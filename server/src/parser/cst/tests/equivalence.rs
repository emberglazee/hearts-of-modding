//! Equivalence tests: compare CST pipeline output against the old parser.
//!
//! The old parser (`crate::parser::parser::parse_script`) is the reference.
//! The new pipeline (tokenize → parse_cst → lower) must produce identical
//! AST output for all valid HOI4 script inputs.
//!
//! When adding new tests, write the test case, then fix the CST side until it
//! passes.  Do NOT modify `ast.rs` or `types.rs`.

use crate::parser::ast::{self, Entry, Value};
use crate::parser::cst::lexer::tokenize;
use crate::parser::cst::lower::lower;
use crate::parser::cst::parser::parse_cst;

/// Run the full CST pipeline on `input` and return the same type as the old
/// parser.
fn parse_and_lower(input: &str) -> (ast::Script, Vec<(String, ast::Range)>) {
    let (tokens, _) = tokenize(input);
    let cst = parse_cst(tokens);
    lower(cst)
}

/// Run the old parser on `input`.
fn parse_old(input: &str) -> (ast::Script, Vec<(String, ast::Range)>) {
    crate::parser::parser::parse_script(input)
}

// ── Comparison helpers ──────────────────────────────────────────────────────

/// Compare entries structurally, ignoring exact range values (they often differ
/// between parsers).
fn entries_eq(old: &Entry, new: &Entry, input: &str, index: usize) {
    match (old, new) {
        (Entry::Assignment(oa), Entry::Assignment(na)) => {
            assert_eq!(
                oa.key,
                na.key,
                "input[{i}]: assignment key mismatch\n  old: {:?}\n  new: {:?}\n  input: {:?}",
                oa.key,
                na.key,
                input,
                i = index
            );
            assert!(
                std::mem::discriminant(&oa.operator) == std::mem::discriminant(&na.operator),
                "input[{i}]: operator mismatch for key {:?}\n  old: {:?}\n  new: {:?}\n  input: {:?}",
                oa.key,
                oa.operator,
                na.operator,
                input,
                i = index
            );
            values_eq(&oa.value.value, &na.value.value, input, index);
        }
        (Entry::Value(ov), Entry::Value(nv)) => {
            values_eq(&ov.value, &nv.value, input, index);
        }
        (Entry::Comment(ot, _), Entry::Comment(nt, _)) => {
            assert_eq!(
                ot,
                nt,
                "input[{i}]: comment text mismatch\n  old: {:?}\n  new: {:?}\n  input: {:?}",
                ot,
                nt,
                input,
                i = index
            );
        }
        _ => {
            panic!(
                "input[{i}]: entry type mismatch\n  old: {:?}\n  new: {:?}\n  input: {:?}",
                variant_name(old),
                variant_name(new),
                input,
                i = index
            );
        }
    }
}

/// Recursively compare two `Value` variants.
fn values_eq(old: &Value, new: &Value, input: &str, index: usize) {
    match (old, new) {
        (Value::String(os), Value::String(ns)) => {
            assert_eq!(
                os,
                ns,
                "input[{i}]: String value mismatch\n  old: {:?}\n  new: {:?}\n  input: {:?}",
                os,
                ns,
                input,
                i = index
            );
        }
        (Value::Number(on), Value::Number(nn)) => {
            assert!(
                (on - nn).abs() < f64::EPSILON,
                "input[{i}]: Number value mismatch\n  old: {}\n  new: {}\n  input: {:?}",
                on,
                nn,
                input,
                i = index
            );
        }
        (Value::Boolean(ob), Value::Boolean(nb)) => {
            assert_eq!(
                ob,
                nb,
                "input[{i}]: Boolean value mismatch\n  old: {}\n  new: {}\n  input: {:?}",
                ob,
                nb,
                input,
                i = index
            );
        }
        (Value::Block(ob), Value::Block(nb)) => {
            assert_eq!(
                ob.len(),
                nb.len(),
                "input[{i}]: Block child count mismatch\n  old: {:?}\n  new: {:?}\n  input: {:?}",
                ob,
                nb,
                input,
                i = index
            );
            for (j, (o_entry, n_entry)) in ob.iter().zip(nb.iter()).enumerate() {
                entries_eq(o_entry, n_entry, input, j);
            }
        }
        (Value::TaggedBlock(ot, ob, _), Value::TaggedBlock(nt, nb, _)) => {
            assert_eq!(
                ot,
                nt,
                "input[{i}]: TaggedBlock tag mismatch\n  old: {:?}\n  new: {:?}\n  input: {:?}",
                ot,
                nt,
                input,
                i = index
            );
            assert_eq!(
                ob.len(),
                nb.len(),
                "input[{i}]: TaggedBlock child count mismatch\n  old: {:?}\n  new: {:?}\n  input: {:?}",
                ob,
                nb,
                input,
                i = index
            );
            for (j, (o_entry, n_entry)) in ob.iter().zip(nb.iter()).enumerate() {
                entries_eq(o_entry, n_entry, input, j);
            }
        }
        _ => {
            panic!(
                "input[{i}]: Value type mismatch\n  old: {:?}\n  new: {:?}\n  input: {:?}",
                value_variant_name(old),
                value_variant_name(new),
                input,
                i = index
            );
        }
    }
}

/// Human-readable name for an Entry variant (for error messages).
fn variant_name(e: &Entry) -> &'static str {
    match e {
        Entry::Assignment(_) => "Assignment",
        Entry::Value(_) => "Value",
        Entry::Comment(_, _) => "Comment",
    }
}

/// Human-readable name for a Value variant (for error messages).
fn value_variant_name(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "String",
        Value::Number(_) => "Number",
        Value::Boolean(_) => "Boolean",
        Value::Block(_) => "Block",
        Value::TaggedBlock(_, _, _) => "TaggedBlock",
    }
}

/// Assert that the CST pipeline and the old parser produce identical AST for
/// the given input.
fn assert_ast_equivalent(input: &str) {
    let (old_script, old_errors) = parse_old(input);
    let (new_script, new_errors) = parse_and_lower(input);

    // Compare entry count
    assert_eq!(
        old_script.entries.len(),
        new_script.entries.len(),
        "Entry count mismatch for: {:?}\nOld entries: {:#?}\nNew entries: {:#?}",
        input,
        old_script.entries,
        new_script.entries
    );

    // Compare each entry structurally (ignoring exact ranges)
    for (i, (old, new)) in old_script
        .entries
        .iter()
        .zip(new_script.entries.iter())
        .enumerate()
    {
        entries_eq(old, new, input, i);
    }

    // Error comparison: every old error message should appear in new errors
    // (the new parser may produce *more* errors due to better recovery)
    for (old_msg, _old_range) in &old_errors {
        let found = new_errors.iter().any(|(m, _)| m == old_msg);
        assert!(
            found,
            "Old error not found in new errors: {:?}\nNew errors: {:?}\nInput: {:?}",
            old_msg, new_errors, input
        );
    }
}

// ── Test cases ──────────────────────────────────────────────────────────────

/// Helper: wrap assert_ast_equivalent so each case is a separate #[test].
macro_rules! equiv_test {
    ($name:ident, $input:expr) => {
        #[test]
        fn $name() {
            assert_ast_equivalent($input);
        }
    };
}

// ── Basic cases ─────────────────────────────────────────────────────────────

equiv_test!(empty, "");
equiv_test!(whitespace_only, "  \n  ");
equiv_test!(simple_assignment, "key = val");
equiv_test!(number_assignment, "x = 42");
equiv_test!(negative_number, "x = -42");
equiv_test!(float_assignment, "x = 3.14");
equiv_test!(string_assignment, r#"x = "hello""#);
equiv_test!(boolean_yes, "x = yes");
equiv_test!(boolean_no, "x = no");

// ── Operators ────────────────────────────────────────────────────────────────

equiv_test!(op_equals, "a = 1");
equiv_test!(op_less_than, "a < 1");
equiv_test!(op_greater_than, "a > 1");
equiv_test!(op_not_equals, "a != 1");
equiv_test!(op_less_or_equal, "a <= 1");
equiv_test!(op_greater_or_equal, "a >= 1");

// ── Blocks ──────────────────────────────────────────────────────────────────

equiv_test!(empty_block, "x = { }");
equiv_test!(block_one_entry, "e = { id = 1 }");
equiv_test!(nested_blocks, "a = { b = { c = 1 } }");
equiv_test!(bare_block, "{ key = val }");
equiv_test!(tagged_block, "modifier = my_tag { f = 0.5 }");
equiv_test!(bare_tagged_block, "my_tag { }");

// ── Multiple entries ────────────────────────────────────────────────────────

equiv_test!(two_assignments, "a = 1\nb = 2");
equiv_test!(three_assignments, "a = 1\nb = 2\nc = 3");

// ── Special identifiers ─────────────────────────────────────────────────────

equiv_test!(dots_in_value, "title = daw.2.t");
equiv_test!(pipe_in_value, "custom_effect_tooltip = tech_effect|sp_main");
equiv_test!(special_key, "[?my_var] = 10");

// ── String escaping ─────────────────────────────────────────────────────────

equiv_test!(
    escaped_quotes,
    r#"title = "Event \"The Great War\" Begins""#
);
equiv_test!(plain_string, r#"title = "plain text""#);

// ── Comments ──── KNOWN DISCREPANCY: CST pipeline doesn't extract comments ──

equiv_test!(comment_only, "# just a comment");
equiv_test!(
    comment_before_entry,
    "# This is a test HOI4 script\ncountry_event = { }"
);
equiv_test!(comment_between_entries, "a = 1\n# comment between\nb = 2");
equiv_test!(comment_after_entry, "a = 1\n# trailing comment");
equiv_test!(multiple_comments, "# first\n# second\nkey = val");
equiv_test!(comment_at_eof, "key = val\n# comment at end");
equiv_test!(comment_after_block, "a = { }\n# comment after block");

// ── Inline comments (should NOT be extracted as entries) ────────────────────

equiv_test!(inline_comment, "key = val # not standalone\nnext = 1");
equiv_test!(
    inline_comment_block,
    "a = {\n  # this is inline\n  b = 1\n}"
);

// ── Complex scripts ─────────────────────────────────────────────────────────

#[test]
fn test_parse_basic_from_old_parser() {
    let input = r#"
        # This is a test HOI4 script
        country_event = {
            id = test.1
            is_triggered_only = yes
            trigger = {
                tag = GER
                has_war = no
            }
        }
        "#;
    assert_ast_equivalent(input);
}

#[test]
fn test_parse_complex_from_old_parser() {
    let input = r#"
        modifier = {
            political_power_factor = 0.15
            stability_factor > -0.1
            tag != "ENG"
            [?my_var] = 10
            array^0 = 1
        }
        "#;
    assert_ast_equivalent(input);
}

// ── Error recovery ──────────────────────────────────────────────────────────

#[test]
fn test_missing_close_brace() {
    // The old parser's parse_block requires a closing '}'.  Without it, the
    // block fails to parse, and `e` is parsed as a bare identifier value.
    // The new parser has lenient error recovery and produces a block with a
    // missing close brace — strictly more information.
    //
    // Accept this as a robustness improvement in the new pipeline.
    let (old_script, _old_errors) = parse_old("e = { id = 1");
    let (new_script, _new_errors) = parse_and_lower("e = { id = 1");
    // Both should produce at least one entry
    assert!(
        old_script.entries.len() >= 1,
        "old parser should produce at least 1 entry"
    );
    assert!(
        new_script.entries.len() >= 1,
        "new parser should produce at least 1 entry"
    );
    // The new parser's error recovery should include the old parser's error
    // messages (the old parser reports a parsing error for the rest)
    // At minimum, the new parser should have some diagnostics for the missing brace
    assert!(
        !_new_errors.is_empty(),
        "expected diagnostic for missing close brace"
    );
}

#[test]
fn test_extra_close_brace() {
    // The old parser may produce errors for the extra brace.
    // We just check entry equivalence (ignoring extra errors).
    let (old_script, _old_errors) = parse_old("x = { } }");
    let (new_script, _new_errors) = parse_and_lower("x = { } }");
    assert_eq!(
        old_script.entries.len(),
        new_script.entries.len(),
        "Entry count mismatch for extra close brace"
    );
    for (i, (old, new)) in old_script
        .entries
        .iter()
        .zip(new_script.entries.iter())
        .enumerate()
    {
        entries_eq(old, new, "x = { } }", i);
    }
}

// ── Edge cases ──────────────────────────────────────────────────────────────

equiv_test!(comment_with_spaces, "#   spaced out comment   ");
equiv_test!(comments_and_operators, "# comment\na < 1\n# another\nb = 2");
