//! Build course strokes using the shared osmo download cache.

use anyhow::{Context, Result, ensure};
use language_utils::{Language, StrokeTable};
use osmo::Store;

pub async fn table<'a>(
    language: Language,
    texts: impl IntoIterator<Item = &'a str>,
    store: &Store,
) -> Result<StrokeTable> {
    let client = reqwest::Client::new();
    stroke_order_sources::table(language, texts, |url| {
        let client = &client;
        async move {
            let key = format!("strokes/{url}");
            if let Some(bytes) = store.read(&key).await {
                return Ok(bytes);
            }
            ensure!(
                !crate::cache_only(),
                "stroke source missing from cache: {url}"
            );
            let bytes = client
                .get(url)
                .send()
                .await
                .with_context(|| format!("Failed to fetch stroke source {url}"))?
                .error_for_status()?
                .bytes()
                .await?
                .to_vec();
            if let Err(error) = store.write(&key, &bytes).await {
                eprintln!("Warning: Failed to cache stroke source {url}: {error}");
            }
            Ok(bytes)
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Downloads pinned sources once, then rebuilds with networking forbidden.
    #[tokio::test]
    #[ignore = "downloads real Japanese stroke sources"]
    async fn japanese_download_and_cache() {
        let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
        let store = Store::open(dir.path());
        let texts = ["日本語を学ぶ。", "猫"];
        let downloaded = table(Language::Japanese, texts, &store).await.unwrap();
        assert!(downloaded.contains_key("日"));
        assert!(downloaded.contains_key("猫"));
        for url in [
            stroke_order_sources::KANJIVG_URL,
            stroke_order_sources::MMAH_URL,
        ] {
            assert!(store.read(&format!("strokes/{url}")).await.is_some());
        }
        crate::set_cache_only(true);
        let cached = table(Language::Japanese, texts, &store).await;
        crate::set_cache_only(false);
        assert_eq!(downloaded, cached.unwrap());
        let forms = downloaded.values().flatten().count();
        let points: usize = downloaded
            .values()
            .flatten()
            .flat_map(|glyph| &glyph.strokes)
            .map(|stroke| stroke.points.len())
            .sum();
        println!(
            "Japanese strokes: {} units, {forms} forms, {points} points; cached rebuild identical",
            downloaded.len()
        );
    }
}
