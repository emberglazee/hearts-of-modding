//! The `analyze` route is a second front door to the same rules, so it gets a
//! test that proves the door is open: a fixture mod with one known error must
//! come back as JSON with that error in it, and a clean file must exit 0.

use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(name: &str, body: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("hom-analyze-{}-{name}", std::process::id()));
    let characters = root.join("common/characters");
    std::fs::create_dir_all(&characters).expect("fixture dir");
    let file = characters.join("zz_test.txt");
    std::fs::write(&file, body).expect("fixture file");
    file
}

fn analyze(file: &Path) -> (Option<i32>, serde_json::Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_hom-lsp"))
        .arg("analyze")
        .arg(file)
        // No game path: the data tables are compiled in, so a fixture can be
        // validated on a machine that has never installed the game.
        .arg("--no-game")
        .arg("--json")
        .output()
        .expect("run hom-lsp analyze");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "stdout is not JSON ({e}): {stdout}\nstderr: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (out.status.code(), json)
}

#[test]
fn a_wrong_leader_role_skill_is_reported_and_exits_nonzero() {
    let file = fixture(
        "wrong-skill",
        "characters = {\n\
         \tZZ_admiral = {\n\
         \t\tname = ZZ_admiral\n\
         \t\tnavy_leader = {\n\
         \t\t\tskill = 4\n\
         \t\t\tplanning_skill = 2\n\
         \t\t}\n\
         \t}\n\
         }\n",
    );
    let (code, json) = analyze(&file);
    assert_eq!(code, Some(1), "diagnostics must exit 1: {json}");
    let diags = json["diagnostics"].as_array().expect("diagnostics array");
    let codes: Vec<&str> = diags.iter().filter_map(|d| d["code"].as_str()).collect();
    assert!(
        codes.contains(&"HOM1005"),
        "expected HOM1005, got {codes:?}: {json}"
    );
    let hom = diags
        .iter()
        .find(|d| d["code"] == "HOM1005")
        .expect("HOM1005 entry");
    assert_eq!(hom["line"], 6, "reported line: {json}");
}

#[test]
fn a_clean_character_file_exits_zero() {
    let file = fixture(
        "clean",
        "characters = {\n\
         \tZZ_admiral = {\n\
         \t\tname = ZZ_admiral\n\
         \t\tnavy_leader = {\n\
         \t\t\tskill = 4\n\
         \t\t\tmaneuvering_skill = 2\n\
         \t\t}\n\
         \t}\n\
         }\n",
    );
    let (code, json) = analyze(&file);
    assert_eq!(code, Some(0), "a clean file must exit 0: {json}");
    assert_eq!(json["diagnostics"].as_array().map(Vec::len), Some(0));
}

#[test]
fn tokens_are_decoded_with_their_types() {
    let file = fixture(
        "tokens",
        "characters = {\n\
         \tZZ_admiral = {\n\
         \t\tnavy_leader = {\n\
         \t\t\tskill = 4\n\
         \t\t}\n\
         \t}\n\
         }\n",
    );
    let out = Command::new(env!("CARGO_BIN_EXE_hom-lsp"))
        .arg("analyze")
        .arg(&file)
        .arg("--no-game")
        .arg("--json")
        .arg("--tokens")
        .output()
        .expect("run hom-lsp analyze");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    let tokens = json["tokens"].as_array().expect("tokens array");
    let skill = tokens
        .iter()
        .find(|t| t["text"] == "skill")
        .unwrap_or_else(|| panic!("no `skill` token: {json}"));
    assert_eq!(
        skill["type"], "property",
        "a documented block parameter must highlight as a property: {json}"
    );
}
