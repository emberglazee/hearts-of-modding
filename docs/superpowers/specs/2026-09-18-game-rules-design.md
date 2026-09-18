# Game Rules Support — Design Spec

**Date:** 2026-09-18
**Status:** approved §§1–3 by Embi (chat, 2026-09-18); severities for HOM5012/5013
**pending empirical confirmation** (probe mod built, awaiting one game launch).
**Goal:** Give HOI4 game rules the same first-class treatment as every other
script entity — scan `common/game_rules/*.txt`, validate both the definition
files and the ~2,700 `has_game_rule` usage sites, and provide completion,
highlighting, hover and goto-definition for rule and option tokens.

## Background

Game rules are the pre-game options in the Game Rules screen (`allow_wargoals`,
`hardcore_mode`, the per-country `*_ai_behavior` set). Script reads them with
exactly one construct — the `has_game_rule` trigger — in focuses, events,
on_actions, decisions and scripted triggers/effects.

Hearts of Modding supports them **partially today**, and the gap is silent:

- `backend.rs:1868-1872` inserts `group`, `required_dlc`, `exclude_dlc`,
  `allow_achievements` as static keywords (cosmetic highlighting only).
- `hoi4_data.json` carries `has_game_rule` as a `value_trigger` with
  `"parameters": {}` and an example that is factually wrong
  (`option = yes`); no rule/option token is known to the server.
- **Zero validation exists.** A typo'd rule token or an option token that does
  not belong to that rule evaluates false, produces no `error.log` line and no
  tooltip — the gate is simply dead and the content silently misbehaves.

Measured corpus (2026-09-18, game 1.19.3.0):

| Fact | Value |
|---|---|
| Vanilla rule tokens in `common/game_rules/00_game_rules.txt` | 86 |
| Hearts of Minecraft rule tokens (mod adds 14, redefines 20) | 34 |
| `has_game_rule` call sites (vanilla / Hearts of Minecraft) | 2253 / 420 |
| Call-site shapes | **100%** `has_game_rule = { rule = X option = Y }`; keys inside are only `rule` and `option`; no bare-value form exists anywhere |
| Rule-level keys used | `name`, `group`, `desc`, `icon`, `required_dlc`, `allow_achievements`, `option`, `default` — nothing else |
| Option-block keys used | `name`, `text`, `desc`, `allow_achievements`, plus `required_dlc` (69) and `exclude_dlc` (14) |
| `allow_achievements` occurrences | rule level, option level, and inside `default = { }` |
| Naive rule×option cross-check | **0 real errors** in vanilla; the only 4 hits in Hearts of Minecraft are inside comments (a commented-out `PLC_ai_behavior` block) |

### Rule resolution: the FIRST definition of a token wins (engine-verified)

Settled 2026-09-18 by the probe's load-time validation channel (game 1.19.3.0;
engine messages quoted verbatim in §9):

- `has_game_rule = { rule = allow_wargoals option = PDX_PROBE_FREE_X }` — where
  `PDX_PROBE_FREE_X` is an option the probe's **own redeclaration** of
  `allow_wargoals` declares — is rejected:
  `game rule option PDX_PROBE_FREE_X is not valid for the rule allow_wargoals`,
  while vanilla's `FREE_25`, which that redeclaration omits, stays **valid**.
  Redeclaring a vanilla rule therefore does nothing: the first definition
  (vanilla, loaded first) is the one that binds.
- The same holds within one mod: `PDX_probe_rule_b`, declared in
  `00_rules_probe_a.txt` and again in `01_rules_probe_b.txt` with a disjoint
  option set, validates `FROM_FILE_A` and rejects `FROM_FILE_B`. First file wins.

Consequences for the server:

1. `LayeredValue`'s highest-priority-wins resolution is the **wrong model for
   game rules**. The registry must resolve to the *first* definition in load
   order (vanilla → parent mod → submod, filename order within a root) and its
   option set — no union across layers, no mod-overrides-vanilla.
2. Validation of an `option` token must run against that first-definition set.
   A mod redefinition's options are not merely lower priority; they are invalid.
3. Hearts of Minecraft is unaffected in practice today: all 20 of its vanilla
   redeclarations are identical to vanilla in every field the comparison covers
   (name, group, icon, rule- and option-level `allow_achievements`, DLC fields,
   the option list and its order) — i.e. they are no-ops. A future edit inside
   one of those blocks would silently never apply, which is worth recording for
   the mod authors rather than fixing in the LSP.

Residual unknown (single mechanism vs two): vanilla-beats-mod and
first-file-beats-second-file are both consistent with "first definition wins in
load order", which is what the LSP will implement. A mod-vs-mod conflict across
different roots was not probed.

## §1 Data model

New `scanner/game_rule_scanner.rs`, following the standard scanner shape
(`scan_*_files(files, filter) -> HashMap<String, T>` + a shared
`extract_*` used by the full scan and the incremental updater):

```rust
pub struct GameRuleOption {
    pub name: InternedStr,               // the token has_game_rule matches
    pub text_key: Option<InternedStr>,   // loc key (`text =`)
    pub desc_key: Option<InternedStr>,
    pub is_default: bool,                // came from a `default = { }` block
    pub allow_achievements: Option<bool>,
    pub required_dlc: Option<InternedStr>,
    pub exclude_dlc: Option<InternedStr>,
    pub path: InternedStr,
    pub range: ast::Range,               // the option block's `name` value range
}

pub struct GameRule {
    pub name: String,                    // == block token (path/range convention)
    pub loc_key: Option<InternedStr>,    // `name = "RULE_..."` — a loc KEY, not an identifier
    pub desc_key: Option<InternedStr>,
    pub groups: Vec<InternedStr>,
    pub icon: Option<InternedStr>,
    pub required_dlc: Option<InternedStr>,
    pub allow_achievements: Option<bool>, // rule level, incl. inside `default`
    pub options: Vec<GameRuleOption>,     // deduped case-insensitively, order preserved
    pub path: InternedStr,
    pub range: ast::Range,                // the rule block key
}
```

**Registries** (both `DashMap<InternedStr, LayeredValue<T>>` with a companion
`*_file_index`, so `retain_path!`/`remove_path!` stay O(K)):

- `game_rules` — keyed by the block token (`allow_wargoals`, `SAV_ai_behavior`).
- `game_rule_options` — keyed `"<rule_lower>::<option_lower>"`. Option identity
  is rule-scoped: `RANDOM` appears in 7 different rules, `LIMITED` in several,
  so a flat name map would collide.

Both are declared in `for_each_standard_scanner!`
(`scanner/registry.rs`), which generates the `find_definition` arm, the
`entity_names` arm and the `rebuild_all_file_indices` entry:

```rust
$mac!(game_rule_scanner, GameRule, GameRule, game_rules, "common/game_rules", &["txt"]);
```

**Extraction rules** (all measured against the corpus):

- `name`, `group`, `desc`, `icon`, `required_dlc` yield loc keys or sprite
  names and may be written quoted or bare; resolve both identically.
- Inside an option block `name` is an **identifier** (the token), while `text`
  and `desc` are loc keys. The same is true of a `default = { }` block, except
  that a `default` block's `text`/`desc` describe the rule.
- Options are deduped case-insensitively, order preserved; `is_default` marks
  the option that a `default` block supplies.
- Keys may appear in any order (`option` before `default` is common in vanilla).
- Comment stripping is mandatory — the parser already produces comment entries,
  but the token cross-check must not read commented-out rules (this is exactly
  how the "4 unresolved PLC usages" artefact appeared in the measurement).

**Definition schema** in `hoi4_data.json` (`definitions`):

```json
"game_rule":        { "file_types": ["GameRules"], "parameters": {
    "name": …, "desc": …, "icon": …, "group": …, "required_dlc": …, "allow_achievements": … } },
"game_rule_option": { "file_types": ["GameRules"], "parameters": {
    "name": …, "text": …, "desc": …, "allow_achievements": …, "required_dlc": …, "exclude_dlc": … } }
```

This clears the unknown-entity flood for definition-file bodies (otherwise
`group`/`required_dlc` etc. are undefined), adds definition sub-key completion
for free, and follows the same pattern as `focus` / `decision`.

## §2 Integration points (six)

| # | Site | Change |
|---|---|---|
| 1 | `scanner/registry.rs` | the `for_each_standard_scanner!` entry above |
| 2 | `data/scanner_data.rs` | fields `game_rules`, `game_rule_options`, `game_rules_file_index`, `game_rule_options_file_index`; the macro covers the first index, the second gets one manual `rebuild_index!` |
| 3 | `scanner/incremental_scanner.rs` | `FileCategory::GameRules` + `as_str` entry; `classify_file` arm for `/common/game_rules/`; `update_from_ast` arm; `remove_path_from_scanner_data` arms for both maps |
| 4 | `scanner/orchestrator.rs` + `lsp/handler.rs` | `scan_game_rules(overlay)` via `scan_dashmap_overlay!`, added to the startup `tokio::join!`; `update_entity_token_context()` then makes the new names highlightable with no further wiring |
| 5 | `lsp/semantic_tokens.rs` | `EntityKind::GameRule` → `Enum` (rule token at definition sites; this is what the registry's entity arm already produces), `EntityKind::GameRuleOption` → `EnumMember`; one value-side branch: `parent_key == "rule"` resolves the value as a rule, and an `option` value inside an enclosing `has_game_rule` chain resolves as an option (the chain test is what keeps event `option = { }` blocks untouched) |
| 6 | `lsp/completion_handler.rs` | inside a `has_game_rule` body (`find_enclosing_block_key_chain`, the same helper the block-params walk uses): offer `rule`/`option` with `0_` sort_text, and on an `option` value read the sibling `rule` value and offer **only that rule's option tokens**, default first. Additive — the generic list below is untouched |

Goto-definition and hover come from the same registry: `find_identifier_at`
already returns `(identifier, …, context_key)`, so `rule = allow_wargoals`
arrives as `("allow_wargoals", context="rule")` and `option = MONARCHIST` as
`("MONARCHIST", context="option")`. The option lookup is rule-scoped when the
context supplies the rule and falls back to the flat key otherwise; hover
renders the rule's loc key, group, DLC requirement and its option list.

**Deliberately out of scope:** rename (the `RenameableSymbol` whitelist at
`rename.rs:171-183` covers 8 entity kinds and ends in `_ => return None`; game
rules are not added by this spec) and any change to the keyword set in
`backend.rs:1868-1872` (those four keys stay as they are — the definition schema
now owns real highlighting for them).

## §3 Validation — usage sites

`rules/game_rules.rs`, an `AstVisitor` + `after_walk`, registered in
`Backend::check_semantic`; `ValidationContext` gains a `game_rules` reference.

| Code | Fires when | Severity |
|---|---|---|
| `HOM5012` | `rule = X` inside `has_game_rule` where X resolves in no scanned `game_rules` (case-insensitive) | **ERROR** |
| `HOM5013` | `option = Y` where the rule resolves but Y is not one of *that resolved rule's* options | **ERROR** |
| `HOM5014` | `has_game_rule` block missing `rule` or missing `option` | **ERROR** |

**All three at ERROR — engine-verified, not aspirational.** The engine validates
every `has_game_rule` block statically at load and logs a matching message for
each of these three cases, then drops the trigger
(`trigger.cpp:117: Trigger failed to validate`). The LSP emits exactly the class
of problem the engine itself reports, at the engine's own severity class:

| Engine message (`triggerimplementation.cpp`) | LSP code |
|---|---|
| `9803`: `game rule <X> does not exist` | `HOM5012` |
| `9823`: `game rule option <Y> is not valid for the rule <X>` | `HOM5013` |
| `9808`: `rule option is not specified` | `HOM5014` (no `option`) |
| `9796`: `game rule is not specified` | `HOM5014` (no `rule`) |

Two nuances this evidence forces:

- **DLC gating makes a declaration conditional.** A rule carrying `required_dlc`
  (or `exclude_dlc`) whose condition is not met is *not registered* — the engine
  then reports every use of it as `does not exist`. Verified on this machine: the
  two vanilla rules gated behind "Thunder at Our Gates" (`INS_ai_behavior`,
  `SIA_ai_behavior`) are absent because that DLC is not installed, so **vanilla's
  own `common/on_actions/00_on_actions.txt` logs four such errors per load in a
  stock install**. The server therefore treats a DLC-gated declaration as
  sufficient for `HOM5012` silence — a deliberate false negative, since resolving
  installed-DLC state is out of scope (§8).
- The engine validates **eagerly at parse time**, so commented-out blocks are
  never parsed and stay silent automatically — no extra guard needed for the
  commented `PLC_ai_behavior` usages in Hearts of Minecraft.

- Only blocks whose key is `has_game_rule` are considered; `rule` / `option`
  elsewhere in the language is untouched (`option` is a transparent block used
  by events, `rule` is unremarkable).
- Case-insensitive resolution both ways, mirroring the engine (probe F5
  measures whether the engine really is insensitive to option-token case; if it
  is not, matching tightens to case-sensitive for options).
- **Bite requirement:** the two sites in Hearts of Minecraft that a naive check
  flags are inside comments — the parser must not produce diagnostics for
  commented-out blocks (they are not AST entries), and the probe mod provides
  the live negative control.

## §4 Validation — definition files

Second visitor in the same module, gated on `FileCategory::GameRules`.

| Code | Fires when |
|---|---|
| `HOM5015` | rule block missing required `name` / `group`; or an option block missing `name` / `text` |
| `HOM5016` | duplicate rule token across files (a mod redefinition without `replace_path`), or duplicate option token inside one rule — the engine keeps the last, so the earlier definition is unreachable |
| `HOM5017` | loc key (`name`, `desc`, `text`, `group`) resolves in neither the scanned loc registry nor vanilla, compared case-insensitively — the game renders the raw key with no `error.log` signal |
| `HOM5018` | `icon` names a sprite that is not in the sprite registry |
| `HOM5019` | an option sets `allow_achievements = yes` under a rule-level `allow_achievements = no` — the option-level value can never take effect |

Severity split: **HOM5015/5016/5019 = WARN**, **HOM5017/5018 = INFO**. The two
INFO codes are the ones most likely to have a source outside the scanned set (a
DLC loc file, a submod's own art) and both degrade only the UI. HOM5015 is a
WARN rather than ERROR because a rule without `name`/`group` still loads and
still gates script.

Codes are allocated in `validation/advanced_validation.rs` in the existing
HOM50xx band (HOM5011 is the highest allocated today).

## §5 Semantic tokens

- Definition site: the rule's block token and the `name =` values of its option
  blocks resolve through the registry arms the macro generates
  (`Enum` / `EnumMember`). A `default = { }` block's `name` value is an option
  token like any other (engine-verified, §9 F8), so it takes `EnumMember` too.
- Usage site: `has_game_rule`, `rule` and `option` are keyword-keyed already;
  the change is the **value** side —
  `rule = allow_wargoals` → `Enum`,
  `option = MONARCHIST` (inside a `has_game_rule` chain) → `EnumMember`.
  Everything else in the tokenizer's fallback path is unchanged, so a bad token
  keeps the neutral string colour rather than disappearing.

## §6 Hover and goto-definition

- Hover on a rule token: rule loc name/desc, group, DLC requirement, option
  count, and the option list (default marked).
- Hover on an option token: its loc text/desc and the rule it belongs to.
- Goto-definition: rule token → the rule block; option token → the option
  block's `name` value (rule-scoped key first, flat key as fallback).
- Both go through `EntityLookup::find_definition`, which the macro arm extends;
  no new lookup path is introduced.

## §7 Testing plan

| Layer | Test |
|---|---|
| Scanner unit | synthetic AST via the real parser: full rule with options/default, option-before-default ordering, quoted and bare keys, mixed-case option tokens, comments containing rule-like tokens |
| Scanner registry | `rebuild_all_file_indices` populates both indices; `remove_path` deletes both rule and option entries (the add→edit→delete regression shape) |
| Rules | `TestCtx`-based tests per code (HOM5012–5019), each with a positive **and** a control case (declared rule+option → clean), plus a commented-out block → clean |
| Completion | cursor inside `has_game_rule` → `rule`/`option` offered; cursor on an `option` value with a sibling `rule` → exactly that rule's options; event `option = { }` → unchanged |
| Semantic tokens | definition rule token → Enum; `rule =` value → Enum; option value inside the chain → EnumMember; the same token outside the chain → unchanged |
| End-to-end | probe mod (§9) in a real game load: `error.log` clean, and the resolved verdicts match |

CI bar: `cargo test --bin hom-lsp`, `cargo clippy --all-targets -- -D warnings`,
`cargo fmt` (repo gates). CHANGELOG entry per repo convention: one line, bold
summary, explanation continuing on the same line.

## §8 Not in scope

- Rename participation for rule/option tokens.
- Validating `required_dlc` / `exclude_dlc` against a DLC registry (no such
  registry exists in the server today; the values are free-form strings).
- A dedicated `hoi4.validator.gameRules` setting — the new codes follow the
  existing always-on unknown-entity behaviour (HOM5001–5003), which is the same
  class of check. Revisit only if the probe shows the checks can misfire.
- Any change to game-rule *loading* or the keyword set.

## §9 Probe status

The probe mod `probe_game_rules` (user mod dir, pointer `.mod` alongside)
carries a `# F<n>` marker on each `has_game_rule = {` line, and
`scripts/analyze_game_rules_probe.py` maps the engine's own `file:line`
citations back to those keys. Two channels: **load-time validation**
(`logs/error.log`, populated as soon as the game reaches a game-start screen)
and **runtime evaluation** (`logs/game.log`, needs one campaign day).

### Settled by the load-time channel (2026-09-18, 1.19.3.0)

| Key | Verdict | Consequence |
|---|---|---|
| F1 | `game rule pdx_probe_nonexistent_rule does not exist` | HOM5012 = ERROR |
| F2 | `game rule option pdx_probe_nonexistent_option is not valid for the rule allow_wargoals` | HOM5013 = ERROR |
| F3 | clean — vanilla `FREE_25` still valid despite the redeclaration omitting it | first definition wins (§1) |
| F3b | the redeclaration's own option is **rejected** | a mod cannot extend a vanilla rule |
| F4a / F4b | `FROM_FILE_A` valid, `FROM_FILE_B` rejected | first file wins within a mod |
| F5 | differently-cased rule **and** option tokens accepted | case-insensitive matching, both directions |
| F8 | a token declared only inside `default = { }` is a valid target | default-block tokens belong in the option set (affects §5/§6) |
| F9 | `rule option is not specified` | HOM5014 = ERROR |
| F10 | `game rule is not specified` | HOM5014 = ERROR |
| Control | F7 validated clean | the probe's own rules registered; the run is trustworthy |

### Outstanding (needs one campaign day)

- **F6a/F6b** — with no `default` block, which option does the engine
  preselect? Both tokens validate, so only the runtime/UI answer distinguishes
  "first option block is the implicit default" (the vanilla header's claim) from
  a changed rule. Affects §6 hover/completion ordering.
- **F11–F16 runtime halves** — F11 (defective definition still matchable),
  F12 (which duplicate option token survives), F13 (bad icon: rule still gates),
  F14/F15 (a rule carrying `required_dlc`/`exclude_dlc` that the machine fails:
  expect the rule to be dropped — the mechanism the vanilla INS/SIA errors in
  §3 exposed), F16 (option-level DLC gating: option dropped, or rule killed).
  Their load-time rows currently read `does not exist` only because those rules
  had not been written yet when the log was produced — the next load is the
  real test.
- The engine's exact messages are reproduced verbatim by the analyzer's
  "engine messages" section; encode them in the skill reference rather than
  paraphrasing.

### Residual (not probed)

- Mod-vs-mod conflict across different roots (see §1).
- Whether `allow_achievements` correctness matters to the engine at all: no
  probe case exists for a wrong `allow_achievements` value, so §4's HOM5019 sits
  on corpus evidence only.

## §10 File inventory

**New**

- `server/src/scanner/game_rule_scanner.rs`
- `server/src/rules/game_rules.rs`
- `server/src/tests/game_rules.rs`
- `~/.hermes/skills/gaming/hoi4-modding/scripts/analyze_game_rules_probe.py`
- `~/.hermes/skills/gaming/hoi4-modding/references/game-rules-empirical.md` (post-probe)
- mod dir: `probe_game_rules.mod` + `probe_game_rules/` — 4 game-rules files (`00_rules_probe_a.txt`, `01_rules_probe_b.txt`, `02_rules_probe_defects.txt`, `99_rules_probe_z.txt`), 1 on_actions file (13 → 19 `# F<n>`-marked checks), 1 probe-only loc file, descriptor — under `~/.local/share/Paradox Interactive/Hearts of Iron IV/mod/`
- FTS (For Tomorrow's Sake) is the second live mod in the same playset — `~/git/github/emberglazee/Hearts-Of-Minecraft-For-Tomorrows-Sake`, symlinked from the mod dir as `hom-fts`, zero `replace_path` entries and no `game_rules` directory of its own, so it contributes no rules but does consume the parent's

**Modified**

- `server/src/{backend.rs, main.rs}` — registry, `ValidationContext` field, visitor/rule registration
- `server/src/data/{scanner_data.rs, entity_lookup.rs, hoi4_data.rs}`
- `server/src/scanner/{registry.rs, orchestrator.rs, incremental_scanner.rs}`
- `server/src/lsp/{handler.rs, completion_handler.rs, semantic_tokens.rs}`
- `server/src/validation/advanced_validation.rs`
- `server/assets/hoi4_data.json`
- `server/src/tests/mod.rs`
- `CHANGELOG.md`
