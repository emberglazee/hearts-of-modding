use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValueType {
    Bool,
    Int,
    Float,
    String,
    Block,
    Scope,
    Variable,
    Anything,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub key: String,
    pub value_type: ValueType,
    pub description: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub triggers: HashMap<String, Rule>,
    pub effects: HashMap<String, Rule>,
}

impl Schema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_cwt(&mut self, content: &str, is_trigger: bool) {
        let mut current_description = None;
        let mut current_scopes = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("###") {
                current_description = Some(line[3..].trim().to_string());
                continue;
            }

            if line.starts_with("## scope") {
                if let Some(eq) = line.find('=') {
                    let scopes_part = &line[eq + 1..].trim();
                    if scopes_part.starts_with('{') {
                        let inner = scopes_part.trim_matches('{').trim_matches('}');
                        current_scopes = inner.split_whitespace().map(|s| s.to_string()).collect();
                    } else {
                        current_scopes = vec![scopes_part.to_string()];
                    }
                }
                continue;
            }

            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            // alias[trigger:tag] = bool
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    let alias_type = &line[start + 1..end];
                    if let Some(colon) = alias_type.find(':') {
                        let kind = &alias_type[..colon];
                        let key = &alias_type[colon + 1..];

                        if (is_trigger && kind == "trigger") || (!is_trigger && kind == "effect") {
                            let value_part = if line.contains("==") {
                                line.split("==").nth(1).unwrap_or("").trim()
                            } else {
                                line.split('=').nth(1).unwrap_or("").trim()
                            };

                            let value_type = match value_part {
                                "replace_me_bool" | "bool" => ValueType::Bool,
                                "replace_me_comparison" | "int" => ValueType::Int,
                                "float" => ValueType::Float,
                                "replace_me_character" | "replace_me_country_tag" | "replace_me_country_scope" | "string" => ValueType::String,
                                "replace_me" => ValueType::Anything,
                                _ => ValueType::Anything,
                            };

                            let rule = Rule {
                                key: key.to_string(),
                                value_type,
                                description: current_description.take(),
                                scopes: current_scopes.clone(),
                            };

                            if is_trigger {
                                self.triggers.insert(key.to_string(), rule);
                            } else {
                                self.effects.insert(key.to_string(), rule);
                            }
                        }
                    }
                }
            }
            current_description = None;
            current_scopes.clear();
        }
    }
}
