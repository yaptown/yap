/// Universal Dependencies morphological features
use crate::{Language, PartOfSpeech};
use schemars::JsonSchema;

pub trait FeatureSet {
    fn name() -> &'static str;
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool;
}

/// This feature typically applies to pronouns, pronominal adjectives (determiners), pronominal numerals (quantifiers) and pronominal adverbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum PronType {}

/// Some languages (especially Slavic) have a complex system of numerals. For example, in the school grammar of Czech, the main part of speech is “numeral”, it includes almost everything where counting is involved and there are various subtypes. It also includes interrogative, relative, indefinite and demonstrative words referring to numbers (words like kolik / how many, tolik / so many, několik / some, a few), so at the same time we may have a non-empty value of PronType. (In English, these words are called quantifiers and they are considered a subgroup of determiners.)
///
/// From the syntactic point of view, some numtypes behave like adjectives and some behave like adverbs. We tag them ADJ and ADV respectively. Thus the NumType feature applies to several different parts of speech:
///
/// NUM: cardinal numerals
/// DET: quantifiers
/// ADJ: definite adjectival, e.g. ordinal numerals
/// ADV: adverbial (e.g. ordinal and multiplicative) numerals, both definite and pronominal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum NumType {}

/// Boolean feature of pronouns, determiners or adjectives. It tells whether the word is possessive.
///
/// While many tagsets would have “possessive” as one of the various pronoun types, this feature is intentionally separate from PronType, as it is orthogonal to pronominal types. Several of the pronominal types can be optionally possessive, and adjectives can too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Poss {
    Yes,
}

/// Boolean feature, typically of pronouns or determiners. It tells whether the word is reflexive, i.e. refers to the subject of its clause.
///
/// While many tagsets would have “reflexive” as one of the various pronoun types, this feature is intentionally separate from PronType. When used with pronouns and determiners, it should be combined with PronType=Prs, regardless whether they really distinguish the Person feature (in some languages they do, in others they do not).
///
/// Note that forms that are canonically reflexive sometimes have other functions in the language, too. The feature Reflex=Yes denotes the word type, not its actual function in context (which can be distinguished by dependency relation types). Hence the feature is not restricted to situations where the word is used truly reflexively.
///
/// For example, reflexive clitics in European languages often have a wide array of possible functions (middle, passive, inchoative, impersonal, or even as a lexical morpheme). Besides that, reflexives in some languages are also used for emphasis (while other languages have separate emphatic pronouns), and in some languages they signal reciprocity (while other languages have separate reciprocal pronouns). Using Reflex=Yes with all of them has the benefit that they can be easily identified (however, if it is possible for the annotators to distinguish contexts where a reflexive pronoun is used reciprocally or emphatically, it is possible to combine Reflex=Yes with PronType=Rcp or PronType=Emp, instead of PronType=Prs).
///
/// Note that while some languages also have reflexive verbs, these are in fact fused verbs with reflexive pronouns, as in Spanish despertarse or Russian проснуться (both meaning “to wake up”). Thus in these cases the fused token will be split to two syntactic words, one of them being a reflexive pronoun. In languages where the reflexive pronoun is not split, it may be more appropriate to mark the verb as the middle Voice than using Reflex=Yes with the verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Reflex {}

/// Clusivity is a feature of first-person plural personal pronouns. As such, it can also be reflected by inflection of verbs, e.g. in Plains Cree (Wolvengrey 2011 p. 66).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Clusivity {}

/// Gender is usually a lexical feature of nouns and inflectional feature of other parts of speech (pronouns, adjectives, determiners, numerals, verbs) that mark agreement with nouns. In English gender affects only the choice of the personal pronoun (he / she / it) and the feature is usually not encoded in English tagsets.
///
/// See also the related feature of Animacy.
///
/// African languages have an analogous feature of noun classes: there might be separate grammatical categories for flat objects, long thin objects etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Gender {}

/// Similarly to Gender (and to the African noun classes), animacy is usually a lexical feature of nouns and inflectional feature of other parts of speech (pronouns, adjectives, determiners, numerals, verbs) that mark agreement with nouns. Some languages distinguish only gender, some only animacy, and in some languages both gender and animacy play a role in the grammar. (Some non-UD tagsets then combine the two features into an extended system of genders; however, in UD the two features are annotated separately.)
///
/// Similarly to gender, the values of animacy refer to semantic properties of the noun, but this is only an approximation, referring to the prototypical members of the categroy. There are nouns that are treated as grammatically animate, although semantically the are inanimate.
///
/// The following table is an example of a three-way animacy distinction (human – animate nonhuman – inanimate) in the declension of the masculine determiner który “which” in Polish (boldface forms in the upper and lower rows differ from the middle row):
///
/// gender     sg-nom     sg-gen     sg-dat     sg-acc     sg-ins     sg-loc     pl-nom     pl-gen     pl-dat     pl-acc     pl-ins     pl-loc
/// animate human     który     którego     któremu     którego     którym     którym     którzy     których     którym     których     którymi     których
/// animate non-human     który     którego     któremu     którego     którym     którym     które     których     którym     które     którymi     których
/// inanimate     który     którego     któremu     który     którym     którym     które     których     którym     które     którymi     których
/// In the corresponding paradigm of Czech, only two values are distinguished: masculine animate and masculine inanimate:
///
/// gender     sg-nom     sg-gen     sg-dat     sg-acc     sg-ins     sg-loc     pl-nom     pl-gen     pl-dat     pl-acc     pl-ins     pl-loc
/// animate     který     kterého     kterému     kterého     kterým     kterém     kteří     kterých     kterým     které     kterými     kterých
/// inanimate     který     kterého     kterému     který     kterým     kterém     které     kterých     kterým     které     kterými     kterých
/// More generally: Some languages distinguish animate vs. inanimate (e.g. Czech masculines), some languages distinguish human vs. non-human (e.g. Yuwan, a Ryukyuan language), and others distinguish three values, human vs. non-human animate vs. inanimate (e.g. Polish masculines).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Animacy {}

/// NounClass is similar to Gender and Animacy because it is to a large part a lexical category of nouns and other parts of speech inflect for it to show agreement (pronouns, adjectives, determiners, numerals, verbs).
///
/// The distinction between gender and noun class is not sharp and is partially conditioned by the traditional terminology of a given language family. In general, the feature is called gender if the number of possible values is relatively low (typically 2-4) and the partition correlates with sex of people and animals. In language families where the number of categories is high (10-20), the feature is usually called noun class. No language family uses both the features.
///
/// In Bantu languages, the noun class also encodes Number; therefore it is a lexical-inflectional feature of nouns. The words should be annotated with the Number feature in addition to NounClass, despite the fact that people who know Bantu could infer the number from the noun class. The lemma of the noun should be its singular form.
///
/// The set of values of this feature is specific for a language family or group. Within the group, it is possible to identify classes that have similar meaning across languages (although some classes may have merged or disappeared in some languages in the group). The value of the NounClass feature consists of a short identifier of the language group (e.g., Bantu), and the number of the class (there is a standardized class numbering system accepted by scholars of the various Bantu languages; similar numbering systems should be created for the other families that have noun classes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum NounClass {}

/// Number is usually an inflectional feature of nouns and, depending on language, other parts of speech (pronouns, adjectives, determiners, numerals, verbs) that mark agreement with nouns.
///
/// In languages where noun phrases are pluralized using a specific function word (pluralizer), this function word is tagged DET and Number=Plur is its lexical feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Number {}

/// Case is usually an inflectional feature of nouns and, depending on language, other parts of speech (pronouns, adjectives, determiners, numerals, verbs) that mark agreement with nouns.
///
/// Case can also be a lexical feature of adpositions and describe the case meaning that the adposition contributes to the nominal in which it appears. (This usage of the feature is typical for languages that do not have case morphology on nouns. For languages that have both adpositions and morphological case, the traditional set of cases is determined by the nominal forms and it does not cover adpositional meanings.) In some non-UD tagsets, case of adpositions is used as a valency feature (saying that the adposition requires its nominal argument to be in that morphological case); however, annotating adposition valency case in UD treebanks would be superfluous because the same case feature can be found at the nominal to which the adposition belongs.
///
/// Case helps specify the role of the noun phrase in the sentence, especially in free-word-order languages. For example, the nominative and accusative cases often distinguish subject and object of the verb, while in fixed-word-order languages these functions would be distinguished merely by the positions of the nouns in the sentence.
///
/// Here on the level of morphosyntactic features we are dealing with case expressed morphologically, i.e. by bound morphemes (affixes). Note that on a higher level case can be understood more broadly as the role, and it can be also expressed by adding an adposition to the noun. What is expressed by affixes in one language can be expressed using adpositions in another language. Cf. the case dependency label.
///
/// Examples
/// [cs] nominative matka “mother”, genitive matky, dative matce, accusative matku, vocative matko, locative matce, instrumental matkou
/// [de] nominative der Mann “the man”, genitive des Mannes, dative dem Mann, accusative den Mann
/// [en] nominative/direct case he, she, accusative/oblique case him, her.
/// The descriptions of the individual case values below include semantic hints about the prototypical meaning of the case. Bear in mind that quite often a case will be used for a meaning that is totally unrelated to the meaning mentioned here. Valency of verbs, adpositions and other words will determine that the noun phrase must be in a particular grammatical case to fill a particular valency slot (semantic role). It is much the same as trying to explain the meaning of prepositions: most people would agree that the central meaning of English in is location in space or time but there are phrases where the meaning is less locational: In God we trust. Say it in English.
///
/// Note that Indian corpora based on the so-called Paninian model use a related feature called vibhakti. It is a merger of the Case feature described here and of various postpositions. Values of the feature are language-dependent because they are copies of the relevant morphemes (either bound morphemes or postpositions). Vibhakti can be mapped on the Case values described here if we know 1. which source values are bound morphemes (postpositions are separate nodes for us) and 2. what is their meaning. For instance, the genitive case (Gen) in Bengali is marked using the suffix -ra (-র), i.e. vib=era. In Hindi, the suffix has been split off the noun and it is now written as a separate word – the postposition kā/kī/ke (का/की/के). Even if the postpositional phrase can be understood as a genitive noun phrase, the noun is not in genitive. Instead, the postposition requires that it takes one of three case forms that are marked directly on the noun: the oblique case (Acc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Case {}

/// Definiteness is typically a feature of nouns, adjectives and articles. Its value distinguishes whether we are talking about something known and concrete, or something general or unknown. It can be marked on definite and indefinite articles, or directly on nouns, adjectives etc. In Arabic, definiteness is also called the “state”.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Definite {}

/// Deixis is typically a feature of demonstrative pronouns, determiners, and adverbs. Its value classifies the location of the referred entity with respect to the location of the speaker or of the hearer. The common distinction is distance (proximate vs. remote entities); in some languages, elevation is distinguished as well (e.g., the entity is located higher or lower than the speaker).
///
/// If it is necessary to distinguish the person whose location is the reference point (speaker or hearer), the feature DeixisRef can be used in addition to Deixis. See also the Wolof examples below. DeixisRef is not needed if all deictic expressions in the language are relative to the same person (probably the speaker).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Deixis {}

/// DeixisRef is a feature of demonstrative pronouns, determiners, and adverbs, accompanying Deixis when necessary. Deixis encodes position of an entity relative to either the speaker or the hearer. If it is necessary to distinguish the person whose location is the reference point (speaker or hearer), DeixisRef is used. DeixisRef is not needed if all deictic expressions in the language are relative to the same person (probably the speaker), or if they do not distinguish the reference point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum DeixisRef {}

/// Degree of comparison is typically an inflectional feature of some adjectives and adverbs. A different flavor of degree is diminutives and augmentatives, which often apply to nouns but are not restricted to them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Degree {}

/// Even though the name of the feature seems to suggest that it is used exclusively with verbs, it is not the case. Some verb forms in some languages actually form a gray zone between verbs and other parts of speech (nouns, adjectives and adverbs). For instance, participles may be either classified as verbs or as adjectives, depending on language and context. In both cases VerbForm=Part may be used to separate them from other verb forms or other types of adjectives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum VerbForm {}

/// Mood is a feature that expresses modality and subclassifies finite verb forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Mood {}

/// Tense is typically a feature of verbs. It may also occur with other parts of speech (nouns, adjectives, adverbs), depending on whether borderline word forms such as participles are classified as verbs or as the other category.
///
/// Tense is a feature that specifies the time when the action took / takes / will take place, in relation to a reference point. The reference is often the moment of producing the sentence, but it can be also another event in the context. In some languages (e.g. English), some tenses are actually combinations of tense and aspect. In other languages (e.g. Czech), aspect and tense are separate, although not completely independent of each other.
///
/// Note that we are defining features that apply to a single word. If a tense is constructed periphrastically (two or more words, e.g. auxiliary verb indicative + participle of the main verb) and none of the participating words are specific to this tense, then the features will probably not directly reveal the tense. For instance, [en] I had been there is past perfect (pluperfect) tense, formed periphrastically by the simple past tense of the auxiliary to have and the past participle of the main verb to be. The auxiliary will be tagged VerbForm=Fin|Mood=Ind|Tense=Past and the participle will have VerbForm=Part|Tense=Past; none of the two will have Tense=Pqp. On the other hand, Portuguese can form the pluperfect morphologically as just one word, such as estivera, which will thus be tagged VerbForm=Fin|Mood=Ind|Tense=Pqp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Tense {}

/// Aspect is typically a feature of verbs. It may also occur with other parts of speech (nouns, adjectives, adverbs), depending on whether borderline word forms such as gerunds and participles are classified as verbs or as the other category.
///
/// Aspect is a feature that specifies duration of the action in time, whether the action has been completed etc. In some languages (e.g. English), some tenses are actually combinations of tense and aspect. In other languages (e.g. Czech), aspect and tense are separate, although not completely independent of each other.
///
/// In Czech and other Slavic languages, aspect is a lexical feature. Pairs of imperfective and perfective verbs exist and are often morphologically related but the space is highly irregular and the verbs are considered to belong to separate lemmas.
///
/// Since we proceed bottom-up, the current standard covers only a few aspect values found in corpora. See Wikipedia (http://en.wikipedia.org/wiki/Grammatical_aspect) for a long list of other possible aspects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Aspect {}

/// Voice is typically a feature of verbs. It may also occur with other parts of speech (nouns, adjectives, adverbs), depending on whether borderline word forms such as gerunds and participles are classified as verbs or as the other category.
///
/// For Indo-European speakers, voice means mainly the active-passive distinction. In other languages, other shades of verb meaning are categorized as voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Voice {}

/// Evidentiality is the morphological marking of a speaker’s source of information (Aikhenvald, 2004). It is sometimes viewed as a category of mood and modality.
///
/// Many different values are attested in the world’s languages. At present we only cover the firsthand vs. non-firsthand distinction, needed in Turkish. It distinguishes there the normal past tense (firsthand, also definite past tense, seen past tense) from the so-called miş-past (non-firsthand, renarrative, indefinite, heard past tense).
///
/// Aikhenvald also distinguishes reported evidentiality, occurring in Estonian and Latvian, among others. We currently use the quotative Mood for this.
///
/// Note: Evident is a new universal feature in UD version 2. It was used as a language-specific feature (under the name Evidentiality) in UD v1 for Turkish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Evident {}

/// Polarity is typically a feature of verbs, adjectives, sometimes also adverbs and nouns in languages that negate using bound morphemes. In languages that negate using a function word, Polarity is used to mark that function word, unless it is a pro-form already marked with PronType=Neg (see below).
///
/// Positive polarity (affirmativeness) is rarely, if at all, encoded using overt morphology. The feature value Polarity=Pos is usually used to signal that a lemma has negative forms but this particular form is not negative. Using the feature in such cases is somewhat optional for words that can be negated but rarely are. Language-specific documentation should define under which circumstances the positive polarity is annotated.
///
/// In Czech, for instance, all verbs and adjectives can be negated using the prefix ne-.
///
/// In English, verbs are negated using the particle not. English adjectives can be negated with not, or sometimes using prefixes (wise – unwise, probable – improbable), although the use of prefixes is less productive than in Czech. In general, only the most grammatical (as opposed to lexical) forms of negation should receive Polarity=Neg.
///
/// Note that Polarity=Neg is not the same thing as PronType=Neg. For pronouns and other pronominal parts of speech there is no such binary opposition as for verbs and adjectives. (There is no such thing as “affirmative pronoun”.)
///
/// The Polarity feature can be also used to distinguish response interjections yes and no.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Polarity {}

/// Person is typically feature of personal and possessive pronouns / determiners, and of verbs. On verbs it is in fact an agreement feature that marks the person of the verb’s subject (some languages, e.g. Basque, can also mark person of objects). Person marked on verbs makes it unnecessary to always add a personal pronoun as subject and thus subjects are sometimes dropped (pro-drop languages).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Person {}

/// Various languages have various means to express politeness or respect; some of the means are morphological. Three to four dimensions of politeness are distinguished in linguistic literature. The Polite feature currently covers (and mixes) two of them; a more elaborate system of feature values may be devised in future versions of UD if needed. The two axes covered are:
///
/// speaker-referent axis (meant to include the addressee when he happens to be the referent)
/// speaker-addressee axis (word forms depend on who is the addressee, although the addressee is not referred to)
/// Changing pronouns and/or person and/or number of the verb forms when respectable persons are addressed in Indo-European languages belongs to the speaker-referent axis because the honorific pronouns are used to refer to the addressee.
///
/// In Czech, formal second person has the same form for singular and plural, and is identical to informal second person plural. This involves both the pronoun and the finite verb but not a participle, which has no special formal form (that is, formal singular is identical to informal singular, not to informal plural).
///
/// In German, Spanish or Hindi, both number and person are changed (informal third person is used as formal second person) and in addition, special pronouns are used that only occur in the formal register ([de] Sie; [es] usted, ustedes; [hi] आप āpa).
///
/// In Japanese, verbs and other words have polite and informal forms but the polite forms are not referring to the addressee (they are not in second person). They are just used because of who the addressee is, even if the topic does not involve the addressee at all. This kind of polite language is called teineigo (丁寧語) and belongs to the speaker-addressee axis. Nevertheless, we currently use the same values for both axes, i.e. Polite=Form can be used for teineigo too. This approach may be refined in future.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum Polite {}

impl FeatureSet for PronType {
    fn name() -> &'static str {
        "PronType"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Applies to pronouns, pronominal adjectives (determiners), pronominal numerals (quantifiers) and pronominal adverbs
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Pron | PartOfSpeech::Det | PartOfSpeech::Num | PartOfSpeech::Adv
                )
            }
        }
    }
}

impl FeatureSet for NumType {
    fn name() -> &'static str {
        "NumType"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // NUM: cardinal numerals, DET: quantifiers, ADJ: ordinal numerals, ADV: adverbial numerals
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Num | PartOfSpeech::Det | PartOfSpeech::Adj | PartOfSpeech::Adv
                )
            }
        }
    }
}

impl FeatureSet for Poss {
    fn name() -> &'static str {
        "Poss"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Boolean feature of pronouns, determiners or adjectives
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Pron | PartOfSpeech::Det | PartOfSpeech::Adj
                )
            }
        }
    }
}

impl FeatureSet for Reflex {
    fn name() -> &'static str {
        "Reflex"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Typically of pronouns or determiners
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(pos, PartOfSpeech::Pron | PartOfSpeech::Det)
            }
        }
    }
}

impl FeatureSet for Clusivity {
    fn name() -> &'static str {
        "Clusivity"
    }
    fn applies_to(language: Language, _pos: PartOfSpeech) -> bool {
        // Feature of first-person plural pronouns and can be reflected in verb inflection
        // Not used in our current language set
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => false,
        }
    }
}

impl FeatureSet for Gender {
    fn name() -> &'static str {
        "Gender"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Lexical feature of nouns and inflectional feature of pronouns, adjectives, determiners, numerals, verbs
        // In English, only affects pronouns
        match language {
            Language::English => matches!(pos, PartOfSpeech::Pron),
            Language::French | Language::Spanish | Language::German | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Noun
                        | PartOfSpeech::Propn
                        | PartOfSpeech::Pron
                        | PartOfSpeech::Adj
                        | PartOfSpeech::Det
                        | PartOfSpeech::Num
                        | PartOfSpeech::Verb
                        | PartOfSpeech::Aux
                )
            }
        }
    }
}

impl FeatureSet for Animacy {
    fn name() -> &'static str {
        "Animacy"
    }
    fn applies_to(language: Language, _pos: PartOfSpeech) -> bool {
        // Lexical feature of nouns and inflectional feature of other parts of speech
        // Mainly used in Slavic languages (Polish, Czech), not common in our current language set
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => false,
        }
    }
}

impl FeatureSet for NounClass {
    fn name() -> &'static str {
        "NounClass"
    }
    fn applies_to(language: Language, _pos: PartOfSpeech) -> bool {
        // Mainly for Bantu and other African languages, not in our current language set
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => false,
        }
    }
}

impl FeatureSet for Number {
    fn name() -> &'static str {
        "Number"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Inflectional feature of nouns and other parts of speech (pronouns, adjectives, determiners, numerals, verbs)
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Noun
                        | PartOfSpeech::Propn
                        | PartOfSpeech::Pron
                        | PartOfSpeech::Adj
                        | PartOfSpeech::Det
                        | PartOfSpeech::Num
                        | PartOfSpeech::Verb
                        | PartOfSpeech::Aux
                )
            }
        }
    }
}

impl FeatureSet for Case {
    fn name() -> &'static str {
        "Case"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Inflectional feature of nouns and other parts of speech; can also be lexical feature of adpositions
        match language {
            Language::English => {
                // English has minimal case, mainly in pronouns
                matches!(pos, PartOfSpeech::Pron)
            }
            Language::German => {
                // German has case on nouns, pronouns, adjectives, determiners, adpositions
                matches!(
                    pos,
                    PartOfSpeech::Noun
                        | PartOfSpeech::Propn
                        | PartOfSpeech::Pron
                        | PartOfSpeech::Adj
                        | PartOfSpeech::Det
                        | PartOfSpeech::Adp
                )
            }
            Language::Korean => {
                // Korean has case particles
                matches!(
                    pos,
                    PartOfSpeech::Noun
                        | PartOfSpeech::Propn
                        | PartOfSpeech::Pron
                        | PartOfSpeech::Adp
                )
            }
            Language::French | Language::Spanish => {
                // French and Spanish don't have morphological case
                false
            }
        }
    }
}

impl FeatureSet for Definite {
    fn name() -> &'static str {
        "Definite"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Feature of nouns, adjectives and articles
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Noun
                        | PartOfSpeech::Propn
                        | PartOfSpeech::Adj
                        | PartOfSpeech::Det
                )
            }
        }
    }
}

impl FeatureSet for Deixis {
    fn name() -> &'static str {
        "Deixis"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Feature of demonstrative pronouns, determiners, and adverbs
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Pron | PartOfSpeech::Det | PartOfSpeech::Adv
                )
            }
        }
    }
}

impl FeatureSet for DeixisRef {
    fn name() -> &'static str {
        "DeixisRef"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Feature of demonstrative pronouns, determiners, and adverbs
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Pron | PartOfSpeech::Det | PartOfSpeech::Adv
                )
            }
        }
    }
}

impl FeatureSet for Degree {
    fn name() -> &'static str {
        "Degree"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Inflectional feature of adjectives and adverbs; diminutives/augmentatives apply to nouns
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Adj
                        | PartOfSpeech::Adv
                        | PartOfSpeech::Noun
                        | PartOfSpeech::Propn
                )
            }
        }
    }
}

impl FeatureSet for VerbForm {
    fn name() -> &'static str {
        "VerbForm"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Applies to verbs and participles (which may be classified as adjectives)
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Verb | PartOfSpeech::Aux | PartOfSpeech::Adj
                )
            }
        }
    }
}

impl FeatureSet for Mood {
    fn name() -> &'static str {
        "Mood"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Subclassifies finite verb forms
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(pos, PartOfSpeech::Verb | PartOfSpeech::Aux)
            }
        }
    }
}

impl FeatureSet for Tense {
    fn name() -> &'static str {
        "Tense"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Feature of verbs; may occur with participles classified as other parts of speech
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Verb
                        | PartOfSpeech::Aux
                        | PartOfSpeech::Adj
                        | PartOfSpeech::Noun
                        | PartOfSpeech::Adv
                )
            }
        }
    }
}

impl FeatureSet for Aspect {
    fn name() -> &'static str {
        "Aspect"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Feature of verbs; may occur with gerunds and participles classified as other parts of speech
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Verb
                        | PartOfSpeech::Aux
                        | PartOfSpeech::Adj
                        | PartOfSpeech::Noun
                        | PartOfSpeech::Adv
                )
            }
        }
    }
}

impl FeatureSet for Voice {
    fn name() -> &'static str {
        "Voice"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Feature of verbs; may occur with gerunds and participles classified as other parts of speech
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Verb
                        | PartOfSpeech::Aux
                        | PartOfSpeech::Adj
                        | PartOfSpeech::Noun
                        | PartOfSpeech::Adv
                )
            }
        }
    }
}

impl FeatureSet for Evident {
    fn name() -> &'static str {
        "Evident"
    }
    fn applies_to(language: Language, _pos: PartOfSpeech) -> bool {
        // Evidentiality marking, mainly for Turkish, Estonian, Latvian (not in our current language set)
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => false,
        }
    }
}

impl FeatureSet for Polarity {
    fn name() -> &'static str {
        "Polarity"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Feature of verbs, adjectives, adverbs, nouns (negation); also function words and interjections (yes/no)
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Verb
                        | PartOfSpeech::Aux
                        | PartOfSpeech::Adj
                        | PartOfSpeech::Adv
                        | PartOfSpeech::Noun
                        | PartOfSpeech::Part
                        | PartOfSpeech::Intj
                        | PartOfSpeech::Adp
                )
            }
        }
    }
}

impl FeatureSet for Person {
    fn name() -> &'static str {
        "Person"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Feature of personal and possessive pronouns/determiners, and of verbs
        match language {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Pron | PartOfSpeech::Det | PartOfSpeech::Verb | PartOfSpeech::Aux
                )
            }
        }
    }
}

impl FeatureSet for Polite {
    fn name() -> &'static str {
        "Polite"
    }
    fn applies_to(language: Language, pos: PartOfSpeech) -> bool {
        // Politeness marking on pronouns and verbs
        // Mentioned specifically for German, Spanish, Korean
        match language {
            Language::German | Language::Spanish | Language::Korean => {
                matches!(
                    pos,
                    PartOfSpeech::Pron | PartOfSpeech::Det | PartOfSpeech::Verb | PartOfSpeech::Aux
                )
            }
            Language::French | Language::English => {
                // French and English can have polite forms but less morphologically marked
                matches!(
                    pos,
                    PartOfSpeech::Pron | PartOfSpeech::Det | PartOfSpeech::Verb | PartOfSpeech::Aux
                )
            }
        }
    }
}
