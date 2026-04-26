use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Global,
    Country,
    State,
    Unit,
    Character,
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
            Scope::Unknown => "Unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "country" | "ger" | "eng" | "fra" | "ita" | "jap" | "sov" | "usa" => Scope::Country,
            "state" => Scope::State,
            "unit" => Scope::Unit,
            "character" => Scope::Character,
            _ => Scope::Unknown,
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