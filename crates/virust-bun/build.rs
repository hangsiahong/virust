use std::fs;
use std::path::PathBuf;

fn main() {
    let bundled_dir = PathBuf::from("bundled");
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bundled");

    // Copy bundled files to output directory
    if bundled_dir.exists() {
        println!("cargo:rerun-if-changed=bundled/renderer.js");
        println!("cargo:rerun-if-changed=bundled/package.json");
    }
}
