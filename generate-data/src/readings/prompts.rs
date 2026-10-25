//! Prompts and numbered evidence, separate from batch orchestration.
use super::*;

const ATTACHMENT_NOTE: &str = "This reading contains a sound change within a word. An altered onset belongs to an element attached after a preceding part of the same compound or numeral-counter expression. Gemination depends on a following consonant in a compound or suffix. An independent word normally keeps its plain lexical reading.";

pub(super) const OCCURRENCE_CHECK: &str = "We could not turn this answer into a reliable annotation. Listed choices need the actual sense and reading indices shown in this menu, because guessing an index can select a different vocabulary card. We also check that the pronunciation covers the complete marked span and preserves its written kana: an omitted particle or a neighbour's syllable would otherwise produce plausible-looking but misplaced ruby. Conditional compound sound changes also need an attached neighbour; a sentence edge, punctuation or whitespace cannot supply one. The rejected answer is included so that you can revisit those details.";

pub(super) const REPAIR_NOTE: &str = "The occurrence annotator could not find a suitable meaning or reading for these examples. Its suggestions are evidence to check against the sentences, rather than authoritative replacements. Please repair the existing hierarchy additively, referring to an existing sense through grounding.existing_sense when it already has the right meaning, or using null for a new meaning. Other occurrences have already attested the existing meanings, so replacing or deleting those would lose successful decisions. Citing each forced missing-meaning example in grounding.examples lets us check that the repair actually covers the examples that prompted it. A new meaning using a listed reading does not require reopening the candidate list.";

const FRAGMENT_NOTE: &str = "A gram is a learning chunk, not necessarily a complete phrase or clause. It can start or end inside a larger constituent, so the surrounding sentence supplies the missing syntax. Please describe and select the meaning of the marked words in that sentence, rather than completing the fragment into a different assertion. A general sense is not limited to the particular objects or participants used to illustrate it; a compatible contextual paraphrase does not create a new meaning. Genuine lexical meaning differences still need separate senses. These chunks remain vocabulary in their own right even when their parts are transparent.";

fn language_policy(language: Language) -> &'static str {
    match language {
        Language::Japanese => {
            "For Japanese, we would like the natural everyday reading in this context. \
            The dictionary's priority marks are helpful defaults, while a clear name, \
            register, or grammatical construction may call for a less common form. \
            Those marks belong to the particular spelling and reading shown: a common \
            spelling elsewhere in the same entry does not make an exceptional spelling \
            common too. Please carry the evidence about archaic or literary use, unusual \
            kanji or kana, gikun, and suffix attachment into the decision, because a \
            dictionary listing on its own does not establish that a form is natural here.\n\
            \n\
            Rendaku and gemination are pronunciation changes within a word, rather than \
            new meanings. For example, 桁 is けた on its own, while 橋桁 is はしげた; \
            the voiced form belongs to the compound. Numeral-counter expressions can \
            have the same kind of fused pronunciation. The menu marks synthetic forms \
            with this attachment condition so that they remain available where needed \
            without looking like ordinary standalone alternatives. A dictionary reading \
            restricted to a noun or verb-stem suffix likewise needs that construction; \
            following a finite verb is a different grammatical environment. When the \
            marked span is the second element of a compound directly attached to a \
            preceding noun or stem, consider its conventional voiced onset even if \
            that preceding element lies outside the span. Rendaku is common there, \
            not mandatory: established unvoiced compounds keep their lexical form. \
            After a particle, a completed clause, or a word boundary, an independent \
            word normally keeps its plain onset. Mere adjacency to written kana does \
            not establish compound attachment. This distinction matters because our \
            gram boundaries can split one compound or join parts of separate words.\n\
            \n\
            For kanji-written words, normalize conversational shortening or expressive \
            reshaping only when it is a pronunciation of the same lexical word, rather \
            than an established alternative reading. For example, 御免 receives ごめん \
            rather than an inferred drawn-out ごめーん; explicitly written ごめーん \
            keeps the author's spelling. This lets learners connect kanji with the \
            written word without reconstructing a speaker's delivery. Distinct \
            established lexical readings still compete on meaning, construction, and \
            the natural register of this sentence: a more formal, longer, or more \
            literal reading is not inherently preferable. Kanji spelling is not itself \
            a cue for a formal or bookish reading. Regular lexical compound sound \
            changes remain part of the reading.\n\
            \n\
            Our learner annotation uses hiragana and keeps written kana in their aligned \
            positions, including は and へ even when they are spoken わ and え. We check \
            that alignment mechanically because a copied lemma ending or a neighbouring \
            syllable can sound plausible while putting ruby over the wrong text. The \
            surface as inflected is what needs a reading. A short irregular word reading \
            can still be legitimate: mora count is not a count of Han characters. \
            Punctuation contributes no sound. One explicit learner spelling convention \
            is that the kanji-written 言う lemma uses the いう family, including its \
            inflections, rather than the conversational ゆう family. This is a convention \
            for that lemma, not a change to unrelated words containing ゆ."
        }
        Language::ChineseSimplified => {
            "For Mandarin, please establish what the marked text means before choosing \
            its pronunciation. We use standard mainland readings, with the dictionary's \
            region, name, and old-variant evidence kept visible so that a regional form \
            or a common-noun reading of a name character does not quietly become the \
            default. For example, 钻 as the action of drilling is zuān, while the noun \
            for a drill is zuàn; the intended meaning settles that choice.\n\
            \n\
            In a personal name or nickname, a character takes the reading \
            supported by the meaning or established name actually in use. A dictionary \
            entry labelled as a surname applies when the character is functioning as \
            a family name; being part of a name is not enough to establish that role. \
            For example, the surname reading 翟 zhái does not replace dí when the \
            character is used with its ordinary pheasant meaning. Given names and \
            nicknames may use ordinary character readings instead. Matching written \
            characters tells us which readings are possible, but an unrelated \
            homograph's meaning and pronunciation do not carry over automatically. \
            Please keep that distinction visible in the sources and sense gloss.\n\
            \n\
            Within the selected meaning, our annotation follows the exact CEDICT primary \
            form. The distinction between a primary and an also-pronounced or reference \
            form matters because a lexical neutral tone is not simply an optional way \
            of saying every word. For example, 笤帚 is tiáo zhou, with a neutral second \
            syllable, whereas 围裙 is wéi qún, with its full second tone. We therefore \
            follow the selected primary in either direction, rather than preferring \
            whichever alternative has more neutral syllables. A reopened pronunciation \
            follows the same convention. Tone similarity by itself does not establish \
            a meaning: 东西 dōng xi for a thing and dōng xī for east and west remain \
            distinct. The source sense IDs let us keep that semantic boundary explicit.\n\
            \n\
            A phrase can have several complete dictionary coverings, including a whole \
            headword and different combinations of smaller entries. Please consider \
            what each covering means in this sentence. A longest entry may be an idiom \
            that does not fit the actual construction, while a shorter internal entry \
            can carry a lexical pronunciation that character-by-character reading misses. \
            Once its meaning fits, a headword's primary takes precedence over the tones \
            of its separate characters. Where no covering headword applies, the lexical \
            atom or character tones remain the fallback.\n\
            \n\
            Structural 得 linking a verb or adjective to a degree, result, or potential \
            complement is neutral de, unlike lexical 得 dé meaning to obtain. A whole \
            headword for an emphatic rhetorical expression does not override that \
            productive grammar just because the characters coincide. Likewise, final \
            了 in a refusal or change-of-state response is neutral le, while resultative \
            了 expressing completion or ability is liǎo. Keep these meanings distinct \
            in the menu, including inside longer chunks: punctuation or a following \
            clause does not turn a refusal into an inability complement. Cite only \
            sources for the actual construction, reopening a missing reading when \
            needed, because a source for a different use can force the wrong tones.\n\
            \n\
            We use tone-marked pinyin with one syllable per Han character and neutral \
            tone left unmarked, because the annotation represents lexical reading rather \
            than a particular spoken realization. This excludes third-tone sandhi and \
            the sandhi spellings yí/yì for 一 or bú for 不. It does not turn a dictionary \
            headword's lexical neutral 一 or 不 into a full tone. In a productive \
            verb–一–verb construction without such a headword, the character's lexical \
            yī is the fallback. Our display convention writes 啊 as a throughout, \
            including interjection contours and the ya/wa/na allophones."
        }
        _ => unreachable!(),
    }
}

pub(super) fn hierarchy_system_prompt(language: Language) -> String {
    let name = language.frozen_prompt_name();
    format!(
        "We are building a {name} course for a language-learning app. Its sentences \
        are segmented into vocabulary items, called grams, which can be a word or a \
        phrase. We would like your help organizing one gram's dictionary entries into \
        senses, each with its readings. Every sense becomes its own vocabulary card, \
        and its readings supply the pronunciation annotation shown above the gram.\n\
        \n\
        The request gives the gram, its dictionary candidates, its atoms with their \
        lemmas and parts of speech, and numbered sentences with the gram marked «like \
        this». The dictionary evidence includes source sense IDs, primary forms, usage \
        notes, and spelling-bound priorities. For a phrase, the atom entries help show \
        what its parts mean; a composed candidate is not automatically an idiom.\n\
        \n\
        Please make a sense for each distinct meaning, including meanings that share \
        a pronunciation, because a learner meets those as different words that happen \
        to sound alike. Synonymous dictionary glosses belong together, as do alternative \
        pronunciations of one meaning: splitting those would teach a pronunciation \
        variant as a new vocabulary item. The examples help establish use, rather than \
        limiting the dictionary's inventory. A later step counts which meanings the \
        corpus attests and drops unused ones, so an unused sense costs nothing while \
        a meaning omitted here is unavailable to that step. A contextual specialization \
        of an existing meaning does not by itself call for a new card.\n\
        \n\
        {FRAGMENT_NOTE}\n\
        \n\
        Each sense needs a concise {name} gloss that tells it apart from the others. \
        The next step reads these glosses when choosing the meaning in each sentence, \
        so the difference needs to be clear even when the readings are identical. \
        A gram with only one meaning still needs that gloss and its readings; it is \
        the implicit base meaning rather than an additional vocabulary card.\n\
        \n\
        In grounding.sources, please cite the displayed source IDs whose meanings \
        actually apply to the sense. An empty list is appropriate when dictionary \
        evidence does not cover that meaning. These references connect your semantic \
        decision to its primary pronunciation, so a shared sound alone is not a reason \
        to cite a source. grounding.examples lists the example indices demonstrating \
        the meaning when examples are available.\n\
        \n\
        Each whole_gram_reading is one complete pronunciation of the entire marked \
        gram as written and inflected. Its alternatives are whole pronunciations, \
        rather than separate entries for the component syllables or atoms, because \
        the annotation has to cover the whole word or phrase. For a listed candidate, \
        please copy both its zero-based index and its complete text, leaving example_line \
        null. We check that pair to catch accidental index mistakes. If a demonstrated \
        modern use needs a reading outside the list, set reopen to true and supply \
        that reading with index null and the example_line that needs it. An open gram \
        with no listed candidates needs this citation for every reading.\n\
        \n\
        On the first pass, grounding.existing_sense is null. A repair request also \
        supplies the existing_hierarchy and examples that have already attested its \
        meanings. Please return additions or extensions, referring to an existing \
        sense by its exact ID when that meaning already fits, or using null for a \
        genuinely new meaning. Keeping the existing meanings, glosses, and references \
        intact preserves the successful decisions in other sentences. Each forced \
        missing-meaning example needs an explicit citation in grounding.examples of \
        the sense that covers it, including when an existing sense already suffices; \
        that is how we check that the repair addresses the examples that caused it.\n\
        \n\
        {}\n\
        \n\
        Please keep the response concise.",
        language_policy(language)
    )
}

const COVERING_PARTS_NOTE: &str = "Each covering lists positions in covering_parts. A part keeps the real dictionary sources for its span, so those source IDs can be cited independently of the other parts. Sharing the part table avoids repeating the same evidence across alternative coverings.";

#[derive(Default)]
struct CoveringView {
    parts: Vec<serde_json::Value>,
    readings: BTreeMap<String, Vec<Vec<usize>>>,
}

fn covering_view<'a>(
    coverings: impl IntoIterator<Item = (&'a String, &'a Vec<coverings::Covering>)>,
) -> CoveringView {
    let mut view = CoveringView::default();
    let mut indices = BTreeMap::new();
    for (reading, paths) in coverings {
        let paths = paths
            .iter()
            .map(|parts| {
                parts
                    .iter()
                    .map(|part| {
                        let key = serde_json::to_string(part).unwrap();
                        *indices.entry(key).or_insert_with(|| {
                            let index = view.parts.len();
                            let mut value = serde_json::to_value(part).unwrap();
                            value["part_index"] = serde_json::json!(index);
                            view.parts.push(value);
                            index
                        })
                    })
                    .collect()
            })
            .collect();
        view.readings.insert(reading.clone(), paths);
    }
    view
}

fn install_covering_view(prompt: &mut serde_json::Value, view: CoveringView) {
    if !view.parts.is_empty() {
        prompt["coverings"] = serde_json::json!(view.readings);
        prompt["covering_parts"] = serde_json::json!(view.parts);
        prompt["covering_note"] = serde_json::json!(COVERING_PARTS_NOTE);
    }
}

pub(super) fn add_coverings(prompt: &mut serde_json::Value, candidates: &Candidates) {
    install_covering_view(prompt, covering_view(&candidates.coverings));
}

pub(super) fn hierarchy_prompt(
    gram: &Gram<String>,
    candidates: &Candidates,
    language: Language,
) -> serde_json::Value {
    let coverings = covering_view(&candidates.coverings);
    let readings: Vec<_> = candidates
        .ordered_readings()
        .into_iter()
        .enumerate()
        .map(|(index, reading)| {
            let mut entry = serde_json::json!({"index":index,"reading":reading});
            if let Some(senses) = candidates.senses.get(reading).filter(|s| !s.is_empty()) {
                entry["senses"] = serde_json::json!(senses);
            }
            if let Some(evidence) = candidates.evidence.get(reading).filter(|s| !s.is_empty()) {
                entry["evidence"] = serde_json::json!(evidence);
            }
            if let Some(parts) = coverings.readings.get(reading) {
                entry["coverings"] = serde_json::json!(parts);
            }
            entry
        })
        .collect();
    let mut prompt = serde_json::json!({"gram":gram.to_display_string(language),"candidates":{"open":candidates.open,"atoms":candidates.atoms,"readings":readings},"examples":[]});
    if !coverings.parts.is_empty() {
        prompt["candidates"]["covering_parts"] = serde_json::json!(coverings.parts);
        prompt["candidates"]["covering_note"] = serde_json::json!(COVERING_PARTS_NOTE);
    }
    prompt
}

pub(super) fn evidence_priority(evidence: &[DictionaryEvidence]) -> (bool, bool, u8, bool) {
    evidence
        .iter()
        .map(|e| {
            let marked = e.notes.iter().any(|n| {
                [
                    "&rK;",
                    "&ok;",
                    "&ik;",
                    "&arch;",
                    "&lit;",
                    "rarely-used kanji form",
                    "rarely used kanji form",
                    "search-only",
                    "out-dated",
                    "archaic",
                    "out-dated or obsolete",
                    "irregular kanji usage",
                    "archaism",
                    "literary",
                ]
                .iter()
                .any(|tag| n.contains(tag))
            });
            let frequency = e
                .priority
                .iter()
                .filter_map(|p| p.strip_prefix("nf")?.parse::<u8>().ok())
                .min()
                .unwrap_or(u8::MAX);
            (
                marked,
                e.priority.is_empty(),
                frequency,
                e.form != "primary",
            )
        })
        .min()
        .unwrap_or((false, true, u8::MAX, true))
}

pub(super) fn numbered_examples(examples: &[Occurrence]) -> serde_json::Value {
    serde_json::json!(
        examples
            .iter()
            .enumerate()
            .map(|(index, o)| serde_json::json!({"index":index,"sentence":o.marked()}))
            .collect::<Vec<_>>()
    )
}

const HIERARCHY_RETRY_NOTE: &str = "Our checks could not use the previous hierarchy. The rejected answer and its specific mismatches are included below so you can repair the actual problem instead of guessing what went wrong. Displayed candidate indices name particular complete readings, and source references must come from the supplied dictionary evidence. Please use the reported index/text pairs and unknown references to check those choices, while retaining the meanings and readings that were already sound. An unlisted reading still needs reopen, a null index, and an example_line showing its use. A repair also needs the supplied existing sense references and coverage of each forced missing-meaning example. The marked gram is the pronunciation scope; neighbouring words provide context, not additional syllables.";

pub(super) fn hierarchy_retry(request: &HierarchyRequest, feedback: &serde_json::Value) -> String {
    format!(
        "{}\n\n{HIERARCHY_RETRY_NOTE}\nRejected answer and checks: {feedback}",
        request.prompt
    )
}

pub(super) fn occurrence_system_prompt(language: Language) -> String {
    format!(
        "Please identify the sense and pronunciation of the gram marked «like this». \
        Your answer sets the learner's ruby over it and which sense card the sentence trains.\n\
        \n\
        It helps to establish what the marked text means in the whole sentence before \
        choosing a reading. A familiar dictionary entry or an upstream part-of-speech \
        tag may describe a different use of those characters. The marked phrase can \
        cross a word boundary, so its actual construction matters more than finding \
        the longest familiar headword. The dictionary notes and the atoms' evidence \
        are there to help make that distinction. The explicit surface field is \
        exactly the text to pronounce; the rest of the sentence supplies context.\n\
        \n\
        {FRAGMENT_NOTE}\n\
        \n\
        The response has an answer object describing either a listed choice or an \
        escape. If a listed meaning and reading fit, please use decision listed and \
        return choices as sense index and zero-based reading index, most likely first. \
        Additional choices are useful when the writing cannot distinguish the \
        alternatives. Sense 0 is a \
        single unsplit meaning. Earlier plain, primary, or high-priority forms are \
        defaults rather than verdicts: context can establish a marked register, name, \
        or attachment that calls for another form. This lets the menu convey useful \
        dictionary evidence without substituting frequency for meaning.\n\
        \n\
        The menu can be incomplete, and the other kind of answer is how we repair \
        it. If no listed reading fits, or the menu is empty, please use decision \
        none_of_these and give the complete whole_gram_reading. If a pronunciation \
        is listed but the meaning here needs a different translation, the escape \
        also carries missing_meaning as a short target-language gloss. When only \
        the pronunciation is missing, missing_meaning is null. A nuance or contextual \
        specialization of a listed meaning is not automatically a missing meaning. \
        Keeping meaning and pronunciation separate matters because even a uniquely \
        matching reading can belong to a meaning absent from the menu. The \
        missing_meaning field belongs only to the escape branch because it asks us \
        to add a meaning; an explanation of an already listed sense is not such a \
        request. The two answer shapes keep those intentions distinct.\n\
        \n\
        The reading belongs to exactly the marked surface as inflected. A neighbour's \
        syllable, a dropped particle, or a substituted lemma could sound plausible \
        while giving the learner the wrong annotation. Please check the phrase from \
        left to right, preserving its written particles and kana in place; punctuation \
        contributes no pronunciation.\n\
        \n\
        {}",
        language_policy(language)
    )
}

fn occurrence_value(
    occurrence: &Occurrence,
    hierarchy: &[SenseReadings],
    language: Language,
) -> serde_json::Value {
    let senses: Vec<_> = hierarchy
        .iter()
        .map(|sense| {
            let readings: Vec<_> = sense
                .readings
                .iter()
                .enumerate()
                .map(|(index, reading)| {
                    let synthetic =
                        language == Language::Japanese && sense.attachment_only(reading);
                    let mut entry = serde_json::json!({"index":index,"reading":reading});
                    if synthetic {
                        entry["attachment"] = serde_json::json!(ATTACHMENT_NOTE);
                    }
                    entry
                })
                .collect();
            serde_json::json!({"sense":sense.index,"gloss":sense.gloss,"readings":readings})
        })
        .collect();
    serde_json::json!({"language":language.frozen_prompt_name(),"surface":&occurrence.text[occurrence.bytes.0..occurrence.bytes.1],"sentence":occurrence.marked(),"hierarchy":senses})
}

pub(super) fn occurrence_prompt(
    occurrence: &Occurrence,
    hierarchy: &[SenseReadings],
    language: Language,
) -> String {
    occurrence_value(occurrence, hierarchy, language).to_string()
}

pub(super) fn occurrence_sense_followup(
    occurrence: &Occurrence,
    compatible: &[SenseReadings],
    original: &[SenseReadings],
    candidates: Option<&Candidates>,
    language: Language,
    reading: &str,
) -> String {
    let mut prompt = occurrence_value(occurrence, compatible, language);
    let mut attached = Vec::new();
    if language == Language::Japanese {
        for sense in compatible {
            let original = original.iter().find(|s| s.index == sense.index).unwrap();
            for reading in &sense.readings {
                if original.attachment_only(reading) {
                    attached.push(reading);
                }
            }
        }
    }
    prompt["compound_or_counter_attachment_only"] = serde_json::json!(attached);
    if let Some(candidates) = candidates {
        prompt["atoms"] = serde_json::json!(candidates.atoms);
        let evidence: BTreeMap<_, _> = candidates
            .evidence
            .iter()
            .filter(|(r, _)| compatible.iter().any(|s| s.readings.contains(r)))
            .collect();
        prompt["dictionary_evidence"] = serde_json::json!(evidence);
        let coverings = covering_view(
            candidates
                .coverings
                .iter()
                .filter(|(r, _)| compatible.iter().any(|s| s.readings.contains(r))),
        );
        install_covering_view(&mut prompt, coverings);
    }
    format!(
        "{prompt}\n\nWe have already checked the pronunciation {reading}, and would \
        now like your help settling its meaning in this sentence. This smaller menu \
        contains the senses with compatible pronunciations. A matching pronunciation \
        does not establish which meaning is intended, even when there is only one \
        listed sense, because the dictionary may be missing another meaning with \
        the same sound. Please use the listed decision with explicit sense and \
        reading indices when their gloss fits here. If the intended meaning is absent, \
        the none_of_these decision repeats the already checked whole_gram_reading \
        and supplies a short missing_meaning gloss. That field asks us to add a \
        meaning rather than explain a listed choice, so the escape lets us repair \
        the gap instead of crediting \
        the wrong vocabulary card. Any attachment-only forms retain that condition \
        in this focused menu."
    )
}

pub(super) fn verification_system_prompt() -> &'static str {
    "We are checking a pronunciation annotation for a Japanese language-learning \
    sentence. The proposed reading did not compose from our dictionary's character \
    and word readings, but that is only a reason to ask for help: ordinary names, \
    colloquial forms, and unusual spellings can still be perfectly legitimate.\n\
    \n\
    Please decide whether the proposed whole_gram_reading pronounces exactly the \
    text marked «like this» in the supplied sentence. The surrounding sentence helps \
    identify the intended reading, but its sounds do not belong in the marked span. \
    Including a neighbour's syllable, dropping an attached kana, or substituting \
    the dictionary lemma for the inflected surface would put the wrong ruby over \
    the learner's word. We need the complete marked text, no more and no less.\n\
    \n\
    If the reading is legitimate in this context, please mark it verified. If it \
    is not, please offer a corrected_whole_gram_reading in hiragana when you can \
    confidently supply one, or leave the correction null when you cannot. A brief \
    reason should point to the relevant sounds or span boundaries so we can audit \
    the decision. Written kana remain aligned with the surface, and punctuation \
    contributes no sound."
}

pub(super) fn occurrence_retry(prompt: &str) -> String {
    format!(
        "{prompt}\n\nWe could not interpret the previous answer reliably. The rejected \
        answer and check are included so that you can reconsider its indices and \
        marked span. If a listed meaning and reading fit, please use the listed \
        decision inside answer, with their displayed indices in a nonempty choices \
        list. Otherwise, the none_of_these decision supplies one complete \
        whole_gram_reading of the marked surface and missing_meaning. That gloss \
        is for a meaning the menu needs to add, even if its pronunciation is already \
        listed; it can be null when only a reading is missing. These distinct answer \
        shapes let us recover the intended annotation without confusing an \
        explanation of a listed choice with a request to add a new meaning."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_fragment_guidance_reaches_b_and_c_in_both_languages() {
        for language in [Language::Japanese, Language::ChineseSimplified] {
            assert!(hierarchy_system_prompt(language).contains(FRAGMENT_NOTE));
            assert!(occurrence_system_prompt(language).contains(FRAGMENT_NOTE));
        }
        let japanese = language_policy(Language::Japanese);
        assert!(japanese.contains("same lexical word"));
        assert!(japanese.contains("Distinct established lexical readings"));
        assert!(japanese.contains("Kanji spelling is not itself a cue"));
        assert!(!japanese.contains("An informal scene alone"));
    }

    #[test]
    fn c_surface_is_exact_and_shared_with_the_focused_followup() {
        let occurrence = Occurrence {
            text: "前へ行くんじゃ".into(),
            gram_index: 1,
            bytes: ("前".len(), "前へ行く".len()),
            spans: vec![],
        };
        let hierarchy = implicit_hierarchy(vec!["へいく".into()]);
        for prompt in [
            occurrence_prompt(&occurrence, &hierarchy, Language::Japanese),
            occurrence_sense_followup(
                &occurrence,
                &hierarchy,
                &hierarchy,
                None,
                Language::Japanese,
                "へいく",
            ),
        ] {
            let value = serde_json::Deserializer::from_str(&prompt)
                .into_iter::<serde_json::Value>()
                .next()
                .unwrap()
                .unwrap();
            assert_eq!(value["surface"], "へ行く");
            assert_eq!(value["sentence"], "前«へ行く»んじゃ");
        }
    }
}
