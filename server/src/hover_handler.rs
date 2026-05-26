use crate::Backend;
use crate::ast;
use crate::loc_parser;
use crate::loc_preview::paradox_to_markdown;
use crate::lsp_convert::ast_range_to_lsp;
use crate::modifier_display;
use crate::modifier_format::format_modifier_value;
use crate::parser;
use crate::scope;
use crate::symbol_search::find_identifier_at;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

impl Backend {
    pub(crate) async fn handle_hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;
        let map_config = crate::map_config::get_map_config(std::path::Path::new("."));

        if let Some(content) = self.documents.get(&uri) {
            if uri.ends_with(".yml") {
                let (locs, _, _) = loc_parser::parse_loc_file(&content, &uri);
                let global_loc = self.scanner_data.localization();
                for entry in locs.values() {
                    // Check key
                    if position.line == entry.range.start_line
                        && position.character >= entry.range.start_col
                        && position.character <= entry.range.end_col
                    {
                        let mut hover_text = format!("### 🌐 Localization: {}\n\n", entry.key);

                        // Add achievement context
                        let achievements = self.scanner_data.achievements();
                        if entry.key.ends_with("_NAME") {
                            let ach_id = &entry.key[..entry.key.len() - 5];
                            if let Some(ach) = achievements.get(ach_id) {
                                hover_text.push_str(&format!(
                                    "**Context:** Name for {} `{}`\n\n",
                                    if ach.is_ribbon {
                                        "Ribbon"
                                    } else {
                                        "Achievement"
                                    },
                                    ach_id
                                ));
                                hover_text.push_str(&format!(
                                    "Defined in: {}\n\n---\n\n",
                                    self.make_file_link(&ach.path)
                                ));
                            }
                        } else if entry.key.ends_with("_DESC") {
                            let ach_id = &entry.key[..entry.key.len() - 5];
                            if let Some(ach) = achievements.get(ach_id) {
                                hover_text.push_str(&format!(
                                    "**Context:** Description for {} `{}`\n\n",
                                    if ach.is_ribbon {
                                        "Ribbon"
                                    } else {
                                        "Achievement"
                                    },
                                    ach_id
                                ));
                                hover_text.push_str(&format!(
                                    "Defined in: {}\n\n---\n\n",
                                    self.make_file_link(&ach.path)
                                ));
                            }
                        }

                        hover_text.push_str(&format!("**Raw:** `{}`\n\n", entry.value));
                        hover_text.push_str("**Preview:**\n\n");
                        hover_text.push_str(&paradox_to_markdown(&entry.value, Some(&global_loc)));

                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: hover_text,
                            }),
                            range: Some(ast_range_to_lsp(&entry.range)),
                        }));
                    }
                    // Check value
                    if position.line == entry.range.start_line
                        && position.character >= entry.value_start_col
                        && position.character <= entry.value_start_col + entry.value.len() as u32
                    {
                        let mut hover_text = "### 👁️ Localization Preview\n\n".to_string();
                        hover_text.push_str(&paradox_to_markdown(&entry.value, Some(&global_loc)));

                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: hover_text,
                            }),
                            range: Some(Range {
                                start: Position {
                                    line: entry.range.start_line,
                                    character: entry.value_start_col,
                                },
                                end: Position {
                                    line: entry.range.start_line,
                                    character: entry.value_start_col + entry.value.len() as u32,
                                },
                            }),
                        }));
                    }
                }
                return Ok(None);
            } else if uri.ends_with("/map/buildings.txt") || uri.ends_with("\\map\\buildings.txt") {
                if let Some(line) = content.lines().nth(position.line as usize) {
                    let mut hover_text = String::from("### 🏗️ Map Building Definition\n\n");
                    hover_text.push_str("`State ID (integer); building ID (string); X position; Y position; Z position; Rotation; Adjacent sea province (integer)`\n\n---\n\n");

                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 7 {
                        let mut current_col = 0;
                        let mut hovered_index = None;
                        for (i, part) in parts.iter().enumerate() {
                            let end_col = current_col + part.len() as u32;
                            if position.character >= current_col && position.character <= end_col {
                                hovered_index = Some(i);
                                break;
                            }
                            current_col = end_col + 1;
                        }

                        match hovered_index {
                            Some(0) => {
                                if let Ok(state_id) = parts[0].parse::<u32>() {
                                    let states = self.scanner_data.states();
                                    if let Some(state) = states.get(&state_id) {
                                        let loc = self.scanner_data.localization();
                                        let state_name =
                                            if let Some(loc_entry) = loc.get(&state.name) {
                                                loc_entry.value.clone()
                                            } else {
                                                state.name.clone()
                                            };
                                        hover_text.push_str(&format!(
                                            "**Hovered:** State ID `{}` (🗺️ {})\n",
                                            state_id, state_name
                                        ));
                                    } else {
                                        hover_text.push_str(&format!(
                                            "**Hovered:** State ID `{}`\n",
                                            state_id
                                        ));
                                    }
                                }
                            }
                            Some(1) => {
                                hover_text.push_str(&format!(
                                    "**Hovered:** Building ID `{}`\n",
                                    parts[1]
                                ));
                            }
                            Some(2) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** X Position `{}`\n", parts[2]));
                            }
                            Some(3) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Y Position `{}`\n", parts[3]));
                            }
                            Some(4) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Z Position `{}`\n", parts[4]));
                            }
                            Some(5) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Rotation `{}`\n", parts[5]));
                            }
                            Some(6) => {
                                if let Ok(prov_id) = parts[6].parse::<u32>() {
                                    let provs = self.scanner_data.provinces();
                                    if let Some(province) = provs.get(&prov_id) {
                                        hover_text.push_str(&format!("**Hovered:** Adjacent Sea Province `{}` (Coastal: {}, Terrain: {})\n", prov_id, province.is_coastal, province.terrain));
                                    } else {
                                        hover_text.push_str(&format!(
                                            "**Hovered:** Adjacent Sea Province `{}`\n",
                                            prov_id
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_text,
                        }),
                        range: None,
                    }));
                }
                return Ok(None);
            } else if uri.ends_with("/map/unitstacks.txt") || uri.ends_with("\\map\\unitstacks.txt")
            {
                if let Some(line) = content.lines().nth(position.line as usize) {
                    let mut hover_text = String::from("### 🪖 Unit Stack Definition\n\n");
                    hover_text.push_str("`Province ID (integer); Type (integer); X position; Y position; Z position; Rotation; Offset`\n\n---\n\n");

                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 7 {
                        let mut current_col = 0;
                        let mut hovered_index = None;
                        for (i, part) in parts.iter().enumerate() {
                            let end_col = current_col + part.len() as u32;
                            if position.character >= current_col && position.character <= end_col {
                                hovered_index = Some(i);
                                break;
                            }
                            current_col = end_col + 1;
                        }

                        match hovered_index {
                            Some(0) => {
                                if let Ok(prov_id) = parts[0].parse::<u32>() {
                                    let provs = self.scanner_data.provinces();
                                    if let Some(province) = provs.get(&prov_id) {
                                        hover_text.push_str(&format!("**Hovered:** Province ID `{}` (Coastal: {}, Terrain: {})\n", prov_id, province.is_coastal, province.terrain));
                                    } else {
                                        hover_text.push_str(&format!(
                                            "**Hovered:** Province ID `{}`\n",
                                            prov_id
                                        ));
                                    }
                                }
                            }
                            Some(1) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Stack Type `{}`\n", parts[1]));
                            }
                            Some(2) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** X Position `{}`\n", parts[2]));
                            }
                            Some(3) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Y Position `{}`\n", parts[3]));
                            }
                            Some(4) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Z Position `{}`\n", parts[4]));
                            }
                            Some(5) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Rotation `{}`\n", parts[5]));
                            }
                            Some(6) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Offset `{}`\n", parts[6]));
                            }
                            _ => {}
                        }
                    }

                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_text,
                        }),
                        range: None,
                    }));
                }
                return Ok(None);
            } else if uri.ends_with("/map/weatherpositions.txt")
                || uri.ends_with("\\map\\weatherpositions.txt")
            {
                if let Some(line) = content.lines().nth(position.line as usize) {
                    let mut hover_text = String::from("### ☁️ Weather Position Definition\n\n");
                    hover_text.push_str("`Strategic Region ID (integer); X position; Y position; Z position; Size (string: small or large)`\n\n---\n\n");

                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 5 {
                        let mut current_col = 0;
                        let mut hovered_index = None;
                        for (i, part) in parts.iter().enumerate() {
                            let end_col = current_col + part.len() as u32;
                            if position.character >= current_col && position.character <= end_col {
                                hovered_index = Some(i);
                                break;
                            }
                            current_col = end_col + 1;
                        }

                        match hovered_index {
                            Some(0) => {
                                if let Ok(region_id) = parts[0].parse::<u32>() {
                                    let regions = self.scanner_data.strategic_regions();
                                    if let Some(region) = regions.get(&region_id) {
                                        let loc = self.scanner_data.localization();
                                        let region_name =
                                            if let Some(loc_entry) = loc.get(&region.name) {
                                                loc_entry.value.clone()
                                            } else {
                                                region.name.clone()
                                            };
                                        hover_text.push_str(&format!(
                                            "**Hovered:** Strategic Region ID `{}` (🗺️ {})\n",
                                            region_id, region_name
                                        ));
                                    } else {
                                        hover_text.push_str(&format!(
                                            "**Hovered:** Strategic Region ID `{}`\n",
                                            region_id
                                        ));
                                    }
                                }
                            }
                            Some(1) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** X Position `{}`\n", parts[1]));
                            }
                            Some(2) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Y Position `{}`\n", parts[2]));
                            }
                            Some(3) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Z Position `{}`\n", parts[3]));
                            }
                            Some(4) => {
                                hover_text.push_str(&format!("**Hovered:** Size `{}`\n", parts[4]));
                            }
                            _ => {}
                        }
                    }

                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_text,
                        }),
                        range: None,
                    }));
                }
                return Ok(None);
            } else if uri.ends_with("/map/supply_nodes.txt")
                || uri.ends_with("\\map\\supply_nodes.txt")
            {
                let hover_text = String::from(
                    "### 📦 Supply Node Definition\n\n`Level (integer) Province ID (integer)`",
                );
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: None,
                }));
            } else if uri.ends_with("/map/railways.txt") || uri.ends_with("\\map\\railways.txt") {
                let hover_text = String::from(
                    "### 🚂 Railway Definition\n\n`Level (integer) Amount of provinces (integer) List of provinces (space-separated integers)`",
                );
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: None,
                }));
            } else if uri.ends_with(&map_config.adjacencies) {
                if let Some(line) = content.lines().nth(position.line as usize) {
                    let mut hover_text = String::from("### 🚢 Adjacency Definition\n\n");
                    hover_text.push_str("`Start province ID; End province ID; Adjacency type; Through province ID; Starting X; Starting Y; Ending X; Ending Y; Adjacency rule; Comment`\n\n---\n\n");

                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 2 {
                        let mut current_col = 0;
                        let mut hovered_index = None;
                        for (i, part) in parts.iter().enumerate() {
                            let end_col = current_col + part.len() as u32;
                            if position.character >= current_col && position.character <= end_col {
                                hovered_index = Some(i);
                                break;
                            }
                            current_col = end_col + 1;
                        }

                        match hovered_index {
                            Some(0) | Some(1) | Some(3) => {
                                let label = match hovered_index {
                                    Some(0) => "Start Province ID",
                                    Some(1) => "End Province ID",
                                    _ => "Through Province ID",
                                };
                                if let Ok(prov_id) = parts[hovered_index.unwrap()].parse::<u32>() {
                                    let provs = self.scanner_data.provinces();
                                    if let Some(province) = provs.get(&prov_id) {
                                        hover_text.push_str(&format!(
                                            "**Hovered:** {} `{}` (Terrain: {}, Type: {})\n",
                                            label, prov_id, province.terrain, province.prov_type
                                        ));
                                    } else {
                                        hover_text.push_str(&format!(
                                            "**Hovered:** {} `{}`\n",
                                            label, prov_id
                                        ));
                                    }
                                }
                            }
                            Some(2) => {
                                hover_text.push_str(&format!(
                                    "**Hovered:** Adjacency Type `{}`\n",
                                    parts[2]
                                ));
                            }
                            Some(4) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Starting X `{}`\n", parts[4]));
                            }
                            Some(5) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Starting Y `{}`\n", parts[5]));
                            }
                            Some(6) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Ending X `{}`\n", parts[6]));
                            }
                            Some(7) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Ending Y `{}`\n", parts[7]));
                            }
                            Some(8) => {
                                let rule_name = parts[8].trim();
                                if !rule_name.is_empty() {
                                    let rules = self.scanner_data.adjacency_rules();
                                    if let Some(rule) = rules.get(rule_name) {
                                        let mut rule_info = format!(
                                            "**Hovered:** Adjacency Rule `{}`\n",
                                            rule_name
                                        );
                                        if !rule.required_provinces.is_empty() {
                                            rule_info.push_str(&format!(
                                                "- Required Provinces: `{:?}`\n",
                                                rule.required_provinces
                                            ));
                                        }
                                        if let Some(icon) = rule.icon {
                                            rule_info.push_str(&format!("- Icon ID: `{}`\n", icon));
                                        }
                                        hover_text.push_str(&rule_info);
                                    } else {
                                        hover_text.push_str(&format!(
                                            "**Hovered:** Adjacency Rule `{}`\n",
                                            rule_name
                                        ));
                                    }
                                } else {
                                    hover_text.push_str("**Hovered:** Adjacency Rule (None)\n");
                                }
                            }
                            Some(9) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Comment `{}`\n", parts[9]));
                            }
                            _ => {}
                        }
                    }

                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_text,
                        }),
                        range: None,
                    }));
                }
                return Ok(None);
            } else if uri.ends_with(&map_config.definitions) {
                if let Some(line) = content.lines().nth(position.line as usize) {
                    let mut hover_text = String::from("### 🗺️ Province Definition\n\n");
                    hover_text.push_str("`Province ID; R; G; B; Province type; Coastal status; Terrain; Continent`\n\n---\n\n");

                    let parts: Vec<&str> = line.split(';').collect();
                    if parts.len() >= 8 {
                        let mut current_col = 0;
                        let mut hovered_index = None;
                        for (i, part) in parts.iter().enumerate() {
                            let end_col = current_col + part.len() as u32;
                            if position.character >= current_col && position.character <= end_col {
                                hovered_index = Some(i);
                                break;
                            }
                            current_col = end_col + 1;
                        }

                        match hovered_index {
                            Some(0) => {
                                hover_text.push_str(&format!(
                                    "**Hovered:** Province ID `{}`\n",
                                    parts[0]
                                ));
                            }
                            Some(1) => {
                                hover_text.push_str(&format!("**Hovered:** Red `{}`\n", parts[1]));
                            }
                            Some(2) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Green `{}`\n", parts[2]));
                            }
                            Some(3) => {
                                hover_text.push_str(&format!("**Hovered:** Blue `{}`\n", parts[3]));
                            }
                            Some(4) => {
                                hover_text.push_str(&format!(
                                    "**Hovered:** Province Type `{}` (land, sea, or lake)\n",
                                    parts[4]
                                ));
                            }
                            Some(5) => {
                                hover_text.push_str(&format!(
                                    "**Hovered:** Coastal Status `{}` (true or false)\n",
                                    parts[5]
                                ));
                            }
                            Some(6) => {
                                hover_text
                                    .push_str(&format!("**Hovered:** Terrain `{}`\n", parts[6]));
                            }
                            Some(7) => {
                                hover_text.push_str(&format!(
                                    "**Hovered:** Continent ID `{}`\n",
                                    parts[7]
                                ));
                            }
                            _ => {}
                        }
                    }

                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_text,
                        }),
                        range: None,
                    }));
                }
                return Ok(None);
            } else if uri.ends_with(".csv") {
                return Ok(None); // Skip CSV files for script hover
            }

            {
                let (script, _) = self.ensure_ast_cached(&uri).unwrap_or_else(|| {
                    let (s, e) = parser::parse_script(&content);
                    (Arc::new(s), e)
                });
                let mut scope_stack = scope::ScopeStack::new(scope::Scope::Global);
                let achievements = self.scanner_data.achievements();
                if let Some((identifier, final_scopes, assigned_value, context_key)) =
                    find_identifier_at(&script, position, &mut scope_stack, &achievements)
                {
                    let mut hover_text = String::new();
                    let context_key_lower = context_key.as_ref().map(|s| s.to_lowercase());

                    fn push_section(full_text: &mut String, section: &str) {
                        if !full_text.is_empty() && !full_text.ends_with("---\n\n") {
                            full_text.push_str("\n\n---\n\n");
                        }
                        full_text.push_str(section);
                    }

                    // Show scope stack
                    let is_music = final_scopes.iter().any(|s| {
                        *s == scope::Scope::MusicTrack || *s == scope::Scope::MusicStation
                    });
                    let is_achievement = final_scopes
                        .iter()
                        .any(|s| *s == scope::Scope::Achievement || *s == scope::Scope::Ribbon);

                    let mut scope_text = String::from(if is_music {
                        "### 🎵 Music Definition Stack\n"
                    } else if is_achievement {
                        "### 🏆 Achievement Context Stack\n"
                    } else {
                        "### 🔍 Scope Stack\n"
                    });

                    for (i, s) in final_scopes.iter().enumerate() {
                        if i > 0 {
                            scope_text.push_str(" > ");
                        }
                        scope_text.push_str(s.as_str());
                    }
                    push_section(&mut hover_text, &scope_text);

                    // Achievement specialized hover
                    if let Some(achievement) = achievements.get(&identifier) {
                        let mut ach_text = if achievement.is_ribbon {
                            format!("### 🎀 Ribbon: `{}`\n", identifier)
                        } else {
                            format!("### 🏆 Achievement: `{}`\n", identifier)
                        };

                        let loc = self.scanner_data.localization();

                        let name_key = format!("{}_NAME", identifier);
                        if let Some(name_loc) = loc.get(&name_key) {
                            ach_text.push_str(&format!(
                                "\n**Name (`{}`):** {}\n",
                                name_key,
                                paradox_to_markdown(&name_loc.value, Some(&loc))
                            ));
                        } else {
                            ach_text.push_str(&format!("\n**Name:** *Missing `{}`*\n", name_key));
                        }

                        let desc_key = format!("{}_DESC", identifier);
                        if let Some(desc_loc) = loc.get(&desc_key) {
                            ach_text.push_str(&format!(
                                "\n**Description (`{}`):** {}\n",
                                desc_key,
                                paradox_to_markdown(&desc_loc.value, Some(&loc))
                            ));
                        } else {
                            ach_text.push_str(&format!(
                                "\n**Description:** *Missing `{}`*\n",
                                desc_key
                            ));
                        }

                        ach_text.push_str(&format!(
                            "\n---\nDefined in: {}",
                            self.make_file_link(&achievement.path)
                        ));
                        push_section(&mut hover_text, &ach_text);
                    }

                    // Check for states
                    let mut id_opt = None;
                    if let Some(ast::Value::Number(n)) = &assigned_value {
                        id_opt = Some(*n as u32);
                    } else if let Ok(n) = identifier.parse::<u32>() {
                        id_opt = Some(n);
                    }

                    if let Some(id) = id_opt {
                        let states = self.scanner_data.states();
                        if let Some(state) = states.get(&id) {
                            // To prevent false positives, we only show this if the identifier is explicitly related to states
                            // Or if the identifier *is* the number (meaning it's an element in an array, like in any_state_of)
                            let ident_lower = identifier.to_lowercase();
                            let is_state_key = ident_lower.contains("state")
                                || ident_lower.contains("capital")
                                || (ident_lower == "id"
                                    && context_key_lower.as_deref() == Some("state"))
                                || ident_lower == "add_core_of"
                                || ident_lower == "add_claim_by"
                                || (identifier.parse::<u32>().is_ok()
                                    && context_key_lower
                                        .as_ref()
                                        .is_some_and(|ck| ck.contains("state")));

                            if is_state_key {
                                let loc = self.scanner_data.localization();
                                let state_name = if let Some(loc_entry) = loc.get(&state.name) {
                                    paradox_to_markdown(&loc_entry.value, Some(&loc))
                                } else {
                                    state.name.clone()
                                };

                                push_section(
                                    &mut hover_text,
                                    &format!(
                                        "### 🗺️ State: {}\n\nID: `{}`\n\nDefined in: {}",
                                        state_name,
                                        id,
                                        self.make_file_link(&state.path)
                                    ),
                                );
                            }
                        }

                        let provinces = self.scanner_data.provinces();
                        if let Some(province) = provinces.get(&id) {
                            let ident_lower = identifier.to_lowercase();
                            let is_province_key = ident_lower.contains("province")
                                || ident_lower == "victory_points"
                                || (identifier.parse::<u32>().is_ok()
                                    && context_key_lower.as_ref().is_some_and(|ck| {
                                        ck.contains("province") || ck == "victory_points"
                                    }));

                            if is_province_key {
                                let mut text = format!("### 📍 Province: {}\n\n", id);
                                text.push_str(&format!("**Terrain:** `{}`\n", province.terrain));
                                text.push_str(&format!("**Type:** `{}`\n", province.prov_type));
                                text.push_str(&format!(
                                    "**Coastal:** {}\n",
                                    if province.is_coastal { "Yes" } else { "No" }
                                ));
                                text.push_str(&format!(
                                    "**Continent:** `{}`\n",
                                    province.continent
                                ));
                                text.push_str(&format!(
                                    "**Color (RGB):** `{}, {}, {}`\n",
                                    province.rgb.0, province.rgb.1, province.rgb.2
                                ));
                                push_section(&mut hover_text, &text);
                            }
                        }

                        let regions = self.scanner_data.strategic_regions();
                        if let Some(region) = regions.get(&id) {
                            let ident_lower = identifier.to_lowercase();
                            let is_region_key = ident_lower.contains("strategic_region")
                                || (ident_lower == "id"
                                    && context_key_lower.as_deref() == Some("strategic_region"))
                                || (identifier.parse::<u32>().is_ok()
                                    && context_key_lower
                                        .as_ref()
                                        .is_some_and(|ck| ck.contains("strategic_region")));

                            if is_region_key {
                                let loc = self.scanner_data.localization();
                                let region_name = if let Some(loc_entry) = loc.get(&region.name) {
                                    loc_entry.value.clone()
                                } else {
                                    region.name.clone()
                                };

                                let mut text = format!(
                                    "### 🗺️ Strategic Region: {}\n\nID: `{}`\n\n",
                                    region_name, id
                                );
                                if let Some(weather) = &region.weather {
                                    text.push_str(&format!("**Weather:** `{}`\n", weather));
                                }
                                if let Some(naval) = &region.naval_terrain {
                                    text.push_str(&format!("**Naval Terrain:** `{}`\n", naval));
                                }
                                text.push_str(&format!(
                                    "**Provinces:** `{}`\n",
                                    region.provinces.len()
                                ));
                                text.push_str(&format!(
                                    "\nDefined in: {}",
                                    self.make_file_link(&region.path)
                                ));
                                push_section(&mut hover_text, &text);
                            }
                        }
                    }

                    // Check triggers/effects from hardcoded data
                    if let Some(entity) = crate::TRIGGERS.get(identifier.as_str()) {
                        push_section(
                            &mut hover_text,
                            &format!("### 🔍 Trigger: {}\n\n{}", entity.name, entity.description),
                        );
                    } else if let Some(entity) = crate::EFFECTS.get(identifier.as_str()) {
                        push_section(
                            &mut hover_text,
                            &format!("### ⚡ Effect: {}\n\n{}", entity.name, entity.description),
                        );
                    } else if crate::SCOPES.contains(&identifier.to_uppercase().as_str()) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🎯 Scope: {}\n\nStandard Paradox scope.",
                                identifier.to_uppercase()
                            ),
                        );
                    } else if crate::LOC_COMMANDS.contains(&identifier.as_str()) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🛠️ Localization Command: {}\n\nStandard localization command.",
                                identifier
                            ),
                        );
                    }

                    // Check localization
                    let loc = self.scanner_data.localization();
                    // Try exact match first, then try keys starting with ID:
                    let entry = loc.get(&identifier).or_else(|| {
                        // Find any key that starts with "identifier:"
                        let target = format!("{}:", identifier);
                        loc.iter()
                            .find(|(k, _)| k.starts_with(&target))
                            .map(|(_, e)| e)
                    });

                    if let Some(e) = entry {
                        let mut text = format!("### 🌐 Localization: {}\n\n", e.key);
                        text.push_str(&format!("**Raw:** `{}`\n\n", e.value));
                        text.push_str("**Preview:**\n\n");
                        text.push_str(&paradox_to_markdown(&e.value, Some(&loc)));
                        push_section(&mut hover_text, &text);
                    } else {
                        // Check scripted triggers
                        let st = self.scanner_data.scripted_triggers();
                        if let Some(entity) = st.get(&identifier) {
                            push_section(
                                &mut hover_text,
                                &format!(
                                    "### 📜 Scripted Trigger: {}\n\nDefined in: {}",
                                    identifier,
                                    self.make_file_link(&entity.path)
                                ),
                            );
                        } else {
                            // Check scripted effects
                            let se = self.scanner_data.scripted_effects();
                            if let Some(entity) = se.get(&identifier) {
                                push_section(
                                    &mut hover_text,
                                    &format!(
                                        "### 🛠️ Scripted Effect: {}\n\nDefined in: {}",
                                        identifier,
                                        self.make_file_link(&entity.path)
                                    ),
                                );
                            } else {
                                // Check scripted locs
                                let sl = self.scanner_data.scripted_locs();
                                if let Some(loc) = sl.get(&identifier) {
                                    push_section(
                                        &mut hover_text,
                                        &format!(
                                            "### 📝 Scripted Localisation: {}\n\nDefined in: {}",
                                            identifier,
                                            self.make_file_link(&loc.path)
                                        ),
                                    );
                                }
                            }
                        }
                    }

                    // Check ideologies
                    let id_map = self.scanner_data.ideologies();
                    if let Some(ideology) = id_map.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🗳️ Ideology: {}\n\nDefined in: {}\n\nSub-ideologies: {}",
                                ideology.name,
                                self.make_file_link(&ideology.path),
                                ideology.sub_ideologies.join(", ")
                            ),
                        );
                    }

                    // Check sub-ideologies
                    let sid_map = self.scanner_data.sub_ideologies();
                    if let Some((parent, _, path)) = sid_map.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🗳️ Sub-Ideology: {}\n\nParent Ideology: `{}`\n\nDefined in: {}",
                                identifier,
                                parent,
                                self.make_file_link(path)
                            ),
                        );
                    }

                    // Check traits
                    let t_map = self.scanner_data.traits();
                    if let Some(trait_info) = t_map.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🎖️ Trait: {}\n\nType: `{}`\n\nDefined in: {}",
                                trait_info.name,
                                trait_info.trait_type,
                                self.make_file_link(&trait_info.path)
                            ),
                        );
                    }

                    // Check sprites
                    let s_map = self.scanner_data.sprites();
                    if let Some(sprite) = s_map.get(&identifier) {
                        let mut texture_link = sprite.texture_file.clone();
                        // Attempt to resolve texture path relative to root
                        let gfx_path = std::path::Path::new(&sprite.path);
                        let mut root = gfx_path.parent();
                        while let Some(r) = root {
                            if r.join("interface").exists() || r.join("common").exists() {
                                let full_texture = r.join(&sprite.texture_file);
                                if full_texture.exists() {
                                    texture_link =
                                        self.make_file_link(&full_texture.to_string_lossy());
                                    break;
                                }
                            }
                            root = r.parent();
                        }

                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🖼️ Sprite: {}\n\nTexture: {}\n\nDefined in: {}",
                                sprite.name,
                                texture_link,
                                self.make_file_link(&sprite.path)
                            ),
                        );
                    }

                    // Check events
                    let e_map = self.scanner_data.events();
                    if let Some(event) = e_map.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 📅 Event: {}\n\nType: `{}`\n\nDefined in: {}\n\nTriggers: {}",
                                event.id,
                                event.event_type,
                                self.make_file_link(&event.path),
                                if event.triggered_events.is_empty() {
                                    "None".to_string()
                                } else {
                                    event.triggered_events.join(", ")
                                }
                            ),
                        );
                    }

                    // Check ideas
                    let idea_map = self.scanner_data.ideas();
                    if let Some(idea) = idea_map.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 💡 Idea: {}\n\nCategory: `{}`\n\nDefined in: {}",
                                idea.name,
                                idea.category,
                                self.make_file_link(&idea.path)
                            ),
                        );
                    }

                    // Check characters
                    let char_map = self.scanner_data.characters();
                    if let Some(character) = char_map.get(&identifier) {
                        let mut char_text = format!("### 👤 Character: `{}`\n", identifier);

                        let loc = self.scanner_data.localization();
                        if let Some(name_key) = &character.name {
                            if let Some(name_loc) = loc.get(name_key) {
                                char_text.push_str(&format!(
                                    "\n**Name:** {}\n",
                                    paradox_to_markdown(&name_loc.value, Some(&loc))
                                ));
                            } else {
                                char_text
                                    .push_str(&format!("\n**Name:** *Missing `{}`*\n", name_key));
                            }
                        }

                        if let Some(gender) = &character.gender {
                            char_text.push_str(&format!("**Gender:** {}\n", gender));
                        }

                        if let Some(desc_key) = &character.desc {
                            if let Some(desc_loc) = loc.get(desc_key) {
                                char_text.push_str(&format!(
                                    "**Description:** {}\n",
                                    paradox_to_markdown(&desc_loc.value, Some(&loc))
                                ));
                            } else {
                                char_text.push_str(&format!(
                                    "**Description:** *Missing `{}`*\n",
                                    desc_key
                                ));
                            }
                        }

                        if !character.portraits.is_empty() {
                            char_text.push_str("\n**Portraits:**\n");
                            let s_map = self.scanner_data.sprites();
                            for (cat, sprite_name) in &character.portraits {
                                let mut texture_link = sprite_name.clone();
                                if let Some(sprite) = s_map.get(sprite_name) {
                                    let gfx_path = std::path::Path::new(&sprite.path);
                                    let mut root = gfx_path.parent();
                                    while let Some(r) = root {
                                        if r.join("interface").exists() || r.join("common").exists()
                                        {
                                            let full_texture = r.join(&sprite.texture_file);
                                            if full_texture.exists() {
                                                texture_link = format!(
                                                    "[{}]({})",
                                                    sprite_name,
                                                    self.make_file_link(
                                                        &full_texture.to_string_lossy()
                                                    )
                                                );
                                                break;
                                            }
                                        }
                                        root = r.parent();
                                    }
                                } else if sprite_name.starts_with("gfx/") {
                                    let char_path = std::path::Path::new(&character.path);
                                    let mut root = char_path.parent();
                                    while let Some(r) = root {
                                        if r.join("common").exists() {
                                            let full_texture = r.join(sprite_name);
                                            if full_texture.exists() {
                                                texture_link = format!(
                                                    "[{}]({})",
                                                    sprite_name,
                                                    self.make_file_link(
                                                        &full_texture.to_string_lossy()
                                                    )
                                                );
                                                break;
                                            }
                                        }
                                        root = r.parent();
                                    }
                                }
                                char_text.push_str(&format!("- {}: {}\n", cat, texture_link));
                            }
                        }

                        if !character.roles.is_empty() {
                            char_text.push_str("\n**Roles:**\n");
                            for role in &character.roles {
                                char_text.push_str(&format!("- `{}`", role.role_type));
                                if let Some(ideology) = &role.ideology {
                                    char_text.push_str(&format!(" (Ideology: `{}`)", ideology));
                                }

                                let mut skills = Vec::new();
                                if let Some(s) = role.skill {
                                    skills.push(format!("Skill: {}", s));
                                }
                                if let Some(s) = role.attack_skill {
                                    skills.push(format!("Attack: {}", s));
                                }
                                if let Some(s) = role.defense_skill {
                                    skills.push(format!("Defense: {}", s));
                                }
                                if let Some(s) = role.planning_skill {
                                    skills.push(format!("Planning: {}", s));
                                }
                                if let Some(s) = role.logistics_skill {
                                    skills.push(format!("Logistics: {}", s));
                                }
                                if let Some(s) = role.maneuvering_skill {
                                    skills.push(format!("Maneuvering: {}", s));
                                }
                                if let Some(s) = role.coordination_skill {
                                    skills.push(format!("Coordination: {}", s));
                                }

                                if !skills.is_empty() {
                                    char_text.push_str(&format!(" [{}]", skills.join(", ")));
                                }

                                if !role.traits.is_empty() {
                                    char_text.push_str(&format!(
                                        "\n  - Traits: `{}`",
                                        role.traits.join(", ")
                                    ));
                                }
                                char_text.push('\n');
                            }
                        }

                        char_text.push_str(&format!(
                            "\n---\nDefined in: {}",
                            self.make_file_link(&character.path)
                        ));
                        push_section(&mut hover_text, &char_text);
                    }

                    // Check abilities
                    let ability_map = self.scanner_data.abilities();
                    if let Some(ability) = ability_map.get(&identifier) {
                        let mut text = format!("### ⚔️ Leader Ability: `{}`\n", ability.key);
                        let loc = self.scanner_data.localization();

                        if let Some(name_key) = &ability.name_loc {
                            if let Some(name_loc) = loc.get(name_key) {
                                text.push_str(&format!(
                                    "\n**Name:** {}\n",
                                    paradox_to_markdown(&name_loc.value, Some(&loc))
                                ));
                            } else {
                                text.push_str(&format!("\n**Name:** *Missing `{}`*\n", name_key));
                            }
                        }

                        if let Some(desc_key) = &ability.desc_loc {
                            if let Some(desc_loc) = loc.get(desc_key) {
                                text.push_str(&format!(
                                    "\n**Description:** {}\n",
                                    paradox_to_markdown(&desc_loc.value, Some(&loc))
                                ));
                            }
                        }

                        if let Some(cost) = ability.cost {
                            text.push_str(&format!("\n**Cost:** {}\n", cost));
                        }
                        if let Some(duration) = ability.duration {
                            text.push_str(&format!("\n**Duration:** {} hours\n", duration));
                        }
                        if let Some(cooldown) = ability.cooldown {
                            text.push_str(&format!("\n**Cooldown:** {} hours\n", cooldown));
                        }
                        if let Some(type_name) = &ability.type_name {
                            text.push_str(&format!("\n**Type:** `{}`\n", type_name));
                        }
                        if let Some(cancelable) = ability.cancelable {
                            text.push_str(&format!(
                                "\n**Cancelable:** {}\n",
                                if cancelable { "Yes" } else { "No" }
                            ));
                        }
                        if let Some(sound) = &ability.sound_effect {
                            text.push_str(&format!("\n**Sound Effect:** `{}`\n", sound));
                        }
                        if let Some(icon) = &ability.icon {
                            text.push_str(&format!("\n**Icon:** `{}`\n", icon));
                        }

                        // Block presence indicators
                        let mut blocks = Vec::new();
                        if ability.has_allowed {
                            blocks.push("allowed");
                        }
                        if ability.has_unit_modifiers {
                            blocks.push("unit_modifiers");
                        }
                        if ability.has_one_time_effect {
                            blocks.push("one_time_effect");
                        }
                        if ability.has_ai_will_do {
                            blocks.push("ai_will_do");
                        }
                        if !blocks.is_empty() {
                            text.push_str(&format!("\n**Blocks:** {}\n", blocks.join(", ")));
                        }

                        text.push_str(&format!(
                            "\n---\nDefined in: {}",
                            self.make_file_link(&ability.path)
                        ));
                        push_section(&mut hover_text, &text);
                    }

                    // Check portraits
                    let portrait_map = self.scanner_data.portraits();
                    if let Some(portrait) = portrait_map.get(&identifier) {
                        let block_kind = match portrait.block_type {
                            crate::portrait_scanner::PortraitBlockType::Default => "Default",
                            crate::portrait_scanner::PortraitBlockType::Continent => "Continent",
                            crate::portrait_scanner::PortraitBlockType::Tag => "Country Tag",
                        };

                        let mut text = format!("### 🖼️ Portrait Definition: `{}`\n", portrait.name);
                        text.push_str(&format!("\n**Type:** {}\n", block_kind));
                        if let Some(cont) = &portrait.continent_name {
                            text.push_str(&format!("**Continent:** `{}`\n", cont));
                        }

                        let mut blocks: Vec<String> = Vec::new();
                        if portrait.has_male {
                            blocks.push("male".to_string());
                        }
                        if portrait.has_female {
                            blocks.push("female".to_string());
                        }
                        if portrait.has_army {
                            blocks.push("army".to_string());
                        }
                        if portrait.has_navy {
                            blocks.push("navy".to_string());
                        }
                        if portrait.has_operative {
                            blocks.push("operative".to_string());
                        }
                        if portrait.has_scientist {
                            blocks.push("scientist".to_string());
                        }
                        if portrait.has_political {
                            blocks.push(format!("political [{}]", portrait.ideologies.join(", ")));
                        }
                        if !blocks.is_empty() {
                            text.push_str(&format!("\n**Blocks:** {}\n", blocks.join(", ")));
                        }

                        if !portrait.gfx_entries.is_empty() {
                            let gfx_count = portrait.gfx_entries.len();
                            text.push_str(&format!(
                                "\n**GFX References:** {} sprite(s)\n",
                                gfx_count
                            ));
                            let s_map = self.scanner_data.sprites();
                            for gfx in portrait.gfx_entries.iter().take(10) {
                                if let Some(sprite) = s_map.get(gfx) {
                                    text.push_str(&format!(
                                        "\n- `{}` → `{}`",
                                        gfx, sprite.texture_file
                                    ));
                                } else {
                                    text.push_str(&format!("\n- `{}`", gfx));
                                }
                            }
                            if gfx_count > 10 {
                                text.push_str(&format!("\n- ... and {} more", gfx_count - 10));
                            }
                        }

                        text.push_str(&format!(
                            "\n\n---\nDefined in: {}",
                            self.make_file_link(&portrait.path)
                        ));
                        push_section(&mut hover_text, &text);
                    }

                    // Check AI strategy plans
                    let ap_map = self.scanner_data.ai_strategy_plans();
                    if let Some(plan) = ap_map.get(&identifier) {
                        let mut text = format!("### AI Strategy Plan: `{}`\n", plan.name);

                        let mut blocks = Vec::new();
                        if plan.has_ai_national_focuses {
                            blocks.push("ai_national_focuses");
                        }
                        if plan.has_research {
                            blocks.push("research");
                        }
                        if plan.has_ideas {
                            blocks.push("ideas");
                        }
                        if plan.has_traits {
                            blocks.push("traits");
                        }
                        if plan.has_ai_strategy {
                            blocks.push("ai_strategy");
                        }
                        if plan.has_focus_factors {
                            blocks.push("focus_factors");
                        }
                        if plan.has_weight {
                            blocks.push("weight");
                        }
                        if !blocks.is_empty() {
                            text.push_str(&format!("\n**Blocks:** {}\n", blocks.join(", ")));
                        }

                        text.push_str(&format!(
                            "\n---\nDefined in: {}",
                            self.make_file_link(&plan.path)
                        ));
                        push_section(&mut hover_text, &text);
                    }

                    // Check for modifier blocks (modifier = { ... } or modifiers = { ... })
                    let identifier_lower = identifier.to_lowercase();
                    if (identifier_lower == "modifier"
                        || identifier_lower == "modifiers"
                        || identifier_lower == "unit_modifiers")
                        && matches!(assigned_value, Some(ast::Value::Block(_)))
                    {
                        let mappings = self.scanner_data.modifier_mappings();
                        let formats = self.scanner_data.modifier_formats();
                        let loc = self.scanner_data.localization();

                        let display_service = modifier_display::ModifierDisplayService::new(
                            (*mappings).clone(),
                            (*formats).clone(),
                            (*loc).clone(),
                        );

                        if let Some(value) = &assigned_value {
                            let blocks = display_service.extract_modifier_blocks(value);
                            if !blocks.is_empty() {
                                let section_title = if identifier_lower == "unit_modifiers" {
                                    "### 📊 Unit Modifiers\n\n"
                                } else {
                                    "### 📊 Modifier Block\n\n"
                                };
                                let formatted = display_service.format_all_blocks(&blocks);
                                push_section(
                                    &mut hover_text,
                                    &format!("{}{}", section_title, formatted),
                                );
                            }
                        }
                    }

                    // Check for one_time_effect blocks
                    if identifier_lower == "one_time_effect"
                        && matches!(assigned_value, Some(ast::Value::Block(_)))
                    {
                        if let Some(ast::Value::Block(entries)) = &assigned_value {
                            let mut effect_list = Vec::new();
                            for entry in entries {
                                if let ast::Entry::Assignment(ass) = entry {
                                    let key = &ass.key;
                                    if matches!(&ass.value.value, ast::Value::Block(_)) {
                                        effect_list.push(format!("`{}` {{ ... }}", key));
                                    } else {
                                        effect_list.push(format!("`{}`", key));
                                    }
                                }
                            }
                            if !effect_list.is_empty() {
                                push_section(
                                    &mut hover_text,
                                    &format!(
                                        "### ⚡ One-Time Effects\n\n{}",
                                        effect_list.join("\n")
                                    ),
                                );
                            }
                        }
                    }

                    // Check modifiers
                    let custom_mods = self.scanner_data.custom_modifiers();
                    if let Some(modifier) = custom_mods.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🔧 Custom Modifier: {}\n\nDefined in: {}",
                                identifier,
                                self.make_file_link(&modifier.path)
                            ),
                        );
                    }
                    let mappings = self.scanner_data.modifier_mappings();
                    if let Some(loc_key) = mappings.get(&identifier) {
                        let loc = self.scanner_data.localization();
                        let loc_text = if let Some(e) = loc.get(loc_key) {
                            paradox_to_markdown(&e.value, Some(&loc))
                        } else {
                            loc_key.clone()
                        };

                        let formats = self.scanner_data.modifier_formats();
                        let format_info = formats.get(loc_key);

                        let parsed_val = match assigned_value {
                            Some(ast::Value::Number(val)) => Some(val),
                            Some(ast::Value::String(s)) => s.parse::<f64>().ok(),
                            _ => None,
                        };

                        if let Some(val) = parsed_val {
                            let formatted_val =
                                format_modifier_value(&identifier, val, format_info);
                            push_section(
                                &mut hover_text,
                                &format!("### 📈 {}\n\n{}", loc_text, formatted_val),
                            );
                        } else {
                            push_section(
                                &mut hover_text,
                                &format!(
                                    "### 📉 {}\n\nEngine Modifier: `{}`",
                                    loc_text, identifier
                                ),
                            );
                        }
                    }

                    // Check variables
                    let var_map = self.scanner_data.variables();
                    if let Some(vars) = var_map.get(&identifier) {
                        let paths: Vec<String> =
                            vars.iter().map(|v| self.make_file_link(&v.path)).collect();
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🔢 Variable: {}\n\nUsed/Defined in:\n- {}",
                                identifier,
                                paths.join("\n- ")
                            ),
                        );
                    }

                    // Check event targets
                    let target_map = self.scanner_data.event_targets();
                    if let Some(targets) = target_map.get(&identifier) {
                        let paths: Vec<String> = targets
                            .iter()
                            .map(|t| {
                                format!(
                                    "{} ({})",
                                    self.make_file_link(&t.path),
                                    if t.is_global { "Global" } else { "Local" }
                                )
                            })
                            .collect();
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🎯 Event Target: {}\n\nSaved in:\n- {}",
                                identifier,
                                paths.join("\n- ")
                            ),
                        );
                    }

                    // Check music
                    let m_assets = self.scanner_data.music_assets();
                    if let Some(asset) = m_assets.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🎵 Music Asset: {}\n\nFile: `{}`\n\nDefined in: {}",
                                asset.name,
                                asset.file,
                                self.make_file_link(&asset.path)
                            ),
                        );
                    }

                    let m_stations = self.scanner_data.music_stations();
                    if let Some(station) = m_stations.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 📻 Music Station: {}\n\nDefined in: {}",
                                station.name,
                                self.make_file_link(&station.path)
                            ),
                        );
                    }

                    let m_songs = self.scanner_data.songs();
                    if let Some(song) = m_songs.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🎶 Song: {}\n\nDefined in: {}",
                                song.name,
                                self.make_file_link(&song.path)
                            ),
                        );
                    }

                    // Check sounds
                    let s_sounds = self.scanner_data.sounds();
                    if let Some(sound) = s_sounds.get(&identifier) {
                        let mut file_link = sound.file.clone();
                        // Try to resolve file link
                        let asset_path = std::path::Path::new(&sound.path);
                        if let Some(root) = asset_path.parent().and_then(|p| p.parent()) {
                            let full_sound_path = root.join("sound").join(&sound.file);
                            if full_sound_path.exists() {
                                file_link = self.make_file_link(&full_sound_path.to_string_lossy());
                            }
                        }

                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🔊 Sound: {}\n\nFile: {}\n\nDefined in: {}",
                                sound.name,
                                file_link,
                                self.make_file_link(&sound.path)
                            ),
                        );
                    }

                    let s_effects = self.scanner_data.sound_effects();
                    if let Some(effect) = s_effects.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 🔉 Sound Effect: {}\n\nSounds: `{}`\n\nDefined in: {}",
                                effect.name,
                                effect.sounds.join(", "),
                                self.make_file_link(&effect.path)
                            ),
                        );
                    }

                    let s_falloffs = self.scanner_data.falloffs();
                    if let Some(falloff) = s_falloffs.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 📉 Sound Falloff: {}\n\nDefined in: {}",
                                falloff.name,
                                self.make_file_link(&falloff.path)
                            ),
                        );
                    }

                    let s_categories = self.scanner_data.sound_categories();
                    if let Some(category) = s_categories.get(&identifier) {
                        push_section(
                            &mut hover_text,
                            &format!(
                                "### 📂 Sound Category: {}\n\nEffects: `{}`\n\nDefined in: {}",
                                category.name,
                                category.soundeffects.join(", "),
                                self.make_file_link(&category.path)
                            ),
                        );
                    }

                    if !hover_text.is_empty() {
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: hover_text,
                            }),
                            range: None,
                        }));
                    }
                }
            }
        }
        Ok(None)
    }
}
