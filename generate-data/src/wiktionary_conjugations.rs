use anyhow::Context as _;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use std::path::Path;

pub mod french {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct FrenchVerbConjugation {
        pub infinitive: String,
        pub present_participle: String,
        pub past_participle: String,

        // Indicative mood - simple tenses (6 forms each: je, tu, il, nous, vous, ils)
        pub indicative_present: [String; 6],
        pub indicative_imperfect: [String; 6],
        pub indicative_past_historic: [String; 6],
        pub indicative_future: [String; 6],
        pub indicative_conditional: [String; 6],

        // Subjunctive mood - simple tenses (6 forms each)
        pub subjunctive_present: [String; 6],
        pub subjunctive_imperfect: [String; 6],

        // Imperative mood (3 forms: tu, nous, vous)
        pub imperative: [String; 3],

        // Auxiliary verb for compound tenses
        pub auxiliary: Auxiliary,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum Auxiliary {
        Avoir,
        Être,
    }

    /// Extract the French language section from a Wiktionary page
    fn extract_french_section(document: &Html) -> anyhow::Result<Html> {
        // Find the h2 heading with id="French"
        let h2_selector = Selector::parse("h2#French").unwrap();

        let french_heading = document
            .select(&h2_selector)
            .next()
            .context("Could not find French language section")?;

        // Collect all content until the next h2 (language section)
        let mut french_content = String::new();
        let mut current = french_heading.parent();

        while let Some(node) = current {
            current = node.next_sibling();
            if let Some(current_node) = current {
                // Stop if we hit another h2 (next language section)
                if let Some(elem) = ElementRef::wrap(current_node) {
                    if elem.value().name() == "div" {
                        if let Some(first_child) = elem.first_child() {
                            if let Some(child_elem) = ElementRef::wrap(first_child) {
                                if child_elem.value().name() == "h2" {
                                    break;
                                }
                            }
                        }
                    }
                    french_content.push_str(&elem.html());
                }
            }
        }

        Ok(Html::parse_fragment(&french_content))
    }

    /// Parse a French verb conjugation table from Wiktionary HTML
    pub fn parse_french_verb_conjugation(
        html: &str,
        verb: &str,
    ) -> anyhow::Result<FrenchVerbConjugation> {
        let document = Html::parse_document(html);

        // Extract only the French language section to avoid dialect tables
        let french_section = extract_french_section(&document)?;

        // Parse infinitive (it's the verb itself)
        let infinitive = verb.to_string();

        // Parse present participle
        let present_participle = parse_present_participle(&french_section)?;

        // Parse past participle
        let past_participle = parse_past_participle(&french_section)?;

        // Parse auxiliary verb from compound tense row
        let auxiliary = parse_auxiliary(&french_section)?;

        // Parse indicative tenses
        let indicative_present = parse_tense(&french_section, "present", "indicative")?;
        let indicative_imperfect = parse_tense(&french_section, "imperfect", "indicative")?;
        let indicative_past_historic = parse_tense(&french_section, "past historic", "indicative")?;
        let indicative_future = parse_tense(&french_section, "future", "indicative")?;
        let indicative_conditional = parse_tense(&french_section, "conditional", "indicative")?;

        // Parse subjunctive tenses
        let subjunctive_present = parse_tense(&french_section, "present", "subjunctive")?;
        let subjunctive_imperfect = parse_tense(&french_section, "imperfect", "subjunctive")?;

        // Parse imperative
        let imperative = parse_imperative(&french_section)?;

        Ok(FrenchVerbConjugation {
            infinitive,
            present_participle,
            past_participle,
            indicative_present,
            indicative_imperfect,
            indicative_past_historic,
            indicative_future,
            indicative_conditional,
            subjunctive_present,
            subjunctive_imperfect,
            imperative,
            auxiliary,
        })
    }

    fn parse_present_participle(document: &Html) -> anyhow::Result<String> {
        // Look for the present participle row in the table
        // Format: <span class="Latn form-of lang-fr ppr-form-of">
        let selector = Selector::parse("span.ppr-form-of a").unwrap();

        document
            .select(&selector)
            .next()
            .and_then(|el| el.text().next())
            .map(|s| s.to_string())
            .context("Failed to find present participle")
    }

    fn parse_past_participle(document: &Html) -> anyhow::Result<String> {
        // Look for the past participle row in the table
        // Format: <span class="Latn form-of lang-fr pp-form-of">
        let selector = Selector::parse("span.pp-form-of a").unwrap();

        document
            .select(&selector)
            .next()
            .and_then(|el| el.text().next())
            .map(|s| s.to_string())
            .context("Failed to find past participle")
    }

    fn parse_auxiliary(document: &Html) -> anyhow::Result<Auxiliary> {
        // Look for the compound tense row which mentions the auxiliary
        // Format: present indicative of <i><a href="/wiki/avoir">avoir</a></i> + past participle
        // or: present indicative of <i><a href="/wiki/être">être</a></i> + past participle

        let selector = Selector::parse("th.roa-compound-row").unwrap();

        for element in document.select(&selector) {
            let text = element.text().collect::<String>();
            if text.contains("avoir") {
                return Ok(Auxiliary::Avoir);
            }
            if text.contains("être") {
                return Ok(Auxiliary::Être);
            }
        }

        anyhow::bail!("Failed to find auxiliary verb")
    }

    fn parse_tense(document: &Html, tense: &str, mood: &str) -> anyhow::Result<[String; 6]> {
        // Find the row with the tense name
        let th_selector =
            Selector::parse("th.roa-indicative-left-rail, th.roa-subjunctive-left-rail").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let mood_prefix = match mood {
            "indicative" => "roa-indicative-left-rail",
            "subjunctive" => "roa-subjunctive-left-rail",
            _ => anyhow::bail!("Unknown mood: {}", mood),
        };

        // Find the header for this tense
        let mut tense_row_ref = None;
        for th in document.select(&th_selector) {
            let text = th.text().collect::<String>().to_lowercase();
            if text.contains(&tense.to_lowercase())
                && th.attr("class").unwrap_or("").contains(mood_prefix)
            {
                // Get the parent tr element
                if let Some(parent) = th.parent() {
                    if parent.value().as_element().map(|e| e.name()) == Some("tr") {
                        tense_row_ref = Some(parent);
                        break;
                    }
                }
            }
        }

        let tense_row =
            tense_row_ref.context(format!("Failed to find tense row for {mood} {tense}"))?;

        // Extract the 6 conjugated forms from the td elements in this row
        let mut forms = Vec::new();

        // Iterate through children of the tr element
        for child in tense_row.children() {
            if let Some(element) = child.value().as_element() {
                if element.name() == "td" {
                    // Find the link inside the td
                    let td_elem = scraper::ElementRef::wrap(child).unwrap();
                    if let Some(link) = td_elem.select(&a_selector).next() {
                        if let Some(text) = link.text().next() {
                            forms.push(text.to_string());
                        }
                    }
                }
            }
        }

        if forms.len() != 6 {
            anyhow::bail!(
                "Expected 6 forms for {} {}, found {}",
                mood,
                tense,
                forms.len()
            );
        }

        Ok([
            forms[0].clone(),
            forms[1].clone(),
            forms[2].clone(),
            forms[3].clone(),
            forms[4].clone(),
            forms[5].clone(),
        ])
    }

    fn parse_imperative(document: &Html) -> anyhow::Result<[String; 3]> {
        // Find the imperative row (simple)
        let th_selector = Selector::parse("th.roa-imperative-left-rail").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let mut imperative_row_ref = None;
        for th in document.select(&th_selector) {
            let text = th.text().collect::<String>().to_lowercase();
            if text.contains("simple") {
                // Get the parent tr element
                if let Some(parent) = th.parent() {
                    if parent.value().as_element().map(|e| e.name()) == Some("tr") {
                        imperative_row_ref = Some(parent);
                        break;
                    }
                }
            }
        }

        let imperative_row = imperative_row_ref.context("Failed to find imperative row")?;

        // Extract the 3 forms (skip the first and last which are "—")
        let mut forms = Vec::new();

        // Iterate through children of the tr element
        for child in imperative_row.children() {
            if let Some(element) = child.value().as_element() {
                if element.name() == "td" {
                    let td_elem = scraper::ElementRef::wrap(child).unwrap();
                    let text = td_elem.text().collect::<String>().trim().to_string();

                    // Skip empty cells marked with "—"
                    if text == "—" {
                        continue;
                    }

                    // Extract the link text (the conjugated form)
                    if let Some(link) = td_elem.select(&a_selector).next() {
                        if let Some(text) = link.text().next() {
                            forms.push(text.to_string());
                        }
                    }
                }
            }
        }

        if forms.len() != 3 {
            anyhow::bail!("Expected 3 forms for imperative, found {}", forms.len());
        }

        Ok([forms[0].clone(), forms[1].clone(), forms[2].clone()])
    }

    /// Fetch French verb conjugations from Wiktionary with HTML caching
    ///
    /// # Arguments
    /// * `verbs` - List of verb infinitives to fetch
    /// * `cache_dir` - Directory to cache HTML files (e.g., `.cache/wiktionary/french/`)
    ///
    /// # Returns
    /// HashMap mapping verb infinitives to their conjugations
    pub async fn fetch_french_verb_conjugations(
        verbs: &[String],
        cache_dir: &Path,
    ) -> anyhow::Result<HashMap<String, FrenchVerbConjugation>> {
        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(cache_dir)?;

        let mut results = HashMap::new();
        let mut to_fetch = Vec::new();

        // Check which verbs we already have cached
        for verb in verbs {
            let cache_file = cache_dir.join(format!("{}.html", verb));
            if cache_file.exists() {
                // Try to parse from cached HTML
                match std::fs::read_to_string(&cache_file) {
                    Ok(html) => match parse_french_verb_conjugation(&html, verb) {
                        Ok(conjugation) => {
                            results.insert(verb.clone(), conjugation);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse cached HTML for {verb}: {e}");
                            to_fetch.push(verb);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read cached HTML for {verb}: {e}");
                        to_fetch.push(verb);
                    }
                }
            } else {
                to_fetch.push(verb);
            }
        }

        if to_fetch.is_empty() {
            println!("All {} French verbs loaded from cache", results.len());
            return Ok(results);
        }

        println!(
            "Fetching {} French verb pages from Wiktionary...",
            to_fetch.len()
        );

        // Build HTTP client with YapBot user agent
        let client = reqwest::Client::builder()
            .user_agent("YapBot/1.0 (https://yap.town) reqwest/0.11")
            .build()
            .context("Failed to build HTTP client")?;

        // Fetch each verb
        for (i, verb) in to_fetch.iter().enumerate() {
            if i > 0 && i % 10 == 0 {
                println!("  Fetched {}/{} verbs...", i, to_fetch.len());
                // Rate limiting: sleep between batches
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            // Fetch the Wiktionary page
            let url = format!("https://en.wiktionary.org/wiki/{verb}");
            let response = match client.get(&url).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Failed to fetch {verb}: {e}");
                    continue;
                }
            };

            let html = match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("Failed to read HTML for {verb}: {e}");
                    continue;
                }
            };

            // Save HTML to cache
            let cache_file = cache_dir.join(format!("{}.html", verb));
            if let Err(e) = std::fs::write(&cache_file, &html) {
                eprintln!("Failed to write cache file for {verb}: {e}");
            }

            // Parse the conjugation
            match parse_french_verb_conjugation(&html, verb) {
                Ok(conjugation) => {
                    results.insert((*verb).clone(), conjugation);
                }
                Err(e) => {
                    eprintln!("Failed to parse conjugation for {verb}: {e}");
                }
            }
        }

        println!(
            "Finished fetching French conjugations. Total: {}",
            results.len()
        );

        Ok(results)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;

        #[test]
        fn test_parse_boire() {
            let html = fs::read_to_string("src/wiktionary-examples/fra/boire.txt")
                .expect("Failed to read boire.txt");

            let conjugation = parse_french_verb_conjugation(&html, "boire")
                .expect("Failed to parse boire conjugation");

            assert_eq!(conjugation.infinitive, "boire");
            assert_eq!(conjugation.present_participle, "buvant");
            assert_eq!(conjugation.past_participle, "bu");
            assert_eq!(conjugation.auxiliary, Auxiliary::Avoir);

            // Check indicative present
            assert_eq!(conjugation.indicative_present[0], "bois"); // je
            assert_eq!(conjugation.indicative_present[1], "bois"); // tu
            assert_eq!(conjugation.indicative_present[2], "boit"); // il
            assert_eq!(conjugation.indicative_present[3], "buvons"); // nous
            assert_eq!(conjugation.indicative_present[4], "buvez"); // vous
            assert_eq!(conjugation.indicative_present[5], "boivent"); // ils

            // Check future
            assert_eq!(conjugation.indicative_future[0], "boirai");
            assert_eq!(conjugation.indicative_future[1], "boiras");

            // Check imperative
            assert_eq!(conjugation.imperative[0], "bois"); // tu
            assert_eq!(conjugation.imperative[1], "buvons"); // nous
            assert_eq!(conjugation.imperative[2], "buvez"); // vous
        }

        #[test]
        fn test_parse_aller() {
            let html = fs::read_to_string("src/wiktionary-examples/fra/aller.txt")
                .expect("Failed to read aller.txt");

            let conjugation = parse_french_verb_conjugation(&html, "aller")
                .expect("Failed to parse aller conjugation");

            assert_eq!(conjugation.infinitive, "aller");
            assert_eq!(conjugation.auxiliary, Auxiliary::Être);

            // Check indicative present (irregular)
            assert_eq!(conjugation.indicative_present[0], "vais"); // je
            assert_eq!(conjugation.indicative_present[1], "vas"); // tu
            assert_eq!(conjugation.indicative_present[2], "va"); // il
            assert_eq!(conjugation.indicative_present[3], "allons"); // nous
            assert_eq!(conjugation.indicative_present[4], "allez"); // vous
            assert_eq!(conjugation.indicative_present[5], "vont"); // ils

            // Check future (irregular stem)
            assert_eq!(conjugation.indicative_future[0], "irai");
            assert_eq!(conjugation.indicative_future[1], "iras");
        }

        #[test]
        fn test_parse_avoir() {
            let html = fs::read_to_string("src/wiktionary-examples/fra/avoir.txt")
                .expect("Failed to read avoir.txt");

            let conjugation = parse_french_verb_conjugation(&html, "avoir")
                .expect("Failed to parse avoir conjugation");

            assert_eq!(conjugation.infinitive, "avoir");
            assert_eq!(conjugation.auxiliary, Auxiliary::Avoir);

            // Check indicative present
            assert_eq!(conjugation.indicative_present[0], "ai"); // je
            assert_eq!(conjugation.indicative_present[1], "as"); // tu
            assert_eq!(conjugation.indicative_present[2], "a"); // il
        }

        #[test]
        fn test_parse_etre() {
            let html = fs::read_to_string("src/wiktionary-examples/fra/etre.txt")
                .expect("Failed to read etre.txt");

            let conjugation = parse_french_verb_conjugation(&html, "être")
                .expect("Failed to parse être conjugation");

            assert_eq!(conjugation.infinitive, "être");
            assert_eq!(conjugation.auxiliary, Auxiliary::Avoir);

            // Check indicative present
            assert_eq!(conjugation.indicative_present[0], "suis"); // je
            assert_eq!(conjugation.indicative_present[1], "es"); // tu
            assert_eq!(conjugation.indicative_present[2], "est"); // il
        }

        #[tokio::test]
        #[ignore] // Only run manually to avoid hitting Wiktionary during CI
        async fn test_fetch_verb_conjugations() {
            use tempfile::tempdir;

            let temp_dir = tempdir().unwrap();
            let cache_dir = temp_dir.path().join("french");

            let verbs = vec![
                "parler".to_string(),
                "finir".to_string(),
                "être".to_string(),
            ];

            // First fetch
            let result = fetch_french_verb_conjugations(&verbs, &cache_dir)
                .await
                .expect("Failed to fetch conjugations");

            assert!(result.contains_key("parler"));
            assert!(result.contains_key("finir"));
            assert!(result.contains_key("être"));

            // Verify cache files were created
            assert!(cache_dir.join("parler.html").exists());
            assert!(cache_dir.join("finir.html").exists());
            assert!(cache_dir.join("être.html").exists());

            // Second fetch should use cache
            let result2 = fetch_french_verb_conjugations(&verbs, &cache_dir)
                .await
                .expect("Failed to fetch conjugations from cache");

            assert_eq!(result.len(), result2.len());
        }
    }
}

pub mod spanish {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct SpanishVerbConjugation {
        pub infinitive: String,
        pub gerund: String,
        pub past_participle_masculine_singular: String,
        pub past_participle_feminine_singular: String,

        // Indicative mood - simple tenses (6 forms each: yo, tú, él, nosotros, vosotros, ellos)
        pub indicative_present: [String; 6],
        pub indicative_imperfect: [String; 6],
        pub indicative_preterite: [String; 6],
        pub indicative_future: [String; 6],
        pub indicative_conditional: [String; 6],

        // Subjunctive mood - simple tenses (6 forms each)
        pub subjunctive_present: [String; 6],
        pub subjunctive_imperfect: [String; 6],
        pub subjunctive_future: [String; 6],

        // Imperative mood (5 forms: tú, usted, nosotros, vosotros, ustedes)
        pub imperative: [String; 5],
    }

    /// Extract the Spanish language section from a Wiktionary page
    fn extract_spanish_section(document: &Html) -> anyhow::Result<Html> {
        // Find the h2 heading with id="Spanish"
        let h2_selector = Selector::parse("h2#Spanish").unwrap();

        let spanish_heading = document
            .select(&h2_selector)
            .next()
            .context("Could not find Spanish language section")?;

        // Collect all content until the next h2 (language section)
        let mut spanish_content = String::new();
        let mut current = spanish_heading.parent();

        while let Some(node) = current {
            current = node.next_sibling();
            if let Some(current_node) = current {
                // Stop if we hit another h2 (next language section)
                if let Some(elem) = ElementRef::wrap(current_node) {
                    if elem.value().name() == "div" {
                        if let Some(first_child) = elem.first_child() {
                            if let Some(child_elem) = ElementRef::wrap(first_child) {
                                if child_elem.value().name() == "h2" {
                                    break;
                                }
                            }
                        }
                    }
                    spanish_content.push_str(&elem.html());
                }
            }
        }

        Ok(Html::parse_fragment(&spanish_content))
    }

    /// Parse a Spanish verb conjugation table from Wiktionary HTML
    pub fn parse_spanish_verb_conjugation(
        html: &str,
        verb: &str,
    ) -> anyhow::Result<SpanishVerbConjugation> {
        let document = Html::parse_document(html);

        // Extract only the Spanish language section
        let spanish_section = extract_spanish_section(&document)?;

        // Parse infinitive (it's the verb itself)
        let infinitive = verb.to_string();

        // Parse gerund
        let gerund = parse_gerund(&spanish_section)?;

        // Parse past participles (masculine/feminine singular)
        let (past_participle_masculine_singular, past_participle_feminine_singular) =
            parse_past_participles(&spanish_section)?;

        // Parse indicative tenses
        let indicative_present = parse_tense(&spanish_section, "present", "indicative")?;
        let indicative_imperfect = parse_tense(&spanish_section, "imperfect", "indicative")?;
        let indicative_preterite = parse_tense(&spanish_section, "preterite", "indicative")?;
        let indicative_future = parse_tense(&spanish_section, "future", "indicative")?;
        let indicative_conditional = parse_tense(&spanish_section, "conditional", "indicative")?;

        // Parse subjunctive tenses
        let subjunctive_present = parse_tense(&spanish_section, "present", "subjunctive")?;
        let subjunctive_imperfect = parse_tense(&spanish_section, "imperfect", "subjunctive")?;
        let subjunctive_future = parse_tense(&spanish_section, "future", "subjunctive")?;

        // Parse imperative
        let imperative = parse_imperative(&spanish_section)?;

        Ok(SpanishVerbConjugation {
            infinitive,
            gerund,
            past_participle_masculine_singular,
            past_participle_feminine_singular,
            indicative_present,
            indicative_imperfect,
            indicative_preterite,
            indicative_future,
            indicative_conditional,
            subjunctive_present,
            subjunctive_imperfect,
            subjunctive_future,
            imperative,
        })
    }

    fn parse_gerund(document: &Html) -> anyhow::Result<String> {
        // Look for the gerund row in the table
        // Format: <span class="Latn form-of lang-es gerund-...">
        let selector = Selector::parse("span[class*='gerund'] a").unwrap();

        document
            .select(&selector)
            .next()
            .and_then(|el| el.text().next())
            .map(|s| s.to_string())
            .context("Failed to find gerund")
    }

    fn parse_past_participles(document: &Html) -> anyhow::Result<(String, String)> {
        // Look for past participle rows
        // Masculine singular: <span class="Latn form-of lang-es pp-ms-form-of">
        // Feminine singular: <span class="Latn form-of lang-es pp-fs-form-of">
        let ms_selector = Selector::parse("span[class*='pp'][class*='ms'] a").unwrap();
        let fs_selector = Selector::parse("span[class*='pp'][class*='fs'] a").unwrap();

        let masculine = document
            .select(&ms_selector)
            .next()
            .and_then(|el| el.text().next())
            .map(|s| s.to_string())
            .context("Failed to find masculine past participle")?;

        let feminine = document
            .select(&fs_selector)
            .next()
            .and_then(|el| el.text().next())
            .map(|s| s.to_string())
            .context("Failed to find feminine past participle")?;

        Ok((masculine, feminine))
    }

    fn parse_tense(document: &Html, tense: &str, mood: &str) -> anyhow::Result<[String; 6]> {
        // Spanish uses roa-finite-header for tense headers
        let th_selector = Selector::parse("th.roa-finite-header").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        // Map English tense names to Spanish
        let spanish_tense = match tense {
            "present" => "presente",
            "imperfect" => "imperfecto",
            "preterite" => "pretérito",
            "future" => "futuro",
            "conditional" => "condicional",
            _ => tense, // fallback to English name
        };

        // Find the header for this tense
        let tense_keyword = match mood {
            "indicative" => format!("{} de indicativo", spanish_tense),
            "subjunctive" => format!("{} de subjuntivo", spanish_tense),
            _ => anyhow::bail!("Unknown mood: {}", mood),
        };

        let mut tense_row_ref = None;
        for th in document.select(&th_selector) {
            // Check the title attribute which contains the Spanish name
            if let Some(title) = th.value().attr("title") {
                if title.to_lowercase().contains(&tense_keyword.to_lowercase()) {
                    // Get the parent tr element
                    if let Some(parent) = th.parent() {
                        if parent.value().as_element().map(|e| e.name()) == Some("tr") {
                            tense_row_ref = Some(parent);
                            break;
                        }
                    }
                }
            } else {
                // Fallback: check text content
                let text = th.text().collect::<String>().to_lowercase();
                if text.contains(&tense.to_lowercase()) {
                    // Need to check if this belongs to the right mood by looking at context
                    // For now, just accept it
                    if let Some(parent) = th.parent() {
                        if parent.value().as_element().map(|e| e.name()) == Some("tr") {
                            tense_row_ref = Some(parent);
                            break;
                        }
                    }
                }
            }
        }

        let tense_row =
            tense_row_ref.context(format!("Failed to find tense row for {mood} {tense}"))?;

        // Extract the 6 conjugated forms from the td elements in this row
        let mut forms = Vec::new();

        // Iterate through children of the tr element
        for child in tense_row.children() {
            if let Some(element) = child.value().as_element() {
                if element.name() == "td" {
                    // Find the link inside the td
                    let td_elem = scraper::ElementRef::wrap(child).unwrap();
                    if let Some(link) = td_elem.select(&a_selector).next() {
                        if let Some(text) = link.text().next() {
                            forms.push(text.to_string());
                        }
                    }
                }
            }
        }

        if forms.len() != 6 {
            anyhow::bail!(
                "Expected 6 forms for {} {}, found {}",
                mood,
                tense,
                forms.len()
            );
        }

        Ok([
            forms[0].clone(),
            forms[1].clone(),
            forms[2].clone(),
            forms[3].clone(),
            forms[4].clone(),
            forms[5].clone(),
        ])
    }

    fn parse_imperative(document: &Html) -> anyhow::Result<[String; 5]> {
        // Find the affirmative imperative row
        // The structure is: th.roa-finite-header with title="imperativo afirmativo" or text "affirmative"
        let th_selector = Selector::parse("th.roa-finite-header").unwrap();
        let a_selector = Selector::parse("a").unwrap();
        let span_selector = Selector::parse("span[lang='es']").unwrap();

        let mut imperative_row_ref = None;
        for th in document.select(&th_selector) {
            // Check title attribute for "imperativo afirmativo" or text for "affirmative"
            let matches = if let Some(title) = th.value().attr("title") {
                title.to_lowercase().contains("afirmativo")
            } else {
                let text = th.text().collect::<String>().to_lowercase();
                text.contains("affirmative")
            };

            if matches {
                // Get the parent tr element
                if let Some(parent) = th.parent() {
                    if parent.value().as_element().map(|e| e.name()) == Some("tr") {
                        imperative_row_ref = Some(parent);
                        break;
                    }
                }
            }
        }

        let imperative_row =
            imperative_row_ref.context("Failed to find imperative affirmative row")?;

        // Extract the 5 forms (skip first td which is empty for "yo")
        // Forms are: tú, usted, nosotros, vosotros, ustedes
        let mut forms = Vec::new();

        // Iterate through children of the tr element
        for child in imperative_row.children() {
            if let Some(element) = child.value().as_element() {
                if element.name() == "td" {
                    let td_elem = scraper::ElementRef::wrap(child).unwrap();

                    // Check if this is an empty cell (for yo form)
                    let text = td_elem.text().collect::<String>().trim().to_string();
                    if text.is_empty() {
                        continue;
                    }

                    // Extract the first Spanish link in this cell
                    // (tú cell has two forms - tú and vos, we take the first one for tú)
                    let mut found = false;
                    for span in td_elem.select(&span_selector) {
                        if let Some(link) = span.select(&a_selector).next() {
                            if let Some(form_text) = link.text().next() {
                                forms.push(form_text.to_string());
                                found = true;
                                break; // Take only the first form (tú, not vos)
                            }
                        }
                    }

                    // Fallback: try any link if no Spanish span found
                    if !found {
                        if let Some(link) = td_elem.select(&a_selector).next() {
                            if let Some(form_text) = link.text().next() {
                                forms.push(form_text.to_string());
                            }
                        }
                    }
                }
            }
        }

        if forms.len() != 5 {
            anyhow::bail!("Expected 5 forms for imperative, found {}", forms.len());
        }

        Ok([
            forms[0].clone(),
            forms[1].clone(),
            forms[2].clone(),
            forms[3].clone(),
            forms[4].clone(),
        ])
    }

    /// Fetch Spanish verb conjugations from Wiktionary with HTML caching
    ///
    /// # Arguments
    /// * `verbs` - List of verb infinitives to fetch
    /// * `cache_dir` - Directory to cache HTML files (e.g., `.cache/wiktionary/spanish/`)
    ///
    /// # Returns
    /// HashMap mapping verb infinitives to their conjugations
    pub async fn fetch_spanish_verb_conjugations(
        verbs: &[String],
        cache_dir: &Path,
    ) -> anyhow::Result<HashMap<String, SpanishVerbConjugation>> {
        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(cache_dir)?;

        let mut results = HashMap::new();
        let mut to_fetch = Vec::new();

        // Check which verbs we already have cached
        for verb in verbs {
            let cache_file = cache_dir.join(format!("{}.html", verb));
            if cache_file.exists() {
                // Try to parse from cached HTML
                match std::fs::read_to_string(&cache_file) {
                    Ok(html) => match parse_spanish_verb_conjugation(&html, verb) {
                        Ok(conjugation) => {
                            results.insert(verb.clone(), conjugation);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse cached HTML for {verb}: {e}");
                            to_fetch.push(verb);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read cached HTML for {verb}: {e}");
                        to_fetch.push(verb);
                    }
                }
            } else {
                to_fetch.push(verb);
            }
        }

        if to_fetch.is_empty() {
            println!("All {} Spanish verbs loaded from cache", results.len());
            return Ok(results);
        }

        println!(
            "Fetching {} Spanish verb pages from Wiktionary...",
            to_fetch.len()
        );

        // Build HTTP client with YapBot user agent
        let client = reqwest::Client::builder()
            .user_agent("YapBot/1.0 (https://yap.town) reqwest/0.11")
            .build()
            .context("Failed to build HTTP client")?;

        // Fetch each verb
        for (i, verb) in to_fetch.iter().enumerate() {
            if i > 0 && i % 10 == 0 {
                println!("  Fetched {}/{} verbs...", i, to_fetch.len());
                // Rate limiting: sleep between batches
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            // Fetch the Wiktionary page
            let url = format!("https://en.wiktionary.org/wiki/{verb}");
            let response = match client.get(&url).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Failed to fetch {verb}: {e}");
                    continue;
                }
            };

            let html = match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("Failed to read HTML for {verb}: {e}");
                    continue;
                }
            };

            // Save HTML to cache
            let cache_file = cache_dir.join(format!("{}.html", verb));
            if let Err(e) = std::fs::write(&cache_file, &html) {
                eprintln!("Failed to write cache file for {verb}: {e}");
            }

            // Parse the conjugation
            match parse_spanish_verb_conjugation(&html, verb) {
                Ok(conjugation) => {
                    results.insert((*verb).clone(), conjugation);
                }
                Err(e) => {
                    eprintln!("Failed to parse conjugation for {verb}: {e}");
                }
            }
        }

        println!(
            "Finished fetching Spanish conjugations. Total: {}",
            results.len()
        );

        Ok(results)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;

        #[test]
        fn test_parse_beber() {
            let html = fs::read_to_string("src/wiktionary-examples/spa/beber.txt")
                .expect("Failed to read beber.txt");

            let conjugation = parse_spanish_verb_conjugation(&html, "beber")
                .expect("Failed to parse beber conjugation");

            assert_eq!(conjugation.infinitive, "beber");
            assert_eq!(conjugation.gerund, "bebiendo");
            assert_eq!(conjugation.past_participle_masculine_singular, "bebido");
            assert_eq!(conjugation.past_participle_feminine_singular, "bebida");

            // Check indicative present (yo, tú, él, nosotros, vosotros, ellos)
            assert_eq!(conjugation.indicative_present[0], "bebo");
            assert_eq!(conjugation.indicative_present[1], "bebes");
            assert_eq!(conjugation.indicative_present[2], "bebe");
        }

        #[test]
        fn test_parse_ser() {
            let html = fs::read_to_string("src/wiktionary-examples/spa/ser.txt")
                .expect("Failed to read ser.txt");

            let conjugation = parse_spanish_verb_conjugation(&html, "ser")
                .expect("Failed to parse ser conjugation");

            assert_eq!(conjugation.infinitive, "ser");
            assert_eq!(conjugation.gerund, "siendo");

            // Check indicative present (irregular)
            assert_eq!(conjugation.indicative_present[0], "soy");
            assert_eq!(conjugation.indicative_present[1], "eres");
            assert_eq!(conjugation.indicative_present[2], "es");
        }

        #[test]
        fn test_parse_tener() {
            let html = fs::read_to_string("src/wiktionary-examples/spa/tener.txt")
                .expect("Failed to read tener.txt");

            let conjugation = parse_spanish_verb_conjugation(&html, "tener")
                .expect("Failed to parse tener conjugation");

            assert_eq!(conjugation.infinitive, "tener");
        }

        #[test]
        fn test_parse_venir() {
            let html = fs::read_to_string("src/wiktionary-examples/spa/venir.txt")
                .expect("Failed to read venir.txt");

            let conjugation = parse_spanish_verb_conjugation(&html, "venir")
                .expect("Failed to parse venir conjugation");

            assert_eq!(conjugation.infinitive, "venir");
        }
    }
}
