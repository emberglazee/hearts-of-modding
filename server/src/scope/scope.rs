use crate::data::interner::InternedStr;
use crate::data::layered_value::LayeredValue;
use crate::scanner::achievement_scanner::Achievement;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Global,
    Country,
    State,
    Unit,
    Character,
    MusicStation,
    MusicTrack,
    Achievement,
    Ribbon,
    Idea,
    /// The `hidden_ideas` category inside `ideas = { }`. Works like
    /// `country` as a category keyword, except ideas defined within it
    /// don't show up in the spirit container and don't need a `picture`.
    HiddenIdeaCategory,
    /// Strategic region (air zone / naval region)
    StrategicRegion,
    Unknown,
}

impl Scope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Global => "Global",
            Scope::Country => "Country",
            Scope::State => "State",
            Scope::Unit => "Unit",
            Scope::Character => "Character",
            Scope::MusicStation => "Music Station",
            Scope::MusicTrack => "Music Track",
            Scope::Achievement => "Achievement",
            Scope::Ribbon => "Ribbon",
            Scope::Idea => "Idea",
            Scope::HiddenIdeaCategory => "Hidden Idea Category",
            Scope::StrategicRegion => "Strategic Region",
            Scope::Unknown => "Unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        let s_lower = s.to_ascii_lowercase();
        match s_lower.as_str() {
            "music_station" => Scope::MusicStation,
            "music" => Scope::MusicTrack,
            "state" => Scope::State,
            "strategic_region" => Scope::StrategicRegion,
            "ideas" => Scope::Idea,
            "hidden_ideas" => Scope::HiddenIdeaCategory,
            "country"
            | "ger"
            | "eng"
            | "fra"
            | "ita"
            | "jap"
            | "sov"
            | "usa"
            | "focus_tree"
            | "focus"
            | "shared_focus"
            | "joint_focus"
            | "continuous_focus_palette"
            | "completion_reward"
            | "completion_reward_joint_originator"
            | "completion_reward_joint_member"
            | "select_effect"
            | "bypass_effect"
            | "cancel_effect"
            | "complete_tooltip"
            | "ai_will_do"
            | "available"
            | "available_if_capitulated"
            | "bypass"
            | "bypass_if_unavailable"
            | "allow_branch"
            | "will_lead_to_war_with"
            | "historical_ai"
            | "joint_trigger"
            | "supports_ai_strategy"
            | "cancel_if_invalid"
            | "continue_if_invalid"
            | "allowed"
            | "enable"
            | "daily_cost"
            | "on_start"
            | "immediate"
            | "option"
            | "after"
            | "country_event"
            | "on_action"
            | "modifier"
            | "trigger"
            | "limit"
            | "chance"
            | "any_country"
            | "every_country"
            | "random_country"
            | "any_neighbor_country"
            | "any_allied_country"
            | "any_war_adversary"
            | "any_war_ally"
            | "any_guaranteed_country" => Scope::Country,
            "any_state"
            | "every_state"
            | "random_state"
            | "any_neighbor_state"
            | "any_home_state"
            | "any_owned_state"
            | "any_controlled_state"
            | "any_core_state" => Scope::State,
            "unit" | "any_unit" | "every_unit" | "random_unit" => Scope::Unit,
            "character" | "any_character" | "every_character" | "random_character"
            | "any_unit_leader" | "any_army_leader" | "any_navy_leader" => Scope::Character,
            _ => {
                // HOI4 tags: 3 chars, first uppercase alphabetic, rest uppercase alphanumeric.
                // Reserved words (NOT, AND, TAG, OOB, LOG, NUM, RED) excluded.
                const RESERVED: [&str; 7] = ["NOT", "AND", "TAG", "OOB", "LOG", "NUM", "RED"];
                if s.len() == 3
                    && s.as_bytes()[0].is_ascii_alphabetic()
                    && s.as_bytes()[0].is_ascii_uppercase()
                    && s.as_bytes()[1].is_ascii_alphanumeric()
                    && s.as_bytes()[2].is_ascii_alphanumeric()
                    && !RESERVED.contains(&s)
                {
                    Scope::Country
                } else {
                    Scope::Unknown
                }
            }
        }
    }
}

/// A node on the scope stack with transparency metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeNode {
    pub scope_type: Scope,
    pub is_transparent: bool,
}

impl ScopeNode {
    pub fn new(scope_type: Scope) -> Self {
        Self {
            scope_type,
            is_transparent: false,
        }
    }
}

/// Resolve a key to its semantic scope, with achievement/ribbon overrides.
/// Use this instead of `Scope::from_str` directly when achievements data is available.
/// This ensures the achievement-override logic lives in one place.
pub fn resolve_key_scope(
    key: &str,
    achievements: &DashMap<InternedStr, LayeredValue<Achievement>>,
) -> Scope {
    if let Some(achievement) = achievements.get(key) {
        if achievement.is_ribbon {
            Scope::Ribbon
        } else {
            Scope::Achievement
        }
    } else {
        Scope::from_str(key)
    }
}

pub struct ScopeStack {
    nodes: Vec<ScopeNode>,
}

impl ScopeStack {
    pub fn new(initial: Scope) -> Self {
        Self {
            nodes: vec![ScopeNode::new(initial)],
        }
    }

    pub fn push(&mut self, scope: Scope) {
        self.nodes.push(ScopeNode::new(scope));
    }

    /// Push a scope with explicit transparency flag.
    pub fn push_with(&mut self, scope: Scope, is_transparent: bool) {
        self.nodes.push(ScopeNode {
            scope_type: scope,
            is_transparent,
        });
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.nodes.pop().map(|n| n.scope_type)
    }

    pub fn current(&self) -> Scope {
        self.nodes.last().map_or(Scope::Global, |n| n.scope_type)
    }

    /// Get the current node for transparency checking
    #[allow(dead_code)]
    pub fn current_node(&self) -> Option<&ScopeNode> {
        self.nodes.last()
    }

    /// Returns scopes as a Vec<Scope> for backward compatibility.
    pub fn stack(&self) -> Vec<Scope> {
        self.nodes.iter().map(|n| n.scope_type).collect()
    }

    /// Iterate over Scope values (not ScopeNode).
    pub fn iter(&self) -> impl Iterator<Item = &Scope> {
        self.nodes.iter().map(|n| &n.scope_type)
    }

    /// Get all scopes as a slice of nodes
    #[allow(dead_code)]
    pub fn nodes(&self) -> &[ScopeNode] {
        &self.nodes
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Resolve ROOT/THIS/PREV pointers, using transparency-aware semantics.
    /// - ROOT: First non-transparent scope in the stack
    /// - THIS: Current (top) scope
    /// - PREV: Previous non-transparent scope before the current one
    pub fn resolve_pointer(&self, pointer: &str) -> Scope {
        match pointer.to_ascii_uppercase().as_str() {
            "ROOT" => self
                .nodes
                .iter()
                .skip(1)
                .find(|n| !n.is_transparent)
                .map_or(Scope::Global, |n| n.scope_type),
            "THIS" => self.current(),
            "PREV" => self
                .nodes
                .iter()
                .rev()
                .skip(1)
                .find(|n| !n.is_transparent)
                .map_or(Scope::Global, |n| n.scope_type),
            _ => Scope::Unknown,
        }
    }

    /// Resolve a dot-notation scope chain like `ROOT.owner.capital.controller`.
    ///
    /// Walk the chain using V2 chain target data:
    /// 1. Split on `.`
    /// 2. First segment: resolve via pointer or from_str
    /// 3. Subsequent segments: look up chain_target(current, segment) in V2 data
    /// 4. Returns `(final_scope, is_known)`
    #[allow(dead_code)]
    pub fn resolve_chain(&self, key: &str) -> (Scope, bool) {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.is_empty() {
            return (Scope::Unknown, false);
        }

        // Resolve the first segment
        let first = parts[0];
        let (mut current, mut known) = match first.to_ascii_uppercase().as_str() {
            "ROOT" => (self.resolve_pointer("ROOT"), true),
            "THIS" => (self.resolve_pointer("THIS"), true),
            "PREV" => (self.resolve_pointer("PREV"), true),
            "FROM" => (Scope::Country, true),
            _ => {
                let s = Scope::from_str(first);
                let known = s != Scope::Unknown;
                (s, known)
            }
        };

        // Walk subsequent segments via chain targets
        for segment in &parts[1..] {
            if let Some(target) = crate::data::hoi4_data::lookup_chain_target(&current, segment) {
                current = target.scope;
            } else {
                // Unknown link — try from_str as fallback
                let s = Scope::from_str(segment);
                if s != Scope::Unknown {
                    current = s;
                } else if segment.len() == 3
                    && segment.is_ascii()
                    && segment.as_bytes()[0].is_ascii_alphabetic()
                    && segment.as_bytes()[0].is_ascii_uppercase()
                {
                    // Country tag
                    current = Scope::Country;
                } else {
                    known = false;
                    break;
                }
            }
        }

        (current, known)
    }

    /// Resolve a meta-scope reference dynamically based on the current
    /// scope stack context.
    ///
    /// HOI4 defines several meta-scopes that refer to contextual scopes
    /// rather than named ones:
    ///
    /// | Keyword | Resolves to |
    /// |---------|------------|
    /// | `THIS`  | Current scope (top of stack) |
    /// | `ROOT`  | First non-transparent scope pushed (the entry point of the block) |
    /// | `PREV`  | Previous non-transparent scope (one level up) |
    /// | `PREVPREV` | Two non-transparent scopes up |
    /// | `FROM`  | Event/action source scope — typically `Country` |
    /// | `FROM.FROM` | Chained FROM — typically `Country` |
    ///
    /// Returns `None` when the key is not a meta-scope, so callers can
    /// fall back to [`Scope::from_str`] or [`resolve_key_scope`].
    pub fn resolve_meta_scope(&self, key: &str) -> Option<Scope> {
        let upper = key.to_ascii_uppercase();
        match upper.as_str() {
            // THIS = current scope (top of stack).
            // Always succeeds because the stack is never empty
            // (it always has at least Global).
            "THIS" => Some(self.current()),

            // ROOT = the first non-transparent scope pushed after Global.
            // In HOI4 this is usually Country (events, focuses, decisions),
            // but can be State (state events) or Character (character events).
            "ROOT" => Some(self.resolve_pointer("ROOT")),

            // PREV = previous non-transparent scope (one above current).
            // PREVPREV, PREVPREVPREV, etc. = N non-transparent scopes up.
            // We handle any string made of consecutive "PREV" parts.
            "PREV" | "PREVPREV" | "PREVPREVPREV" | "PREVPREVPREVPREV" => {
                let depth = upper.matches("PREV").count();
                let mut count = 0;
                for node in self.nodes.iter().rev().skip(1) {
                    if !node.is_transparent {
                        count += 1;
                        if count == depth {
                            return Some(node.scope_type);
                        }
                    }
                }
                Some(Scope::Unknown)
            }

            // FROM = source scope in events / targeted effects.
            // Cannot be determined statically without tracking which
            // event/effect fired this block. Default to Country since
            // most senders are countries.
            "FROM" => Some(Scope::Country),

            // Chained FROM references: FROM.FROM, FROM.FROM.FROM.
            // Also default to Country.
            _ if upper.starts_with("FROM.") => {
                let count = upper.matches("FROM").count();
                if count > 3 || count == 0 {
                    Some(Scope::Unknown)
                } else {
                    Some(Scope::Country)
                }
            }

            _ => None,
        }
    }

    /// Resolve a key to its semantic scope, trying meta-scope resolution
    /// first, then falling back to achievement-aware resolution, then
    /// static [`Scope::from_str`].
    ///
    /// This is the preferred single-call API when both a `ScopeStack`
    /// and an achievements map are available.
    pub fn resolve_scope_key(
        &self,
        key: &str,
        achievements: &DashMap<InternedStr, LayeredValue<Achievement>>,
    ) -> Scope {
        self.resolve_meta_scope(key)
            .unwrap_or_else(|| resolve_key_scope(key, achievements))
    }
}
