use crate::ast;
use crate::loc_parser;
use std::collections::HashMap;

/// Represents a single modifier with its value
#[derive(Debug, Clone)]
pub struct ModifierEntry {
    pub key: String,
    pub value: f64,
    pub is_positive: bool,
}

/// Represents different types of modifier blocks
#[derive(Debug, Clone)]
pub enum ModifierBlock {
    /// Simple leaf modifiers: key = value
    Simple(Vec<ModifierEntry>),
    /// Targeted modifier: targeted_modifier = { tag = GER ... }
    Targeted {
        target: String,
        modifiers: Vec<ModifierEntry>,
    },
    /// Equipment bonus: equipment_bonus = { infantry_equipment = { ... } }
    EquipmentBonus {
        equipment_type: String,
        modifiers: Vec<ModifierEntry>,
    },
    /// Hidden modifier: hidden_modifier = { ... }
    Hidden(Vec<ModifierEntry>),
    /// Custom tooltip
    CustomTooltip(String),
}

/// Service for formatting and displaying modifiers
pub struct ModifierDisplayService {
    /// Maps modifier keys to localization keys
    pub mappings: HashMap<String, String>,
    /// Maps localization keys to format strings
    pub formats: HashMap<String, String>,
    /// Localization data
    pub localization: HashMap<String, loc_parser::LocEntry>,
}

impl ModifierDisplayService {
    pub fn new(
        mappings: HashMap<String, String>,
        formats: HashMap<String, String>,
        localization: HashMap<String, loc_parser::LocEntry>,
    ) -> Self {
        Self {
            mappings,
            formats,
            localization,
        }
    }

    /// Extract all modifier blocks from an AST value
    pub fn extract_modifier_blocks(&self, value: &ast::Value) -> Vec<ModifierBlock> {
        let mut blocks = Vec::new();

        if let ast::Value::Block(entries) = value {
            for entry in entries {
                if let ast::Entry::Assignment(ass) = entry {
                    let key_lower = ass.key.to_ascii_lowercase();

                    // Check for special modifier types
                    if key_lower == "targeted_modifier" {
                        if let Some(block) = self.extract_targeted_modifier(&ass.value.value) {
                            blocks.push(block);
                        }
                    } else if key_lower == "equipment_bonus" {
                        if let Some(block) = self.extract_equipment_bonus(&ass.value.value) {
                            blocks.push(block);
                        }
                    } else if key_lower == "hidden_modifier" {
                        if let Some(block) = self.extract_hidden_modifier(&ass.value.value) {
                            blocks.push(block);
                        }
                    } else if key_lower == "custom_effect_tooltip"
                        || key_lower == "custom_modifier_tooltip"
                    {
                        if let ast::Value::String(text) = &ass.value.value {
                            blocks.push(ModifierBlock::CustomTooltip(text.clone()));
                        }
                    } else {
                        // Try to parse as a simple modifier
                        if let Some(val) = self.parse_numeric_value(&ass.value.value) {
                            blocks.push(ModifierBlock::Simple(vec![ModifierEntry {
                                key: ass.key.clone(),
                                value: val,
                                is_positive: self.is_positive_modifier(&ass.key, val),
                            }]));
                        }
                    }
                }
            }
        }

        blocks
    }

    /// Extract targeted_modifier block
    fn extract_targeted_modifier(&self, value: &ast::Value) -> Option<ModifierBlock> {
        if let ast::Value::Block(entries) = value {
            let mut target = String::new();
            let mut modifiers = Vec::new();

            for entry in entries {
                if let ast::Entry::Assignment(ass) = entry {
                    let key_lower = ass.key.to_ascii_lowercase();
                    if key_lower == "tag" {
                        if let ast::Value::String(tag) = &ass.value.value {
                            target = tag.clone();
                        }
                    } else if let Some(val) = self.parse_numeric_value(&ass.value.value) {
                        modifiers.push(ModifierEntry {
                            key: ass.key.clone(),
                            value: val,
                            is_positive: self.is_positive_modifier(&ass.key, val),
                        });
                    }
                }
            }

            if !target.is_empty() && !modifiers.is_empty() {
                return Some(ModifierBlock::Targeted { target, modifiers });
            }
        }
        None
    }

    /// Extract equipment_bonus block
    fn extract_equipment_bonus(&self, value: &ast::Value) -> Option<ModifierBlock> {
        if let ast::Value::Block(entries) = value {
            for entry in entries {
                if let ast::Entry::Assignment(ass) = entry {
                    let equipment_type = ass.key.clone();
                    if let ast::Value::Block(mod_entries) = &ass.value.value {
                        let mut modifiers = Vec::new();
                        for mod_entry in mod_entries {
                            if let ast::Entry::Assignment(mod_ass) = mod_entry {
                                if let Some(val) = self.parse_numeric_value(&mod_ass.value.value) {
                                    modifiers.push(ModifierEntry {
                                        key: mod_ass.key.clone(),
                                        value: val,
                                        is_positive: self.is_positive_modifier(&mod_ass.key, val),
                                    });
                                }
                            }
                        }
                        if !modifiers.is_empty() {
                            return Some(ModifierBlock::EquipmentBonus {
                                equipment_type,
                                modifiers,
                            });
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract hidden_modifier block
    fn extract_hidden_modifier(&self, value: &ast::Value) -> Option<ModifierBlock> {
        if let ast::Value::Block(entries) = value {
            let mut modifiers = Vec::new();
            for entry in entries {
                if let ast::Entry::Assignment(ass) = entry {
                    if let Some(val) = self.parse_numeric_value(&ass.value.value) {
                        modifiers.push(ModifierEntry {
                            key: ass.key.clone(),
                            value: val,
                            is_positive: self.is_positive_modifier(&ass.key, val),
                        });
                    }
                }
            }
            if !modifiers.is_empty() {
                return Some(ModifierBlock::Hidden(modifiers));
            }
        }
        None
    }

    /// Parse a numeric value from AST
    fn parse_numeric_value(&self, value: &ast::Value) -> Option<f64> {
        match value {
            ast::Value::Number(n) => Some(*n),
            ast::Value::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Determine if a modifier is positive (beneficial)
    fn is_positive_modifier(&self, key: &str, value: f64) -> bool {
        let key_lower = key.to_ascii_lowercase();

        // Negative modifiers (bad things)
        let negative_keywords = [
            "cost",
            "attrition",
            "damage",
            "loss",
            "penalty",
            "consumption",
            "tension",
            "threat",
            "resistance",
            "surrender",
            "casualties",
        ];

        for keyword in &negative_keywords {
            if key_lower.contains(keyword) {
                // For negative modifiers, negative values are good
                return value < 0.0;
            }
        }

        // For most modifiers, positive values are good
        value > 0.0
    }

    /// Format a single modifier entry
    pub fn format_modifier(&self, entry: &ModifierEntry) -> String {
        let loc_key = self.mappings.get(&entry.key);
        let loc_text = if let Some(key) = loc_key {
            self.localization
                .get(key)
                .map(|e| e.value.clone())
                .unwrap_or_else(|| key.clone())
        } else {
            // Fallback: convert snake_case to Title Case
            self.humanize_key(&entry.key)
        };

        let formatted_value = self.format_value(&entry.key, entry.value, loc_key);

        // Use emoji indicators for positive/negative
        let indicator = if entry.is_positive { "✓" } else { "✗" };

        format!("  {} **{}**: {}", indicator, loc_text, formatted_value)
    }

    /// Format a modifier value with proper sign, percentage, and decimal places
    fn format_value(&self, key: &str, value: f64, loc_key: Option<&String>) -> String {
        let mut is_percentage = key.ends_with("_factor");
        let mut display_digits = 1;
        let mut is_double_percent = false;

        // Check format string if available
        if let Some(lk) = loc_key {
            if let Some(fmt) = self.formats.get(lk) {
                if fmt.contains("%%") {
                    is_double_percent = true;
                    is_percentage = false;
                } else {
                    is_percentage = fmt.contains('%');
                }

                // Extract decimal places from format string
                for c in fmt.chars().rev() {
                    if c.is_ascii_digit() {
                        display_digits = c.to_digit(10).unwrap() as usize;
                        break;
                    }
                }
            }
        }

        let mut actual_val = value;
        if is_percentage && !is_double_percent {
            actual_val *= 100.0;
        }

        let sign = if actual_val >= 0.0 { "+" } else { "" };
        let mut formatted_num = format!("{}{:.*}", sign, display_digits, actual_val);

        if is_percentage || is_double_percent {
            formatted_num.push('%');
        }

        formatted_num
    }

    /// Convert snake_case to Title Case
    fn humanize_key(&self, key: &str) -> String {
        key.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format a complete modifier block for display
    pub fn format_modifier_block(&self, block: &ModifierBlock) -> String {
        match block {
            ModifierBlock::Simple(modifiers) => {
                let mut lines = vec!["**Modifier Effects:**".to_string()];
                for modifier in modifiers {
                    lines.push(self.format_modifier(modifier));
                }
                lines.join("\n")
            }
            ModifierBlock::Targeted { target, modifiers } => {
                let mut lines = vec![format!("**Against {}:**", target)];
                for modifier in modifiers {
                    lines.push(self.format_modifier(modifier));
                }
                lines.join("\n")
            }
            ModifierBlock::EquipmentBonus {
                equipment_type,
                modifiers,
            } => {
                let equipment_name = self.humanize_key(equipment_type);
                let mut lines = vec![format!("**{} Bonus:**", equipment_name)];
                for modifier in modifiers {
                    lines.push(self.format_modifier(modifier));
                }
                lines.join("\n")
            }
            ModifierBlock::Hidden(modifiers) => {
                let mut lines = vec!["**Hidden Modifier:**".to_string()];
                for modifier in modifiers {
                    lines.push(self.format_modifier(modifier));
                }
                lines.join("\n")
            }
            ModifierBlock::CustomTooltip(text) => {
                format!("**Custom Tooltip:**\n{}", text)
            }
        }
    }

    /// Format multiple modifier blocks
    pub fn format_all_blocks(&self, blocks: &[ModifierBlock]) -> String {
        if blocks.is_empty() {
            return String::new();
        }

        blocks
            .iter()
            .map(|block| self.format_modifier_block(block))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_key() {
        let service = ModifierDisplayService::new(HashMap::new(), HashMap::new(), HashMap::new());

        assert_eq!(service.humanize_key("stability_factor"), "Stability Factor");
        assert_eq!(
            service.humanize_key("political_power_gain"),
            "Political Power Gain"
        );
    }

    #[test]
    fn test_is_positive_modifier() {
        let service = ModifierDisplayService::new(HashMap::new(), HashMap::new(), HashMap::new());

        // Positive modifiers
        assert!(service.is_positive_modifier("stability_factor", 0.1));
        assert!(!service.is_positive_modifier("stability_factor", -0.1));

        // Negative modifiers (cost is bad)
        assert!(service.is_positive_modifier("political_power_cost", -0.1));
        assert!(!service.is_positive_modifier("political_power_cost", 0.1));
    }

    #[test]
    fn test_format_value() {
        let service = ModifierDisplayService::new(HashMap::new(), HashMap::new(), HashMap::new());

        // Percentage modifier
        assert_eq!(
            service.format_value("stability_factor", 0.05, None),
            "+5.0%"
        );
        assert_eq!(
            service.format_value("stability_factor", -0.05, None),
            "-5.0%"
        );

        // Non-percentage modifier
        assert_eq!(
            service.format_value("political_power_gain", 0.5, None),
            "+0.5"
        );
    }
}
