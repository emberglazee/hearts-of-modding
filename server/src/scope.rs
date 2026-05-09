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
            Scope::Unknown => "Unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "music_station" => Scope::MusicStation,
            "music" => Scope::MusicTrack,
            "state" => Scope::State,
            "country" | "ger" | "eng" | "fra" | "ita" | "jap" | "sov" | "usa" |
            "focus_tree" | "focus" | "shared_focus" | "completion_reward" | "select_effect" | 
            "ai_will_do" | "available" | "bypass" | "allow_branch" | "will_lead_to_war_with" | 
            "on_start" | "immediate" | "option" | "after" | "country_event" | "on_action" |
            "modifier" | "trigger" | "limit" | "chance" |
            "any_country" | "every_country" | "random_country" | "any_neighbor_country" | 
            "any_allied_country" | "any_war_adversary" | "any_war_ally" | "any_guaranteed_country" => Scope::Country,
            "any_state" | "every_state" | "random_state" | "any_neighbor_state" | 
            "any_home_state" | "any_owned_state" | "any_controlled_state" | "any_core_state" => Scope::State,
            "unit" | "any_unit" | "every_unit" | "random_unit" => Scope::Unit,
            "character" | "any_character" | "every_character" | "random_character" |
            "any_unit_leader" | "any_army_leader" | "any_navy_leader" => Scope::Character,
            _ => {
                if s.len() == 3 && s.chars().all(|c| c.is_ascii_alphabetic()) {
                    Scope::Country
                } else {
                    Scope::Unknown
                }
            }
        }
    }
}

pub struct ScopeStack {
    stack: Vec<Scope>,
}

impl ScopeStack {
    pub fn new(initial: Scope) -> Self {
        Self { stack: vec![initial] }
    }

    pub fn push(&mut self, scope: Scope) {
        self.stack.push(scope);
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.stack.pop()
    }

    #[allow(dead_code)]
    pub fn current(&self) -> Scope {
        *self.stack.last().unwrap_or(&Scope::Global)
    }

    #[allow(dead_code)]
    pub fn stack(&self) -> &[Scope] {
        &self.stack
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Scope> {
        self.stack.iter()
    }
}