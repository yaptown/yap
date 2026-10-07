//! Deck-independent, ephemeral onboarding. Hosts render the view and execute
//! effects; persisted answers still use the existing deck-selection events.
use crate::deck_selection::{
    DailyReviewTarget, ExperienceLevel, HeardAbout, Motivation, OnboardingSelections,
};
use crate::{get_daily_goal_options, get_language_metadata};
use language_utils::Language;
use serde::{Deserialize, Serialize};

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardingPurpose {
    App,
    AnkiDeck,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardingStep {
    HeardAbout,
    Motivation,
    Experience,
    Achievements,
    SrsTeaser,
    SrsIntro,
    SrsConclusion,
    StudyGoal,
    Notifications,
    Ready,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OnboardingChoice {
    HeardAbout { value: HeardAbout },
    Motivation { value: Motivation },
    Experience { value: ExperienceLevel },
    StudyGoal { value: DailyReviewTarget },
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingState {
    pub target_language: Language,
    pub purpose: OnboardingPurpose,
    pub steps: Vec<OnboardingStep>,
    pub step_index: u32,
    pub heard_about: Option<HeardAbout>,
    pub selections: OnboardingSelections,
    pub review_count: u8,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OnboardingEvent {
    Choose {
        choice: OnboardingChoice,
    },
    Next,
    Back,
    StartFromScratch,
    NotificationsDone,
    /// The SDK can initialize after mounting. Only the first step may change
    /// the itinerary, and refreshing it must not discard the selected answer.
    RefreshNotificationOffer {
        offer: bool,
    },
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OnboardingEffect {
    SaveHeardAbout { value: HeardAbout },
    Complete { selections: OnboardingSelections },
    Exit,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingTransition {
    pub state: OnboardingState,
    pub effects: Vec<OnboardingEffect>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingOption {
    pub choice: OnboardingChoice,
    pub label: String,
    pub detail: Option<String>,
    pub selected: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingAchievement {
    /// Illustration id from the yap-icons repo.
    pub icon: String,
    pub text: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingStudy {
    pub title: String,
    pub authors: String,
    pub year: u32,
    pub journal: String,
    pub url: String,
}

/// One stretch of the forgetting-curve illustration: memory starts full at
/// `start` and decays to `retained` by `end`. Times and memory run from 0 to 1.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingCurve {
    pub start: f64,
    pub end: f64,
    pub retained: f64,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingChart {
    pub x_label: String,
    pub y_label: String,
    pub accessibility_label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OnboardingContent {
    Choices {
        options: Vec<OnboardingOption>,
    },
    Achievements {
        items: Vec<OnboardingAchievement>,
    },
    Studies {
        studies: Vec<OnboardingStudy>,
        conclusion: String,
    },
    /// Every curve after the first starts at a review.
    Review {
        eyebrow: String,
        title_emphasis: String,
        curves: Vec<OnboardingCurve>,
        caption: String,
        review_label: Option<String>,
        learned: bool,
        learned_title: String,
        learned_body: String,
        chart: OnboardingChart,
    },
    /// Words drifting into memory and staying there.
    Remember {
        words: Vec<String>,
    },
    Notifications {
        icon: String,
        body: String,
        enable_label: String,
        enabling_label: String,
        skip_label: String,
    },
    Ready {
        icon: String,
        body: String,
        start_fresh_label: Option<String>,
    },
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingPrimary {
    pub label: String,
    pub enabled: bool,
    pub show_arrow: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingView {
    pub step: OnboardingStep,
    pub step_number: u32,
    pub total_steps: u32,
    pub progress_percent: f64,
    pub progress_label: String,
    pub back_label: String,
    pub navigation_title: String,
    pub title: String,
    pub content: OnboardingContent,
    pub primary: Option<OnboardingPrimary>,
}

#[bridgerton::bridge]
pub fn onboarding_start(
    target_language: Language,
    has_heard_about: bool,
    offer_notifications: bool,
    purpose: OnboardingPurpose,
) -> OnboardingState {
    use OnboardingStep::*;
    let steps = match purpose {
        OnboardingPurpose::App => {
            let mut steps = vec![];
            if !has_heard_about {
                steps.push(HeardAbout);
            }
            steps.extend([
                Motivation,
                Experience,
                Achievements,
                SrsTeaser,
                SrsIntro,
                SrsConclusion,
                StudyGoal,
            ]);
            if offer_notifications {
                steps.push(Notifications);
            }
            steps.push(Ready);
            steps
        }
        OnboardingPurpose::AnkiDeck => vec![Experience],
    };
    OnboardingState {
        target_language,
        purpose,
        steps,
        step_index: 0,
        heard_about: None,
        selections: OnboardingSelections::default(),
        review_count: 0,
    }
}

impl OnboardingState {
    fn step(&self) -> &OnboardingStep {
        &self.steps[self.step_index as usize]
    }
    fn move_to(&mut self, index: u32) {
        self.heard_about = None;
        self.review_count = 0;
        self.step_index = index;
    }
    fn advance(&mut self) {
        self.move_to(self.step_index + 1);
    }
    fn proceed(&mut self, effects: &mut Vec<OnboardingEffect>) {
        match self.step() {
            OnboardingStep::HeardAbout => {
                effects.push(OnboardingEffect::SaveHeardAbout {
                    value: self.heard_about.clone().unwrap(),
                });
                self.advance();
            }
            OnboardingStep::SrsIntro if self.review_count < LEARNED_AFTER => self.review_count += 1,
            _ if self.step_index as usize == self.steps.len() - 1 => effects.push(
                self.complete(self.selections.experience_level == Some(ExperienceLevel::New)),
            ),
            _ => self.advance(),
        }
    }
    fn complete(&self, starting_fresh: bool) -> OnboardingEffect {
        OnboardingEffect::Complete {
            selections: OnboardingSelections {
                starting_fresh,
                ..self.selections.clone()
            },
        }
    }
}

#[bridgerton::bridge]
pub fn onboarding_reduce(
    mut state: OnboardingState,
    event: OnboardingEvent,
) -> OnboardingTransition {
    use OnboardingEvent::*;
    let mut effects = vec![];
    match event {
        Choose { choice } => {
            match (state.step(), choice) {
                (OnboardingStep::HeardAbout, OnboardingChoice::HeardAbout { value }) => {
                    state.heard_about = Some(value)
                }
                (OnboardingStep::Motivation, OnboardingChoice::Motivation { value }) => {
                    state.selections.motivation = Some(value)
                }
                (OnboardingStep::Experience, OnboardingChoice::Experience { value }) => {
                    state.selections.experience_level = Some(value)
                }
                (OnboardingStep::StudyGoal, OnboardingChoice::StudyGoal { value }) => {
                    state.selections.study_goal = Some(value)
                }
                _ => return OnboardingTransition { state, effects },
            }
            // Picking an answer is the whole step; there is no Continue to press.
            state.proceed(&mut effects);
        }
        Back => {
            if state.step_index == 0 {
                state.move_to(0);
                effects.push(OnboardingEffect::Exit);
            } else {
                state.move_to(state.step_index - 1);
            }
        }
        Next => {
            if onboarding_view(state.clone())
                .primary
                .is_some_and(|p| p.enabled)
            {
                state.proceed(&mut effects);
            }
        }
        StartFromScratch if *state.step() == OnboardingStep::Ready => {
            effects.push(state.complete(true))
        }
        NotificationsDone if *state.step() == OnboardingStep::Notifications => state.advance(),
        RefreshNotificationOffer { offer }
            if state.step_index == 0 && state.purpose == OnboardingPurpose::App =>
        {
            state
                .steps
                .retain(|step| *step != OnboardingStep::Notifications);
            if offer {
                state
                    .steps
                    .insert(state.steps.len() - 1, OnboardingStep::Notifications);
            }
        }
        _ => {}
    }
    OnboardingTransition { state, effects }
}

#[bridgerton::bridge]
#[bridgerton::stable]
pub fn onboarding_view(state: OnboardingState) -> OnboardingView {
    use OnboardingContent::*;
    let metadata = get_language_metadata(state.target_language);
    let language = &metadata.native_name;
    let step = state.step().clone();
    let is_new = state.selections.experience_level == Some(ExperienceLevel::New);
    let mut primary = Some(OnboardingPrimary {
        label: if state.purpose == OnboardingPurpose::AnkiDeck {
            yap_i18n::tr!("onboarding-set-up-my-deck")
        } else {
            yap_i18n::tr!("common-continue")
        },
        enabled: true,
        show_arrow: true,
    });
    let mut choices = |title: String,
                       values: Vec<(OnboardingChoice, String, Option<String>)>,
                       selected: Option<OnboardingChoice>| {
        // Choosing advances by itself.
        primary = None;
        (
            title,
            Choices {
                options: values
                    .into_iter()
                    .map(|(choice, label, detail)| OnboardingOption {
                        selected: Some(&choice) == selected.as_ref(),
                        choice,
                        label,
                        detail,
                    })
                    .collect(),
            },
        )
    };
    let (title, content) = match step {
        OnboardingStep::HeardAbout => choices(
            yap_i18n::tr!("onboarding-how-did-you-hear-about-yap"),
            [
                (
                    HeardAbout::FriendsOrFamily,
                    yap_i18n::tr!("onboarding-friends-or-family"),
                ),
                (HeardAbout::Reddit, "Reddit".to_owned()),
                (HeardAbout::TikTok, "TikTok".to_owned()),
                (
                    HeardAbout::GoogleSearch,
                    yap_i18n::tr!("onboarding-google-search"),
                ),
                (HeardAbout::YouTube, "YouTube".to_owned()),
                (HeardAbout::Other, yap_i18n::tr!("onboarding-other")),
            ]
            .into_iter()
            .map(|(value, label)| (OnboardingChoice::HeardAbout { value }, label, None))
            .collect(),
            state
                .heard_about
                .map(|value| OnboardingChoice::HeardAbout { value }),
        ),
        OnboardingStep::Motivation => choices(
            yap_i18n::tr!(
                "onboarding-why-are-you-learning-language",
                language = (language).to_string()
            ),
            [
                (
                    Motivation::SpendTimeProductively,
                    yap_i18n::tr!("onboarding-spend-time-productively"),
                ),
                (
                    Motivation::SupportMyEducation,
                    yap_i18n::tr!("onboarding-support-my-education"),
                ),
                (
                    Motivation::ConnectWithPeople,
                    yap_i18n::tr!("onboarding-connect-with-people"),
                ),
                (
                    Motivation::BoostMyCareer,
                    yap_i18n::tr!("onboarding-boost-my-career"),
                ),
                (
                    Motivation::PrepareForTravel,
                    yap_i18n::tr!("onboarding-prepare-for-travel"),
                ),
                (
                    Motivation::JustForFun,
                    yap_i18n::tr!("onboarding-just-for-fun"),
                ),
                (Motivation::Other, yap_i18n::tr!("onboarding-other")),
            ]
            .into_iter()
            .map(|(value, label)| (OnboardingChoice::Motivation { value }, label, None))
            .collect(),
            state
                .selections
                .motivation
                .map(|value| OnboardingChoice::Motivation { value }),
        ),
        OnboardingStep::Experience => choices(
            yap_i18n::tr!(
                "onboarding-how-much-language-do-you-know",
                language = (language).to_string()
            ),
            [
                (
                    ExperienceLevel::New,
                    yap_i18n::tr!(
                        "onboarding-i-m-new-to-language",
                        language = (language).to_string()
                    ),
                ),
                (
                    ExperienceLevel::CommonWords,
                    yap_i18n::tr!("onboarding-i-know-some-common-words"),
                ),
                (
                    ExperienceLevel::BasicConversations,
                    yap_i18n::tr!("onboarding-i-can-have-basic-conversations"),
                ),
                (
                    ExperienceLevel::VariousTopics,
                    yap_i18n::tr!("onboarding-i-can-talk-about-various-topics"),
                ),
                (
                    ExperienceLevel::MostTopics,
                    yap_i18n::tr!("onboarding-i-can-discuss-most-topics-in-detail"),
                ),
            ]
            .into_iter()
            .map(|(value, label)| (OnboardingChoice::Experience { value }, label, None))
            .collect(),
            state
                .selections
                .experience_level
                .map(|value| OnboardingChoice::Experience { value }),
        ),
        OnboardingStep::Achievements => (
            yap_i18n::tr!("onboarding-here-s-what-you-can-achieve"),
            Achievements {
                items: [
                    (
                        "ui.vocabulary",
                        yap_i18n::tr!("onboarding-build-a-large-vocabulary"),
                    ),
                    (
                        "ui.media",
                        yap_i18n::tr!(
                            "onboarding-enjoy-media",
                            language = (metadata.common_name).to_string()
                        ),
                    ),
                    (
                        "ui.speakers",
                        yap_i18n::tr!(
                            "onboarding-understand-speakers",
                            language = (metadata.common_name).to_string()
                        ),
                    ),
                ]
                .map(|(icon, text)| OnboardingAchievement {
                    icon: icon.into(),
                    text,
                })
                .into(),
            },
        ),
        OnboardingStep::SrsTeaser => (
            yap_i18n::tr!("onboarding-yap-is-based-on-one-scientifically-proven-idea"),
            Studies {
                studies: studies(),
                conclusion: yap_i18n::tr!("onboarding-spaced-repetition"),
            },
        ),
        OnboardingStep::SrsIntro => {
            let learned = state.review_count >= LEARNED_AFTER;
            primary = Some(OnboardingPrimary {
                label: if learned {
                    yap_i18n::tr!("common-continue")
                } else {
                    yap_i18n::tr!("common-review")
                },
                enabled: true,
                show_arrow: learned,
            });
            let shown = usize::from(state.review_count.min(LEARNED_AFTER - 1)) + 1;
            (
                yap_i18n::tr!("onboarding-memory-heading.before"),
                Review {
                    eyebrow: yap_i18n::tr!("onboarding-how-yap-works"),
                    title_emphasis: yap_i18n::tr!("onboarding-memory-heading.emphasis"),
                    curves: forgetting_curves().into_iter().take(shown).collect(),
                    caption: [
                        yap_i18n::tr!("onboarding-you-learn-a-new-word-and-right-away-you"),
                        yap_i18n::tr!(
                            "onboarding-reviewing-it-just-before-it-slips-away-brings-it"
                        ),
                        yap_i18n::tr!("onboarding-and-each-time-you-forget-it-more-slowly"),
                        yap_i18n::tr!(
                            "onboarding-so-the-reviews-can-get-further-and-further-apart"
                        ),
                    ][shown - 1]
                        .clone(),
                    review_label: (1..=3).contains(&state.review_count).then(|| {
                        yap_i18n::tr!("onboarding-review-count", count = state.review_count)
                    }),
                    learned,
                    learned_title: yap_i18n::tr!("onboarding-word-learned"),
                    learned_body: yap_i18n::tr!("onboarding-that-word-is-now-in-long-term-memory"),
                    chart: OnboardingChart {
                        x_label: yap_i18n::tr!("onboarding-time"),
                        y_label: yap_i18n::tr!("onboarding-memory"),
                        accessibility_label: yap_i18n::tr!(
                            "onboarding-forgetting-curve-chart-showing-how-spaced-repetition-helps-memory"
                        ),
                    },
                },
            )
        }
        OnboardingStep::SrsConclusion => {
            primary.as_mut().unwrap().label = yap_i18n::tr!("onboarding-set-a-goal");
            (
                yap_i18n::tr!("onboarding-that-s-how-yap-makes-sure-you-remember-everything"),
                Remember {
                    words: sample_words(state.target_language).map(String::from).into(),
                },
            )
        }
        OnboardingStep::StudyGoal => choices(
            yap_i18n::tr!("onboarding-set-a-daily-study-goal"),
            get_daily_goal_options()
                .into_iter()
                .map(|g| {
                    (
                        OnboardingChoice::StudyGoal { value: g.value },
                        yap_i18n::tr!("onboarding-study-minutes", minutes = g.minutes),
                        Some(yap_i18n::tr!(
                            "onboarding-first-week-words",
                            count = g.estimated_first_week_words
                        )),
                    )
                })
                .collect(),
            state
                .selections
                .study_goal
                .map(|value| OnboardingChoice::StudyGoal { value }),
        ),
        OnboardingStep::Notifications => {
            primary = None;
            (
                yap_i18n::tr!("onboarding-we-ll-remind-you-to-practice-so-it-becomes"),
                Notifications {
                    icon: "ui.reminder".into(),
                    body: yap_i18n::tr!("onboarding-a-small-daily-reminder-makes-it-easy-to-stay"),
                    enable_label: yap_i18n::tr!("onboarding-enable-reminders"),
                    enabling_label: yap_i18n::tr!("onboarding-enabling"),
                    skip_label: yap_i18n::tr!("common-not-now"),
                },
            )
        }
        OnboardingStep::Ready => {
            primary.as_mut().unwrap().label = if is_new {
                metadata.lets_go
            } else {
                yap_i18n::tr!("onboarding-find-my-level")
            };
            (
                if is_new {
                    yap_i18n::tr!("onboarding-let-s-start-from-the-beginning")
                } else {
                    yap_i18n::tr!("onboarding-now-let-s-find-the-best-place-to-start")
                },
                Ready {
                    icon: metadata.icon,
                    body: if is_new {
                        yap_i18n::tr!(
                            "onboarding-we-ll-build-your-language-foundation-step-by",
                            language = (language).to_string()
                        )
                    } else {
                        yap_i18n::tr!(
                            "onboarding-since-you-already-know-some-language-we-can",
                            language = (language).to_string()
                        )
                    },
                    start_fresh_label: (!is_new)
                        .then(|| yap_i18n::tr!("onboarding-start-from-scratch")),
                },
            )
        }
    };
    let step_number = state.step_index + 1;
    let total_steps = state.steps.len() as u32;
    OnboardingView {
        step,
        step_number,
        total_steps,
        progress_percent: f64::from(step_number) / f64::from(total_steps) * 100.0,
        progress_label: yap_i18n::tr!(
            "onboarding-step-step-number-of-total-steps",
            step_number = step_number,
            total_steps = total_steps
        ),
        back_label: yap_i18n::tr!("common-back"),
        navigation_title: metadata.yaptown_name,
        title,
        content,
        primary,
    }
}

/// Reviews in the forgetting-curve demo before the word counts as learned.
const LEARNED_AFTER: u8 = 4;

/// Each review restarts the curve, and each restart decays more slowly.
fn forgetting_curves() -> [OnboardingCurve; LEARNED_AFTER as usize] {
    let mut start = 0.0;
    [(0.12, 0.25), (0.2, 0.4), (0.3, 0.55), (0.38, 0.7)].map(|(width, retained)| {
        let curve = OnboardingCurve {
            start,
            end: start + width,
            retained,
        };
        start += width;
        curve
    })
}

/// Everyday words for the "remember everything" illustration.
fn sample_words(language: Language) -> [&'static str; 8] {
    use Language::*;
    match language {
        English => [
            "hello",
            "thanks",
            "friend",
            "tomorrow",
            "music",
            "beautiful",
            "coffee",
            "love",
        ],
        French => [
            "bonjour", "merci", "ami", "demain", "musique", "beau", "café", "amour",
        ],
        SpanishLatinAmerican | SpanishPeninsular => [
            "hola", "gracias", "amigo", "mañana", "música", "bonito", "café", "amor",
        ],
        German => [
            "hallo", "danke", "Freund", "morgen", "Musik", "schön", "Kaffee", "Liebe",
        ],
        Italian => [
            "ciao", "grazie", "amico", "domani", "musica", "bello", "caffè", "amore",
        ],
        PortugueseBrazilian | PortugueseEuropean => [
            "olá", "obrigado", "amigo", "amanhã", "música", "bonito", "café", "amor",
        ],
        Russian => [
            "привет",
            "спасибо",
            "друг",
            "завтра",
            "музыка",
            "красивый",
            "кофе",
            "любовь",
        ],
        Korean => [
            "안녕",
            "고마워",
            "친구",
            "내일",
            "음악",
            "예쁘다",
            "커피",
            "사랑",
        ],
        Japanese => [
            "こんにちは",
            "ありがとう",
            "友達",
            "明日",
            "音楽",
            "きれい",
            "コーヒー",
            "愛",
        ],
        ChineseSimplified => ["你好", "谢谢", "朋友", "明天", "音乐", "漂亮", "咖啡", "爱"],
        ChineseTraditional => ["你好", "謝謝", "朋友", "明天", "音樂", "漂亮", "咖啡", "愛"],
        Hindi => [
            "नमस्ते",
            "धन्यवाद",
            "दोस्त",
            "कल",
            "संगीत",
            "सुंदर",
            "कॉफ़ी",
            "प्यार",
        ],
        Thai => ["สวัสดี", "ขอบคุณ", "เพื่อน", "พรุ่งนี้", "เพลง", "สวย", "กาแฟ", "รัก"],
    }
}

fn studies() -> Vec<OnboardingStudy> {
    [
        (
            "Memory: A Contribution to Experimental Psychology",
            "Ebbinghaus, H.",
            1885,
            "Teachers College, Columbia University",
            "https://psychclassics.yorku.ca/Ebbinghaus/index.htm",
        ),
        (
            "Distributed practice in verbal recall tasks: A review and quantitative synthesis",
            "Cepeda, N.J., Pashler, H., Vul, E., Wixted, J.T., & Rohrer, D.",
            2006,
            "Psychological Bulletin, 132(3), 354–380",
            "https://doi.org/10.1037/0033-2909.132.3.354",
        ),
        (
            "The Critical Importance of Retrieval for Learning",
            "Karpicke, J.D. & Roediger, H.L.",
            2008,
            "Science, 319(5865), 966–968",
            "https://doi.org/10.1126/science.1152408",
        ),
        (
            "A Stochastic Shortest Path Algorithm for Optimizing Spaced Repetition Scheduling",
            "Ye, J.J., Su, J., & Cao, Y.",
            2022,
            "KDD ’22: Proceedings of the 28th ACM SIGKDD Conference, 4381–4390",
            "https://dl.acm.org/doi/10.1145/3534678.3539081?cid=99660547150",
        ),
    ]
    .into_iter()
    .map(|(title, authors, year, journal, url)| OnboardingStudy {
        title: title.into(),
        authors: authors.into(),
        year,
        journal: journal.into(),
        url: url.into(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send(state: &mut OnboardingState, event: OnboardingEvent) -> Vec<OnboardingEffect> {
        let transition = onboarding_reduce(state.clone(), event);
        *state = transition.state;
        transition.effects
    }
    fn next(state: &mut OnboardingState) -> Vec<OnboardingEffect> {
        send(state, OnboardingEvent::Next)
    }
    /// Choice steps have no primary button: picking an answer moves on.
    fn choose(state: &mut OnboardingState, choice: OnboardingChoice) -> Vec<OnboardingEffect> {
        assert!(view(state).primary.is_none());
        assert!(next(state).is_empty());
        let before = state.step_index;
        let effects = send(state, OnboardingEvent::Choose { choice });
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, OnboardingEffect::Complete { .. }))
                || state.step_index == before + 1
        );
        effects
    }
    fn view(state: &OnboardingState) -> OnboardingView {
        onboarding_view(state.clone())
    }

    #[test]
    fn anki_onboarding_only_asks_experience_and_records_it() {
        for level in [ExperienceLevel::New, ExperienceLevel::CommonWords] {
            let mut s =
                onboarding_start(Language::French, false, true, OnboardingPurpose::AnkiDeck);
            assert_eq!(s.steps, [OnboardingStep::Experience]);
            assert_eq!(view(&s).progress_label, "Step 1 of 1");
            assert_eq!(view(&s).progress_percent, 100.0);
            for offer in [false, true] {
                assert!(
                    send(&mut s, OnboardingEvent::RefreshNotificationOffer { offer }).is_empty()
                );
                assert_eq!(s.steps, [OnboardingStep::Experience]);
            }
            let effects = choose(
                &mut s,
                OnboardingChoice::Experience {
                    value: level.clone(),
                },
            );
            let [OnboardingEffect::Complete { selections }] = &effects[..] else {
                panic!()
            };
            assert_eq!(
                selections,
                &OnboardingSelections {
                    starting_fresh: level == ExperienceLevel::New,
                    experience_level: Some(level),
                    motivation: None,
                    study_goal: None,
                }
            );
            assert_eq!(s.step_index, 0);
        }
    }

    #[test]
    fn full_flow_preserves_canonical_copy_and_emits_existing_answers() {
        use OnboardingStep::*;
        let mut s = onboarding_start(Language::French, false, true, OnboardingPurpose::App);
        assert_eq!(
            s.steps,
            [
                HeardAbout,
                Motivation,
                Experience,
                Achievements,
                SrsTeaser,
                SrsIntro,
                SrsConclusion,
                StudyGoal,
                Notifications,
                Ready
            ]
        );
        assert_eq!(view(&s).progress_label, "Step 1 of 10");
        assert!(matches!(
            &choose(
                &mut s,
                OnboardingChoice::HeardAbout {
                    value: crate::deck_selection::HeardAbout::Reddit,
                },
            )[..],
            [OnboardingEffect::SaveHeardAbout {
                value: crate::deck_selection::HeardAbout::Reddit
            }]
        ));
        assert!(s.heard_about.is_none());
        assert_eq!(view(&s).title, "Why are you learning Français?");
        choose(
            &mut s,
            OnboardingChoice::Motivation {
                value: crate::deck_selection::Motivation::JustForFun,
            },
        );
        assert_eq!(view(&s).title, "How much Français do you know?");
        choose(
            &mut s,
            OnboardingChoice::Experience {
                value: ExperienceLevel::New,
            },
        );
        let OnboardingContent::Achievements { items } = view(&s).content else {
            panic!()
        };
        assert_eq!(
            items.iter().map(|i| i.text.as_str()).collect::<Vec<_>>(),
            [
                "Build a large vocabulary",
                "Enjoy French media",
                "Understand how French speakers actually talk"
            ]
        );
        assert_eq!(view(&s).primary.unwrap().label, "Continue");
        next(&mut s);
        let OnboardingContent::Studies { studies, .. } = view(&s).content else {
            panic!()
        };
        assert_eq!(studies.len(), 4);
        assert_eq!(
            studies[3].journal,
            "KDD ’22: Proceedings of the 28th ACM SIGKDD Conference, 4381–4390"
        );
        next(&mut s);
        for count in 0..=4 {
            assert_eq!(s.review_count, count);
            let OnboardingContent::Review {
                learned,
                review_label,
                curves,
                ..
            } = view(&s).content
            else {
                panic!()
            };
            assert_eq!(learned, count == 4);
            // The demo opens on the first forgetting curve, never an empty chart.
            assert_eq!(curves.len(), usize::from(count.min(3)) + 1);
            assert_eq!(curves[0].start, 0.0);
            assert!(curves.windows(2).all(|w| w[0].end == w[1].start));
            assert_eq!(
                review_label.as_deref(),
                match count {
                    1 => Some("1 review"),
                    2 => Some("2 reviews"),
                    3 => Some("3 reviews"),
                    _ => None,
                }
            );
            let primary = view(&s).primary.unwrap();
            assert_eq!(primary.label, if count < 4 { "Review" } else { "Continue" });
            assert_eq!(primary.show_arrow, count == 4);
            next(&mut s);
        }
        assert!((forgetting_curves().last().unwrap().end - 1.0).abs() < 1e-9);
        assert_eq!(*s.step(), SrsConclusion);
        assert_eq!(s.review_count, 0);
        assert_eq!(view(&s).primary.unwrap().label, "Set a goal");
        let OnboardingContent::Remember { words } = view(&s).content else {
            panic!()
        };
        assert_eq!(words[0], "bonjour");
        next(&mut s);
        let OnboardingContent::Choices { options } = view(&s).content else {
            panic!()
        };
        assert_eq!(options[1].label, "10 min/day");
        assert_eq!(
            options[1].detail.as_deref(),
            Some("~50 words in your first week")
        );
        choose(
            &mut s,
            OnboardingChoice::StudyGoal {
                value: DailyReviewTarget::Regular,
            },
        );
        assert_eq!(*s.step(), Notifications);
        assert!(view(&s).primary.is_none());
        next(&mut s);
        assert_eq!(*s.step(), Notifications);
        send(&mut s, OnboardingEvent::NotificationsDone);
        assert_eq!(view(&s).primary.unwrap().label, "Allons-y !");
        let OnboardingContent::Ready {
            body,
            start_fresh_label,
            ..
        } = view(&s).content
        else {
            panic!()
        };
        assert_eq!(body, "We'll build your Français foundation step by step.");
        assert!(start_fresh_label.is_none());
        let effects = next(&mut s);
        let [OnboardingEffect::Complete { selections }] = &effects[..] else {
            panic!()
        };
        assert_eq!(
            selections,
            &OnboardingSelections {
                starting_fresh: true,
                motivation: Some(crate::deck_selection::Motivation::JustForFun),
                experience_level: Some(ExperienceLevel::New),
                study_goal: Some(DailyReviewTarget::Regular)
            }
        );
    }

    #[test]
    fn navigation_resets_transient_answers_but_keeps_course_choices() {
        let mut s = onboarding_start(Language::French, false, false, OnboardingPurpose::App);
        assert!(matches!(
            &send(&mut s, OnboardingEvent::Back)[..],
            [OnboardingEffect::Exit]
        ));
        send(
            &mut s,
            OnboardingEvent::RefreshNotificationOffer { offer: true },
        );
        choose(
            &mut s,
            OnboardingChoice::HeardAbout {
                value: HeardAbout::Other,
            },
        );
        // Only the first step may change the itinerary.
        send(
            &mut s,
            OnboardingEvent::RefreshNotificationOffer { offer: false },
        );
        assert!(s.steps.contains(&OnboardingStep::Notifications));
        choose(
            &mut s,
            OnboardingChoice::Motivation {
                value: Motivation::Other,
            },
        );
        send(&mut s, OnboardingEvent::Back);
        // Going back shows the earlier answer.
        let OnboardingContent::Choices { options } = view(&s).content else {
            panic!()
        };
        assert!(options.iter().any(|o| o.selected));
        send(&mut s, OnboardingEvent::Back);
        assert_eq!(*s.step(), OnboardingStep::HeardAbout);
        assert!(s.heard_about.is_none());
        assert_eq!(s.selections.motivation, Some(Motivation::Other));
        let OnboardingContent::Choices { options } = view(&s).content else {
            panic!()
        };
        assert!(!options.iter().any(|o| o.selected));
        s.step_index = s
            .steps
            .iter()
            .position(|s| *s == OnboardingStep::SrsIntro)
            .unwrap() as u32;
        next(&mut s);
        send(&mut s, OnboardingEvent::Back);
        assert_eq!(s.review_count, 0);
        next(&mut s);
        assert_eq!(s.review_count, 0);
    }

    #[test]
    fn optional_steps_and_experienced_completion() {
        let mut s = onboarding_start(
            Language::SpanishLatinAmerican,
            true,
            false,
            OnboardingPurpose::App,
        );
        assert_eq!(s.steps.len(), 8);
        assert_eq!(*s.step(), OnboardingStep::Motivation);
        assert!(!s.steps.contains(&OnboardingStep::Notifications));
        s.step_index = s.steps.len() as u32 - 1;
        s.selections.experience_level = Some(ExperienceLevel::CommonWords);
        assert_eq!(view(&s).primary.unwrap().label, "Find my level");
        let OnboardingContent::Ready {
            body,
            start_fresh_label,
            ..
        } = view(&s).content
        else {
            panic!()
        };
        assert_eq!(
            body,
            "Since you already know some Español, we can skip ahead to where you belong."
        );
        assert_eq!(start_fresh_label.as_deref(), Some("Start from scratch"));
        assert!(
            matches!(&next(&mut s)[..], [OnboardingEffect::Complete { selections }] if !selections.starting_fresh)
        );
        assert!(
            matches!(&send(&mut s, OnboardingEvent::StartFromScratch)[..], [OnboardingEffect::Complete { selections }] if selections.starting_fresh)
        );
    }
}
