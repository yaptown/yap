use anyhow::Context as _;
use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

mod french {
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

    /// Fetch French verb conjugations from Wiktionary with caching
    ///
    /// # Arguments
    /// * `verbs` - List of verb infinitives to fetch
    /// * `cache_file` - Path to the cache file (JSONL format)
    /// * `failures_file` - Path to the failures file (one verb per line)
    ///
    /// # Returns
    /// HashMap mapping verb infinitives to their conjugations
    pub async fn fetch_french_verb_conjugations(
        verbs: &[String],
        cache_file: &Path,
        failures_file: &Path,
    ) -> anyhow::Result<HashMap<String, FrenchVerbConjugation>> {
        // Load cached conjugations
        let mut cached: HashMap<String, FrenchVerbConjugation> = if cache_file.exists() {
            let file = File::open(cache_file)?;
            let reader = BufReader::new(file);
            reader
                .lines()
                .filter_map(|line| {
                    let line = line.ok()?;
                    let (verb, conj): (String, FrenchVerbConjugation) =
                        serde_json::from_str(&line).ok()?;
                    Some((verb, conj))
                })
                .collect()
        } else {
            HashMap::new()
        };

        // Load previous failures to avoid re-fetching
        let failures: HashSet<String> = if failures_file.exists() {
            let file = File::open(failures_file)?;
            let reader = BufReader::new(file);
            reader
                .lines()
                .filter_map(|line| line.ok())
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        } else {
            HashSet::new()
        };

        // Determine which verbs need to be fetched
        let to_fetch: Vec<&String> = verbs
            .iter()
            .filter(|verb| !cached.contains_key(*verb) && !failures.contains(*verb))
            .collect();

        if to_fetch.is_empty() {
            return Ok(cached);
        }

        println!(
            "Fetching {} verb conjugations from Wiktionary...",
            to_fetch.len()
        );

        // Build HTTP client with YapBot user agent
        let client = reqwest::Client::builder()
            .user_agent("YapBot/1.0 (https://yap.town) reqwest/0.11")
            .build()
            .context("Failed to build HTTP client")?;

        // Open files for appending
        let mut cache_writer = OpenOptions::new()
            .create(true)
            .append(true)
            .open(cache_file)?;

        let mut failures_writer = OpenOptions::new()
            .create(true)
            .append(true)
            .open(failures_file)?;

        // Fetch each verb
        for (i, verb) in to_fetch.iter().enumerate() {
            if i > 0 && i % 10 == 0 {
                println!("  Fetched {}/{} verbs...", i, to_fetch.len());
                // Rate limiting: sleep between batches
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }

            // Fetch the Wiktionary page
            let url = format!("https://en.wiktionary.org/wiki/{}", verb);
            let response = match client.get(&url).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("Failed to fetch {}: {}", verb, e);
                    writeln!(failures_writer, "{}", verb)?;
                    continue;
                }
            };

            let html = match response.text().await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("Failed to read HTML for {}: {}", verb, e);
                    writeln!(failures_writer, "{}", verb)?;
                    continue;
                }
            };

            // Parse the conjugation
            match parse_french_verb_conjugation(&html, verb) {
                Ok(conjugation) => {
                    // Write to cache
                    let json = serde_json::to_string(&(verb, &conjugation))?;
                    writeln!(cache_writer, "{}", json)?;

                    // Add to in-memory cache
                    cached.insert((*verb).clone(), conjugation);
                }
                Err(e) => {
                    eprintln!("Failed to parse conjugation for {}: {}", verb, e);
                    writeln!(failures_writer, "{}", verb)?;
                }
            }
        }

        println!(
            "Finished fetching conjugations. Total cached: {}",
            cached.len()
        );

        Ok(cached)
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
            let cache_file = temp_dir.path().join("conjugations_cache.jsonl");
            let failures_file = temp_dir.path().join("conjugations_failures.txt");

            let verbs = vec![
                "parler".to_string(),
                "finir".to_string(),
                "être".to_string(),
            ];

            // First fetch
            let result = fetch_french_verb_conjugations(&verbs, &cache_file, &failures_file)
                .await
                .expect("Failed to fetch conjugations");

            assert!(result.contains_key("parler"));
            assert!(result.contains_key("finir"));
            assert!(result.contains_key("être"));

            // Verify cache file was created
            assert!(cache_file.exists());

            // Second fetch should use cache
            let result2 = fetch_french_verb_conjugations(&verbs, &cache_file, &failures_file)
                .await
                .expect("Failed to fetch conjugations from cache");

            assert_eq!(result.len(), result2.len());
        }
    }
}
