use crate::achievement_scanner::Achievement;
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
pub fn resolve_key_scope(key: &str, achievements: &DashMap<String, Achievement>) -> Scope {
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
}
