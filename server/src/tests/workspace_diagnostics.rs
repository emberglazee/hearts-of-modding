#[cfg(test)]
mod tests {
    use crate::backend::cleared_problem_uris;
    use std::collections::HashSet;

    fn set(uris: &[&str]) -> HashSet<String> {
        uris.iter().map(|u| u.to_string()).collect()
    }

    #[test]
    fn test_fixed_file_is_cleared() {
        // Flagged by the last scan, clean now (e.g. fixed via git checkout).
        let previous = set(&["file:///a.txt", "file:///b.txt"]);
        let current = set(&["file:///b.txt"]);
        let cleared = cleared_problem_uris(&previous, &current);
        assert_eq!(cleared, vec!["file:///a.txt".to_string()]);
    }

    #[test]
    fn test_still_broken_file_is_untouched() {
        let previous = set(&["file:///a.txt", "file:///b.txt"]);
        let current = set(&["file:///a.txt", "file:///b.txt"]);
        assert!(cleared_problem_uris(&previous, &current).is_empty());
    }

    #[test]
    fn test_deleted_file_is_cleared() {
        // Vanished from the scan entirely (deleted/unreadable) — must still
        // get `[]`, otherwise its Problems entries stick forever.
        let previous = set(&["file:///gone.txt", "file:///b.txt"]);
        let current = set(&["file:///b.txt"]);
        let cleared = cleared_problem_uris(&previous, &current);
        assert_eq!(cleared, vec!["file:///gone.txt".to_string()]);
    }

    #[test]
    fn test_new_problem_is_not_cleared() {
        // Only in the current set — needs its diagnostics published, never `[]`.
        let previous = set(&["file:///b.txt"]);
        let current = set(&["file:///b.txt", "file:///new.txt"]);
        assert!(cleared_problem_uris(&previous, &current).is_empty());
    }
}
