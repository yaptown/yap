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
    pub selected: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OnboardingAchievement {
    pub emoji: String,
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
        hint: Option<String>,
    },
    Achievements {
        items: Vec<OnboardingAchievement>,
    },
    Studies {
        studies: Vec<OnboardingStudy>,
        conclusion: String,
    },
    Review {
        eyebrow: String,
        title_emphasis: String,
        demo_reviews: u8,
        review_label: Option<String>,
        learned: bool,
        learned_title: String,
        learned_body: String,
        chart: OnboardingChart,
    },
    Growth {
        chart: OnboardingChart,
    },
    Notifications {
        body: String,
        enable_label: String,
        enabling_label: String,
        skip_label: String,
    },
    Ready {
        flag: String,
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
        Choose { choice } => match (state.step(), choice) {
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
            _ => {}
        },
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
                match state.step() {
                    OnboardingStep::HeardAbout => {
                        effects.push(OnboardingEffect::SaveHeardAbout {
                            value: state.heard_about.clone().unwrap(),
                        });
                        state.advance();
                    }
                    OnboardingStep::SrsIntro if state.review_count < 4 => state.review_count += 1,
                    _ if state.step_index as usize == state.steps.len() - 1 => {
                        effects.push(state.complete(
                            state.selections.experience_level == Some(ExperienceLevel::New),
                        ))
                    }
                    _ => state.advance(),
                }
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
pub fn onboarding_view(state: OnboardingState) -> OnboardingView {
    use OnboardingContent::*;
    let metadata = get_language_metadata(state.target_language);
    let language = &metadata.native_name;
    let step = state.step().clone();
    let is_new = state.selections.experience_level == Some(ExperienceLevel::New);
    let mut primary = Some(OnboardingPrimary {
        label: if state.purpose == OnboardingPurpose::AnkiDeck {
            "Set up my deck"
        } else {
            "Continue"
        }
        .into(),
        enabled: true,
        show_arrow: true,
    });
    let mut choices = |title: String,
                       values: Vec<(OnboardingChoice, String)>,
                       selected: Option<OnboardingChoice>,
                       hint| {
        primary.as_mut().unwrap().enabled = selected.is_some();
        (
            title,
            Choices {
                options: values
                    .into_iter()
                    .map(|(choice, label)| OnboardingOption {
                        selected: Some(&choice) == selected.as_ref(),
                        choice,
                        label,
                    })
                    .collect(),
                hint,
            },
        )
    };
    let (title, content) = match step {
        OnboardingStep::HeardAbout => choices(
            "How did you hear about Yap?".into(),
            [
                (HeardAbout::FriendsOrFamily, "Friends or family"),
                (HeardAbout::Reddit, "Reddit"),
                (HeardAbout::TikTok, "TikTok"),
                (HeardAbout::GoogleSearch, "Google Search"),
                (HeardAbout::YouTube, "YouTube"),
                (HeardAbout::Other, "Other"),
            ]
            .into_iter()
            .map(|(value, label)| (OnboardingChoice::HeardAbout { value }, label.into()))
            .collect(),
            state
                .heard_about
                .map(|value| OnboardingChoice::HeardAbout { value }),
            None,
        ),
        OnboardingStep::Motivation => choices(
            format!("Why are you learning {language}?"),
            [
                (Motivation::SpendTimeProductively, "Spend time productively"),
                (Motivation::SupportMyEducation, "Support my education"),
                (Motivation::ConnectWithPeople, "Connect with people"),
                (Motivation::BoostMyCareer, "Boost my career"),
                (Motivation::PrepareForTravel, "Prepare for travel"),
                (Motivation::JustForFun, "Just for fun"),
                (Motivation::Other, "Other"),
            ]
            .into_iter()
            .map(|(value, label)| (OnboardingChoice::Motivation { value }, label.into()))
            .collect(),
            state
                .selections
                .motivation
                .map(|value| OnboardingChoice::Motivation { value }),
            None,
        ),
        OnboardingStep::Experience => choices(
            format!("How much {language} do you know?"),
            [
                (ExperienceLevel::New, format!("I'm new to {language}")),
                (
                    ExperienceLevel::CommonWords,
                    "I know some common words".into(),
                ),
                (
                    ExperienceLevel::BasicConversations,
                    "I can have basic conversations".into(),
                ),
                (
                    ExperienceLevel::VariousTopics,
                    "I can talk about various topics".into(),
                ),
                (
                    ExperienceLevel::MostTopics,
                    "I can discuss most topics in detail".into(),
                ),
            ]
            .into_iter()
            .map(|(value, label)| (OnboardingChoice::Experience { value }, label))
            .collect(),
            state
                .selections
                .experience_level
                .map(|value| OnboardingChoice::Experience { value }),
            None,
        ),
        OnboardingStep::Achievements => (
            "Here's what you can achieve".into(),
            Achievements {
                items: [
                    ("💬", "Converse with confidence"),
                    ("📚", "Build a large vocabulary"),
                    ("🔄", "Develop a lasting learning habit"),
                ]
                .map(|(emoji, text)| OnboardingAchievement {
                    emoji: emoji.into(),
                    text: text.into(),
                })
                .into(),
            },
        ),
        OnboardingStep::SrsTeaser => (
            "Yap is based on one scientifically proven idea:".into(),
            Studies {
                studies: studies(),
                conclusion: "Spaced repetition.".into(),
            },
        ),
        OnboardingStep::SrsIntro => {
            let learned = state.review_count > 3;
            primary = Some(OnboardingPrimary {
                label: if learned { "Continue" } else { "Review" }.into(),
                enabled: true,
                show_arrow: learned,
            });
            (
                "Every time you review a word, you'll remember it for ".into(),
                Review {
                    eyebrow: "How Yap works".into(),
                    title_emphasis: "longer.".into(),
                    demo_reviews: state.review_count,
                    review_label: match state.review_count {
                        1 => Some("review".into()),
                        2 | 3 => Some("reviews".into()),
                        _ => None,
                    },
                    learned,
                    learned_title: "Word learned".into(),
                    learned_body: "That word is now in long-term memory!".into(),
                    chart: OnboardingChart {
                        x_label: "TIME →".into(),
                        y_label: String::new(),
                        accessibility_label:
                            "Forgetting curve chart showing how spaced repetition helps memory"
                                .into(),
                    },
                },
            )
        }
        OnboardingStep::SrsConclusion => {
            primary.as_mut().unwrap().label = "Set a goal".into();
            (
                "That's why if you study a little bit every day, you'll learn a lot.".into(),
                Growth {
                    chart: OnboardingChart {
                        x_label: "Days".into(),
                        y_label: "Words learned".into(),
                        accessibility_label: "Days → Words learned".into(),
                    },
                },
            )
        }
        OnboardingStep::StudyGoal => {
            let goals = get_daily_goal_options();
            let hint = goals
                .iter()
                .find(|g| Some(&g.value) == state.selections.study_goal.as_ref())
                .map(|g| {
                    format!(
                        "That's ~{} words in your first week!",
                        g.estimated_first_week_words
                    )
                });
            choices(
                "Set a daily study goal".into(),
                goals
                    .into_iter()
                    .map(|g| {
                        let label = format!("{} min/day — {:?}", g.minutes, g.value);
                        (OnboardingChoice::StudyGoal { value: g.value }, label)
                    })
                    .collect(),
                state
                    .selections
                    .study_goal
                    .map(|value| OnboardingChoice::StudyGoal { value }),
                hint,
            )
        }
        OnboardingStep::Notifications => {
            primary = None;
            ("We'll remind you to practice so it becomes a habit!".into(), Notifications {
                body: "A small daily reminder makes it easy to stay consistent and reach your goals.".into(),
                enable_label: "Enable reminders".into(), enabling_label: "Enabling...".into(),
                skip_label: "Not now".into(),
            })
        }
        OnboardingStep::Ready => {
            primary.as_mut().unwrap().label = if is_new {
                metadata.lets_go
            } else {
                "Find my level".into()
            };
            (
                if is_new {
                    "Let's start from the beginning!"
                } else {
                    "Now let's find the best place to start"
                }
                .into(),
                Ready {
                    flag: metadata.flag,
                    body: if is_new {
                        format!("We'll build your {language} foundation step by step.")
                    } else {
                        format!(
                            "Since you already know some {language}, we can skip ahead to where you belong."
                        )
                    },
                    start_fresh_label: (!is_new).then(|| "Start from scratch".into()),
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
        progress_label: format!("Step {step_number} of {total_steps}"),
        back_label: "Back".into(),
        navigation_title: metadata.yaptown_name,
        title,
        content,
        primary,
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
    fn choose(state: &mut OnboardingState, choice: OnboardingChoice) {
        let before = view(state).primary.unwrap();
        assert_eq!(before.label, "Continue");
        send(state, OnboardingEvent::Choose { choice });
        let after = view(state).primary.unwrap();
        assert_eq!(after.label, "Continue");
        assert!(after.enabled);
        assert!(after.show_arrow);
    }
    fn view(state: &OnboardingState) -> OnboardingView {
        onboarding_view(state.clone())
    }

    #[test]
    fn anki_onboarding_only_asks_experience_and_records_it() {
        for level in [ExperienceLevel::New, ExperienceLevel::CommonWords] {
            let mut s =
                onboarding_start(Language::French, false, true, OnboardingPurpose::AnkiDeck);
            assert_eq!(s.purpose, OnboardingPurpose::AnkiDeck);
            assert_eq!(s.steps, [OnboardingStep::Experience]);
            assert_eq!(view(&s).progress_label, "Step 1 of 1");
            assert_eq!(view(&s).progress_percent, 100.0);
            assert_eq!(view(&s).primary.unwrap().label, "Set up my deck");
            assert!(!view(&s).primary.unwrap().enabled);
            assert!(next(&mut s).is_empty());
            send(
                &mut s,
                OnboardingEvent::Choose {
                    choice: OnboardingChoice::Experience {
                        value: level.clone(),
                    },
                },
            );
            for offer in [false, true] {
                assert!(
                    send(&mut s, OnboardingEvent::RefreshNotificationOffer { offer }).is_empty()
                );
                assert_eq!(s.steps, [OnboardingStep::Experience]);
                assert_eq!(s.selections.experience_level, Some(level.clone()));
            }
            assert!(view(&s).primary.unwrap().enabled);
            let effects = next(&mut s);
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
        assert!(!view(&s).primary.unwrap().enabled);
        assert!(next(&mut s).is_empty());
        assert_eq!(s.step_index, 0);
        choose(
            &mut s,
            OnboardingChoice::HeardAbout {
                value: crate::deck_selection::HeardAbout::Reddit,
            },
        );
        assert!(matches!(
            &next(&mut s)[..],
            [OnboardingEffect::SaveHeardAbout {
                value: crate::deck_selection::HeardAbout::Reddit
            }]
        ));
        assert!(s.heard_about.is_none());
        assert_eq!(view(&s).title, "Why are you learning Français?");
        assert!(!view(&s).primary.unwrap().enabled);
        choose(
            &mut s,
            OnboardingChoice::Motivation {
                value: crate::deck_selection::Motivation::JustForFun,
            },
        );
        next(&mut s);
        assert_eq!(view(&s).title, "How much Français do you know?");
        assert!(!view(&s).primary.unwrap().enabled);
        choose(
            &mut s,
            OnboardingChoice::Experience {
                value: ExperienceLevel::New,
            },
        );
        next(&mut s);
        assert_eq!(view(&s).primary.unwrap().label, "Continue");
        next(&mut s);
        assert_eq!(view(&s).primary.as_ref().unwrap().label, "Continue");
        assert!(view(&s).primary.unwrap().enabled);
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
                ..
            } = view(&s).content
            else {
                panic!()
            };
            assert_eq!(learned, count == 4);
            assert_eq!(
                review_label.as_deref(),
                match count {
                    1 => Some("review"),
                    2 | 3 => Some("reviews"),
                    _ => None,
                }
            );
            let primary = view(&s).primary.unwrap();
            assert_eq!(primary.label, if count < 4 { "Review" } else { "Continue" });
            assert_eq!(primary.show_arrow, count == 4);
            next(&mut s);
        }
        assert_eq!(*s.step(), SrsConclusion);
        assert_eq!(s.review_count, 0);
        assert!(view(&s).primary.as_ref().unwrap().enabled);
        assert_eq!(view(&s).primary.unwrap().label, "Set a goal");
        next(&mut s);
        assert!(!view(&s).primary.unwrap().enabled);
        choose(
            &mut s,
            OnboardingChoice::StudyGoal {
                value: DailyReviewTarget::Regular,
            },
        );
        let OnboardingContent::Choices { hint, options } = view(&s).content else {
            panic!()
        };
        assert_eq!(
            hint.as_deref(),
            Some("That's ~50 words in your first week!")
        );
        assert_eq!(options[1].label, "10 min/day — Regular");
        assert!(options[1].selected);
        next(&mut s);
        assert!(view(&s).primary.is_none());
        next(&mut s);
        assert_eq!(*s.step(), Notifications);
        send(&mut s, OnboardingEvent::NotificationsDone);
        assert!(view(&s).primary.as_ref().unwrap().enabled);
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
        choose(
            &mut s,
            OnboardingChoice::HeardAbout {
                value: HeardAbout::Other,
            },
        );
        send(
            &mut s,
            OnboardingEvent::RefreshNotificationOffer { offer: true },
        );
        assert_eq!(s.heard_about, Some(HeardAbout::Other));
        next(&mut s);
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
        assert!(s.heard_about.is_none());
        assert_eq!(s.selections.motivation, Some(Motivation::Other));
        // Back out of the referral step clears its transient selection too.
        choose(
            &mut s,
            OnboardingChoice::HeardAbout {
                value: HeardAbout::Other,
            },
        );
        send(&mut s, OnboardingEvent::Back);
        assert!(s.heard_about.is_none());
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
