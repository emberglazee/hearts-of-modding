use crate::ast;
use std::path::{Path, PathBuf};
use tower_lsp_server::ls_types::{
    DiagnosticRelatedInformation, DiagnosticTag, Location, Position, Range, Uri,
};

pub fn ast_range_to_lsp(range: &ast::Range) -> Range {
    Range {
        start: Position {
            line: range.start_line,
            character: range.start_col,
        },
        end: Position {
            line: range.end_line,
            character: range.end_col,
        },
    }
}

pub fn ast_tag_to_lsp(tag: &ast::DiagnosticTag) -> DiagnosticTag {
    match tag {
        ast::DiagnosticTag::Unnecessary => DiagnosticTag::UNNECESSARY,
        ast::DiagnosticTag::Deprecated => DiagnosticTag::DEPRECATED,
    }
}

pub fn ast_related_info_to_lsp(
    info: &ast::DiagnosticRelatedInformation,
) -> DiagnosticRelatedInformation {
    DiagnosticRelatedInformation {
        location: Location {
            uri: info
                .location
                .uri
                .parse::<Uri>()
                .unwrap_or_else(|_| Uri::from_file_path(&info.location.uri).unwrap()),
            range: ast_range_to_lsp(&info.location.range),
        },
        message: info.message.clone(),
    }
}

pub fn ast_range_to_lsp_location(range: &ast::Range, path: &str) -> Location {
    Location {
        uri: Uri::from_file_path(
            Path::new(path)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(path)),
        )
        .unwrap(),
        range: ast_range_to_lsp(range),
    }
}
