pub fn format_modifier_value(key: &str, val: f64, format_str: Option<&String>) -> String {
    let mut is_percentage = key.ends_with("factor");
    let mut display_digits = 1;
    let mut is_double_percent = false;

    if let Some(fmt) = format_str {
        if fmt.contains("%%") {
            is_double_percent = true;
            is_percentage = false;
        } else {
            is_percentage = fmt.contains('%');
        }

        for c in fmt.chars().rev() {
            if c.is_ascii_digit() {
                display_digits = c.to_digit(10).unwrap() as usize;
                break;
            }
        }
    }

    let mut actual_val = val;
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
