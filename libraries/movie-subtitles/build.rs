use std::{env, fs, path::PathBuf};

fn main() {
    let dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("corrections");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
        .collect();
    files.sort();
    let mut table = String::from("&[\n");
    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        table.push_str(&format!(
            "({:?}, include_str!({:?})),\n",
            path.file_stem().unwrap().to_str().unwrap(),
            path.to_str().unwrap()
        ));
    }
    table.push_str("]\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("corrections.rs"),
        table,
    )
    .unwrap();
}
