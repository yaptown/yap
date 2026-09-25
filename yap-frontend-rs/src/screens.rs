//! Serializable screen snapshots. Rust owns content and deck actions;
//! hosts own navigation and presentation state.
use crate::*;

/// Account encouragement and authentication copy shared by both hosts.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountCopy {
    pub prompt_title: String,
    pub prompt_body: String,
    pub prompt_action: String,
    pub sign_in_action: String,
    pub dialog_title: String,
    pub dialog_description: String,
    pub sign_in_tab: String,
    pub sign_up_tab: String,
    pub email_label: String,
    pub password_label: String,
    pub sign_in_button: String,
    pub signing_in_button: String,
    pub sign_up_button: String,
    pub signing_up_button: String,
    pub forgot_password: String,
    pub audio_needs_account_title: String,
    pub audio_needs_account_body: String,
}

#[bridgerton::bridge]
pub fn account_copy() -> AccountCopy {
    AccountCopy {
        prompt_title: "Log in or create an account to make sure you don't lose your progress!"
            .into(),
        prompt_body: "Your learning data is currently only stored on this device.".into(),
        prompt_action: "Create Account".into(),
        sign_in_action: "Sign In".into(),
        dialog_title: "Welcome to Yap.Town".into(),
        dialog_description: "Sign in or create an account to sync your progress across devices"
            .into(),
        sign_in_tab: "Sign In".into(),
        sign_up_tab: "Sign Up".into(),
        email_label: "Email".into(),
        password_label: "Password".into(),
        sign_in_button: "Sign In".into(),
        signing_in_button: "Signing in...".into(),
        sign_up_button: "Create Account".into(),
        signing_up_button: "Creating account...".into(),
        forgot_password: "Forgot your password?".into(),
        audio_needs_account_title: "Please log in to play audio".into(),
        audio_needs_account_body:
            "Audio playback requires an account to access the text-to-speech service.".into(),
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdleKind {
    NothingToDo,
    FirstRun,
    NeedsMoreCards,
    AllCaughtUp,
}

/// One card type's share of a review plan: "3 reading cards" over the words.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewPlanGroup {
    pub heading: String,
    pub cards: Vec<String>,
}

/// The set of reviews the learner commits to with one button: today's lockup
/// offer, or releasing more cards later in the day.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewPlanView {
    pub target_language: Language,
    pub title: String,
    /// Reading, then listening, then pronunciation; empty groups are omitted.
    pub groups: Vec<ReviewPlanGroup>,
    pub accept_label: String,
    pub event: DeckEvent,
    pub week: Vec<DayProgress>,
}

impl ReviewPlanView {
    fn new(
        target_language: Language,
        cards: Vec<CardSummary>,
        event: DeckEvent,
        week: Vec<DayProgress>,
    ) -> Self {
        let label = |card: &CardSummary| match card.card_indicator {
            CardIndicator::WrittenGram { .. } => "reading",
            CardIndicator::ListeningGram { .. } => "listening",
            CardIndicator::LetterPronunciation { .. } => "pronunciation",
        };
        let groups = ["reading", "listening", "pronunciation"]
            .into_iter()
            .filter_map(|group| {
                let cards: Vec<String> = cards
                    .iter()
                    .filter(|card| label(card) == group)
                    .map(|card| card.card_text.clone())
                    .collect();
                (!cards.is_empty()).then(|| ReviewPlanGroup {
                    heading: format!(
                        "{} {group} {}",
                        cards.len(),
                        if cards.len() == 1 { "card" } else { "cards" }
                    ),
                    cards,
                })
            })
            .collect();
        ReviewPlanView {
            target_language,
            title: "Today's review plan:".into(),
            groups,
            accept_label: "Let's go!".into(),
            event,
            week,
        }
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SentenceListOptionView {
    pub category: SentenceListCategory,
    pub label: String,
    pub selection: Option<SentenceListSelection>,
}

/// The floating commit for a curriculum the learner has browsed to but not
/// chosen yet: present iff the draft differs from the persisted selection.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SwitchCurriculumView {
    pub label: String,
    pub event: DeckEvent,
}

/// A sentence with one emphasized run, so hosts can style the run (a
/// target-language word, an uppercased curriculum name) without owning the
/// words around it. `emphasis` may be empty.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmphasizedText {
    pub before: String,
    pub emphasis: String,
    pub after: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdleView {
    pub target_language: Language,
    pub kind: IdleKind,
    /// The prominent call-to-action shown right under the title when there is nothing schedulable.
    pub smart_add_label: Option<String>,
    /// Whether the sentence-list panel (progress, learn/next buttons, manual add) appears.
    pub show_sentence_list: bool,
    pub title: String,
    pub body: String,
    pub info: NoCardsReadyInfo,
    pub manual_add_heading: String,
    pub manual_add_options: Vec<ManualAddOption>,
    /// Feeds `next_review_line` on the hosts' own tick when `body` is empty.
    pub next_due: Option<CardSummary>,
    /// "Soon you'll hit 10% on <CURRICULUM>!" atop the sentence-list panel.
    pub curriculum_headline: EmphasizedText,
    /// "Learn 5 new cards to hit 10%": the smart-add button inside the panel.
    pub curriculum_learn_label: Option<String>,
    /// "Next movie" / "Next lesson", once the curriculum is exhausted.
    pub next_sentence_list_label: Option<String>,
    /// Shown instead of a next-list button when nothing is left anywhere.
    pub all_learned_note: Option<String>,
    /// Essential-course footnote about everyday-language coverage.
    pub level_note: Option<String>,
    pub banned_notice: Option<String>,
    pub week: Vec<DayProgress>,
    pub navigation: SentenceListNavigation,
    pub sentence_list_options: Vec<SentenceListOptionView>,
    pub progress: SentenceListProgress,
    pub sentence_list_label: String,
    pub next_sentence_list: Option<SentenceListSelection>,
    /// Commits the browsed curriculum. Hosts append it just before any add
    /// from this view, so adding cards is what switches — there is no
    /// separate switch button here (Goals keeps one).
    pub commit_curriculum: Option<DeckEvent>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IdleScreenView {
    AudioPending {
        count: u32,
        online: bool,
        week: Vec<DayProgress>,
    },
    ReviewPlanOffer(ReviewPlanView),
    StudyPlanComplete {
        title: String,
        next_due: Option<CardSummary>,
        plan: Box<ReviewPlanView>,
    },
    Idle(Box<IdleView>),
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoalOptionView {
    pub target: DailyReviewTarget,
    pub label: String,
    pub duration_label: String,
    pub minutes: u32,
    pub event: DeckEvent,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccomplishmentView {
    pub target_language: Language,
    /// "Goal Reached! …" with the weekday message; Home's Up Next repeats it.
    pub heading: String,
    pub accomplishment: Accomplishment,
    pub today: TodaySummary,
    pub streak: u32,
    pub percent_known: f64,
    pub words_known: u32,
    pub target: DailyReviewTarget,
    pub goals: Vec<GoalOptionView>,
    pub days: Vec<DayProgress>,
}

/// Text that depends on the clock, with the instant it would next change so a
/// host can schedule exactly one refresh instead of polling.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LiveText {
    pub text: EmphasizedText,
    pub refresh_at_ms: f64,
}

/// Relative time the way web's react-timeago renders it: round to the largest
/// unit under a minute/hour/day/week/30-day month/365-day year, then
/// "2 days from now" or "3 hours ago". Also returns when the string would
/// next change (always at least a second after `from_ms`).
pub fn relative_time(from_ms: f64, to_ms: f64) -> (String, f64) {
    const MINUTE: f64 = 60.0;
    const HOUR: f64 = 60.0 * MINUTE;
    const DAY: f64 = 24.0 * HOUR;
    const WEEK: f64 = 7.0 * DAY;
    const MONTH: f64 = 30.0 * DAY;
    const YEAR: f64 = 365.0 * DAY;
    // (lower bound of the tier, unit, name)
    const TIERS: [(f64, f64, &str); 7] = [
        (0.0, 1.0, "second"),
        (MINUTE, MINUTE, "minute"),
        (HOUR, HOUR, "hour"),
        (DAY, DAY, "day"),
        (WEEK, WEEK, "week"),
        (MONTH, MONTH, "month"),
        (YEAR, YEAR, "year"),
    ];
    let future = to_ms >= from_ms;
    let seconds = ((to_ms - from_ms).abs() / 1000.0).round();
    let tier = TIERS
        .iter()
        .rposition(|(lower, _, _)| seconds >= *lower)
        .unwrap_or(0);
    let (lower, unit, name) = TIERS[tier];
    let value = (seconds / unit).round();
    let plural = if value == 1.0 { "" } else { "s" };
    let suffix = if future { "from now" } else { "ago" };
    let text = format!("{value} {name}{plural} {suffix}");
    // The text is a function of the whole-second count `seconds`: it changes
    // when that count crosses the rounded-value threshold or the tier bound.
    // Work in whole seconds, then convert back to the raw instant (a raw
    // value rounds to a different whole second half a second early).
    let refresh_at_ms = if future {
        let threshold = ((value - 0.5) * unit).max(lower);
        to_ms - (threshold.ceil() - 0.5) * 1000.0 + 1.0
    } else {
        let upper = TIERS.get(tier + 1).map_or(f64::INFINITY, |t| t.0);
        let threshold = ((value + 0.5) * unit).min(upper);
        to_ms + (threshold.ceil() - 0.5) * 1000.0 + 1.0
    };
    (text, refresh_at_ms.max(from_ms + 1000.0))
}

struct CurriculumCopy {
    curriculum_headline: EmphasizedText,
    curriculum_learn_label: Option<String>,
    next_sentence_list_label: Option<String>,
    all_learned_note: Option<String>,
    level_note: Option<String>,
}

/// The sentence-list panel's copy, in web's words.
fn curriculum_copy(
    progress: &SentenceListProgress,
    info: &NoCardsReadyInfo,
    sentence_list_label: &str,
    next_sentence_list: Option<&SentenceListSelection>,
    selection: Option<&SentenceListSelection>,
    target_language: Language,
) -> CurriculumCopy {
    let done = progress.all_available_learned;
    let milestone =
        next_progress_milestone(progress.percent_known, info.percent_known_after).map(|m| m as u32);
    let curriculum_headline = EmphasizedText {
        before: match (done, info.recommend_more_cards, milestone) {
            (true, _, _) => "You're all done with".into(),
            (false, true, Some(m)) => format!("Soon you'll hit {m}% on"),
            (false, true, None) => "Keep up the momentum on".into(),
            (false, false, _) => "You're doing great on".into(),
        },
        emphasis: sentence_list_label.into(),
        after: "!".into(),
    };
    let curriculum_learn_label = (!done && info.smart_add_count > 0).then(|| {
        let mut label = format!(
            "Learn {} new {}",
            info.smart_add_count,
            if info.smart_add_count == 1 {
                "card"
            } else {
                "cards"
            }
        );
        if let (Some(m), false) = (milestone, info.recommend_more_cards) {
            label.push_str(&format!(" to hit {m}%"));
        }
        label
    });
    let next_sentence_list_label =
        done.then_some(next_sentence_list)
            .flatten()
            .map(|next| match next {
                SentenceListSelection::Movie { .. } => "Next movie".into(),
                SentenceListSelection::PimsleurLesson { .. } => "Next lesson".into(),
            });
    let all_learned_note = (done && next_sentence_list.is_none())
        .then(|| "You've learned all available words!".into());
    let level_note = selection.is_none().then(|| {
        format!(
            "When you complete this level, you'll understand {:.1}% of everyday {}.",
            info.tier_info.percent_of_usage,
            get_language_metadata(target_language).common_name
        )
    });
    CurriculumCopy {
        curriculum_headline,
        curriculum_learn_label,
        next_sentence_list_label,
        all_learned_note,
        level_note,
    }
}

/// "You'll review <word> 2 days from now." for a written card, otherwise
/// "Your next review is 2 days from now." (web copy). Hosts re-ask at
/// `refresh_at_ms` so the countdown stays live without polling; the words and
/// rounding are Rust's so both platforms agree.
#[bridgerton::bridge]
#[bridgerton::stable]
pub fn next_review_line(card: CardSummary, now_ms: f64) -> LiveText {
    let (when, refresh_at_ms) = relative_time(now_ms, card.due_timestamp_ms);
    let text = match card.card_indicator {
        CardIndicator::WrittenGram { .. } => EmphasizedText {
            before: "You'll review ".into(),
            emphasis: card.card_text.clone(),
            after: format!(" {when}."),
        },
        _ => EmphasizedText {
            before: format!("Your next review is {when}."),
            emphasis: String::new(),
            after: String::new(),
        },
    };
    LiveText {
        text,
        refresh_at_ms,
    }
}

impl Deck {
    /// Initial backlog plan, rendered by the same plan view as a release offer.
    pub fn review_plan_view(
        &self,
        banned: Vec<ChallengeRequirements>,
        timestamp_ms: f64,
    ) -> Option<ReviewPlanView> {
        let offer = self.get_lockup_offer(banned, timestamp_ms)?;
        let day = DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64)?
            .with_timezone(&self.context.timezone)
            .date_naive();
        Some(ReviewPlanView::new(
            self.get_target_language(),
            offer.keep_preview(),
            offer.lock_event(),
            self.get_current_week_progress_on(day),
        ))
    }

    /// `is_signed_in` is distinct from connectivity: offline signed-in learners
    /// retain their listening options, just as they do elsewhere in the app.
    pub fn idle_screen_view(
        &self,
        banned: Vec<ChallengeRequirements>,
        sentence_list: Option<SentenceListSelection>,
        online: bool,
        is_signed_in: bool,
        timestamp_ms: f64,
    ) -> IdleScreenView {
        let review = self.get_review_info(banned.clone(), timestamp_ms);
        self.idle_screen_view_with_review(
            banned,
            sentence_list,
            online,
            is_signed_in,
            timestamp_ms,
            &review,
        )
    }

    fn idle_screen_view_with_review(
        &self,
        banned: Vec<ChallengeRequirements>,
        sentence_list: Option<SentenceListSelection>,
        online: bool,
        is_signed_in: bool,
        timestamp_ms: f64,
        review: &ReviewInfo,
    ) -> IdleScreenView {
        let day = DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64)
            .unwrap_or_else(Utc::now)
            .with_timezone(&self.context.timezone)
            .date_naive();

        let week = self.get_current_week_progress_on(day);
        if review.due_but_audio_pending_count() > 0 {
            return IdleScreenView::AudioPending {
                count: review.due_but_audio_pending_count() as u32,
                online,
                week,
            };
        }
        let next_due = self
            .get_all_cards_summary()
            .into_iter()
            .filter(|card| card.due_timestamp_ms > timestamp_ms)
            .min_by(|a, b| a.due_timestamp_ms.total_cmp(&b.due_timestamp_ms));
        if let Some(offer) = self.get_release_offer(timestamp_ms) {
            let plan = ReviewPlanView::new(
                self.get_target_language(),
                offer.release_preview(),
                offer.unlock_event(),
                week,
            );
            if self.get_today_time_spent_on(day) == 0
                || !self.study_plan_was_recently_accepted(timestamp_ms)
            {
                return IdleScreenView::ReviewPlanOffer(plan);
            }
            let minutes = (f64::from(self.get_today_time_spent_on(day)) / 60.0).round() as u32;
            let duration = match minutes {
                0 => "less than a minute".into(),
                1 => "1 minute".into(),
                n => format!("{n} minutes"),
            };
            return IdleScreenView::StudyPlanComplete {
                title: format!("You completed the study plan in {duration}!"),
                next_due: next_due
                    .filter(|card| card.due_timestamp_ms - timestamp_ms < 30.0 * 60.0 * 1000.0),
                plan: Box::new(plan),
            };
        }
        let (curriculum, info) = self.curriculum_view(banned, sentence_list);
        let CurriculumView {
            navigation,
            sentence_list_options,
            progress,
            sentence_list_label,
            next_sentence_list,
            switch_curriculum,
            ..
        } = curriculum;
        let manual_add_options =
            self.get_manual_add_options(navigation.selection.clone(), is_signed_in);
        let idle = get_idle_study_state(
            next_due.is_some(),
            self.num_cards_added(),
            info.smart_add_count,
        );
        let (kind, title, body) = if idle.nothing_to_do {
            (
                IdleKind::NothingToDo,
                "All done!",
                "You've learned all available words!".into(),
            )
        } else if idle.has_never_studied {
            (
                IdleKind::FirstRun,
                "Ready to start learning?",
                "We'll start with a couple words you might know.".into(),
            )
        } else if idle.no_schedulable_cards {
            let easy = info.smart_add_regime == SmartAddRegime::Easy;
            (
                IdleKind::NeedsMoreCards,
                if easy {
                    "Adding cards is how you learn more!"
                } else {
                    "Ready for more?"
                },
                if easy && info.easy_cards_remaining > 0 {
                    format!(
                        "{} more easy {}, then we'll add harder ones.",
                        info.easy_cards_remaining,
                        if info.easy_cards_remaining == 1 {
                            "word"
                        } else {
                            "words"
                        }
                    )
                } else {
                    "Add some cards to keep building your vocabulary.".into()
                },
            )
        } else {
            (IdleKind::AllCaughtUp, "All caught up!", String::new())
        };
        let smart_add_label = smart_add_label(&kind, &info);
        let show_sentence_list = kind == IdleKind::AllCaughtUp;
        let CurriculumCopy {
            curriculum_headline,
            curriculum_learn_label,
            next_sentence_list_label,
            all_learned_note,
            level_note,
        } = curriculum_copy(
            &progress,
            &info,
            &sentence_list_label,
            next_sentence_list.as_ref(),
            navigation.selection.as_ref(),
            self.get_target_language(),
        );
        IdleScreenView::Idle(Box::new(IdleView {
            target_language: self.get_target_language(),
            kind,
            smart_add_label,
            show_sentence_list,
            title: title.into(),
            body,
            manual_add_heading: MANUAL_ADD_HEADING.into(),
            manual_add_options,
            info,
            next_due,
            curriculum_headline,
            curriculum_learn_label,
            next_sentence_list_label,
            all_learned_note,
            level_note,
            banned_notice: (review.due_but_banned_count() > 0).then(|| {
                format!(
                    "{} cards paused by listening/speaking restrictions",
                    review.due_but_banned_count()
                )
            }),
            week,
            navigation,
            sentence_list_options,
            progress,
            sentence_list_label,
            next_sentence_list,
            commit_curriculum: switch_curriculum.map(|switch| switch.event),
        }))
    }

    pub fn accomplishment_view(&self, timestamp_ms: f64) -> Option<AccomplishmentView> {
        let day = DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64)?
            .with_timezone(&self.context.timezone)
            .date_naive();
        if self
            .stats
            .today
            .as_ref()
            .is_none_or(|today| today.day != day)
        {
            return None;
        }
        let today = self.get_today_summary_on(day);
        let accomplishment = self.get_accomplishment()?;
        let percent_known = self.get_percent_of_words_known();
        Some(AccomplishmentView {
            target_language: self.get_target_language(),
            heading: accomplishment_heading(&today.day_of_week),
            accomplishment,
            today,
            streak: self.get_daily_streak_on(day),
            percent_known,
            words_known: (percent_known * self.num_cards_added() as f64).round() as u32,
            target: self.get_daily_review_target_setting(),
            goals: self.goal_options_view(),
            days: self.get_current_week_progress_on(day),
        })
    }
}

/// Curriculum content shared by Idle and Goals. Browsing (tabs, lessons,
/// movies, "next lesson") only moves the host's draft selection — the
/// `sentence_list` input — and `switch_curriculum` carries the one event that
/// commits it, so a `SetSentenceList` is appended exactly once per real switch.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurriculumView {
    pub title: String,
    pub navigation: SentenceListNavigation,
    pub sentence_list_options: Vec<SentenceListOptionView>,
    pub progress: SentenceListProgress,
    pub sentence_list_label: String,
    pub next_sentence_list: Option<SentenceListSelection>,
    pub switch_curriculum: Option<SwitchCurriculumView>,
    pub has_movies: bool,
    pub has_pimsleur: bool,
}

fn sentence_list_category_label(category: SentenceListCategory) -> &'static str {
    match category {
        SentenceListCategory::Essential => "Essential",
        SentenceListCategory::Movie => "Movies",
        SentenceListCategory::Pimsleur => "Pimsleur",
    }
}

fn smart_add_label(kind: &IdleKind, info: &NoCardsReadyInfo) -> Option<String> {
    match kind {
        IdleKind::FirstRun => Some("Start learning".into()),
        IdleKind::NeedsMoreCards => Some(format!(
            "Add {} {}{}",
            info.smart_add_count,
            if info.smart_add_regime == SmartAddRegime::Easy {
                "easy "
            } else {
                ""
            },
            if info.smart_add_count == 1 {
                "card"
            } else {
                "cards"
            },
        )),
        IdleKind::NothingToDo | IdleKind::AllCaughtUp => None,
    }
}

impl Deck {
    fn curriculum_navigation(
        &self,
        sentence_list: Option<SentenceListSelection>,
    ) -> SentenceListNavigation {
        let pack = &self.context.language_pack;
        let available = |frequencies: &language_utils::language_pack::FrequencyList| {
            !frequencies.entries.is_empty() && frequencies.total_count > 0
        };
        let has_movies = pack.movies.keys().any(|id| {
            pack.source_gram_frequencies
                .get(&language_utils::FrequencySourceId::Movie(id.clone()))
                .is_some_and(available)
        });
        let has_pimsleur = pack
            .source_gram_frequencies
            .iter()
            .any(|(source, frequencies)| {
                matches!(source, language_utils::FrequencySourceId::PimsleurLesson(_))
                    && available(frequencies)
            });
        get_sentence_list_navigation(sentence_list, has_movies, has_pimsleur)
    }

    fn curriculum_view(
        &self,
        banned: Vec<ChallengeRequirements>,
        sentence_list: Option<SentenceListSelection>,
    ) -> (CurriculumView, NoCardsReadyInfo) {
        let navigation = self.curriculum_navigation(sentence_list);
        let has_movies = navigation.categories.contains(&SentenceListCategory::Movie);
        let has_pimsleur = navigation
            .categories
            .contains(&SentenceListCategory::Pimsleur);
        let info = self.get_no_cards_ready_info(banned, navigation.selection.clone());
        let progress = self
            .get_sentence_list_progress(navigation.selection.clone(), info.tier_info.percent_known);
        let sentence_list_label = match &navigation.selection {
            None => format!(
                "{} {} Level {}",
                info.tier_info.name,
                get_language_metadata(self.get_target_language()).common_name,
                info.tier_info.level
            ),
            Some(SentenceListSelection::Movie { id }) => self
                .get_movie_metadata(vec![id.clone()])
                .first()
                .map(|m| m.title.clone())
                .unwrap_or_else(|| "Movie".into()),
            Some(SentenceListSelection::PimsleurLesson { level, lesson }) => {
                format!("Pimsleur Level {level}, Lesson {lesson}")
            }
        };
        let next_sentence_list = if progress.all_available_learned {
            match navigation.selection {
                Some(SentenceListSelection::Movie { .. }) => self.get_best_movie_sentence_list(),
                Some(SentenceListSelection::PimsleurLesson { .. }) => {
                    self.get_best_pimsleur_sentence_list()
                }
                None => None,
            }
        } else {
            None
        };
        let sentence_list_options = navigation
            .categories
            .iter()
            .map(|category| {
                let selection = if navigation.categories[navigation.selected_index] == *category {
                    navigation.selection.clone()
                } else {
                    self.get_sentence_list_for_category(*category)
                };
                SentenceListOptionView {
                    category: *category,
                    label: sentence_list_category_label(*category).into(),
                    selection,
                }
            })
            .collect();
        let switch_curriculum =
            (navigation.selection != self.get_sentence_list()).then(|| SwitchCurriculumView {
                label: format!("Switch curriculum to {sentence_list_label}"),
                event: self.change_sentence_list(navigation.selection.clone()),
            });
        (
            CurriculumView {
                title: "Curriculum".into(),
                navigation,
                sentence_list_options,
                progress,
                sentence_list_label,
                next_sentence_list,
                switch_curriculum,
                has_movies,
                has_pimsleur,
            },
            info,
        )
    }

    fn goal_options_view(&self) -> Vec<GoalOptionView> {
        get_daily_goal_options()
            .into_iter()
            .map(|option| GoalOptionView {
                label: format!("{:?}", option.value),
                duration_label: format!("{}m", option.minutes),
                target: option.value.clone(),
                minutes: option.minutes,
                event: self.set_daily_review_target(option.value),
            })
            .collect()
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpNextView {
    /// "Up next · Flashcard"
    pub eyebrow: String,
    pub headline: String,
    /// Picks the hosts' icon beside the eyebrow.
    pub kind: UpNextKind,
    /// "Review 4 cards": the only place Home states the due count.
    pub action_label: String,
    pub due_count: u64,
    /// Present when the scheduler has no challenge, including audio and plan states.
    pub idle: Option<IdleScreenView>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum UpNextKind {
    Flashcard,
    Listening,
    Pronunciation,
    Translation,
    Transcription,
    Other,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoalCardView {
    pub title: String,
    /// "Advanced French": the title without its level, which `level_label` badges.
    pub name: String,
    pub level_label: String,
    pub tier_name: String,
    pub level: u32,
    pub total_levels: u32,
    /// Progress within this level, on a 0–100 scale (not overall vocabulary coverage).
    pub percent: f64,
    pub percent_label: String,
    pub subtitle: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreakCardView {
    pub title: String,
    pub days: u32,
    pub days_label: String,
    pub today_label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatsCardView {
    pub title: String,
    pub total_cards: u64,
    pub cards_label: String,
    /// Overall vocabulary coverage, on the accessor's 0–1 scale.
    pub percent_known: f64,
    pub percent_known_label: String,
}

/// A big number with a caption under it, like Home's XP and card tiles.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeStatView {
    pub value: String,
    pub caption: String,
    pub note: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeekCardView {
    pub title: String,
    pub today_label: String,
    pub days: Vec<DayProgress>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DictionaryCardView {
    pub title: String,
    pub search_placeholder: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeScreenView {
    pub title: String,
    pub course_flag: String,
    pub course_label: String,
    /// "A little French, every day."
    pub greeting: String,
    pub native_language: Language,
    pub target_language: Language,
    pub up_next: UpNextView,
    /// Absent while Up Next shows the curriculum card, whose own bar already
    /// tracks the level.
    pub goal: Option<GoalCardView>,
    pub week: WeekCardView,
    pub xp: HomeStatView,
    pub cards: HomeStatView,
    pub dictionary: DictionaryCardView,
    pub due_count: u64,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DueSummaryView {
    pub title: String,
    pub ready_now: u64,
    pub total: u64,
    pub label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatTileView {
    pub eyebrow: String,
    pub value: String,
    pub caption: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrequencyKnowledgeTick {
    pub value: f64,
    pub label: String,
}

#[bridgerton::bridge]
pub fn frequency_rank_label(rank: f64) -> String {
    if rank >= 1000.0 {
        format!("{:.1}k", rank / 1000.0)
    } else {
        format!("{rank:.0}")
    }
}

/// Both hosts use the same rank ticks and compact labels.
#[bridgerton::bridge]
pub fn frequency_knowledge_ticks() -> Vec<FrequencyKnowledgeTick> {
    [
        2.0, 4.0, 6.0, 8.0, 15.0, 50.0, 90.0, 300.0, 700.0, 1500.0, 5000.0, 10000.0,
    ]
    .into_iter()
    .map(|value| FrequencyKnowledgeTick {
        value,
        label: frequency_rank_label(value),
    })
    .collect()
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatsScreenView {
    pub target_language: Language,
    pub title: String,
    pub xp: f64,
    pub total_reviews: u64,
    pub tiles: Vec<StatTileView>,
    pub percent_known: f64,
    pub due: DueSummaryView,
    pub leeches: Vec<CardSummary>,
    pub leech_count: u64,
    pub leeches_label: String,
    pub frequency_knowledge_chart_data: Vec<FrequencyKnowledgePoint>,
    pub frequency_knowledge_chart_title: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoalsScreenView {
    pub title: String,
    pub target_language: Language,
    pub tier_info: TierInfo,
    pub goal: GoalCardView,
    pub daily_goal_title: String,
    pub daily_goal: DailyReviewTarget,
    pub daily_goal_label: String,
    pub daily_goal_options: Vec<GoalOptionView>,
    pub curriculum: CurriculumView,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DueWordsScreenView {
    pub target_language: Language,
    pub title: String,
    pub ready_now: u64,
    pub total: u64,
    pub summary_label: String,
    pub cards: Vec<CardSummary>,
}

impl Deck {
    fn goal_card_view(&self, tier: &TierInfo) -> GoalCardView {
        let language = get_language_metadata(self.get_target_language()).common_name;
        GoalCardView {
            title: format!("{} {language} Level {}", tier.name, tier.level),
            name: format!("{} {language}", tier.name),
            level_label: format!("Level {} of {}", tier.level, tier.total_levels),
            tier_name: tier.name.clone(),
            level: tier.level,
            total_levels: tier.total_levels,
            percent: tier.percent_known,
            percent_label: format!("{:.0}%", tier.percent_known),
            subtitle: format!(
                "Unlocks {:.1}% of everyday {language}",
                tier.percent_of_usage,
            ),
        }
    }

    fn streak_card_view(&self, timestamp_ms: f64) -> StreakCardView {
        let day = DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64)
            .unwrap_or_else(Utc::now)
            .with_timezone(&self.context.timezone)
            .date_naive();
        let days = self.get_daily_streak_on(day);
        StreakCardView {
            title: "Streak".into(),
            days,
            days_label: format!("{days} {}", if days == 1 { "day" } else { "days" }),
            today_label: format!(
                "{} / {} min today",
                self.get_today_time_spent_on(day) / 60,
                self.get_daily_review_target() / 60,
            ),
        }
    }

    fn stats_card_view(&self) -> StatsCardView {
        let total_cards = self.get_all_cards_summary().len() as u64;
        let percent_known = self.get_percent_of_words_known();
        StatsCardView {
            title: "Stats".into(),
            total_cards,
            cards_label: format!(
                "{total_cards} {}",
                if total_cards == 1 { "card" } else { "cards" }
            ),
            percent_known,
            percent_known_label: format!(
                "{:.1}% of everyday {}",
                percent_known * 100.0,
                get_language_metadata(self.get_target_language()).common_name,
            ),
        }
    }

    fn due_summary_view(&self, ready_now: u64, total: u64) -> DueSummaryView {
        DueSummaryView {
            title: "Due words".into(),
            ready_now,
            total,
            label: format!(
                "{ready_now} ready now · {total} {}",
                if total == 1 { "card" } else { "cards" }
            ),
        }
    }
}

/// Only projects the selected challenge; it never chooses a card or sentence.
fn challenge_preview(
    challenge: &Challenge<Gram<String>>,
    language: Language,
) -> (String, UpNextKind, String) {
    let (headline, kind, label) = match challenge {
        Challenge::FlashCardReview { indicator, .. } => match indicator {
            CardIndicator::WrittenGram { gram } => (
                gram.to_display_string(language),
                UpNextKind::Flashcard,
                "Flashcard",
            ),
            CardIndicator::ListeningGram { .. } => (
                "Listen to the word".into(),
                UpNextKind::Listening,
                "Listening",
            ),
            CardIndicator::LetterPronunciation { pattern, .. } => {
                (pattern.clone(), UpNextKind::Pronunciation, "Pronunciation")
            }
        },
        Challenge::PronunciationChallenge { pattern, .. } => {
            (pattern.clone(), UpNextKind::Pronunciation, "Pronunciation")
        }
        Challenge::TranslateComprehensibleSentence(sentence) => (
            sentence.target_language.clone(),
            UpNextKind::Translation,
            "Translation",
        ),
        Challenge::TranscribeComprehensibleSentence(_) => (
            "Listen and fill in the blanks".into(),
            UpNextKind::Transcription,
            "Transcription",
        ),
    };
    (headline, kind, label.into())
}

#[bridgerton::bridge]
impl Deck {
    #[bridgerton::stable]
    pub fn home_screen_view(&self, inputs: ReviewScreenInputs) -> HomeScreenView {
        let (step, review, _) = self.review_step(inputs.clone());
        let due_count = review.due_count() as u64;
        let is_challenge = matches!(step, ReviewStep::Challenge(_));
        let shows_curriculum = matches!(&step, ReviewStep::Idle(idle)
            if matches!(idle.as_ref(), IdleScreenView::Idle(view) if view.show_sentence_list));
        // The idle view already computed the tier; otherwise pay for it only when the goal shows.
        let goal_tier_info = (!shows_curriculum).then(|| match &step {
            ReviewStep::Idle(idle) if let IdleScreenView::Idle(view) = idle.as_ref() => {
                view.info.tier_info.clone()
            }
            _ => {
                let navigation = self.curriculum_navigation(inputs.sentence_list.clone());
                self.get_no_cards_ready_info(inputs.banned.clone(), navigation.selection)
                    .tier_info
            }
        });
        let mut kind = UpNextKind::Other;
        let (headline, kind_label, idle) = match step {
            ReviewStep::Challenge(view) => {
                let (headline, challenge_kind, kind_label) =
                    challenge_preview(&view.challenge, self.get_target_language());
                kind = challenge_kind;
                (headline, kind_label, None)
            }
            ReviewStep::PlacementTest(session) => {
                let info = get_placement_session_info(session);
                let headline = if !info.finished {
                    "Placement Test"
                } else if info.too_advanced {
                    "You might be too advanced"
                } else {
                    "Ready to Start!"
                };
                (headline.into(), "Placement test".into(), None)
            }
            ReviewStep::ReviewPlan(_) => {
                ("Your study plan is ready".into(), "Study plan".into(), None)
            }
            ReviewStep::SetDisplayName => {
                ("Choose Your Display Name".into(), "Profile".into(), None)
            }
            ReviewStep::Accomplishment(view) => (view.heading, "Accomplishment".into(), None),
            ReviewStep::Idle(idle) => {
                let (headline, kind_label) = match idle.as_ref() {
                    IdleScreenView::Idle(view) => (view.title.clone(), "Review"),
                    IdleScreenView::AudioPending { online, .. } => (
                        if *online {
                            "Preparing audio…"
                        } else {
                            "Connect to download audio"
                        }
                        .into(),
                        "Audio pending",
                    ),
                    IdleScreenView::ReviewPlanOffer(_) => {
                        ("Your study plan is ready".into(), "Study plan")
                    }
                    IdleScreenView::StudyPlanComplete { title, .. } => {
                        (title.clone(), "Study plan")
                    }
                };
                (headline, kind_label.into(), Some(*idle))
            }
        };
        let language = get_language_metadata(self.get_target_language());
        let streak = self.streak_card_view(inputs.timestamp_ms);
        let stats = self.stats_card_view();
        let day = DateTime::<Utc>::from_timestamp_millis(inputs.timestamp_ms as i64)
            .unwrap_or_else(Utc::now)
            .with_timezone(&self.context.timezone)
            .date_naive();
        HomeScreenView {
            title: "Home".into(),
            course_flag: language.flag.clone(),
            course_label: format!("Learning {}", language.common_name),
            greeting: format!("A little {}, every day.", language.common_name),
            native_language: self.context.course.native_language,
            target_language: self.get_target_language(),
            up_next: UpNextView {
                eyebrow: format!("Up next · {kind_label}"),
                headline,
                kind,
                action_label: if !is_challenge {
                    "Continue".into()
                } else {
                    // A held challenge can outlive the due queue, but it's still a card.
                    let count = due_count.max(1);
                    format!(
                        "Review {count} {}",
                        if count == 1 { "card" } else { "cards" }
                    )
                },
                due_count,
                idle,
            },
            goal: goal_tier_info.map(|tier_info| self.goal_card_view(&tier_info)),
            week: WeekCardView {
                title: "This week".into(),
                today_label: streak.today_label,
                days: self.get_current_week_progress_on(day),
            },
            xp: HomeStatView {
                value: format!("{:.0}", self.get_xp()),
                caption: "XP earned".into(),
                note: None,
            },
            cards: HomeStatView {
                value: stats.total_cards.to_string(),
                caption: if stats.total_cards == 1 {
                    "Card studied"
                } else {
                    "Cards studied"
                }
                .into(),
                note: Some(stats.percent_known_label),
            },
            dictionary: DictionaryCardView {
                title: "Find a word".into(),
                search_placeholder: format!(
                    "Search {} or {}",
                    get_language_metadata(self.get_target_language()).common_name,
                    get_language_metadata(self.context.course.native_language).common_name,
                ),
            },
            due_count,
        }
    }

    pub fn stats_screen_view(
        &self,
        banned: Vec<ChallengeRequirements>,
        timestamp_ms: f64,
    ) -> StatsScreenView {
        let stats = self.stats_card_view();
        let review = self.get_review_info(banned, timestamp_ms);
        let leeches = self.get_leeches();
        let leech_count = leeches.len() as u64;
        let xp = self.get_xp();
        let total_reviews = self.get_total_reviews();
        let streak = self.streak_card_view(timestamp_ms);
        StatsScreenView {
            target_language: self.get_target_language(),
            title: stats.title,
            xp,
            total_reviews,
            tiles: vec![
                StatTileView {
                    eyebrow: "XP".into(),
                    value: format!("{xp:.0}"),
                    caption: None,
                },
                StatTileView {
                    eyebrow: "Reviews".into(),
                    value: total_reviews.to_string(),
                    caption: None,
                },
                StatTileView {
                    eyebrow: streak.title,
                    value: streak.days_label,
                    caption: Some(streak.today_label),
                },
                StatTileView {
                    eyebrow: "Vocabulary".into(),
                    value: format!("{:.1}%", stats.percent_known * 100.0),
                    caption: Some(format!(
                        "of everyday {}",
                        get_language_metadata(self.get_target_language()).common_name
                    )),
                },
            ],
            percent_known: stats.percent_known,
            due: self.due_summary_view(review.due_count() as u64, stats.total_cards),
            leeches,
            leech_count,
            leeches_label: format!(
                "{leech_count} {}",
                if leech_count == 1 { "leech" } else { "leeches" }
            ),
            frequency_knowledge_chart_data: self.get_frequency_knowledge_chart_data(),
            frequency_knowledge_chart_title: "Word knowledge by frequency".into(),
        }
    }

    pub fn goals_screen_view(
        &self,
        banned: Vec<ChallengeRequirements>,
        sentence_list: Option<SentenceListSelection>,
    ) -> GoalsScreenView {
        let (curriculum, info) = self.curriculum_view(banned, sentence_list);
        GoalsScreenView {
            title: "Goals".into(),
            target_language: self.get_target_language(),
            goal: self.goal_card_view(&info.tier_info),
            tier_info: info.tier_info,
            daily_goal_title: "Daily goal".into(),
            daily_goal: self.get_daily_review_target_setting(),
            daily_goal_label: format!("{} min per day", self.get_daily_review_target() / 60),
            daily_goal_options: self.goal_options_view(),
            curriculum,
        }
    }

    pub fn due_words_view(
        &self,
        banned: Vec<ChallengeRequirements>,
        timestamp_ms: f64,
    ) -> DueWordsScreenView {
        let review = self.get_review_info(banned, timestamp_ms);
        let summary = self.due_summary_view(
            review.due_count() as u64,
            self.get_all_cards_summary().len() as u64,
        );
        // Use the same scheduled list as Review (including bans, locks and audio
        // readiness), rather than filtering cards by their timestamps ourselves.
        let cards = review
            .due_cards
            .iter()
            .filter_map(|indicator| self.card_to_summary(indicator, self.cards.get(indicator)?))
            .collect();
        DueWordsScreenView {
            target_language: self.get_target_language(),
            title: summary.title,
            ready_now: summary.ready_now,
            total: summary.total,
            summary_label: summary.label,
            cards,
        }
    }
}

/// Host-owned context and the challenge held until the deck or restrictions change.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewScreenInputs {
    pub banned: Vec<ChallengeRequirements>,
    pub sentence_list: Option<SentenceListSelection>,
    pub online: bool,
    pub is_signed_in: bool,
    pub needs_display_name: bool,
    pub display_name_dismissed: bool,
    pub has_access_token: bool,
    pub starting_fresh: Option<bool>,
    pub history_known: bool,
    pub dismissed_accomplishment_at_review: Option<u64>,
    pub placement: Option<PlacementSession>,
    pub current_challenge: Option<Challenge<Gram<String>>>,
    pub timestamp_ms: f64,
}

/// A challenge and optional captured reducer state. Live selections start fresh.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeView {
    pub challenge: Challenge<Gram<String>>,
    pub transcription: Option<TranscriptionState>,
    pub translation: Option<TranslationState>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "view")]
pub enum ReviewStep {
    PlacementTest(PlacementSession),
    ReviewPlan(Box<ReviewPlanView>),
    SetDisplayName,
    Accomplishment(Box<AccomplishmentView>),
    Challenge(Box<ChallengeView>),
    Idle(Box<IdleScreenView>),
}

/// The entire review screen, captured by the Review fixture variant.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewScreenView {
    pub show_account_prompt: bool,
    pub native_language: Language,
    pub target_language: Language,
    pub step: ReviewStep,
    pub progress: f64,
    pub total_reviews: u64,
    pub total_count: u64,
    pub offer_engagement: bool,
    pub online: bool,
}

#[bridgerton::bridge]
impl Deck {
    /// Selects PlacementTest → ReviewPlan → SetDisplayName → Accomplishment
    /// → held (or next) Challenge → Idle, in that priority order on both hosts.
    /// A held challenge takes precedence over idle even when nothing is due.
    /// Hosts retain a selected challenge until the deck or restrictions change;
    /// `None` is never held, so newly ready challenges can surface from idle.
    #[bridgerton::stable]
    pub fn review_screen_view(&self, inputs: ReviewScreenInputs) -> ReviewScreenView {
        let (step, review, prompts) = self.review_step(inputs.clone());
        let total_reviews = self.get_total_reviews();
        let day = DateTime::<Utc>::from_timestamp_millis(inputs.timestamp_ms as i64)
            .unwrap_or_else(Utc::now)
            .with_timezone(&self.context.timezone)
            .date_naive();
        ReviewScreenView {
            show_account_prompt: !inputs.is_signed_in,
            native_language: self.context.course.native_language,
            target_language: self.get_target_language(),
            step,
            progress: (f64::from(self.get_today_time_spent_on(day))
                / f64::from(self.get_daily_review_target()).max(1.0))
            .clamp(0.0, 1.0),
            total_reviews,
            total_count: review.total_count() as u64,
            offer_engagement: prompts.offer_engagement,
            online: inputs.online,
        }
    }
}

impl Deck {
    /// Selects PlacementTest → ReviewPlan → SetDisplayName → Accomplishment
    /// → held (or next) Challenge → Idle, in that priority order on both hosts.
    /// A held challenge takes precedence over idle even when nothing is due.
    /// Hosts retain a selected challenge until the deck or restrictions change;
    /// `None` is never held, so newly ready challenges can surface from idle.
    fn review_step(
        &self,
        inputs: ReviewScreenInputs,
    ) -> (ReviewStep, ReviewInfo, disclosure::ReviewPrompts) {
        let review = self.get_review_info(inputs.banned.clone(), inputs.timestamp_ms);
        let total_reviews = self.get_total_reviews();
        let prompts = get_review_prompts(
            total_reviews,
            review.total_count(),
            ReviewPromptContext {
                is_idle: review.due_count() == 0 && inputs.current_challenge.is_none(),
                is_online: inputs.online,
                is_signed_in: inputs.is_signed_in,
                needs_display_name: inputs.needs_display_name,
                display_name_dismissed: inputs.display_name_dismissed,
                has_access_token: inputs.has_access_token,
            },
        );
        let step = if self.should_offer_placement_test(inputs.starting_fresh, inputs.history_known)
        {
            ReviewStep::PlacementTest(
                inputs
                    .placement
                    .map(|session| {
                        self.refresh_placement_session(session.clone())
                            .unwrap_or(session)
                    })
                    .unwrap_or_else(|| self.start_placement_session()),
            )
        } else if let Some(plan) = self.review_plan_view(inputs.banned.clone(), inputs.timestamp_ms)
        {
            ReviewStep::ReviewPlan(Box::new(plan))
        } else if prompts.offer_display_name {
            ReviewStep::SetDisplayName
        } else if let Some(accomplishment) = self.accomplishment_view(inputs.timestamp_ms)
            && inputs.dismissed_accomplishment_at_review != Some(total_reviews)
        {
            ReviewStep::Accomplishment(Box::new(accomplishment))
        } else if let Some(challenge) = inputs
            .current_challenge
            .or_else(|| review.get_next_challenge(self))
        {
            ReviewStep::Challenge(Box::new(ChallengeView {
                challenge,
                transcription: None,
                translation: None,
            }))
        } else {
            ReviewStep::Idle(Box::new(self.idle_screen_view_with_review(
                inputs.banned,
                inputs.sentence_list,
                inputs.online,
                inputs.is_signed_in,
                inputs.timestamp_ms,
                &review,
            )))
        };
        (step, review, prompts)
    }
}

fn accomplishment_heading(day: &str) -> String {
    let message = match day {
        "Monday" => "Monday can't stop you!",
        "Tuesday" => "Solid Tuesday session!",
        "Wednesday" => "Midweek momentum!",
        "Thursday" => "Thursday well spent!",
        "Friday" => "Happy Friday!",
        "Saturday" => "Weekend warrior!",
        "Sunday" => "So much for the day of rest!",
        _ => "Nice!",
    };
    format!("Goal Reached! {message}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn account_copy_matches_web() {
        let copy = account_copy();
        assert_eq!(copy.prompt_action, "Create Account");
        assert_eq!(copy.dialog_title, "Welcome to Yap.Town");
        assert_eq!(copy.signing_in_button, "Signing in...");
    }

    #[test]
    fn review_plan_groups_cards_by_type_in_a_fixed_order() {
        let card = |card_indicator, text: &str| CardSummary {
            card_indicator,
            due_timestamp_ms: 0.0,
            state: "new".into(),
            card_text: text.into(),
            card_subtitle: None,
        };
        let gram = Gram(vec![]);
        let plan = ReviewPlanView::new(
            Language::French,
            vec![
                card(CardIndicator::ListeningGram { gram: gram.clone() }, "de"),
                card(
                    CardIndicator::LetterPronunciation {
                        pattern: "ch".into(),
                        position: language_utils::PatternPosition::Anywhere,
                    },
                    "[ch]",
                ),
                card(CardIndicator::ListeningGram { gram: gram.clone() }, "et"),
                card(
                    CardIndicator::WrittenGram {
                        gram: TaggedGram { gram, sense: None },
                    },
                    "peut",
                ),
            ],
            DeckEvent::Language(LanguageEvent {
                target_language: Language::French,
                native_language: Language::English,
                content: LanguageEventContent::LockCardsExcept { keep: vec![] },
            }),
            vec![],
        );
        assert_eq!(plan.title, "Today's review plan:");
        assert_eq!(plan.accept_label, "Let's go!");
        assert_eq!(
            plan.groups,
            vec![
                ReviewPlanGroup {
                    heading: "1 reading card".into(),
                    cards: vec!["peut".into()],
                },
                ReviewPlanGroup {
                    heading: "2 listening cards".into(),
                    cards: vec!["de".into(), "et".into()],
                },
                ReviewPlanGroup {
                    heading: "1 pronunciation card".into(),
                    cards: vec!["[ch]".into()],
                },
            ]
        );
    }

    fn inputs() -> ReviewScreenInputs {
        ReviewScreenInputs {
            banned: vec![],
            sentence_list: None,
            online: true,
            is_signed_in: true,
            needs_display_name: false,
            display_name_dismissed: false,
            has_access_token: true,
            starting_fresh: Some(true),
            history_known: true,
            dismissed_accomplishment_at_review: None,
            placement: None,
            current_challenge: None,
            timestamp_ms: Utc
                .with_ymd_and_hms(2026, 9, 21, 12, 0, 0)
                .unwrap()
                .timestamp_millis() as f64,
        }
    }

    #[test]
    fn account_prompt_tracks_identity_even_offline() {
        let deck = with_due_cards();
        for is_signed_in in [true, false] {
            for online in [true, false] {
                let mut inputs = inputs();
                inputs.is_signed_in = is_signed_in;
                inputs.has_access_token = is_signed_in;
                inputs.online = online;
                assert_eq!(
                    deck.review_screen_view(inputs).show_account_prompt,
                    !is_signed_in
                );
            }
        }
    }

    #[test]
    fn relative_time_matches_react_timeago() {
        let now = 1_700_000_000_000.0;
        let s = 1000.0;
        for (delta, expected) in [
            (0.0, "0 seconds from now"),
            (1.0 * s, "1 second from now"),
            (59.0 * s, "59 seconds from now"),
            (60.0 * s, "1 minute from now"),
            (90.0 * s, "2 minutes from now"),
            (3600.0 * s, "1 hour from now"),
            (23.0 * 3600.0 * s, "23 hours from now"),
            (39.0 * 3600.0 * s, "2 days from now"),
            (6.0 * 86400.0 * s, "6 days from now"),
            (7.0 * 86400.0 * s, "1 week from now"),
            (29.0 * 86400.0 * s, "4 weeks from now"),
            (45.0 * 86400.0 * s, "2 months from now"),
            (400.0 * 86400.0 * s, "1 year from now"),
            (-3.0 * 3600.0 * s, "3 hours ago"),
        ] {
            assert_eq!(relative_time(now, now + delta).0, expected, "delta {delta}");
        }
    }

    #[test]
    fn relative_time_refreshes_exactly_when_the_text_changes() {
        let now = 1_700_000_000_000.0;
        let hour = 3_600_000.0;
        for delta_ms in [
            30_000.0,
            59_400.0,
            61_000.0,
            90_000.0,
            5.0 * hour,
            39.0 * hour,
            8.0 * 24.0 * hour,
            -3.0 * hour,
        ] {
            let to = now + delta_ms;
            let (text, refresh_at) = relative_time(now, to);
            assert!(refresh_at >= now + 1000.0, "{text}: refresh in the future");
            let floored = refresh_at == now + 1000.0;
            // Unchanged right up to the refresh instant (unless the 1s floor
            // moved it past a sub-second rounding boundary)…
            if !floored {
                assert_eq!(
                    relative_time(refresh_at - 2.0, to).0,
                    text,
                    "{text}: stable before"
                );
            }
            // …and different at it.
            let (after, _) = relative_time(refresh_at, to);
            assert!(
                after != text || floored,
                "{text} did not change at its refresh instant ({after})"
            );
        }
        assert_eq!(
            relative_time(now, now + 39.0 * hour).1,
            now + 3.0 * hour + 501.0
        );
    }

    #[test]
    fn next_review_line_names_written_cards_only() {
        let now = inputs().timestamp_ms;
        let deck = with_due_cards();
        let mut card = deck.get_all_cards_summary().into_iter().next().unwrap();
        card.due_timestamp_ms = now + 2.0 * 86_400_000.0;
        let line = next_review_line(card.clone(), now).text;
        match card.card_indicator {
            CardIndicator::WrittenGram { .. } => {
                assert_eq!(line.before, "You'll review ");
                assert_eq!(line.emphasis, card.card_text);
                assert_eq!(line.after, " 2 days from now.");
            }
            _ => {
                assert_eq!(line.before, "Your next review is 2 days from now.");
                assert!(line.emphasis.is_empty() && line.after.is_empty());
            }
        }
        let listening = CardSummary {
            card_indicator: CardIndicator::ListeningGram {
                gram: match card.card_indicator {
                    CardIndicator::WrittenGram { gram } => gram.gram,
                    CardIndicator::ListeningGram { gram } => gram,
                    other => panic!("unexpected {other:?}"),
                },
            },
            ..card
        };
        assert_eq!(
            next_review_line(listening, now).text.before,
            "Your next review is 2 days from now."
        );
    }

    #[test]
    fn home_and_review_agree_on_onboarding_steps() {
        let mut deck = Deck::default();
        deck.stats.total_reviews = 25;
        let profile_inputs = ReviewScreenInputs {
            needs_display_name: true,
            ..inputs()
        };
        assert!(matches!(
            deck.review_screen_view(profile_inputs.clone()).step,
            ReviewStep::SetDisplayName
        ));
        let home = deck.home_screen_view(profile_inputs);
        assert!(home.up_next.idle.is_none());
        assert_eq!(home.up_next.headline, "Choose Your Display Name");
        assert_eq!(home.up_next.eyebrow, "Up next · Profile");

        let deck = Deck::default();
        let placement_inputs = ReviewScreenInputs {
            starting_fresh: Some(false),
            ..inputs()
        };
        assert!(matches!(
            deck.review_screen_view(placement_inputs.clone()).step,
            ReviewStep::PlacementTest(_)
        ));
        let home = deck.home_screen_view(placement_inputs);
        assert!(home.up_next.idle.is_none());
        assert_eq!(home.up_next.headline, "Placement Test");
        assert_eq!(home.up_next.eyebrow, "Up next · Placement test");
    }

    #[test]
    fn home_previews_the_challenge_held_by_review() {
        let deck = Deck::default();
        let ready_deck = with_due_cards();
        let held = ready_deck
            .get_review_info(vec![], inputs().timestamp_ms)
            .get_next_challenge(&ready_deck)
            .unwrap();
        let inputs = ReviewScreenInputs {
            current_challenge: Some(held.clone()),
            ..inputs()
        };
        let review = deck.review_screen_view(inputs.clone());
        let ReviewStep::Challenge(view) = review.step else {
            panic!("held challenge should win over idle")
        };
        assert_eq!(json(&view.challenge), json(&held));
        let home = deck.home_screen_view(inputs);
        let (headline, kind, kind_label) = challenge_preview(&held, deck.get_target_language());
        assert_eq!(home.up_next.headline, headline);
        assert_eq!(home.up_next.kind, kind);
        assert_eq!(home.up_next.eyebrow, format!("Up next · {kind_label}"));
        assert!(home.up_next.idle.is_none());
        assert_eq!(home.due_count, 0);
        assert_eq!(home.up_next.action_label, "Review 1 card");
    }

    #[test]
    fn manual_add_options_have_web_copy_and_actionable_events() {
        for (kind, singular, plural) in [
            (
                CardType::TargetLanguage,
                "Learn 1 French → English card",
                "Learn 2 French → English cards",
            ),
            (
                CardType::Listening,
                "Learn 1 French listening card",
                "Learn 2 French listening cards",
            ),
            (
                CardType::LetterPronunciation,
                "Learn 1 French pronunciation card",
                "Learn 2 French pronunciation cards",
            ),
        ] {
            let course = Deck::default().context.course;
            assert_eq!(manual_add_label(1, kind, course), singular);
            assert_eq!(manual_add_label(2, kind, course), plural);
        }
        let deck = Deck::default();
        assert!(
            deck.get_manual_add_option(CardType::Listening, None)
                .is_none()
        );
        for signed_in in [false, true] {
            let options = deck.get_manual_add_options(None, signed_in);
            assert!(!options.is_empty());
            for option in options {
                assert!(option.count > 0);
                assert_eq!(
                    option.label,
                    manual_add_label(option.count, option.card_type, deck.context.course)
                );
                if !signed_in {
                    assert_ne!(option.card_type, CardType::Listening);
                }
                let DeckEvent::Language(LanguageEvent {
                    content: LanguageEventContent::AddCards { cards, .. },
                    ..
                }) = option.event
                else {
                    panic!("manual add must add cards");
                };
                assert_eq!(cards.len(), option.count as usize);
            }
        }
        let IdleScreenView::Idle(idle) =
            deck.idle_screen_view(vec![], None, true, true, inputs().timestamp_ms)
        else {
            panic!("new deck should be idle");
        };
        assert_eq!(idle.manual_add_heading, MANUAL_ADD_HEADING);
    }

    fn with_due_cards() -> Deck {
        let deck = Deck::default();
        let event = deck
            .get_no_cards_ready_info(vec![], None)
            .smart_add_event
            .unwrap();
        let context = deck.context.clone();
        let state = <Deck as weapon::AppState>::process_event(
            DeckState::from(deck),
            &context,
            &weapon::data_model::Timestamped {
                timestamp: DateTime::from_timestamp_millis(inputs().timestamp_ms as i64).unwrap(),
                within_device_events_index: 0,
                timezone: context.timezone,
                event,
            },
        );
        <Deck as weapon::AppState>::finalize(state, &context)
    }

    fn json(value: impl Serialize) -> serde_json::Value {
        serde_json::to_value(value).unwrap()
    }

    #[test]
    fn home_idle_and_goals_share_curriculum_and_smart_add() {
        let deck = Deck::default();
        for selection in [
            None,
            Some(SentenceListSelection::Movie {
                id: "unavailable".into(),
            }),
            Some(SentenceListSelection::PimsleurLesson {
                level: 1,
                lesson: 1,
            }),
        ] {
            for signed_in in [false, true] {
                let mut inputs = inputs();
                inputs.sentence_list = selection.clone();
                inputs.is_signed_in = signed_in;
                let home = deck.home_screen_view(inputs.clone());
                let goals = deck.goals_screen_view(vec![], selection.clone());
                let IdleScreenView::Idle(idle) = home.up_next.idle.unwrap() else {
                    panic!("new deck should be idle");
                };
                assert_eq!(home.up_next.headline, idle.title);
                assert_eq!(home.goal.is_none(), idle.show_sentence_list);
                assert_eq!(json(home.goal), json(goals.goal));
                assert_eq!(json(&goals.curriculum.navigation), json(idle.navigation));
                assert_eq!(
                    json(goals.curriculum.sentence_list_options),
                    json(idle.sentence_list_options)
                );
                assert_eq!(json(goals.curriculum.progress), json(idle.progress));
                assert_eq!(
                    goals.curriculum.sentence_list_label,
                    idle.sentence_list_label
                );
                assert_eq!(
                    json(
                        goals
                            .curriculum
                            .switch_curriculum
                            .as_ref()
                            .map(|switch| &switch.event)
                    ),
                    json(&idle.commit_curriculum)
                );
                // The floating commit exists exactly when the browsed draft
                // differs from the deck's persisted curriculum.
                assert_eq!(
                    goals.curriculum.switch_curriculum.is_some(),
                    goals.curriculum.navigation.selection != deck.get_sentence_list()
                );
                if let Some(switch) = &goals.curriculum.switch_curriculum {
                    assert_eq!(
                        switch.label,
                        format!(
                            "Switch curriculum to {}",
                            goals.curriculum.sentence_list_label
                        )
                    );
                }
                assert_eq!(
                    json(deck.get_manual_add_options(
                        goals.curriculum.navigation.selection.clone(),
                        signed_in,
                    )),
                    json(idle.manual_add_options)
                );
                assert_eq!(
                    goals.curriculum.has_movies,
                    goals
                        .curriculum
                        .navigation
                        .categories
                        .contains(&SentenceListCategory::Movie)
                );
                assert_eq!(
                    goals.curriculum.has_pimsleur,
                    goals
                        .curriculum
                        .navigation
                        .categories
                        .contains(&SentenceListCategory::Pimsleur)
                );
                assert_eq!(
                    json(goals.daily_goal_options),
                    json(deck.goal_options_view())
                );
            }
        }
    }

    #[test]
    fn stats_tiles_separate_labels_values_and_captions() {
        let deck = Deck::default();
        let view = deck.stats_screen_view(vec![], inputs().timestamp_ms);
        assert_eq!(
            view.tiles
                .iter()
                .map(|tile| tile.eyebrow.as_str())
                .collect::<Vec<_>>(),
            ["XP", "Reviews", "Streak", "Vocabulary"]
        );
        assert_eq!(view.tiles[0].value, "0");
        assert_eq!(view.tiles[1].value, "0");
        assert!(view.tiles[0].caption.is_none());
        assert!(view.tiles[1].caption.is_none());
        assert_eq!(
            view.tiles[3].value,
            format!("{:.1}%", view.percent_known * 100.0)
        );
        assert_eq!(view.tiles[3].caption.as_deref(), Some("of everyday French"));
        let options = deck.goal_options_view();
        assert_eq!(options[0].label, "Casual");
        assert_eq!(options[0].duration_label, "5m");
    }

    #[test]
    fn chart_ticks_share_numeric_values_and_compact_labels() {
        let ticks = frequency_knowledge_ticks();
        assert_eq!(
            ticks.iter().map(|tick| tick.value).collect::<Vec<_>>(),
            [
                2.0, 4.0, 6.0, 8.0, 15.0, 50.0, 90.0, 300.0, 700.0, 1500.0, 5000.0, 10000.0
            ]
        );
        assert_eq!(
            ticks
                .iter()
                .map(|tick| tick.label.as_str())
                .collect::<Vec<_>>(),
            [
                "2", "4", "6", "8", "15", "50", "90", "300", "700", "1.5k", "5.0k", "10.0k"
            ]
        );
    }

    #[test]
    fn home_and_curriculum_labels_are_shared_display_copy() {
        let home = Deck::default().home_screen_view(inputs());
        assert_eq!(home.title, "Home");
        assert_eq!(home.course_label, "Learning French");
        assert_eq!(home.up_next.action_label, "Continue");

        let deck = with_due_cards();
        let home = deck.home_screen_view(inputs());
        assert_eq!(home.up_next.due_count, 1);
        assert_eq!(home.up_next.action_label, "Review 1 card");

        for (category, expected) in [
            (SentenceListCategory::Essential, "Essential"),
            (SentenceListCategory::Movie, "Movies"),
            (SentenceListCategory::Pimsleur, "Pimsleur"),
        ] {
            assert_eq!(sentence_list_category_label(category), expected);
        }
        let goals = deck.goals_screen_view(vec![], None);
        for option in goals.curriculum.sentence_list_options {
            assert_eq!(option.label, sentence_list_category_label(option.category));
        }
    }

    #[test]
    fn home_previews_the_review_scheduler_and_due_screens_agree() {
        let deck = with_due_cards();
        let inputs = inputs();
        let review = deck.get_review_info(vec![], inputs.timestamp_ms);
        let challenge = review
            .get_next_challenge(&deck)
            .expect("added cards should be ready");
        let home = deck.home_screen_view(inputs.clone());
        let (headline, kind, kind_label) =
            challenge_preview(&challenge, deck.get_target_language());
        assert_eq!(home.up_next.headline, headline);
        assert_eq!(home.up_next.kind, kind);
        assert_eq!(home.up_next.eyebrow, format!("Up next · {kind_label}"));
        assert!(home.up_next.idle.is_none());
        assert_eq!(home.due_count, review.due_count() as u64);
        assert_eq!(home.up_next.due_count, home.due_count);
        assert_eq!(
            home.up_next.action_label,
            format!("Review {} card", home.due_count)
        );
        let percent_known = deck.get_percent_of_words_known();
        assert!(percent_known > 0.0);
        assert_eq!(
            home.cards.note,
            Some(format!("{:.1}% of everyday French", percent_known * 100.0)),
        );
        let due = deck.due_words_view(vec![], inputs.timestamp_ms);
        assert_eq!(
            json(&due.cards),
            json(deck.due_card_summaries(inputs.timestamp_ms))
        );
        assert_eq!(due.ready_now, due.cards.len() as u64);
        for banned in [
            vec![],
            vec![
                ChallengeRequirements::Text,
                ChallengeRequirements::Listening,
                ChallengeRequirements::Speaking,
            ],
        ] {
            let stats = deck.stats_screen_view(banned.clone(), inputs.timestamp_ms);
            let due = deck.due_words_view(banned.clone(), inputs.timestamp_ms);
            let home = deck.home_screen_view(ReviewScreenInputs {
                banned,
                ..inputs.clone()
            });
            assert_eq!(stats.due.ready_now, due.ready_now);
            assert_eq!(home.due_count, due.ready_now);
            assert_eq!(stats.due.total, due.total);
            assert_eq!(due.total, deck.get_all_cards_summary().len() as u64);
            assert_eq!(stats.xp, deck.get_xp());
            assert_eq!(stats.total_reviews, deck.get_total_reviews());
            assert_eq!(stats.percent_known, deck.get_percent_of_words_known());
            assert_eq!(stats.leech_count, stats.leeches.len() as u64);
            assert_eq!(json(stats.leeches), json(deck.get_leeches()));
            assert_eq!(
                json(stats.frequency_knowledge_chart_data),
                json(deck.get_frequency_knowledge_chart_data())
            );
        }
    }

    #[test]
    fn locked_cards_stay_in_totals_but_not_ready_counts() {
        let mut deck = with_due_cards();
        deck.locked_cards.extend(deck.cards.keys().copied());
        let inputs = ReviewScreenInputs {
            timestamp_ms: inputs().timestamp_ms + 60_000.0,
            ..inputs()
        };
        let home = deck.home_screen_view(inputs.clone());
        let due = deck.due_words_view(vec![], inputs.timestamp_ms);
        let stats = deck.stats_screen_view(vec![], inputs.timestamp_ms);
        assert_eq!(due.ready_now, 0);
        assert!(due.cards.is_empty());
        assert!(due.total > 0);
        assert_eq!(stats.due.total, due.total);
        assert_eq!(stats.due.ready_now, 0);
        assert_eq!(home.due_count, 0);
        assert_eq!(home.up_next.headline, "Your study plan is ready");
        assert!(matches!(
            home.up_next.idle,
            Some(IdleScreenView::ReviewPlanOffer(_))
        ));
    }

    #[test]
    fn streak_cards_use_the_requested_local_day() {
        let mut deck = with_due_cards();
        deck.context.timezone = chrono::FixedOffset::east_opt(2 * 60 * 60).unwrap();
        deck.stats.today.as_mut().unwrap().time_spent_seconds = 125;
        let before_midnight = Utc
            .with_ymd_and_hms(2026, 9, 21, 21, 59, 0)
            .unwrap()
            .timestamp_millis() as f64;
        let after_midnight = before_midnight + 60_000.0;
        let today = deck.streak_card_view(before_midnight);
        let tomorrow = deck.streak_card_view(after_midnight);
        assert_eq!(
            today.today_label,
            format!("2 / {} min today", deck.get_daily_review_target() / 60)
        );
        assert_eq!(
            tomorrow.today_label,
            format!("0 / {} min today", deck.get_daily_review_target() / 60)
        );
        let home = deck.home_screen_view(ReviewScreenInputs {
            timestamp_ms: after_midnight,
            ..inputs()
        });
        let stats = deck.stats_screen_view(vec![], after_midnight);
        assert_eq!(home.week.today_label, tomorrow.today_label);
        assert_eq!(stats.tiles[2].eyebrow, tomorrow.title);
        assert_eq!(stats.tiles[2].value, tomorrow.days_label);
        assert_eq!(stats.tiles[2].caption, Some(tomorrow.today_label));
    }

    #[test]
    fn challenge_previews_cover_every_kind_without_revealing_listening_answers() {
        for (fixture, expected_kind, expected_prompt) in [
            (
                include_str!("../../fixtures/challenges/flashcard-written.json"),
                "Flashcard",
                None,
            ),
            (
                include_str!("../../fixtures/challenges/flashcard-listening.json"),
                "Listening",
                Some("Listen to the word"),
            ),
            (
                include_str!("../../fixtures/challenges/pronunciation.json"),
                "Pronunciation",
                None,
            ),
            (
                include_str!("../../fixtures/challenges/translation-empty.json"),
                "Translation",
                None,
            ),
            (
                include_str!("../../fixtures/challenges/dictation-empty.json"),
                "Transcription",
                Some("Listen and fill in the blanks"),
            ),
        ] {
            let Fixture::Review(view) = parse_fixture(fixture.into()).unwrap() else {
                panic!("expected review fixture")
            };
            let ReviewStep::Challenge(challenge) = view.step else {
                panic!("expected challenge")
            };
            let (headline, _, kind) = challenge_preview(&challenge.challenge, view.target_language);
            assert_eq!(kind, expected_kind);
            assert!(!headline.is_empty());
            if let Some(prompt) = expected_prompt {
                assert_eq!(headline, prompt);
            }
        }
    }

    #[test]
    fn goal_and_overall_coverage_use_their_distinct_percentage_scales() {
        let deck = Deck::default();
        let goal = deck.goal_card_view(&TierInfo {
            tier: 1,
            name: "Elementary".into(),
            level: 5,
            total_levels: 7,
            percent_known: 70.0,
            percent_of_usage: 24.8,
        });
        assert_eq!(goal.title, "Elementary French Level 5");
        assert_eq!(goal.percent_label, "70%");
        assert_eq!(goal.name, "Elementary French");
        assert_eq!(goal.level_label, "Level 5 of 7");
        assert_eq!(goal.subtitle, "Unlocks 24.8% of everyday French");
        let home = deck.home_screen_view(inputs());
        assert_eq!(home.course_flag, "🇫🇷");
        assert_eq!(home.greeting, "A little French, every day.");
        assert_eq!(home.cards.value, "0");
        assert_eq!(home.cards.caption, "Cards studied");
        assert_eq!(
            home.cards.note,
            Some(format!(
                "{:.1}% of everyday French",
                deck.get_percent_of_words_known() * 100.0
            ))
        );
        assert_eq!(
            home.dictionary.search_placeholder,
            "Search French or English"
        );
        assert_eq!(home.xp.value, "0");
        assert_eq!(home.week.days.len(), 7);
        assert_eq!(
            home.week.today_label,
            format!("0 / {} min today", deck.get_daily_review_target() / 60)
        );
    }
}
