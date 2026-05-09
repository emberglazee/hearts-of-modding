use tower_lsp::lsp_types::{Color, ColorInformation, ColorPresentation, TextEdit, Range as LspRange};
use crate::ast;
use crate::defines_parser::GameDefines;

/// Color modifiers from game defines
#[derive(Debug, Clone)]
pub struct ColorModifiers {
    pub country_color_saturation: f64,
    pub country_color_brightness: f64,
    pub country_ui_color_saturation: f64,
    pub country_ui_color_brightness: f64,
}

impl Default for ColorModifiers {
    fn default() -> Self {
        // HOI4 default values
        Self {
            country_color_saturation: 0.6,
            country_color_brightness: 0.9,
            country_ui_color_saturation: 0.6,
            country_ui_color_brightness: 0.9,
        }
    }
}

impl ColorModifiers {
    /// Load color modifiers from game defines
    pub fn from_defines(defines: &GameDefines) -> Self {
        Self {
            country_color_saturation: defines.defines.get("COUNTRY_COLOR_SATURATION_MODIFIER")
                .copied().unwrap_or(0.6),
            country_color_brightness: defines.defines.get("COUNTRY_COLOR_BRIGHTNESS_MODIFIER")
                .copied().unwrap_or(0.9),
            country_ui_color_saturation: defines.defines.get("COUNTRY_UI_COLOR_SATURATION_MODIFIER")
                .copied().unwrap_or(0.6),
            country_ui_color_brightness: defines.defines.get("COUNTRY_UI_COLOR_BRIGHTNESS_MODIFIER")
                .copied().unwrap_or(0.9),
        }
    }
}

/// Enhanced color information with context
#[derive(Debug, Clone)]
pub struct EnhancedColorInfo {
    pub range: ast::Range,
    pub color: Color,
    pub is_ui_color: bool,
    pub format: ColorFormat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorFormat {
    RGB,
    HSV,
}

/// Apply game color modifiers to RGB values
pub fn apply_color_modifiers(r: f64, g: f64, b: f64, is_ui: bool, modifiers: &ColorModifiers) -> (f64, f64, f64) {
    // Convert to HSV
    let (h, s, v) = rgb_to_hsv(r, g, b);

    // Apply modifiers
    let (sat_mod, bright_mod) = if is_ui {
        (modifiers.country_ui_color_saturation, modifiers.country_ui_color_brightness)
    } else {
        (modifiers.country_color_saturation, modifiers.country_color_brightness)
    };

    let modified_s = (s * sat_mod).min(1.0);
    let modified_v = (v * bright_mod).min(1.0);

    // Convert back to RGB
    hsv_to_rgb(h, modified_s, modified_v)
}

/// Remove game color modifiers from RGB values (for editing)
pub fn remove_color_modifiers(r: f64, g: f64, b: f64, is_ui: bool, modifiers: &ColorModifiers) -> (f64, f64, f64) {
    // Convert to HSV
    let (h, s, v) = rgb_to_hsv(r, g, b);

    // Remove modifiers
    let (sat_mod, bright_mod) = if is_ui {
        (modifiers.country_ui_color_saturation, modifiers.country_ui_color_brightness)
    } else {
        (modifiers.country_color_saturation, modifiers.country_color_brightness)
    };

    let original_s = if sat_mod > 0.0 { (s / sat_mod).min(1.0) } else { s };
    let original_v = if bright_mod > 0.0 { (v / bright_mod).min(1.0) } else { v };

    // Convert back to RGB
    hsv_to_rgb(h, original_s, original_v)
}

/// Convert RGB to HSV (0-1 range for all values)
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

/// Convert HSV to RGB (0-1 range for all values)
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

/// Generate color presentations for both RGB and HSV formats
pub fn generate_color_presentations(
    color: &Color,
    range: LspRange,
    is_ui: bool,
    modifiers: &ColorModifiers,
) -> Vec<ColorPresentation> {
    let mut presentations = Vec::new();

    // Remove game modifiers to get original color
    let (r_orig, g_orig, b_orig) = remove_color_modifiers(
        color.red as f64,
        color.green as f64,
        color.blue as f64,
        is_ui,
        modifiers,
    );

    // RGB format (0-255)
    let r = (r_orig * 255.0).round() as u32;
    let g = (g_orig * 255.0).round() as u32;
    let b = (b_orig * 255.0).round() as u32;

    let rgb_text = format!("{{ {} {} {} }}", r, g, b);
    presentations.push(ColorPresentation {
        label: format!("RGB: {}", rgb_text),
        text_edit: Some(TextEdit {
            range,
            new_text: rgb_text,
        }),
        additional_text_edits: None,
    });

    // HSV format (0-1 range)
    let (h, s, v) = rgb_to_hsv(r_orig, g_orig, b_orig);
    let hsv_text = format!("hsv {{ {:.3} {:.3} {:.3} }}", h, s, v);
    presentations.push(ColorPresentation {
        label: format!("HSV: {}", hsv_text),
        text_edit: Some(TextEdit {
            range,
            new_text: hsv_text,
        }),
        additional_text_edits: None,
    });

    presentations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hsv() {
        // Red
        let (h, s, v) = rgb_to_hsv(1.0, 0.0, 0.0);
        assert!((h - 0.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);

        // Green
        let (h, s, v) = rgb_to_hsv(0.0, 1.0, 0.0);
        assert!((h - 0.333).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hsv_to_rgb() {
        // Red
        let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
        assert!((r - 1.0).abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!(b.abs() < 0.01);
    }

    #[test]
    fn test_color_modifiers() {
        let modifiers = ColorModifiers::default();

        // Apply and remove should be inverse operations
        let (r, g, b) = (1.0, 0.5, 0.0);
        let (r_mod, g_mod, b_mod) = apply_color_modifiers(r, g, b, false, &modifiers);
        let (r_orig, g_orig, b_orig) = remove_color_modifiers(r_mod, g_mod, b_mod, false, &modifiers);

        assert!((r - r_orig).abs() < 0.01);
        assert!((g - g_orig).abs() < 0.01);
        assert!((b - b_orig).abs() < 0.01);
    }
}
