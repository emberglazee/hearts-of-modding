use crate::parser::ast;
use crate::utils::lsp_convert::ast_range_to_lsp;
use tower_lsp_server::ls_types::{Color, ColorInformation};

pub fn find_colors(script: &ast::Script) -> Vec<ColorInformation> {
    let mut colors = Vec::new();
    for entry in &script.entries {
        find_colors_in_entry(entry, &mut colors, &[], &script.source);
    }
    colors
}

fn find_colors_in_entry(
    entry: &ast::Entry,
    colors: &mut Vec<ColorInformation>,
    parent_keys: &[&str],
    content: &str,
) {
    if let ast::Entry::Assignment(ass) = entry {
        let is_color_context = ass.key_text(content).to_ascii_lowercase().contains("color")
            || parent_keys.contains(&"textcolors");
        let mut keys: Vec<String> = parent_keys.iter().map(|s| s.to_string()).collect();
        keys.push(ass.key_text(content).to_string());
        let keys_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
        find_colors_in_value(&ass.value, colors, is_color_context, &keys_refs, content);
    } else if let ast::Entry::Value(val) = entry {
        find_colors_in_value(val, colors, false, parent_keys, content);
    }
}

fn find_colors_in_value(
    val: &ast::NodeedValue,
    colors: &mut Vec<ColorInformation>,
    is_color_context: bool,
    parent_keys: &[&str],
    content: &str,
) {
    match &val.value {
        ast::Value::Block(entries) => {
            let nums: Vec<f64> = entries
                .iter()
                .filter_map(|e| {
                    if let ast::Entry::Value(v) = e {
                        match &v.value {
                            ast::Value::Number(n) => Some(*n),
                            ast::Value::String(s) => s.resolve(content).parse::<f64>().ok(),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if nums.len() == 3 && is_color_context {
                let is_255 = nums.iter().any(|&n| n > 1.0);
                let (r, g, b) = if is_255 {
                    (
                        (nums[0] / 255.0) as f32,
                        (nums[1] / 255.0) as f32,
                        (nums[2] / 255.0) as f32,
                    )
                } else {
                    (nums[0] as f32, nums[1] as f32, nums[2] as f32)
                };
                colors.push(ColorInformation {
                    range: ast_range_to_lsp(&val.range),
                    color: Color {
                        red: r,
                        green: g,
                        blue: b,
                        alpha: 1.0,
                    },
                });
            } else {
                for e in entries {
                    find_colors_in_entry(e, colors, parent_keys, content);
                }
            }
        }
        ast::Value::TaggedBlock(tag, entries, _) => {
            let nums: Vec<f64> = entries
                .iter()
                .filter_map(|e| {
                    if let ast::Entry::Value(v) = e {
                        match &v.value {
                            ast::Value::Number(n) => Some(*n),
                            ast::Value::String(s) => s.resolve(content).parse::<f64>().ok(),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if nums.len() == 3 {
                let tag_lower = tag.resolve(content).to_ascii_lowercase();
                if tag_lower == "rgb" {
                    let r = (nums[0] / 255.0) as f32;
                    let g = (nums[1] / 255.0) as f32;
                    let b = (nums[2] / 255.0) as f32;
                    colors.push(ColorInformation {
                        range: ast_range_to_lsp(&val.range),
                        color: Color {
                            red: r,
                            green: g,
                            blue: b,
                            alpha: 1.0,
                        },
                    });
                } else if tag_lower == "hsv" {
                    let (r, g, b) = hsv_to_rgb(nums[0], nums[1], nums[2]);
                    colors.push(ColorInformation {
                        range: ast_range_to_lsp(&val.range),
                        color: Color {
                            red: r as f32,
                            green: g as f32,
                            blue: b as f32,
                            alpha: 1.0,
                        },
                    });
                }
            } else {
                for e in entries {
                    find_colors_in_entry(e, colors, parent_keys, content);
                }
            }
        }
        _ => {}
    }
}

pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let angle = h * 360.0;
    let c = v * s;
    let x = c * (1.0 - ((angle / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = if angle < 60.0 {
        (c, x, 0.0)
    } else if angle < 120.0 {
        (x, c, 0.0)
    } else if angle < 180.0 {
        (0.0, c, x)
    } else if angle < 240.0 {
        (0.0, x, c)
    } else if angle < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r_prime + m, g_prime + m, b_prime + m)
}

#[allow(dead_code)]
pub fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        60.0 * (((g - b) / delta) % 6.0)
    } else if (max - g).abs() < 1e-6 {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max < 1e-6 { 0.0 } else { delta / max };
    let v = max;

    (h / 360.0, s, v)
}
