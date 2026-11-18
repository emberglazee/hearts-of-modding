//! Leader-role skill stats: a `*_skill` key that belongs to a DIFFERENT role.
//!
//! HOI4 has two leader stat vocabularies. Corps commanders and field marshals
//! take `planning_skill` and `logistics_skill`; naval commanders take
//! `maneuvering_skill` and `coordination_skill`. All three share `skill`,
//! `attack_skill` and `defense_skill`. The engine reads a role block's stats
//! positionally against that role's own schema, so writing an army stat on a
//! naval commander does not error — it silently drops the stat, and the leader
//! arrives with the default value instead of the one the script asked for.
//!
//! That silence is what this rule replaces. It fires only when the key is a
//! documented leader stat (so typos and unrelated parameters are out of scope),
//! the enclosing block is a leader role (derived from the parameter table, not
//! a hardcoded list), and the stat is not one this role documents. Both
//! directions are covered, and so are both ways a role is written: the
//! `add_*_role` / `create_*` effects and the `corps_commander` /
//! `field_marshal` / `navy_leader` blocks in `common/characters`.

use crate::data::hoi4_data;
use crate::parser::ast;
use crate::rules::{ParentContext, ValidationContext, ValidationRule};
use crate::scope::scope::ScopeStack;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString};

pub(crate) struct LeaderRoleSkillRule;

impl ValidationRule for LeaderRoleSkillRule {
    fn check_assignment(
        &self,
        ass: &ast::Assignment,
        ctx: &ValidationContext,
        _scope: &ScopeStack,
        _pushed_scope: bool,
        parent: ParentContext<'_>,
        diags: &mut Vec<Diagnostic>,
    ) {
        let key = ass.key_text(ctx.source);

        // Cheap gate first: only `*_skill` keys can be a misplaced stat.
        if !is_skill_key(key) {
            return;
        }

        let Some(block) = parent.key else {
            return;
        };
        let Some(schema) = hoi4_data::leader_role(block) else {
            return;
        };
        if schema.skills.iter().any(|s| s.eq_ignore_ascii_case(key)) {
            return;
        }
        if !hoi4_data::is_leader_skill(key) {
            return;
        }

        let owners = hoi4_data::leader_roles_accepting(key);
        let owners = owner_labels(&owners, schema.block);
        let mut message = format!("'{}' is not a parameter of {}. ", key, schema.block);
        if !owners.is_empty() {
            message.push_str(&format!("It belongs to {}. ", join(&owners)));
        }
        message.push_str(&format!("This block takes {}.", schema.skills.join(", ")));

        diags.push(Diagnostic {
            range: ctx.range(&ass.key_range),
            severity: Some(DiagnosticSeverity::WARNING),
            message,
            code: Some(NumberOrString::String(
                crate::validation::advanced_validation::WRONG_LEADER_ROLE_SKILL.to_string(),
            )),
            source: Some("Hearts of Modding".to_string()),
            ..Default::default()
        });
    }
}

/// `attack_skill`, `planning_skill`, ... — the last `_`-segment is `skill`.
/// Also true for the bare `skill` key, which every role documents and the
/// schema check above resolves before this matters.
fn is_skill_key(key: &str) -> bool {
    key.rsplit('_')
        .next()
        .is_some_and(|last| last.eq_ignore_ascii_case("skill"))
}

/// One label per role, not one per spelling of it.
///
/// `planning_skill` is accepted by six blocks — `corps_commander`,
/// `field_marshal`, `add_corps_commander_role`, `add_field_marshal_role`,
/// `create_corps_commander`, `create_field_marshal` — which are three roles
/// written two ways. Naming all six in the diagnostic is noise; the bare
/// definition form is the name a modder recognises, so that one wins.
fn owner_labels(owners: &[&'static str], current: &str) -> Vec<&'static str> {
    let current_family = family(current);
    let mut picked: Vec<(&'static str, &'static str)> = Vec::new();
    for owner in owners {
        let owner_family = family(owner);
        if owner_family == current_family {
            continue;
        }
        match picked.iter_mut().find(|(_, f)| *f == owner_family) {
            Some((label, _)) => {
                if preferred(owner, label) {
                    *label = owner;
                }
            }
            None => picked.push((owner, owner_family)),
        }
    }
    picked.sort_unstable();
    picked.into_iter().map(|(label, _)| label).collect()
}

/// `add_corps_commander_role` and `create_corps_commander` are the same role as
/// `corps_commander`, which is also the shortest and the most readable.
fn family(block: &str) -> &str {
    let stripped = block
        .strip_prefix("add_")
        .or_else(|| block.strip_prefix("create_"))
        .unwrap_or(block);
    stripped.strip_suffix("_role").unwrap_or(stripped)
}

/// A bare definition name beats an effect spelling; between two effect
/// spellings the shorter one reads better.
fn preferred(candidate: &str, current: &str) -> bool {
    let is_effect = |s: &str| s.starts_with("add_") || s.starts_with("create_");
    match (is_effect(candidate), is_effect(current)) {
        (false, true) => true,
        (true, false) => false,
        _ => candidate.len() < current.len(),
    }
}

/// `a`, `a and b`, `a, b and c`.
fn join(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_string(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::{family, join, owner_labels, preferred};

    #[test]
    fn join_reads_as_a_list() {
        assert_eq!(join(&[]), "");
        assert_eq!(join(&["a"]), "a");
        assert_eq!(join(&["a", "b"]), "a and b");
        assert_eq!(join(&["a", "b", "c"]), "a, b and c");
    }

    #[test]
    fn effect_spellings_collapse_onto_the_definition_name() {
        assert_eq!(family("add_corps_commander_role"), "corps_commander");
        assert_eq!(family("create_field_marshal"), "field_marshal");
        assert_eq!(family("navy_leader"), "navy_leader");
        assert!(preferred("corps_commander", "add_corps_commander_role"));
        assert!(!preferred("add_corps_commander_role", "corps_commander"));
    }

    #[test]
    fn owners_collapse_to_one_label_per_role() {
        let owners = [
            "add_corps_commander_role",
            "add_field_marshal_role",
            "corps_commander",
            "create_corps_commander",
            "create_field_marshal",
            "field_marshal",
        ];
        assert_eq!(
            owner_labels(&owners, "add_naval_commander_role"),
            vec!["corps_commander", "field_marshal"]
        );
        // The block's own role is never named back at it.
        assert!(
            owner_labels(&owners, "add_corps_commander_role")
                .iter()
                .all(|l| *l != "corps_commander")
        );
    }
}
