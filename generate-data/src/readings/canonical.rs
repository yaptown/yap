//! Presentation normalization for open/reopened readings, never pronunciation guessing.
use language_utils::{
    Language,
    readings::{canonical_kana, is_han, is_ruby_base},
};
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;

/// Kana is written pronunciation evidence: inflections must survive the reading.
/// Punctuation contributes nothing, and each Han run needs some pronunciation.
pub(super) fn japanese_pattern(surface: &str) -> regex::Regex {
    let mut pattern = String::from(r"\A");
    let mut in_han = false;
    for c in surface.nfkc() {
        if is_ruby_base(c, Language::Japanese) {
            if !in_han {
                pattern.push_str(".+");
            }
            in_han = true;
        } else {
            in_han = false;
            let c = canonical_kana(c);
            if ('ぁ'..='ゖ').contains(&c) || c == 'ー' {
                pattern.push_str(&regex::escape(&c.to_string()));
            }
        }
    }
    pattern.push_str(r"\z");
    regex::Regex::new(&pattern).unwrap()
}

pub(super) fn canonicalize(language: Language, base: &str, reading: &str) -> Option<String> {
    if language == Language::Japanese {
        let kana: String = reading
            .nfkc()
            .map(canonical_kana)
            .filter(|&c| ('ぁ'..='ゖ').contains(&c) || c == 'ー')
            .collect();
        return (!kana.is_empty()).then_some(kana);
    }
    let characters: Vec<_> = base.nfkc().filter(|&c| is_han(c)).collect();
    let pinyin = pinyin(reading, characters.len())?;
    Some(
        characters
            .into_iter()
            .zip(pinyin.split_whitespace())
            .map(|(c, syllable)| match (c, syllable) {
                ('一', "yí" | "yì") => "yī",
                ('不', "bú") => "bù",
                ('啊', s) if matches!(untoned(s).as_str(), "a" | "ya" | "wa" | "na") => "a",
                _ => syllable,
            })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn untoned(s: &str) -> String {
    s.nfd()
        .filter(|c| !matches!(c, '\u{0304}' | '\u{0301}' | '\u{030c}' | '\u{0300}'))
        .nfc()
        .collect()
}

/// Tone-only neutral/full variants. This relation never identifies a meaning.
/// Different full tones (rèn/rén) are not variants.
pub(super) fn tone_variant(a: &str, b: &str) -> bool {
    let a: Vec<_> = a.split_whitespace().collect();
    let b: Vec<_> = b.split_whitespace().collect();
    a.len() == b.len()
        && a.iter().zip(b).all(|(a, b)| {
            *a == b || (untoned(a) == untoned(b) && (untoned(a) == *a || untoned(b) == b))
        })
}

/// Normalize Japanese particle spellings against the menu. Chinese readings
/// keep their lexical tones here: a flat menu carries no semantic evidence.
/// Neutral headword folds happen only within an installed B sense.
pub(super) fn menu_form<'a>(
    language: Language,
    base: &str,
    reading: &str,
    menu: impl IntoIterator<Item = &'a String>,
) -> Option<String> {
    let reading = canonicalize(language, base, reading)?;
    if language != Language::Japanese {
        return Some(reading);
    }
    let options: BTreeSet<_> = menu
        .into_iter()
        .filter_map(|r| canonicalize(language, base, r))
        .collect();
    if options.contains(&reading) {
        return Some(reading);
    }
    let mut matches = options
        .iter()
        .filter(|option| japanese_particle_variant(base, &reading, option));
    match (matches.next(), matches.next()) {
        (Some(r), None) => Some(r.clone()),
        _ => Some(reading),
    }
}

fn japanese_particle_variant(base: &str, spoken: &str, written: &str) -> bool {
    let spoken: Vec<_> = spoken.chars().collect();
    let written: Vec<_> = written.chars().collect();
    if spoken.len() != written.len() {
        return false;
    }
    // Mark differences before aligning, so a Han wildcard cannot consume them.
    // Only an actual written particle can license its phonetic spelling.
    let mut aligned = String::new();
    for (a, b) in spoken.into_iter().zip(written) {
        aligned.push(match (a, b) {
            (a, b) if a == b => a,
            ('わ', 'は') => 'H',
            ('え', 'へ') => 'E',
            _ => return false,
        });
    }
    let mut pattern = japanese_pattern(base).as_str().replace(".+", "[^HE]+");
    pattern = pattern.replace('は', "[はH]").replace('へ', "[へE]");
    regex::Regex::new(&pattern).unwrap().is_match(&aligned)
}

/// Whether `changed` is `plain` after rendaku (voicing of the first mora) or
/// onbin gemination (く/き/ち/つ becoming っ). The relation is directional, so
/// the plain form is always the one a sound-changed sense merges into.
pub(super) fn sound_changed(changed: &str, plain: &str) -> bool {
    let a: Vec<_> = changed.chars().collect();
    let b: Vec<_> = plain.chars().collect();
    a.len() == b.len()
        && a != b
        && a.iter().zip(&b).enumerate().all(|(i, (&a, &b))| {
            let mut voiced = a.to_string().nfd().collect::<Vec<_>>().into_iter();
            a == b
                || (i == 0
                    && voiced.next() == Some(b)
                    && matches!(voiced.next(), Some('\u{3099}' | '\u{309a}')))
                || (a == 'っ' && "くきちつ".contains(b))
        })
}

/// The hard surface guards shared by corrections and occurrence escapes.
pub(super) fn checked_surface_reading(
    language: Language,
    base: &str,
    reading: &str,
) -> Option<String> {
    let reading = canonicalize(language, base, reading)?;
    if language == Language::Japanese && !japanese_pattern(base).is_match(&reading) {
        return None;
    }
    Some(reading)
}

// Standard untoned Hanyu Pinyin syllables, plus syllabic interjections and the
// neutral erhua suffix r. Keeping the inventory explicit excludes English words.
const SYLLABLES: &str = "a ai an ang ao ba bai ban bang bao bei ben beng bi bia bian biao bie bin bing bo bu ca cai can cang cao ce cen ceng cha chai chan chang chao che chen cheng chi chong chou chu chua chuai chuan chuang chui chun chuo ci cong cou cu cuan cui cun cuo da dai dan dang dao de dei den deng di dia dian diao die ding diu dong dou du duan dui dun duo e ei en eng er fa fan fang fei fen feng fo fou fu ga gai gan gang gao ge gei gen geng gong gou gu gua guai guan guang gui gun guo ha hai han hang hao he hei hen heng hong hou hu hua huai huan huang hui hun huo ji jia jian jiang jiao jie jin jing jiong jiu ju juan jue jun ka kai kan kang kao ke kei ken keng kong kou ku kua kuai kuan kuang kui kun kuo la lai lan lang lao le lei leng li lia lian liang liao lie lin ling liu lo long lou lu luan lun luo lü lüe ma mai man mang mao me mei men meng mi mian miao mie min ming miu mo mou mu na nai nan nang nao ne nei nen neng ni nian niang niao nie nin ning niu nong nou nu nuan nuo nü nüe o ou pa pai pan pang pao pei pen peng pi pian piao pie pin ping po pou pu qi qia qian qiang qiao qie qin qing qiong qiu qu quan que qun ran rang rao re ren reng ri rong rou ru ruan rui run ruo sa sai san sang sao se sen seng sha shai shan shang shao she shei shen sheng shi shou shu shua shuai shuan shuang shui shun shuo si song sou su suan sui sun suo ta tai tan tang tao te teng ti tian tiao tie ting tong tou tu tuan tui tun tuo wa wai wan wang wei wen weng wo wu xi xia xian xiang xiao xie xin xing xiong xiu xu xuan xue xun ya yan yang yao ye yi yin ying yo yong you yu yuan yue yun za zai zan zang zao ze zei zen zeng zha zhai zhan zhang zhao zhe zhei zhen zheng zhi zhong zhou zhu zhua zhuai zhuan zhuang zhui zhun zhuo zi zong zou zu zuan zui zun zuo m n ng hm hng r";
const VOWELS: [&str; 6] = ["aāáǎà", "eēéěè", "iīíǐì", "oōóǒò", "uūúǔù", "üǖǘǚǜ"];

fn pinyin(text: &str, syllable_count: usize) -> Option<String> {
    if syllable_count == 0 {
        return None;
    }
    let text = text
        .nfkc()
        .collect::<String>()
        .to_lowercase()
        .replace("u:", "ü")
        .replace('v', "ü");
    let mut letters: Vec<(char, u8)> = Vec::new();
    let mut boundaries = BTreeSet::new();
    for c in text.nfd() {
        let tone = match c {
            '\u{0304}' => Some(1),
            '\u{0301}' => Some(2),
            '\u{030c}' => Some(3),
            '\u{0300}' => Some(4),
            _ => None,
        };
        if let Some(tone) = tone {
            let last = letters.last_mut()?;
            if last.1 != 0 {
                return None;
            }
            last.1 = tone;
        } else if c == '\u{0308}' {
            let last = letters.last_mut()?;
            if last.0 != 'u' {
                return None;
            }
            last.0 = 'ü';
        } else if c.is_ascii_lowercase() {
            letters.push((c, 0));
        } else if let Some(tone @ 0..=5) = c.to_digit(10) {
            let last = letters.last_mut()?;
            if last.1 != 0 {
                return None;
            }
            last.1 = if tone == 0 { 5 } else { tone as u8 };
            boundaries.insert(letters.len());
        } else if c.is_alphabetic() || c.is_numeric() {
            return None;
        } else {
            // Spaces, hyphens and apostrophes preserve known syllable boundaries.
            boundaries.insert(letters.len());
        }
    }
    static DICTIONARY: std::sync::LazyLock<BTreeSet<&str>> =
        std::sync::LazyLock::new(|| SYLLABLES.split_whitespace().collect());
    let dictionary = &*DICTIONARY;
    fn solve(
        letters: &[(char, u8)],
        boundaries: &BTreeSet<usize>,
        dictionary: &BTreeSet<&str>,
        start: usize,
        memo: &mut BTreeMap<usize, Option<Vec<String>>>,
    ) -> Option<Vec<String>> {
        if start == letters.len() {
            return Some(Vec::new());
        }
        if let Some(answer) = memo.get(&start) {
            return answer.clone();
        }
        let end_limit = boundaries
            .range(start + 1..)
            .next()
            .copied()
            .unwrap_or(letters.len())
            .min(start + 6);
        // Segment independently, then check the Han count: never split a valid
        // syllable unnaturally just to make a malformed answer fit.
        for end in (start + 1..=end_limit).rev() {
            let part = &letters[start..end];
            let base: String = part.iter().map(|&(c, _)| c).collect();
            if !dictionary.contains(base.as_str())
                || part.iter().filter(|(_, t)| *t != 0).count() > 1
            {
                continue;
            }
            if let Some(mut rest) = solve(letters, boundaries, dictionary, end, memo) {
                let tone = part
                    .iter()
                    .find_map(|(_, t)| (*t != 0).then_some(*t))
                    .unwrap_or(0);
                rest.insert(0, mark_tone(&base, tone));
                memo.insert(start, Some(rest.clone()));
                return Some(rest);
            }
        }
        memo.insert(start, None);
        None
    }
    solve(&letters, &boundaries, dictionary, 0, &mut BTreeMap::new())
        .filter(|s| s.len() == syllable_count)
        .map(|s| s.join(" "))
}

fn mark_tone(base: &str, tone: u8) -> String {
    if tone == 0 || tone == 5 {
        return base.to_owned();
    }
    let mut chars: Vec<_> = base.chars().collect();
    let index = chars
        .iter()
        .position(|&c| c == 'a')
        .or_else(|| chars.iter().position(|&c| c == 'e'))
        .or_else(|| {
            base.contains("ou")
                .then(|| chars.iter().position(|&c| c == 'o').unwrap())
        })
        .or_else(|| chars.iter().rposition(|c| "aeiouü".contains(*c)))
        .unwrap_or_else(|| chars.iter().rposition(|c| "mn".contains(*c)).unwrap_or(0));
    if let Some(row) = VOWELS.iter().find(|row| row.starts_with(chars[index])) {
        chars[index] = row.chars().nth(tone as usize).unwrap();
        chars.into_iter().collect()
    } else {
        let mark = ['\0', '\u{0304}', '\u{0301}', '\u{030c}', '\u{0300}'][tone as usize];
        let mut result = String::new();
        for (i, c) in chars.into_iter().enumerate() {
            result.push(c);
            if i == index {
                result.push(mark);
            }
        }
        result.nfc().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_alone_does_not_guess_the_iu_lemma() {
        for (text, reading, expected) in [
            ("言う", "ゆう", "ゆう"),
            ("言った", "ゆった", "ゆった"),
            ("言わない", "ゆわない", "ゆわない"),
            ("結う", "ゆう", "ゆう"),
            ("言葉", "ことば", "ことば"),
            ("夕方", "ゆうがた", "ゆうがた"),
        ] {
            assert_eq!(
                checked_surface_reading(Language::Japanese, text, reading).as_deref(),
                Some(expected)
            );
        }
    }

    #[test]
    fn flat_menus_do_not_assign_a_chinese_meaning_by_tone() {
        for (base, reading, options, expected) in [
            ("朋友", "péng yǒu", vec!["péng you"], "péng yǒu"),
            ("两个", "liǎng ge", vec!["liǎng gè"], "liǎng ge"),
            ("东西", "dōng xi", vec!["dōng xi", "dōng xī"], "dōng xi"),
            ("东西", "dōng xī", vec!["dōng xi", "dōng xī"], "dōng xī"),
            ("当年", "dàng nián", vec!["dāng nián"], "dàng nián"),
            ("么", "ma", vec!["má"], "ma"),
            ("么", "me", vec!["ma", "me", "má"], "me"),
        ] {
            let options: Vec<String> = options.into_iter().map(str::to_owned).collect();
            assert_eq!(
                menu_form(Language::ChineseSimplified, base, reading, &options).as_deref(),
                Some(expected)
            );
        }
        assert!(tone_variant("dài shàng", "dài shang"));
        assert!(tone_variant("dài shang", "dài shàng"));
        assert!(!tone_variant("ma", "me"));
        assert!(!tone_variant("rèn", "rén"));
    }

    #[test]
    fn a_allophones_follow_the_written_character() {
        for reading in ["a", "ā", "á", "ǎ", "à", "ya", "wa", "na", "yā", "wá", "nà"] {
            assert_eq!(
                canonicalize(Language::ChineseSimplified, "啊", reading).as_deref(),
                Some("a")
            );
        }
        assert_eq!(
            canonicalize(Language::ChineseSimplified, "哪", "na").as_deref(),
            Some("na")
        );
    }

    #[test]
    fn particles_take_the_menus_written_kana_without_rewriting_lexical_kana() {
        let menu = |m: &[&str]| m.iter().map(|r| (*r).to_owned()).collect::<Vec<_>>();
        for (base, reading, offered, expected) in [
            ("私は", "わたしわ", menu(&["わたしは"]), "わたしは"),
            ("学校へ", "がっこうえ", menu(&["がっこうへ"]), "がっこうへ"),
            ("わけ", "わけ", menu(&["わけ"]), "わけ"),
            // The literal surface pattern accepts this misalignment; only the
            // menu shows that は is the particle, not 母's second mora.
            (
                "母は私に",
                "ははわわたしに",
                menu(&["はははわたしに"]),
                "はははわたしに",
            ),
            // わ written as わ is never a particle, whatever the menu offers.
            (
                "私は",
                "わたしは",
                menu(&["わたしは", "わたしわ"]),
                "わたしは",
            ),
        ] {
            assert_eq!(
                menu_form(Language::Japanese, base, reading, &offered).as_deref(),
                Some(expected),
                "{base}: {reading}"
            );
        }
        assert!(japanese_pattern("母は私に").is_match("ははわわたしに"));
        // Without a menu there is nothing to align against, so the spelling
        // stands and the surface guard rejects it rather than guessing.
        assert_eq!(
            checked_surface_reading(Language::Japanese, "私は", "わたしわ"),
            None
        );
    }

    #[test]
    fn sound_changes_do_not_erase_unrelated_pronunciations() {
        for (a, b) in [
            ("つくり", "づくり"),
            ("かた", "がた"),
            ("いち", "いっ"),
            ("がく", "がっ"),
        ] {
            assert!(sound_changed(b, a), "{b} from {a}");
            assert!(!sound_changed(a, b), "{a} is the plain form of {b}");
        }
        assert!(sound_changed("ぽん", "ほん"));
        assert!(!sound_changed("かた", "かた"));
        assert!(!sound_changed("ほう", "かた"));
        assert!(!sound_changed("いっ", "い"));
        // Only the first mora voices in rendaku.
        assert!(!sound_changed("あが", "あか"));
    }
    #[test]
    fn okurigana_rejects_lemma_copy_on_every_reading_path() {
        let pattern = japanese_pattern("入った");
        assert!(pattern.is_match("はいった"));
        assert!(pattern.is_match("いった"));
        assert!(!pattern.is_match("はいる"));
        assert!(!pattern.is_match("はいったよ"));
        assert!(japanese_pattern("ああ、神様").is_match("ああかみさま"));
        assert!(japanese_pattern("食ベた").is_match("たべた"));
        assert!(japanese_pattern("取り消す").is_match("とりけす"));
        assert!(japanese_pattern("三ヶ月").is_match("さんかげつ"));
        assert!(japanese_pattern("三ヵ月").is_match("さんかげつ"));
        assert!(japanese_pattern("霞ヶ関").is_match("かすみがせき"));
        assert!(!japanese_pattern("取り消す").is_match("とけす"));
    }

    #[test]
    fn lexical_tones_follow_the_written_yi_and_bu_characters() {
        for (surface, reading, expected) in [
            ("你一个人", "nǐ yí gè rén", "nǐ yī gè rén"),
            ("一点", "yì diǎn", "yī diǎn"),
            ("不是", "bú shì", "bù shì"),
            ("移步", "yí bù", "yí bù"),
            ("八个", "bā ge", "bā ge"),
            ("八个", "bā gè", "bā gè"),
        ] {
            assert_eq!(
                canonicalize(Language::ChineseSimplified, surface, reading).as_deref(),
                Some(expected)
            );
        }
    }
    #[test]
    fn wave_dash_and_long_vowel_mark_have_one_japanese_reading() {
        for reading in ["たべてま〜す", "たべてま～す", "たべてまーす"] {
            assert_eq!(
                checked_surface_reading(Language::Japanese, "食べてま〜す", reading).as_deref(),
                Some("たべてまーす")
            );
        }
    }
    #[test]
    fn japanese_presentation_is_canonical() {
        assert_eq!(
            canonicalize(Language::Japanese, "ああ、神様", "アア、カミサマ！"),
            Some("ああかみさま".into())
        );
        assert_eq!(
            canonicalize(Language::Japanese, "段小樓", "とぁん・しゃおろう"),
            Some("とぁんしゃおろう".into())
        );
        assert_eq!(canonicalize(Language::Japanese, "何", "nani"), None);
    }
    #[test]
    fn pinyin_case_spacing_numbers_and_neutral_tones() {
        for (base, input, output) in [
            ("一个小男孩", "yī gè xiǎo nánhái", "yī gè xiǎo nán hái"),
            ("陈家村", "Chén Jiā Cūn", "chén jiā cūn"),
            ("银行", "yin2 hang2", "yín háng"),
            ("孩子", "hai2 zi5", "hái zi"),
            ("女孩", "nu:3 hai2", "nǚ hái"),
            ("长安", "cháng'ān", "cháng ān"),
            ("花儿", "huār", "huā r"),
            ("哼", "hng", "hng"),
        ] {
            assert_eq!(
                canonicalize(Language::ChineseSimplified, base, input).as_deref(),
                Some(output),
                "{base}: {input}"
            );
        }
        assert_eq!(
            canonicalize(Language::ChineseSimplified, "银行", "yín"),
            None
        );
        assert_eq!(
            canonicalize(Language::ChineseSimplified, "银行", "銀行"),
            None
        );
        assert_eq!(
            canonicalize(Language::ChineseSimplified, "银行", "hello"),
            None
        );
    }
}
