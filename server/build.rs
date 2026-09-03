//! Generates a whitespace-minified copy of the HOI4 data JSON for embedding.
//!
//! `assets/hoi4_data.json` is loaded at compile time via `include_str!` in
//! `src/data/hoi4_data.rs`, so whatever is on disk during compilation ends up
//! in every shipped binary. This build script rewrites the parsed JSON into
//! `OUT_DIR/hoi4_data.min.json` (Cargo creates `OUT_DIR` per build and
//! cleans it up — no temp-file lifecycle to manage), shrinking the embedded
//! data — and therefore every release binary — by the pretty-print whitespace
//! (~144KB at the time of writing) without touching the human-readable source
//! file or requiring a pipeline step before `cargo build`.
//!
//! The round-trip goes through `serde_json::Value`, and the runtime parser in
//! `hoi4_data.rs` is the same `serde_json`, so the data the server sees is
//! byte-for-byte equivalent — only insignificant whitespace is removed.

use std::path::Path;

fn main() {
    let src = Path::new("assets/hoi4_data.json");
    println!("cargo:rerun-if-changed={}", src.display());

    let raw = std::fs::read_to_string(src)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", src.display()));
    let value: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", src.display()));
    let minified = serde_json::to_string(&value)
        .unwrap_or_else(|e| panic!("failed to serialize {}: {e}", src.display()));

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    std::fs::write(Path::new(&out_dir).join("hoi4_data.min.json"), minified)
        .unwrap_or_else(|e| panic!("failed to write minified data: {e}"));
}
