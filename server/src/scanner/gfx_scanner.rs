use crate::data::interner::InternedStr;
use crate::parser::ast;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ColorCode {
    pub symbol: String,
    pub rgb: (u8, u8, u8),
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_color_code_files<F>(files: &[PathBuf], filter: &F) -> HashMap<String, ColorCode>
where
    F: Fn(&Path) -> bool,
{
    let mut codes = HashMap::new();
    let re_entry =
        regex::Regex::new(r#""?(.)"?\s*=\s*\{\s*(\d{1,3})\s+(\d{1,3})\s+(\d{1,3})\s*\}"#).unwrap();

    crate::utils::fs_util::parse_winning_files(files, filter, |path, content| {
        let path_str = path.to_string_lossy().to_string();
        for (_start, block) in find_textcolors_blocks(&content) {
            for cap in re_entry.captures_iter(&block) {
                let symbol = cap[1].to_string();
                let r: u8 = cap[2].parse().unwrap_or(0);
                let g: u8 = cap[3].parse().unwrap_or(0);
                let b: u8 = cap[4].parse().unwrap_or(0);

                let line = block[..cap.get(0).unwrap().start()].matches('\n').count() as u32;
                let col = block[..cap.get(0).unwrap().start()]
                    .rfind('\n')
                    .map(|i| cap.get(0).unwrap().start() - i - 1)
                    .unwrap_or(cap.get(0).unwrap().start());

                codes.insert(
                    symbol.clone(),
                    ColorCode {
                        symbol,
                        rgb: (r, g, b),
                        path: path_str.clone().into(),
                        range: ast::Range {
                            start_line: _start + line,
                            start_col: col as u32,
                            end_line: _start + line,
                            end_col: col as u32 + cap.get(0).unwrap().len() as u32,
                        },
                    },
                );
            }
        }
    });

    codes
}

fn find_textcolors_blocks(content: &str) -> Vec<(u32, String)> {
    let mut blocks = Vec::new();
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    const OPEN: &str = "textcolors = {";

    while i < len {
        if let Some(pos) = content[i..].find(OPEN) {
            let block_start_global = i + pos;
            let block_content_start = block_start_global + OPEN.len();
            let line_offset = content[..block_start_global].matches('\n').count() as u32;

            let mut depth = 1u32;
            let mut j = block_content_start;
            while j < len && depth > 0 {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                j += 1;
            }
            // `j` is one past the closing `}` when the block is closed, and
            // `len` when it is not (truncated file, or a `{` inside a string).
            // The body ends before the brace only when one was actually found:
            // the old `j - 1` gave `len - 1` for an unterminated block, which
            // can split a multi-byte char and — on a file ending at the opening
            // brace — landed *before* the body start. Both panicked the slice
            // below, and a panic here double-panics the orchestrator (the scan
            // macro `.unwrap()`s its `spawn_blocking` handle).
            let block_end = if depth == 0 { j - 1 } else { len };

            blocks.push((
                line_offset,
                content[block_content_start..block_end].to_string(),
            ));

            i = block_end;
        } else {
            break;
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_textcolors_blocks_never_panics_on_truncated_input() {
        // Regression: `block_end` was `j - 1`, which for an unterminated block
        // is `len - 1` — an index that can split a multi-byte char, and which
        // lands *before* the body start when the file ends at the opening
        // brace. Both panicked the body slice.
        assert_eq!(
            find_textcolors_blocks("textcolors = {"),
            vec![(0, String::new())]
        );

        // Unterminated, ending inside a multi-byte char: `é` spans the last two
        // bytes, so `len - 1` was not a char boundary.
        let blocks = find_textcolors_blocks("textcolors = {\n\tx = é");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].1, "\n\tx = é");

        // Unterminated but ASCII-terminated — no panic either way; the body
        // runs to the end of the file.
        assert_eq!(
            find_textcolors_blocks("textcolors = {\n\tx = 1"),
            vec![(0, "\n\tx = 1".to_string())]
        );
    }

    #[test]
    fn test_find_textcolors_blocks_wellformed() {
        // Control: a closed block yields its body without the braces, and
        // non-ASCII entries survive intact (the closing brace is ASCII, so the
        // body end is always a char boundary).
        let blocks =
            find_textcolors_blocks("textcolors = {\n\tred = { 255 0 0 }\n\tcafé = { 1 2 3 }\n}\n");
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].0, 0);
        assert_eq!(blocks[0].1, "\n\tred = { 255 0 0 }\n\tcafé = { 1 2 3 }\n");

        // Nested braces are tracked by depth, not by the first `}`.
        let blocks = find_textcolors_blocks(
            "textcolors = {\n\ta = { 1 2 3 }\n\tb = {\n\t\tc = { 4 5 6 }\n\t}\n}",
        );
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].1.contains("c = { 4 5 6 }"));

        // Two blocks: the second one's line offset is absolute.
        let blocks = find_textcolors_blocks(
            "textcolors = {\n\ta = { 1 2 3 }\n}\ntextcolors = {\n\tb = { 4 5 6 }\n}",
        );
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[1].0, 3);

        // No blocks at all.
        assert!(find_textcolors_blocks("spriteTypes = { }").is_empty());
    }
}
