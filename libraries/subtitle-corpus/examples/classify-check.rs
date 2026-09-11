//! Print what `classify` decides for the films named on the command line
//! (imdb id, path, original language triples), without touching the corpus.
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    for triple in args.chunks(3) {
        let [imdb, path, language] = triple else {
            anyhow::bail!("expected imdb path language triples");
        };
        let source = subtitle_corpus::library::classify(
            imdb,
            &PathBuf::from(path),
            language,
            Path::new("./generate-data/data"),
        )?;
        println!("{imdb} {language}: {source:?}");
    }
    Ok(())
}
