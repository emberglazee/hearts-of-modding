use crate::ast;
use crate::interner::InternedStr;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ColorCode {
    #[allow(dead_code)]
    pub symbol: String,
    pub rgb: (u8, u8, u8),
    pub path: InternedStr,
    pub range: ast::Range,
}

pub fn scan_color_codes<F>(roots: &[std::path::PathBuf], filter: &F) -> HashMap<String, ColorCode>
where
    F: Fn(&Path) -> bool,
{
    let mut codes = HashMap::new();
    let re_entry =
        regex::Regex::new(r#""?(.)"?\s*=\s*\{\s*(\d{1,3})\s+(\d{1,3})\s+(\d{1,3})\s*\}"#).unwrap();

    for root in roots {
        let dir = root.join("interface");
        if !dir.exists() {
            continue;
        }
        crate::fs_util::walk_and_parse_files(&dir, &["gfx"], filter, |path, content| {
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
    }

    codes
}

fn find_textcolors_blocks(content: &str) -> Vec<(u32, String)> {
    let mut blocks = Vec::new();
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if let Some(pos) = content[i..].find("textcolors = {") {
            let block_start_global = i + pos;
            let block_content_start = block_start_global + "textcolors = {".len();
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
            let block_end = j - 1;

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
