pub fn format_csv(content: &str, separator: char) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut max_column_widths: Vec<usize> = Vec::new();

    // First pass: find maximum width for each column
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split(separator).collect();
        for (i, part) in parts.iter().enumerate() {
            let width = part.trim().len();
            if i >= max_column_widths.len() {
                max_column_widths.push(width);
            } else if width > max_column_widths[i] {
                max_column_widths[i] = width;
            }
        }
    }

    // Second pass: format lines
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            lines.push("".to_string());
            continue;
        }
        if trimmed.starts_with('#') {
            lines.push(trimmed.to_string());
            continue;
        }

        let parts: Vec<&str> = trimmed.split(separator).collect();
        let mut formatted_parts: Vec<String> = Vec::new();
        for (i, part) in parts.iter().enumerate() {
            let part_trimmed = part.trim();
            if i < max_column_widths.len() {
                formatted_parts.push(format!(
                    "{:width$}",
                    part_trimmed,
                    width = max_column_widths[i]
                ));
            } else {
                formatted_parts.push(part_trimmed.to_string());
            }
        }
        lines.push(formatted_parts.join(&separator.to_string()));
    }

    if !lines.is_empty() && !content.ends_with('\n') {
        lines.join("\n")
    } else {
        let mut res = lines.join("\n");
        if !res.is_empty() && content.ends_with('\n') {
            res.push('\n');
        }
        res
    }
}
