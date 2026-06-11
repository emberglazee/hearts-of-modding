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
            Scope::Unknown => "Unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        let s_lower = s.to_ascii_lowercase();
        match s_lower.as_str() {
            "music_station" => Scope::MusicStation,
            "music" => Scope::MusicTrack,
            "state" => Scope::State,
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
    stack: Vec<Scope>,
}

impl ScopeStack {
    pub fn new(initial: Scope) -> Self {
        Self {
            stack: vec![initial],
        }
    }

    pub fn push(&mut self, scope: Scope) {
        self.stack.push(scope);
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.stack.pop()
    }

    pub fn current(&self) -> Scope {
        *self.stack.last().unwrap_or(&Scope::Global)
    }

    pub fn stack(&self) -> &[Scope] {
        &self.stack
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Scope> {
        self.stack.iter()
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
    /// | `ROOT`  | First non-Global scope pushed (the entry point of the block) |
    /// | `PREV`  | Parent scope (one level up) |
    /// | `PREVPREV` | Grandparent scope (two levels up) |
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

            // ROOT = the first non-Global scope that was pushed.
            // In HOI4 this is usually Country (events, focuses, decisions),
            // but can be State (state events) or Character (character events).
            "ROOT" => Some(self.stack.get(1).copied().unwrap_or(Scope::Global)),

            // PREV = one level above current (parent scope).
            // PREVPREV, PREVPREVPREV, etc. = N levels up.
            // We handle any string made of consecutive "PREV" parts.
            "PREV" | "PREVPREV" | "PREVPREVPREV" | "PREVPREVPREVPREV" => {
                let depth = upper.matches("PREV").count();
                debug_assert!(depth >= 1, "PREV pattern matched with zero PREVs");
                if self.stack.len() > depth {
                    Some(self.stack[self.stack.len() - 1 - depth])
                } else {
                    Some(Scope::Unknown)
                }
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
