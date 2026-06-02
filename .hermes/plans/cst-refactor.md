# CST Refactor: Tokenizer → CST Parser → AST Shim → Formatter

> **For Hermes:** Use subagent-driven-development to implement this plan phase-by-phase.
> The user guides direction — each major phase is discussed before implementation.

**Goal:** Replace HOI4 script parser with a tokenizer-based CST pipeline that supports
error recovery (brace tracking, error nodes) and lossless formatting (CST transform + serialize).

**Principle:** Keep the existing `ast::*` types completely unchanged. All 50+ semantic
consumers (scanners, LSP handlers, validation rules, utils) keep working through an
AST-lowering shim. Only the parser and formatter see the new types.

**Architecture:**
```
Source text
    │
    ├─▶ Tokenizer (chars → tokens with trivia)
    │       │
    │       ▼
    │   CST Parser (tokens → CstNode tree)  ← error recovery here
    │       │
    │       ├─▶ AST Lowering (CstNode → ast::Entry etc.) → semantic consumers
    │       │
    │       └─▶ CST Formatter (transform tree + serialize) → formatted output
```

**Tech stack:** Rust, no external parsing deps (hand-written char scanner +
recursive descent), existing `ast::*` types untouched, existing `ast::Range` reused.

**Branch:** `refactor/cst-parser` (from main)

---

## Phase 0: Setup

### Task 0.1: Create branch + skeleton

- `git checkout -b refactor/cst-parser`
- Create `server/src/parser/cst/` module:
  - `mod.rs` — re-exports pub API
  - `token.rs` — `CstToken`, `Trivia`, `TokenKind`
  - `types.rs` — `CstScript`, `CstNode`, `CstAssignment`, `CstValue`, `CstBlock`
  - `lexer.rs` — `tokenize(input: &str) -> Vec<CstToken>`
  - `parser.rs` — `parse_cst(tokens) -> CstScript` (recursive descent)
  - `lower.rs` — `cst_to_ast(cst) -> ast::Script`
  - `diagnostic.rs` — parser diagnostic types
- Wire into `server/src/parser/mod.rs` as `pub mod cst;`
- **Commit:** `git add -A && git commit -m "refactor(cst): add module skeleton"`

---

## Phase 1: Foundation — Tokenizer + CST Types

### Task 1.1: TextRange + Diagnostic types

**Objective:** Shared infrastructure for byte-level positioning and error reporting.

**Files:**
- Create: `server/src/parser/cst/diagnostic.rs`
- Create: `server/src/parser/cst/token.rs` (beginning — TextRange type)

**Design:**

```rust
// diagnostic.rs
#[derive(Debug, Clone)]
pub struct CstDiagnostic {
    pub message: String,
    pub range: ast::Range,       // line/col for LSP
    pub severity: Severity,      // Error, Warning
    pub fix: Option<String>,     // auto-fixable suggestion
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity { Error, Warning }
```

```rust
// token.rs — TextRange
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}
```

**Commit:** `refactor(cst): add TextRange, CstDiagnostic, Severity`

---

### Task 1.2: TokenKind + Trivia + CstToken

**Objective:** Define all HOI4 script token types and trivia (whitespace, comments).

**File:** `server/src/parser/cst/token.rs`

**Design:**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Structural
    OpenBrace, CloseBrace,
    // Operators
    OpLessOrEqual, OpGreaterOrEqual, OpNotEquals,
    OpEquals, OpLessThan, OpGreaterThan,
    // Literals
    Ident(String), String(String), Number(f64), Boolean(bool),
    // Comments are attached as trivia, but a standalone comment
    // on its own line is represented as an entry-level CstNode
    Eof,
}

#[derive(Debug, Clone)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub text: String,
    pub range: ast::Range,
}

#[derive(Debug, Clone)]
pub enum TriviaKind {
    Whitespace,  // inline spaces/tabs
    Newline,     // \n or \r\n
    Comment,     // # ... \n
}

#[derive(Debug, Clone)]
pub struct CstToken {
    pub kind: TokenKind,
    pub text: String,
    pub range: ast::Range,
    pub byte_range: TextRange,
    pub leading_trivia: Vec<Trivia>,
}
```

**Key rules for token boundaries:**
- `<=`, `>=`, `!=` are two-char tokens (check second char after seeing `<`, `>`, `!`)
- `=`, `<`, `>` are single-char tokens
- `yes`/`no` are keyword-checked in the parser, tokenized as `Ident`
- `{` and `}` are their own tokens
- `"..."` strings handle `\"` escapes
- Numbers match `-?[0-9]+(\.[0-9]+)?`
- Identifiers include alphanumeric + `_ . : @ [ ] ? ^ $ / - ' % | *`
- Trivia: everything between tokens that isn't a token

**Commit:** `refactor(cst): add TokenKind, Trivia, CstToken types`

---

### Task 1.3: Tokenizer (lexer)

**Objective:** Convert raw source text into `Vec<CstToken>` with trivia attached.

**File:** `server/src/parser/cst/lexer.rs`

**Design:**

```
pub fn tokenize(input: &str) -> (Vec<CstToken>, Vec<CstDiagnostic>)

Algorithm:
  byte_pos = 0
  while byte_pos < input.len():
    // 1. Collect trailing trivia from previous token
    //    (or leading trivia for first token)
    
    // 2. Accumulate leading trivia: whitespace, newlines, comments
    //    - Spaces/tabs → Whitespace trivia
    //    - \n / \r\n → Newline trivia
    //    - # to end of line → Comment trivia
    
    // 3. Determine token kind:
    //    { → OpenBrace, } → CloseBrace
    //    <= → OpLessOrEqual, >= → OpGreaterOrEqual
    //    != → OpNotEquals, = → OpEquals
    //    < → OpLessThan, > → OpGreaterThan
    //    " → string (handle escapes)
    //    digit or .digit or -digit → Number
    //    identifier char → Ident (yes/no handled by parser)
    //    other char → diagnostic error, skip one char
    
    // 4. Create CstToken with:
    //    - kind, text, range (line/col), byte_range
    //    - Leading trivia: ALL trivia accumulated since last token
    
  // After loop: trailing trivia after last token stored separately
```

**Edge cases:**
- BOM `\u{feff}` — strip before tokenizing (like current parser does)
- Empty input → empty token list, no diagnostics
- Only whitespace → empty token list (all whitespace becomes final trivia)
- Mixed `\r\n` — normalize to single newline
- Identifiers starting with `-` followed by digit → number token, not ident
- String with escaped quote `\"` → correct content
- Unterminated string → diagnostic + close at EOF
- Single-character input like `§` → one diagnostic, continue

**Testing approach:**
- Test each token kind individually
- Test trivia collection (leading whitespace, newlines, comments)
- Test string escaping
- Test number edge cases
- Test unterminated strings
- Test error recovery on unexpected chars

**Commit:** `refactor(cst): implement tokenizer with tests`

---

### Task 1.4: CST node types

**Objective:** Define the full CST tree structure.

**File:** `server/src/parser/cst/types.rs`

**Design:**

```rust
#[derive(Debug, Clone)]
pub struct CstScript {
    pub nodes: Vec<CstNode>,
    pub eof_diagnostics: Vec<CstDiagnostic>,
    pub trailing_trivia: Vec<Trivia>,  // whitespace after last token
}

#[derive(Debug, Clone)]
pub enum CstNode {
    Assignment(CstAssignment),
    EntryValue(CstEntryValue),
    EntryComment(Trivia),
    Error(CstDiagnostic),  // parse error at entry level
}

#[derive(Debug, Clone)]
pub struct CstAssignment {
    pub key: CstToken,
    // Note: operator token's leading_trivia has whitespace between key and operator
    pub operator: CstToken,
    pub value: CstValue,
}

#[derive(Debug, Clone)]
pub struct CstEntryValue {
    pub value: Box<CstValue>,
}

#[derive(Debug, Clone)]
pub enum CstValue {
    Ident(CstToken),
    String(CstToken),
    Number(CstToken),
    Boolean(CstToken),
    Block(CstBlock),          // { ... }
    TaggedBlock {
        tag: CstToken,
        block: Box<CstBlock>, // ident { ... }
    },
    Error(CstDiagnostic),
}

#[derive(Debug, Clone)]
pub struct CstBlock {
    pub open_brace: CstToken,
    pub entries: Vec<CstNode>,
    pub close_brace: CloseBrace,  // may be missing on error
}

#[derive(Debug, Clone)]
pub enum CloseBrace {
    Present(CstToken),
    Missing(CstDiagnostic),
}
```

**Key design decisions:**
- `CstAssignment` has `key + operator + value` always. If no operator found,
  the entire thing becomes `CstValue` instead.
- `CstEntryValue` wraps a bare value (no key/operator). This matches `Entry::Value`.
- `CstBlock.close_brace` uses `Result`-like enum to represent missing brace.
- `Trivia` is embedded in each token's `leading_trivia` — the tree doesn't have
  separate trivia nodes between entries. Each token carries the whitespace that
  PRECEDES it.
- `CstNode::EntryComment` captures standalone `# ...` lines that aren't attached
  to any token.

**Commit:** `refactor(cst): add CST node types (CstScript, CstNode, CstValue, CstBlock)`

---

### Task 1.5: CST Parser (recursive descent from tokens)

**Objective:** Convert `Vec<CstToken>` into `CstScript` tree with error recovery.

**File:** `server/src/parser/cst/parser.rs`

**Design:**

```rust
pub struct CstParser {
    tokens: Vec<CstToken>,
    pos: usize,
    diagnostics: Vec<CstDiagnostic>,
    brace_depth: u32,        // for tracking unclosed braces
}

impl CstParser {
    pub fn new(tokens: Vec<CstToken>) -> Self { ... }
    pub fn parse(mut self) -> CstScript { ... }
    
    // Core parse functions:
    fn parse_script(&mut self) -> CstScript;
    fn parse_entry(&mut self) -> Option<CstNode>;
    fn parse_assignment(&mut self, key: CstToken) -> CstAssignment;
    fn parse_value(&mut self) -> CstValue;
    fn parse_block(&mut self) -> CstBlock;
    
    // Error recovery:
    fn recover_to_next_entry(&mut self);
    fn handle_unclosed_braces(&mut self) -> Vec<CstDiagnostic>;
    
    // Token helpers:
    fn peek(&self) -> Option<&CstToken>;
    fn advance(&mut self) -> Option<CstToken>;
    fn expect(&mut self, kind: TokenKind, msg: &str) -> Option<CstToken>;
    fn is_at_end(&self) -> bool;
}
```

**Parse logic:**

```
parse_script():
  while not at end:
    if peek is Comment (standalone comment entry):
      → CstNode::EntryComment
    if peek is Ident:
      key = advance()
      if peek is operator (=, <, >, !=, <=, >=):
        → parse_assignment(key) → CstAssignment
      else:
        → parse_value() → CstEntryValue
    if peek is OpenBrace:
      → parse_block() → CstEntryValue(Value::Block)
    if peek is String, Number, Boolean:
      → parse_value() → CstEntryValue
    else:
      → error: unexpected token → CstNode::Error
      skip one token, continue

parse_assignment(key):
  op_token = advance() (must be an operator)
  val = parse_value()
  if val is Error:
    diagnostic about missing value
  CstAssignment { key, operator: op_token, value: val }

parse_value():
  if peek is OpenBrace:
    → parse_block()
  if peek is Ident(text):
    advance()
    if peek is OpenBrace:
      → CstValue::TaggedBlock { tag: ident_token, block: parse_block() }
    else:
      if text == "yes" or text == "no":
        → CstValue::Boolean(...)
      elif text is valid float:
        → CstValue::Number(...)  // mirrors current parser's fallback
      else:
        → CstValue::Ident(...)
  if peek is String → advance() → CstValue::String
  if peek is Number → advance() → CstValue::Number
  else:
    → error "expected value" → CstValue::Error

parse_block():
  open = advance() (must be OpenBrace)
  while not at_end and peek is not CloseBrace:
    parse entry
  if at_end and brace_depth > 0:
    close = CloseBrace::Missing(diagnostic "expected '}'")
    brace_depth -= 1
  else:
    close = CloseBrace::Present(advance())
  CstBlock { open_brace: open, entries, close_brace: close }
```

**Brace depth tracking at EOF:**
When EOF is reached with `brace_depth > 0`:
1. For each missing `}`, emit a diagnostic: `"Expected '}' to close block"`
2. Attach the diagnostic to the `CstBlock.close_brace` as `CloseBrace::Missing`
3. The content after the missing brace position is preserved (it was parsed as entries)

**Error recovery within a block:**
When an unexpected token is encountered while parsing an entry:
1. Emit diagnostic with the token's position
2. Skip tokens until we find a token that could start a new entry
3. Continue parsing from there

**Testing approach:**
- Test well-formed input produces correct tree
- Test missing `}` at various depths
- Test extra `}` (stray close brace)
- Test empty file
- Test comment-only file
- Test TaggedBlock variants (e.g., `modifier = my_tag { ... }`)
- Test mixed trivia (tabs, spaces, newlines)
- Test edge cases from old parser tests (pipes in idents, `@`, `[?var]`, etc.)

**Commit:** `refactor(cst): implement CST parser with error recovery`

---

## Phase 2: AST Lowering (bridge)

### Task 2.1: CST → AST conversion

**Objective:** Convert `CstScript` → `ast::Script` so all 50+ consumers keep working
WITHOUT any changes.

**File:** `server/src/parser/cst/lower.rs`

**Design:**

```rust
pub fn lower(cst: CstScript) -> (ast::Script, Vec<(String, ast::Range)>) {
    let mut entries = Vec::new();
    let mut diagnostics = Vec::new();
    
    // Process CST diagnostics → entries
    for diag in collect_diagnostics(&cst) {
        diagnostics.push((diag.message, diag.range));
    }
    
    // Process CST nodes → AST entries
    for node in cst.nodes {
        entries.push(lower_node(node));
    }
    
    (ast::Script { entries }, diagnostics)
}

fn lower_node(node: CstNode) -> ast::Entry {
    match node {
        CstNode::Assignment(cst_ass) => {
            ast::Entry::Assignment(ast::Assignment {
                key: cst_ass.key.text,
                key_range: token_to_range(cst_ass.key),
                operator: operator_kind(cst_ass.operator),
                operator_range: token_to_range(cst_ass.operator),
                value: lower_value(cst_ass.value),
            })
        }
        CstNode::EntryValue(ev) => {
            ast::Entry::Value(lower_value_to_nodeed(ev.value))
        }
        CstNode::EntryComment(trivia) => {
            ast::Entry::Comment(trivia.text.trim_start_matches('#').trim().to_string(), trivia.range)
        }
        CstNode::Error(diag) => {
            // Error nodes produce no AST entry (they contribute diagnostics)
            // But we need something in the entries vec...
            // Option: skip them (they're diagnostics, not AST nodes)
            // But that shifts indices... safer: emit Value::String("") with error range
            ast::Entry::Value(ast::NodeedValue {
                value: ast::Value::String(String::new()),
                range: diag.range,
            })
        }
    }
}
```

**Key decisions:**
- `lower_value` mirrors `parse_value` in the old parser: Ident("yes") → Boolean, 
  Ident(parseable as float) → Number, otherwise → String
- `CstNode::Error` emits a placeholder AST entry so entry indices don't shift
- Missing brace diagnostics are added to the diagnostic list

**Commit:** `refactor(cst): implement AST lowering bridge`

---

### Task 2.2: Equivalence testing

**Objective:** Verify the new pipeline produces identical AST to the old parser
for well-formed inputs, and better diagnostics for broken inputs.

**File:** `server/src/parser/cst/tests/equivalence.rs`

**Approach:**

Parse various inputs with both old parser and new pipeline, assert identical AST:

```rust
fn assert_ast_equivalent(input: &str) {
    let (old_script, old_errors) = parser::parse_script(input);
    let (new_script, new_errors) = cst::parse(input);  // tokenize + parse + lower
    assert_eq!(old_script.entries.len(), new_script.entries.len());
    for (old_entry, new_entry) in old_script.entries.iter().zip(new_script.entries.iter()) {
        assert_eq_entries(old_entry, new_entry);
    }
    // Error handling may differ (new parser has MORE errors)
    // So only check that new errors include all old errors
}
```

**Test inputs:**
1. Basic events (from old parser tests) — `test_parse_basic`, `test_parse_complex`
2. Quoted strings with escapes
3. Dots in keys (`daw.2.t`)
4. Pipe in values
5. Special chars in identifiers (`[?var]`, `array^0`)
6. `yes`/`no` booleans
7. Numbers with negative and decimal
8. `TaggedBlock` syntax
9. `Entry::Value` (bare blocks, bare strings)
10. Comments at various positions
11. UTF-8 multibyte chars
12. BOM prefix

Also test on real HOI4 script files from `hoi4-wiki/` directory if available.

**Fix discrepancies:** Run through all failing inputs and adjust the tokenizer,
parser, or lowerer until output matches exactly.

**Commit:** `refactor(cst): add equivalence tests and fix discrepancies`

---

## Phase 3: Switchover

### Task 3.1: Wire new pipeline into `parse_script`

**Objective:** Replace the old `parser::parse_script` with the new pipeline.

**File:** `server/src/parser/parser.rs` (or `server/src/parser/mod.rs`)

**Approach:**

Option A: In-place replacement
```rust
// Old: pub fn parse_script(input: &str) -> (Script, Vec<(String, Range)>)
// New: delegate to cst::parse
pub fn parse_script(input: &str) -> (Script, Vec<(String, Range)>) {
    let cst_result = cst::parse(input);
    cst::cst_to_ast(cst_result)
}
```

Option B: New entry point
```rust
// In mod.rs
pub fn parse_script(input: &str) -> (Script, Vec<(String, Range)>) {
    cst::parse_and_lower(input)
}
```

All 35+ `parser::parse_script(...)` callers continue working unchanged.

**Commit:** `refactor(cst): switch parse_script to new CST pipeline`

---

### Task 3.2: Update Backend to store CST alongside AST

**Objective:** Store the CST in `Backend` so the formatter can access it directly
without re-parsing.

**File:** `server/src/backend.rs`

**Current:**
```rust
document_asts: DashMap<String, (Arc<ast::Script>, Vec<(String, ast::Range)>)>
```

**New:** Either:
- A) Add `document_csts: DashMap<String, Arc<CstScript>>` — separate cache
- B) Combine: `document_parsed: DashMap<String, ParsedDocument>` with both

**Approach A (simpler):**
```rust
document_asts: DashMap<String, (Arc<ast::Script>, Vec<(String, ast::Range)>)>,
document_csts: DashMap<String, Arc<CstScript>>,
```

Update `cache_ast` to also store the CST. Add `ensure_cst_cached` method.

This keeps `ensure_ast_cached` unchanged for all existing callers.

**Commit:** `refactor(cst): cache CST alongside AST in Backend`

---

### Task 3.3: Remove old nom parser code (optional — defer if risky)

**Objective:** Delete `server/src/parser/parser.rs` parser functions that are
now dead code.

**Keep:** The `is_identifier_char` function — it's used by `backend.rs:96`
for word boundary checking in `find_references_in_root`. Move to `parser/mod.rs`
or `utils/`.

**Delete:**
- All `fn *` parser combinators in `parser.rs` (around lines 46-290)
- `fn parse_script` old implementation
- Inline tests that were ported to CST tests

**Commit:** `refactor(cst): remove old nom parser code`

---

## Phase 4: CST Formatter

### Task 4.1: CST Serializer

**Objective:** Convert a `CstScript` back to source text by walking tokens in order.

**File:** `server/src/formatter/serialize.rs` (new `server/src/formatter/` module)

**Design:**

```rust
pub fn serialize(cst: &CstScript) -> String {
    let mut output = String::new();
    serialize_nodes(&cst.nodes, &cst.trailing_trivia, &mut output);
    output
}

fn serialize_nodes(nodes: &[CstNode], trailing_trivia: &[Trivia], out: &mut String) {
    for node in nodes {
        serialize_node(node, out);
    }
    // Emit trailing trivia (whitespace after last token)
    for trivia in trailing_trivia {
        out.push_str(&trivia.text);
    }
}

fn serialize_node(node: &CstNode, out: &mut String) {
    match node {
        CstNode::Assignment(ass) => {
            serialize_token(&ass.key, out);
            serialize_token(&ass.operator, out);
            serialize_value(&ass.value, out);
        }
        CstNode::EntryValue(ev) => {
            serialize_value(&ev.value, out);
        }
        CstNode::EntryComment(trivia) => {
            out.push_str(&trivia.text);  // includes leading trivia (newline + indent)
        }
        CstNode::Error(_) => { /* skip — no text to emit */ }
    }
}

fn serialize_token(token: &CstToken, out: &mut String) {
    // Emit leading trivia (whitespace, newlines, comments)
    for trivia in &token.leading_trivia {
        out.push_str(&trivia.text);
    }
    // Emit token text
    out.push_str(&token.text);
}

fn serialize_value(value: &CstValue, out: &mut String) {
    match value {
        CstValue::Ident(t) | CstValue::String(t) | CstValue::Number(t) | CstValue::Boolean(t) => {
            serialize_token(&t, out);
        }
        CstValue::Block(block) => serialize_block(block, out),
        CstValue::TaggedBlock { tag, block } => {
            serialize_token(&tag, out);
            serialize_block(block, out);
        }
        CstValue::Error(_) => {}
    }
}

fn serialize_block(block: &CstBlock, out: &mut String) {
    serialize_token(&block.open_brace, out);
    serialize_nodes(&block.entries, &[], out);
    match &block.close_brace {
        CloseBrace::Present(token) => serialize_token(token, out),
        CloseBrace::Missing(_) => {
            // Insert a closing brace on a new line at the correct indent
            out.push_str("\n}");
        }
    }
}
```

**Key insight:** The token's `leading_trivia` contains ALL whitespace before
it — so by emitting `trivia.text` for each token, we get the original source
back byte-for-byte.

**Roundtrip test:**
```rust
let input = "...any HOI4 script...";
let cst = parse_cst(tokenize(input));
let output = serialize(&cst);
assert_eq!(input, output);  // lossless!
```

**Commit:** `refactor(formatter): implement CST serializer`

---

### Task 4.2: CST Transformations (formatting rules)

**Objective:** Transform the CST tree to apply formatting rules.

**File:** `server/src/formatter/transform.rs`

**Design:**

Each transform is a function `fn(&mut CstScript)` that walks the tree and
modifies trivia text and token text.

```rust
pub fn format(cst: &mut CstScript) {
    fix_indentation(cst);
    fix_spacing_around_operators(cst);
    fix_brace_newlines(cst);
    fix_brace_spacing(cst);
    fix_trailing_whitespace(cst);
    fix_casing(cst);
    fix_path_separators(cst);
}
```

**Rule: `fix_indentation`**

Walk all tokens. Find the first `\n` in each token's leading trivia. Replace
whitespace after each `\n` with `\t * depth` where depth is computed from
the brace nesting level.

```rust
fn fix_indentation(cst: &mut CstScript) {
    let mut depth = 0u32;
    for node in &mut cst.nodes {
        depth = fix_node_indentation(node, depth);
    }
}
```

**Rule: `fix_spacing_around_operators`**

For each `CstAssignment`, ensure:
- One space before operator (` = ` not `=  ` or `  =`)
- One space after operator

This means modifying the trailing part of the operator's leading trivia
(before the operator token text) and the leading part of the value's
leading trivia (after the operator token text).

**Rule: `fix_brace_newlines`**

For each block:
- If `open_brace.leading_trivia` (after the key/operator/value) contains `\n`,
  replace with a single space (block on same line as opening)
- Except if the block spans multiple lines — keep newline

**Rule: `fix_brace_spacing`**

For single-line blocks `{ ... }`:
- Ensure `{ ` before content, ` }` after content
- Empty blocks → `{}`

**Rule: `fix_casing`**

Replace `key.text` with standard casing for known keywords.

**Rule: `fix_path_separators`**

Replace `\\` and `//` with `/` in string tokens where key is `texturefile`.

**Underlying mechanism:**

Most transforms work by modifying `trivia.text`:
- "   " → " " (normalize spaces)
- "\n\t\t" → "\n\t" (adjust indentation)
- "\n" → " " (inline block)

All modifications preserve the total structure — we only change trivia text
and token text, never remove or reorder tokens.

**Testing:**
- Same test inputs as current formatting module, but output compared via
  CST serialization instead of range patches

**Commit:** `refactor(formatter): implement CST transformations`

---

### Task 4.3: Wire formatter into LSP

**Objective:** Replace the old `formatting.rs` methods with new CST-based ones.

**File:** `server/src/validation/formatting.rs` (rewrite)

**Current pattern:**
```rust
pub(crate) fn collect_styling_fixes(&self, content: &str, fixes: &mut Vec<(Range, String)>)
pub(crate) fn collect_indentation_fixes(&self, ...)
pub(crate) fn collect_assignment_space_fixes(&self, ...)
// ... each produces (Range, String) patches
```

**New pattern:**
```rust
pub(crate) fn format_document(
    &self,
    content: &str,
    uri: &str,
) -> String {
    // 1. Parse to CST (use cached if available)
    let (tokens, _) = tokenize(content);
    let mut cst = parse_cst(tokens);
    
    // 2. Apply formatting transforms
    format(&mut cst);
    
    // 3. Serialize back to text
    serialize(&cst)
}
```

Or more granularly:
```rust
pub(crate) fn collect_formatting_edits(
    &self,
    content: &str,
) -> Vec<TextEdit> {
    let (tokens, _) = tokenize(content);
    let mut cst = parse_cst(tokens);
    format(&mut cst);
    let formatted = serialize(&cst);
    // Diff: produce TextEdits from content → formatted
    compute_text_edits(content, &formatted)
}
```

**Integration with Backend:**
- `Backend::format_document` uses the CST pipeline
- The code actions that trigger formatting are updated to use the new method

**Commit:** `refactor(formatter): wire CST formatter into LSP`

---

### Task 4.4: Remove old formatting code

**Objective:** Delete old range-patch formatting methods.

**Delete:**
- `Backend::collect_styling_fixes`
- `Backend::collect_indentation_fixes`
- `Backend::collect_assignment_space_fixes`
- `Backend::collect_brace_newline_fixes`
- `Backend::collect_brace_space_fixes`
- `Backend::check_and_fix_brace`
- `Backend::collect_path_separator_fixes`
- `Backend::collect_casing_fixes`
- Old code action handlers that reference these

**Commit:** `refactor(formatter): remove old range-patch formatting code`

---

## Phase 5: Cleanup

### Task 5.1: Rename and consolidate

**Objective:** Clean up module organization.

- Move `server/src/parser/cst/` → `server/src/cst/` (top-level module)
  - Rationale: CST is no longer just a parser detail — it's the core
    representation for formatting too
- Or keep it under `parser/` if that's cleaner (it's still a parsing concern)
- Either way, ensure module hierarchy is clean

**Commit:** `chore: consolidate CST module`

---

### Task 5.2: Documentation + memory

**Objective:** Save key design decisions to memory for future sessions.

- Save CST architecture to memory
- Note any non-obvious pitfalls found during implementation

**Commit:** n/a (memory-only)

---

### Task 5.3: Full verification

**Objective:** Everything still works.

```bash
cd server && cargo test && cargo clippy
cd client && npm run compile
```
