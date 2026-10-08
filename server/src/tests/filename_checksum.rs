#[cfg(test)]
mod tests {
    use crate::backend::{non_ascii_checksum_filename_diagnostic, relative_to_root};
    use crate::validation::advanced_validation::NON_ASCII_CHECKSUM_FILENAME;
    use tower_lsp_server::ls_types::{DiagnosticSeverity, NumberOrString};

    fn fires(rel: &str) -> bool {
        non_ascii_checksum_filename_diagnostic(rel).is_some()
    }

    /// The exact file names from Hearts of Minecraft that broke Linux↔Windows
    /// multiplayer checksums (issue #154) — all must fire.
    #[test]
    fn real_world_offenders_fire() {
        for rel in [
            "common/countries/Vésnoran Guard.txt",
            "common/names/vésnoran_names.txt",
            "history/countries/VNG - Vésnoran Guard.txt",
            "history/states/121-§GDivine Island of Emeraldia§!.txt",
            "history/states/293-Transéletvonal.txt",
            "history/states/311-East Österland.txt",
            "history/states/714-Mittel-Königs-Gau.txt",
            "history/states/778-Côte Rocheuse.txt",
            "history/states/874-Sómellék.txt",
            "history/states/887-Próxima.txt",
            "history/states/895-Hügelland.txt",
            "history/states/924-Ostküsten-Bezirk.txt",
        ] {
            assert!(fires(rel), "must fire: {rel}");
        }
    }

    #[test]
    fn ascii_paths_pass() {
        for rel in [
            "history/states/293-Transeletvonal.txt",
            "common/countries/Vesnoran Guard.txt",
            "common/defines/HoM_defines.lua",
            "map/definition.csv",
            "map/default.map",
            "events/my_events.txt",
        ] {
            assert!(!fires(rel), "must not fire: {rel}");
        }
    }

    #[test]
    fn out_of_scope_directories_pass() {
        for rel in [
            "localisation/english/café_l_english.yml",
            "gfx/interface/é.tga",
            "history_notes/é.txt",
            "commonx/é.txt",    // boundary: must not prefix-match "common"
            "gfx/common/é.txt", // "common" must be the FIRST path segment
        ] {
            assert!(!fires(rel), "must not fire: {rel}");
        }
    }

    #[test]
    fn out_of_scope_extensions_pass() {
        for rel in [
            "history/units/é.asset",
            "common/é.json",
            "map/é.png",
            "common/é.txt.bak", // the extension is "bak", not "txt"
        ] {
            assert!(!fires(rel), "must not fire: {rel}");
        }
    }

    /// Evidence covers file names only; a non-ASCII parent directory with an
    /// ASCII file name must stay silent (false negatives beat false positives).
    #[test]
    fn directory_names_are_not_checked() {
        assert!(!fires("common/námés/foo.txt"));
        assert!(!fires("history/states-é/foo.txt"));
    }

    #[test]
    fn windows_separators_fire_too() {
        assert!(fires(r"history\states\874-Sómellék.txt"));
    }

    #[test]
    fn extension_case_insensitive() {
        assert!(fires("history/states/é.TXT"));
    }

    #[test]
    fn diagnostic_shape() {
        let d = non_ascii_checksum_filename_diagnostic("history/states/874-Sómellék.txt").unwrap();
        assert_eq!(d.severity, Some(DiagnosticSeverity::WARNING));
        match d.code {
            Some(NumberOrString::String(ref c)) => assert_eq!(c, NON_ASCII_CHECKSUM_FILENAME),
            other => panic!("unexpected code: {other:?}"),
        }
        assert_eq!(d.range.start.line, 0);
        assert!(d.message.contains("multiplayer"));
    }

    #[test]
    fn relative_to_root_strips() {
        assert_eq!(
            relative_to_root("/m/history/x.txt", "/m"),
            Some("history/x.txt".to_string())
        );
        assert_eq!(
            relative_to_root("/m/sub/history/x.txt", "/m/sub"),
            Some("history/x.txt".to_string())
        );
        assert_eq!(relative_to_root("/m2/x.txt", "/m"), None);
        assert_eq!(relative_to_root("/m", "/m"), None); // the root itself, not a file
        assert_eq!(
            relative_to_root(r"C:\mod\history\x.txt", r"C:\mod"),
            Some("history/x.txt".to_string())
        );
    }
}
