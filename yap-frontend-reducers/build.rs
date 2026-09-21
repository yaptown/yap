//! Writes the web's color tokens from `src/palette.rs` so a plain cargo build
//! keeps `yap-frontend/src/tokens.css` (gitignored) in step with the palette.
#[path = "src/palette.rs"]
#[allow(dead_code)]
mod palette;

fn main() {
    let out = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../yap-frontend/src/tokens.css"
    );
    println!("cargo::rerun-if-changed=src/palette.rs");
    println!("cargo::rerun-if-changed={out}");
    // Builds without the web tree beside us (the yap-mcp Docker image) have
    // nothing to keep in step.
    if !std::path::Path::new(out)
        .parent()
        .is_some_and(|dir| dir.is_dir())
    {
        return;
    }
    let css = palette::render_css();
    if std::fs::read_to_string(out).ok().as_deref() != Some(css.as_str()) {
        std::fs::write(out, css).expect("write tokens.css");
    }
}
