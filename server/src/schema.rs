use crate::ast::DiagnosticSeverity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValueType {
    Bool,
    Int,
    Float,
    String,
    Block,
    Scope(String),
    Variable(String),
    Enum(String),
    Type(String),
    Value(String), // For value[country_flag]
    Alias(String), // For alias_name[trigger]
    Anything,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cardinality {
    pub min: u32,
    pub max: Option<u32>, // None means infinity
}

impl Default for Cardinality {
    fn default() -> Self {
        Self { min: 0, max: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub key: String,
    pub value_type: ValueType,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub push_scope: Option<String>,
    pub cardinality: Cardinality,
    pub severity: Option<DiagnosticSeverity>,
    pub children: Vec<Rule>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub triggers: HashMap<String, Rule>,
    pub effects: HashMap<String, Rule>,
    pub links: HashMap<String, Rule>,
    pub enums: HashMap<String, HashSet<String>>,
    pub types: HashMap<String, Rule>, // Keyed by type name
}

impl Schema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_links(&mut self, script: &crate::ast::Script) {
        for entry in &script.entries {
            if let crate::ast::Entry::Assignment(ass) = entry {
                if ass.key == "links" {
                    if let crate::ast::Value::Block(links_block) = &ass.value.value {
                        for link_entry in links_block {
                            if let crate::ast::Entry::Assignment(link_ass) = link_entry {
                                let key = link_ass.key.clone();
                                let mut output_scope = None;
                                let mut input_scopes = Vec::new();

                                if let crate::ast::Value::Block(link_props) = &link_ass.value.value
                                {
                                    for prop in link_props {
                                        if let crate::ast::Entry::Assignment(prop_ass) = prop {
                                            if prop_ass.key == "output_scope" {
                                                if let crate::ast::Value::String(s) =
                                                    &prop_ass.value.value
                                                {
                                                    output_scope = Some(s.clone());
                                                }
                                            } else if prop_ass.key == "input_scopes" {
                                                match &prop_ass.value.value {
                                                    crate::ast::Value::String(s) => {
                                                        input_scopes.push(s.clone());
                                                    }
                                                    crate::ast::Value::Block(items)
                                                    | crate::ast::Value::TaggedBlock(_, items, _) => {
                                                        for item in items {
                                                            if let crate::ast::Entry::Value(v) =
                                                                item
                                                            {
                                                                if let crate::ast::Value::String(
                                                                    s,
                                                                ) = &v.value
                                                                {
                                                                    input_scopes.push(s.clone());
                                                                }
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }

                                let rule = Rule {
                                    key: key.clone(),
                                    value_type: ValueType::Block,
                                    description: None,
                                    scopes: input_scopes,
                                    push_scope: output_scope,
                                    cardinality: Cardinality::default(),
                                    severity: None,
                                    children: Vec::new(),
                                };
                                self.links.insert(key, rule);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn parse_cwt_ast(&mut self, script: &crate::ast::Script, kind: Option<&str>) {
        let mut metadata_comments = Vec::new();

        for entry in &script.entries {
            match entry {
                crate::ast::Entry::Comment(c, _) => {
                    if c.trim().starts_with("##") || c.trim().starts_with("###") {
                        metadata_comments.push(c.clone());
                    }
                }
                crate::ast::Entry::Assignment(ass) => {
                    self.process_assignment(ass, &metadata_comments, kind);
                    metadata_comments.clear();
                }
                crate::ast::Entry::Value(_) => {
                    metadata_comments.clear();
                }
            }
        }
    }

    fn process_assignment(
        &mut self,
        ass: &crate::ast::Assignment,
        metadata: &[String],
        kind: Option<&str>,
    ) {
        let mut current_description = None;
        let mut current_scopes = Vec::new();
        let mut current_push_scope = None;
        let mut current_cardinality = Cardinality::default();
        let mut current_severity = None;

        for line in metadata {
            let line = line.trim();
            if line.starts_with("###") {
                current_description = Some(line[3..].trim().to_string());
            } else if line.starts_with("## scope") {
                if let Some(eq) = line.find('=') {
                    let scopes_part = line[eq + 1..].trim();
                    if scopes_part.starts_with('{') {
                        let inner = scopes_part.trim_matches('{').trim_matches('}');
                        current_scopes = inner.split_whitespace().map(|s| s.to_string()).collect();
                    } else {
                        current_scopes = vec![scopes_part.to_string()];
                    }
                }
            } else if line.starts_with("## push_scope") {
                if let Some(eq) = line.find('=') {
                    current_push_scope = Some(line[eq + 1..].trim().to_string());
                }
            } else if line.starts_with("## cardinality") {
                if let Some(eq) = line.find('=') {
                    let card_part = line[eq + 1..].trim();
                    let parts: Vec<&str> = card_part.split("..").collect();
                    if parts.len() == 2 {
                        let min = parts[0].parse().unwrap_or(0);
                        let max = if parts[1] == "inf" {
                            None
                        } else {
                            parts[1].parse().ok()
                        };
                        current_cardinality = Cardinality { min, max };
                    }
                }
            } else if line.starts_with("## severity") {
                if let Some(eq) = line.find('=') {
                    match line[eq + 1..].trim().to_lowercase().as_str() {
                        "error" => current_severity = Some(DiagnosticSeverity::Error),
                        "warning" => current_severity = Some(DiagnosticSeverity::Warning),
                        "information" | "info" => {
                            current_severity = Some(DiagnosticSeverity::Information)
                        }
                        "hint" => current_severity = Some(DiagnosticSeverity::Hint),
                        _ => {}
                    }
                }
            }
        }

        // Special handling for enums = { ... }
        if ass.key == "enums" {
            if let crate::ast::Value::Block(entries) = &ass.value.value {
                for entry in entries {
                    if let crate::ast::Entry::Assignment(enum_ass) = entry {
                        if enum_ass.key.starts_with("enum[") && enum_ass.key.ends_with(']') {
                            let enum_name = enum_ass.key[5..enum_ass.key.len() - 1].to_string();
                            let mut values = HashSet::new();
                            if let crate::ast::Value::Block(val_entries) = &enum_ass.value.value {
                                for val_entry in val_entries {
                                    if let crate::ast::Entry::Value(v) = val_entry {
                                        if let crate::ast::Value::String(s) = &v.value {
                                            values.insert(s.clone());
                                        }
                                    }
                                }
                            }
                            self.enums.insert(enum_name, values);
                        }
                    }
                }
            }
            return;
        }

        // Handle alias[kind:key] = value
        if ass.key.starts_with("alias[") && ass.key.contains(':') && ass.key.ends_with(']') {
            let inner = &ass.key[6..ass.key.len() - 1];
            if let Some(colon) = inner.find(':') {
                let alias_kind = &inner[..colon];
                let key = &inner[colon + 1..];

                let rule = Rule {
                    key: key.to_string(),
                    value_type: self.parse_value_type(&ass.value.value),
                    description: current_description,
                    scopes: current_scopes,
                    push_scope: current_push_scope,
                    cardinality: current_cardinality,
                    severity: current_severity,
                    children: self.parse_children(&ass.value.value),
                };

                match alias_kind {
                    "trigger" => {
                        self.triggers.insert(key.to_string(), rule);
                    }
                    "effect" => {
                        self.effects.insert(key.to_string(), rule);
                    }
                    _ => {}
                }
            }
        } else if let Some(k) = kind {
            // General rule assignment (e.g. key = type) if kind is provided
            let rule = Rule {
                key: ass.key.clone(),
                value_type: self.parse_value_type(&ass.value.value),
                description: current_description,
                scopes: current_scopes,
                push_scope: current_push_scope,
                cardinality: current_cardinality,
                severity: current_severity,
                children: self.parse_children(&ass.value.value),
            };
            if k == "trigger" {
                self.triggers.insert(ass.key.clone(), rule);
            } else if k == "effect" {
                self.effects.insert(ass.key.clone(), rule);
            }
        }
    }

    fn parse_value_type(&self, val: &crate::ast::Value) -> ValueType {
        match val {
            crate::ast::Value::String(s) => {
                let s_lower = s.to_lowercase();
                if s_lower == "bool" {
                    ValueType::Bool
                } else if s_lower == "int" {
                    ValueType::Int
                } else if s_lower == "float" {
                    ValueType::Float
                } else if s_lower == "string" {
                    ValueType::String
                } else if s_lower.starts_with("enum[") && s_lower.ends_with(']') {
                    ValueType::Enum(s[5..s.len() - 1].to_string())
                } else if s_lower.starts_with("scope[") && s_lower.ends_with(']') {
                    ValueType::Scope(s[6..s.len() - 1].to_string())
                } else if s_lower.starts_with("type[") && s_lower.ends_with(']') {
                    ValueType::Type(s[5..s.len() - 1].to_string())
                } else if s_lower.starts_with('<') && s_lower.ends_with('>') {
                    ValueType::Type(s[1..s.len() - 1].to_string())
                } else if s_lower.starts_with("value[") && s_lower.ends_with(']') {
                    ValueType::Value(s[6..s.len() - 1].to_string())
                } else if s_lower.starts_with("alias_name[") && s_lower.ends_with(']') {
                    ValueType::Alias(s[11..s.len() - 1].to_string())
                } else {
                    ValueType::Anything
                }
            }
            crate::ast::Value::Block(_) => ValueType::Block,
            _ => ValueType::Anything,
        }
    }

    fn parse_children(&self, val: &crate::ast::Value) -> Vec<Rule> {
        let mut children = Vec::new();
        if let crate::ast::Value::Block(entries) = val {
            let mut metadata_comments = Vec::new();
            for entry in entries {
                match entry {
                    crate::ast::Entry::Comment(c, _) => {
                        if c.trim().starts_with("##") {
                            metadata_comments.push(c.clone());
                        }
                    }
                    crate::ast::Entry::Assignment(ass) => {
                        // For children, we don't have a kind (trigger/effect), it's just a key-value rule
                        let mut current_description = None;
                        let mut current_scopes = Vec::new();
                        let mut current_push_scope = None;
                        let mut current_cardinality = Cardinality::default();
                        let mut current_severity = None;

                        for line in &metadata_comments {
                            let line = line.trim();
                            if line.starts_with("###") {
                                current_description = Some(line[3..].trim().to_string());
                            } else if line.starts_with("## scope") {
                                if let Some(eq) = line.find('=') {
                                    let scopes_part = line[eq + 1..].trim();
                                    if scopes_part.starts_with('{') {
                                        let inner = scopes_part.trim_matches('{').trim_matches('}');
                                        current_scopes = inner
                                            .split_whitespace()
                                            .map(|s| s.to_string())
                                            .collect();
                                    } else {
                                        current_scopes = vec![scopes_part.to_string()];
                                    }
                                }
                            } else if line.starts_with("## push_scope") {
                                if let Some(eq) = line.find('=') {
                                    current_push_scope = Some(line[eq + 1..].trim().to_string());
                                }
                            } else if line.starts_with("## cardinality") {
                                if let Some(eq) = line.find('=') {
                                    let card_part = line[eq + 1..].trim();
                                    let parts: Vec<&str> = card_part.split("..").collect();
                                    if parts.len() == 2 {
                                        let min = parts[0].parse().unwrap_or(0);
                                        let max = if parts[1] == "inf" {
                                            None
                                        } else {
                                            parts[1].parse().ok()
                                        };
                                        current_cardinality = Cardinality { min, max };
                                    }
                                }
                            } else if line.starts_with("## severity") {
                                if let Some(eq) = line.find('=') {
                                    match line[eq + 1..].trim().to_lowercase().as_str() {
                                        "error" => {
                                            current_severity = Some(DiagnosticSeverity::Error)
                                        }
                                        "warning" => {
                                            current_severity = Some(DiagnosticSeverity::Warning)
                                        }
                                        "information" | "info" => {
                                            current_severity = Some(DiagnosticSeverity::Information)
                                        }
                                        "hint" => current_severity = Some(DiagnosticSeverity::Hint),
                                        _ => {}
                                    }
                                }
                            }
                        }

                        // Recurse or create rule
                        let rule = Rule {
                            key: ass.key.clone(),
                            value_type: self.parse_value_type(&ass.value.value),
                            description: current_description,
                            scopes: current_scopes,
                            push_scope: current_push_scope,
                            cardinality: current_cardinality,
                            severity: current_severity,
                            children: self.parse_children(&ass.value.value),
                        };
                        children.push(rule);
                        metadata_comments.clear();
                    }
                    _ => metadata_comments.clear(),
                }
            }
        }
        children
    }

    // Temporary old method for compatibility while refactoring
    pub fn parse_cwt(&mut self, content: &str, is_trigger: bool) {
        {
            let (script, _) = crate::parser::parse_script(content);
            self.parse_cwt_ast(&script, Some(if is_trigger { "trigger" } else { "effect" }));
        }
    }
}
