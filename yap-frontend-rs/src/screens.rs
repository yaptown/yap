//! Serializable screen snapshots. Rust owns content and deck actions;
//! hosts own navigation and presentation state.
use crate::*;

/// Account encouragement and authentication copy shared by both hosts.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountCopy {
    pub delete_account_action: String,
    pub delete_account_title: String,
    pub delete_account_body: String,
    pub delete_account_confirm: String,
    pub deleting_account: String,
    pub delete_account_failed: String,
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
#[bridgerton::stable(strong)]
pub fn account_copy() -> AccountCopy {
    AccountCopy {
        delete_account_action: yap_i18n::tr!("auth-delete-account"),
        delete_account_title: yap_i18n::tr!("auth-delete-your-account"),
        delete_account_body: yap_i18n::tr!(
            "auth-this-permanently-deletes-your-account-and-your-learning-history"
        ),
        delete_account_confirm: yap_i18n::tr!("auth-delete-account"),
        deleting_account: yap_i18n::tr!("auth-deleting-account"),
        delete_account_failed: yap_i18n::tr!(
            "auth-couldn-t-delete-your-account-check-your-connection-and"
        ),
        prompt_title: yap_i18n::tr!("auth-log-in-or-create-an-account-to-make-sure"),
        prompt_body: yap_i18n::tr!("auth-your-learning-data-is-currently-only-stored-on-this"),
        prompt_action: yap_i18n::tr!("auth-create-account"),
        sign_in_action: yap_i18n::tr!("auth-sign-in"),
        dialog_title: yap_i18n::tr!("auth-welcome-to-yap-town"),
        dialog_description: yap_i18n::tr!("auth-sign-in-or-create-an-account-to-sync-your"),
        sign_in_tab: yap_i18n::tr!("auth-sign-in"),
        sign_up_tab: yap_i18n::tr!("auth-sign-up"),
        email_label: yap_i18n::tr!("auth-email"),
        password_label: yap_i18n::tr!("auth-password"),
        sign_in_button: yap_i18n::tr!("auth-sign-in"),
        signing_in_button: yap_i18n::tr!("auth-signing-in"),
        sign_up_button: yap_i18n::tr!("auth-create-account"),
        signing_up_button: yap_i18n::tr!("auth-creating-account"),
        forgot_password: yap_i18n::tr!("auth-forgot-your-password"),
        audio_needs_account_title: yap_i18n::tr!("auth-please-log-in-to-play-audio"),
        audio_needs_account_body: yap_i18n::tr!(
            "auth-audio-playback-requires-an-account-to-access-the-text"
        ),
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
        next_plan: bool,
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
                    heading: match group {
                        "reading" => yap_i18n::tr!("plan-reading-cards", count = cards.len()),
                        "listening" => yap_i18n::tr!("plan-listening-cards", count = cards.len()),
                        _ => yap_i18n::tr!("plan-pronunciation-cards", count = cards.len()),
                    },
                    cards,
                })
            })
            .collect();
        ReviewPlanView {
            target_language,
            title: if next_plan {
                yap_i18n::tr!("review-plan-next-review-plan")
            } else {
                yap_i18n::tr!("review-plan-today-s-review-plan")
            },
            groups,
            accept_label: yap_i18n::tr!("review-plan-let-s-go"),
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
    /// The link under the progress bar that opens the curriculum browser.
    pub change_curriculum_label: String,
    pub banned_notice: Option<String>,
    pub week: Vec<DayProgress>,
    pub navigation: SentenceListNavigation,
    pub sentence_list_options: Vec<SentenceListOptionView>,
    pub progress: SentenceListProgress,
    pub sentence_list_label: String,
    pub next_sentence_list: Option<SentenceListSelection>,
    /// Commits the browsed curriculum. Hosts append it just before any add
    /// from this view, so adding cards is what switches.
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
    /// "5m", under the label in the segmented editor.
    pub duration_label: String,
    /// "5 min", in Home's goal menu.
    pub minutes_label: String,
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
    /// "You studied 10 min! (21 challenges)"
    pub studied: String,
    pub streak: u32,
    /// "2 day streak — keep it up!", absent before the first full day.
    pub streak_label: Option<String>,
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
    let mut args = yap_i18n::FluentArgs::new();
    args.set("count", value);
    args.set("direction", if future { "future" } else { "past" });
    let text = yap_i18n::translate(&format!("time-{name}"), Some(&args));
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
            (true, _, _) => yap_i18n::tr!(
                "curriculum-headline.done",
                m = milestone.unwrap_or_default()
            ),
            (false, true, Some(m)) => yap_i18n::tr!("curriculum-headline.milestone", m = m),
            (false, true, None) => yap_i18n::tr!(
                "curriculum-headline.momentum",
                m = milestone.unwrap_or_default()
            ),
            (false, false, _) => yap_i18n::tr!(
                "curriculum-headline.great",
                m = milestone.unwrap_or_default()
            ),
        },
        emphasis: sentence_list_label.into(),
        after: yap_i18n::tr!(
            "curriculum-headline.after",
            m = milestone.unwrap_or_default()
        ),
    };
    let curriculum_learn_label = (!done && info.smart_add_count > 0).then(|| {
        if let (Some(m), false) = (milestone, info.recommend_more_cards) {
            yap_i18n::tr!(
                "curriculum-learn-milestone",
                count = info.smart_add_count,
                milestone = m
            )
        } else {
            yap_i18n::tr!("curriculum-learn-cards", count = info.smart_add_count)
        }
    });
    let next_sentence_list_label =
        done.then_some(next_sentence_list)
            .flatten()
            .map(|next| match next {
                SentenceListSelection::Movie { .. } => yap_i18n::tr!("curriculum-next-movie"),
                SentenceListSelection::PimsleurLesson { .. } => {
                    yap_i18n::tr!("curriculum-next-lesson")
                }
            });
    let all_learned_note = (done && next_sentence_list.is_none())
        .then(|| yap_i18n::tr!("idle-you-ve-learned-all-available-words"));
    let level_note = selection.is_none().then(|| {
        yap_i18n::tr!(
            "curriculum-level-coverage",
            percent = format!("{:.1}", info.tier_info.percent_of_usage),
            language = (get_language_metadata(target_language).common_name).to_string()
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
            before: yap_i18n::tr!("next-review-word.before", when = when.as_str()),
            emphasis: card.card_text.clone(),
            after: yap_i18n::tr!("next-review-word.after", when = when),
        },
        _ => EmphasizedText {
            before: yap_i18n::tr!("review-your-next-review-is-when", when = (when).to_string()),
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
            self.last_plan_day == Some(day),
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
        if let Some(offer) = self.get_release_offer(banned.clone(), timestamp_ms) {
            let plan = ReviewPlanView::new(
                self.get_target_language(),
                offer.release_preview(),
                offer.unlock_event(),
                week,
                self.last_plan_day == Some(day),
            );
            if self.get_today_time_spent_on(day) == 0
                || !self.study_plan_was_recently_accepted(timestamp_ms)
            {
                return IdleScreenView::ReviewPlanOffer(plan);
            }
            let minutes = (f64::from(self.get_today_time_spent_on(day)) / 60.0).round() as u32;
            return IdleScreenView::StudyPlanComplete {
                title: yap_i18n::tr!("plan-completed", count = minutes),
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
            commit_curriculum,
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
                yap_i18n::tr!("idle-all-done"),
                yap_i18n::tr!("idle-you-ve-learned-all-available-words"),
            )
        } else if idle.has_never_studied {
            (
                IdleKind::FirstRun,
                yap_i18n::tr!("idle-ready-to-start-learning"),
                yap_i18n::tr!("idle-we-ll-start-with-a-couple-words-you-might"),
            )
        } else if idle.no_schedulable_cards {
            let easy = info.smart_add_regime == SmartAddRegime::Easy;
            (
                IdleKind::NeedsMoreCards,
                if easy {
                    yap_i18n::tr!("idle-adding-cards-is-how-you-learn-more")
                } else {
                    yap_i18n::tr!("idle-ready-for-more")
                },
                if easy && info.easy_cards_remaining > 0 {
                    yap_i18n::tr!("idle-easy-words", count = info.easy_cards_remaining)
                } else {
                    yap_i18n::tr!("idle-add-some-cards-to-keep-building-your-vocabulary")
                },
            )
        } else {
            (
                IdleKind::AllCaughtUp,
                yap_i18n::tr!("idle-all-caught-up"),
                String::new(),
            )
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
            title,
            body,
            manual_add_heading: manual_add_heading(),
            manual_add_options,
            info,
            next_due,
            curriculum_headline,
            curriculum_learn_label,
            next_sentence_list_label,
            all_learned_note,
            level_note,
            change_curriculum_label: yap_i18n::tr!("idle-change-curriculum"),
            banned_notice: (review.due_but_banned_count() > 0)
                .then(|| yap_i18n::tr!("idle-paused-cards", count = review.due_but_banned_count())),
            week,
            navigation,
            sentence_list_options,
            progress,
            sentence_list_label,
            next_sentence_list,
            commit_curriculum,
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
        let streak = self.get_daily_streak_on(day);
        Some(AccomplishmentView {
            target_language: self.get_target_language(),
            heading: accomplishment_heading(&today.day_of_week),
            accomplishment,
            studied: yap_i18n::tr!(
                "accomplishment-studied",
                minutes = today.time_spent_seconds / 60,
                count = today.reviews
            ),
            today,
            streak,
            streak_label: (streak > 0).then(|| {
                yap_i18n::tr!(
                    "accomplishment-streak",
                    streak = streak,
                    encouragement = streak_message(streak)
                )
            }),
            percent_known,
            words_known: (percent_known * self.num_cards_added() as f64).round() as u32,
            target: self.get_daily_review_target_setting(),
            goals: self.goal_options_view(),
            days: self.get_current_week_progress_on(day),
        })
    }
}

/// The idle card's curriculum. Its chevrons and "next lesson" only move the
/// host's draft selection — the `sentence_list` input — and
/// `commit_curriculum` is the one event that commits it, so a
/// `SetSentenceList` is appended exactly once per real switch.
struct CurriculumView {
    navigation: SentenceListNavigation,
    sentence_list_options: Vec<SentenceListOptionView>,
    progress: SentenceListProgress,
    sentence_list_label: String,
    next_sentence_list: Option<SentenceListSelection>,
    commit_curriculum: Option<DeckEvent>,
}

fn sentence_list_category_label(category: SentenceListCategory) -> String {
    match category {
        SentenceListCategory::Essential => yap_i18n::tr!("curriculum-essential"),
        SentenceListCategory::Movie => yap_i18n::tr!("common-movies"),
        SentenceListCategory::Pimsleur => "Pimsleur".to_owned(),
    }
}

fn smart_add_label(kind: &IdleKind, info: &NoCardsReadyInfo) -> Option<String> {
    match kind {
        IdleKind::FirstRun => Some(yap_i18n::tr!("idle-start-learning")),
        IdleKind::NeedsMoreCards => Some(if info.smart_add_regime == SmartAddRegime::Easy {
            yap_i18n::tr!("idle-add-easy-cards", count = info.smart_add_count)
        } else {
            yap_i18n::tr!("idle-add-cards", count = info.smart_add_count)
        }),
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
        let info = self.get_no_cards_ready_info(banned, navigation.selection.clone());
        let progress = self
            .get_sentence_list_progress(navigation.selection.clone(), info.tier_info.percent_known);
        let sentence_list_label = match &navigation.selection {
            None => yap_i18n::tr!(
                "curriculum-tier-level",
                tier = (info.tier_info.name).to_string(),
                language =
                    (get_language_metadata(self.get_target_language()).common_name).to_string(),
                level = info.tier_info.level
            ),
            Some(SentenceListSelection::Movie { id }) => self.movie_title(id),
            Some(SentenceListSelection::PimsleurLesson { level, lesson }) => {
                yap_i18n::tr!(
                    "curriculum-pimsleur-level-level-lesson-lesson",
                    level = *level,
                    lesson = *lesson
                )
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
                    label: sentence_list_category_label(*category),
                    selection,
                }
            })
            .collect();
        let commit_curriculum = (navigation.selection != self.get_sentence_list())
            .then(|| self.change_sentence_list(navigation.selection.clone()));
        (
            CurriculumView {
                navigation,
                sentence_list_options,
                progress,
                sentence_list_label,
                next_sentence_list,
                commit_curriculum,
            },
            info,
        )
    }

    fn movie_title(&self, id: &str) -> String {
        self.get_movie_metadata(vec![id.to_string()])
            .first()
            .map(|m| m.title.clone())
            .unwrap_or_else(|| yap_i18n::tr!("curriculum-movie"))
    }

    /// Home's curriculum row, for the persisted selection (or the idle card's
    /// draft, which Home hides this row for anyway).
    fn home_curriculum_view(
        &self,
        tier: &TierInfo,
        navigation: SentenceListNavigation,
    ) -> HomeCurriculumView {
        let language = get_language_metadata(self.get_target_language()).common_name;
        let essential_name = yap_i18n::tr!(
            "curriculum-tier-language",
            tier = tier.name.as_str(),
            language = language.as_str()
        );
        let category = navigation.categories[navigation.selected_index];
        let options = navigation
            .categories
            .iter()
            .map(|category| CurriculumCategoryView {
                category: *category,
                label: match category {
                    SentenceListCategory::Essential => essential_name.clone(),
                    _ => sentence_list_category_label(*category),
                },
            })
            .collect();
        let progress =
            self.get_sentence_list_progress(navigation.selection.clone(), tier.percent_known);
        let (menu_label, item, level_label, subtitle) = match &navigation.selection {
            None => (
                essential_name,
                None,
                Some(yap_i18n::tr!(
                    "curriculum-level-progress",
                    level = tier.level,
                    total = tier.total_levels
                )),
                Some(yap_i18n::tr!(
                    "curriculum-unlocks-coverage",
                    percent = format!("{:.1}", tier.percent_of_usage),
                    language = (language).to_string()
                )),
            ),
            Some(SentenceListSelection::Movie { id }) => (
                yap_i18n::tr!("common-movies"),
                Some(CurriculumItemView {
                    title: self.movie_title(id),
                    movie_id: Some(id.clone()),
                    browse_label: yap_i18n::tr!("curriculum-change"),
                }),
                None,
                None,
            ),
            Some(SentenceListSelection::PimsleurLesson { level, lesson }) => (
                "Pimsleur".to_owned(),
                Some(CurriculumItemView {
                    title: yap_i18n::tr!(
                        "curriculum-level-level-lesson-lesson",
                        level = *level,
                        lesson = *lesson
                    ),
                    movie_id: None,
                    browse_label: yap_i18n::tr!("curriculum-change"),
                }),
                None,
                None,
            ),
        };
        HomeCurriculumView {
            category,
            menu_label,
            options,
            item,
            level_label,
            percent: progress.percent_known,
            percent_label: progress.bar_label,
            subtitle,
        }
    }

    fn goal_options_view(&self) -> Vec<GoalOptionView> {
        get_daily_goal_options()
            .into_iter()
            .map(|option| GoalOptionView {
                label: match option.value {
                    DailyReviewTarget::Casual => yap_i18n::tr!("goal-casual"),
                    DailyReviewTarget::Regular => yap_i18n::tr!("goal-regular"),
                    DailyReviewTarget::Serious => yap_i18n::tr!("goal-serious"),
                    DailyReviewTarget::Intense => yap_i18n::tr!("goal-intense"),
                },
                duration_label: yap_i18n::tr!("goal-minutes-short", minutes = option.minutes),
                minutes_label: yap_i18n::tr!("common-minutes", minutes = option.minutes),
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

/// Home's curriculum row. Its category menu and the curriculum browser commit
/// on the spot: choosing is the whole interaction, so there is no draft to confirm.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeCurriculumView {
    pub category: SentenceListCategory,
    /// The menu button: "Advanced French", "Movies" or "Pimsleur".
    pub menu_label: String,
    /// Choosing one appends `choose_curriculum_category`'s event.
    pub options: Vec<CurriculumCategoryView>,
    /// The chosen movie or lesson; the essential course has nothing to browse.
    pub item: Option<CurriculumItemView>,
    /// "Level 11 of 30", essential course only.
    pub level_label: Option<String>,
    /// Progress through this level or list, on a 0–100 scale.
    pub percent: f64,
    pub percent_label: String,
    /// "Unlocks 67.1% of everyday French", essential course only.
    pub subtitle: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurriculumCategoryView {
    pub category: SentenceListCategory,
    pub label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurriculumItemView {
    pub title: String,
    pub movie_id: Option<String>,
    /// "Change": opens the browser on this category.
    pub browse_label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreakCardView {
    pub title: String,
    pub days: u32,
    pub days_label: String,
    pub today_label: String,
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
    /// "0 / [5 min] today": the emphasis is the daily-goal menu.
    pub today: EmphasizedText,
    pub daily_goal: DailyReviewTarget,
    pub goal_options: Vec<GoalOptionView>,
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
    /// "A little [icon French ⌄], every day.": the course name sits inside the
    /// greeting and is the course switcher, so Home names the language once.
    pub greeting_lead: String,
    pub course_icon: String,
    pub course_name: String,
    pub greeting_tail: String,
    pub native_language: Language,
    pub target_language: Language,
    pub up_next: UpNextView,
    /// Absent while Up Next shows the idle curriculum card, which has its own.
    pub curriculum: Option<HomeCurriculumView>,
    pub week: WeekCardView,
    pub xp: HomeStatView,
    pub cards: HomeStatView,
    pub dictionary: DictionaryCardView,
    pub books: HomeBooksView,
    pub due_count: u64,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeBooksView {
    pub title: String,
    pub caption: String,
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
        yap_i18n::tr!(
            "stats-thousands",
            thousands = format!("{:.1}", rank / 1000.0)
        )
    } else {
        format!("{rank:.0}")
    }
}

/// Both hosts use the same rank ticks and compact labels.
#[bridgerton::bridge]
#[bridgerton::stable(strong)]
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
    pub your_cards: WordListSummaryView,
    pub assumed_known: WordListSummaryView,
    pub leeches: Vec<CardSummary>,
    pub leech_count: u64,
    pub leeches_label: String,
    pub frequency_knowledge_chart_data: Vec<FrequencyKnowledgePoint>,
    pub frequency_knowledge_chart_title: String,
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
    fn streak_card_view(&self, timestamp_ms: f64) -> StreakCardView {
        let day = DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64)
            .unwrap_or_else(Utc::now)
            .with_timezone(&self.context.timezone)
            .date_naive();
        let days = self.get_daily_streak_on(day);
        StreakCardView {
            title: yap_i18n::tr!("stats-streak"),
            days,
            days_label: yap_i18n::tr!("streak-days", count = days),
            today_label: yap_i18n::tr!(
                "stats-today-minutes",
                studied = self.get_today_time_spent_on(day) / 60,
                goal = self.get_daily_review_target() / 60
            ),
        }
    }

    fn percent_known_label(&self) -> String {
        yap_i18n::tr!(
            "stats-language-coverage",
            percent = format!("{:.1}", self.get_percent_of_words_known() * 100.0),
            language = (get_language_metadata(self.get_target_language()).common_name).to_string()
        )
    }

    fn due_summary_view(&self, ready_now: u64) -> DueSummaryView {
        let total = self.get_all_cards_summary().len() as u64;
        DueSummaryView {
            title: yap_i18n::tr!("stats-due-words"),
            ready_now,
            total,
            label: yap_i18n::tr!("due-summary", ready = ready_now, count = total),
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
                yap_i18n::tr!("common-flashcard"),
            ),
            CardIndicator::ListeningGram { .. } => (
                yap_i18n::tr!("screen-listen-to-the-word"),
                UpNextKind::Listening,
                yap_i18n::tr!("common-listening"),
            ),
            CardIndicator::LetterPronunciation { pattern, .. } => (
                pattern.clone(),
                UpNextKind::Pronunciation,
                yap_i18n::tr!("common-pronunciation"),
            ),
        },
        Challenge::PronunciationChallenge { pattern, .. } => (
            pattern.clone(),
            UpNextKind::Pronunciation,
            yap_i18n::tr!("common-pronunciation"),
        ),
        Challenge::TranslateComprehensibleSentence(sentence) => (
            sentence.target_language.clone(),
            UpNextKind::Translation,
            yap_i18n::tr!("common-translation"),
        ),
        Challenge::TranscribeComprehensibleSentence(_) => (
            yap_i18n::tr!("transcription-listen-and-fill-in-the-blanks"),
            UpNextKind::Transcription,
            yap_i18n::tr!("common-transcription"),
        ),
    };
    (headline, kind, label)
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
        let curriculum = (!shows_curriculum).then(|| {
            let navigation = self.curriculum_navigation(inputs.sentence_list.clone());
            // The idle view already computed the tier; otherwise pay for it only when the row shows.
            let tier_info = match &step {
                ReviewStep::Idle(idle) if let IdleScreenView::Idle(view) = idle.as_ref() => {
                    view.info.tier_info.clone()
                }
                _ => {
                    self.get_no_cards_ready_info(
                        inputs.banned.clone(),
                        navigation.selection.clone(),
                    )
                    .tier_info
                }
            };
            self.home_curriculum_view(&tier_info, navigation)
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
                    yap_i18n::tr!("home-placement-test")
                } else if info.too_advanced {
                    yap_i18n::tr!("home-you-might-be-too-advanced")
                } else {
                    yap_i18n::tr!("home-ready-to-start")
                };
                (headline, yap_i18n::tr!("home-placement-test-2"), None)
            }
            ReviewStep::ReviewPlan(_) => (
                yap_i18n::tr!("home-your-study-plan-is-ready"),
                yap_i18n::tr!("home-study-plan"),
                None,
            ),
            ReviewStep::SetDisplayName => (
                yap_i18n::tr!("home-choose-your-display-name"),
                yap_i18n::tr!("home-profile"),
                None,
            ),
            ReviewStep::Accomplishment(view) => {
                (view.heading, yap_i18n::tr!("home-accomplishment"), None)
            }
            ReviewStep::Idle(idle) => {
                let (headline, kind_label) = match idle.as_ref() {
                    IdleScreenView::Idle(view) => {
                        (view.title.clone(), yap_i18n::tr!("common-review"))
                    }
                    IdleScreenView::AudioPending { online, .. } => (
                        if *online {
                            yap_i18n::tr!("home-preparing-audio")
                        } else {
                            yap_i18n::tr!("home-connect-to-download-audio")
                        },
                        yap_i18n::tr!("home-audio-pending"),
                    ),
                    IdleScreenView::ReviewPlanOffer(_) => (
                        yap_i18n::tr!("home-your-study-plan-is-ready"),
                        yap_i18n::tr!("home-study-plan"),
                    ),
                    IdleScreenView::StudyPlanComplete { title, .. } => {
                        (title.clone(), yap_i18n::tr!("home-study-plan"))
                    }
                };
                (headline, kind_label, Some(*idle))
            }
        };
        let language = get_language_metadata(self.get_target_language());
        let cards_studied = self.num_cards_studied();
        let day = DateTime::<Utc>::from_timestamp_millis(inputs.timestamp_ms as i64)
            .unwrap_or_else(Utc::now)
            .with_timezone(&self.context.timezone)
            .date_naive();
        HomeScreenView {
            title: yap_i18n::tr!("common-home"),
            greeting_lead: yap_i18n::tr!("home-greeting.before"),
            course_icon: language.icon.clone(),
            course_name: language.common_name.clone(),
            greeting_tail: yap_i18n::tr!("home-greeting.after"),
            native_language: self.context.course.native_language,
            target_language: self.get_target_language(),
            up_next: UpNextView {
                eyebrow: yap_i18n::tr!(
                    "home-up-next-kind-label",
                    kind_label = (kind_label).to_string()
                ),
                headline,
                kind,
                action_label: if !is_challenge {
                    yap_i18n::tr!("common-continue")
                } else {
                    // A held challenge can outlive the due queue, but it's still a card.
                    let count = due_count.max(1);
                    yap_i18n::tr!("review-cards", count = count)
                },
                due_count,
                idle,
            },
            curriculum,
            week: WeekCardView {
                title: yap_i18n::tr!("home-this-week"),
                today: EmphasizedText {
                    before: yap_i18n::tr!(
                        "home-daily-progress.before",
                        studied = self.get_today_time_spent_on(day) / 60,
                        goal = self.get_daily_review_target() / 60
                    ),
                    emphasis: yap_i18n::tr!(
                        "home-daily-progress.goal",
                        studied = self.get_today_time_spent_on(day) / 60,
                        goal = self.get_daily_review_target() / 60
                    ),
                    after: yap_i18n::tr!(
                        "home-daily-progress.after",
                        studied = self.get_today_time_spent_on(day) / 60,
                        goal = self.get_daily_review_target() / 60
                    ),
                },
                daily_goal: self.get_daily_review_target_setting(),
                goal_options: self.goal_options_view(),
                days: self.get_current_week_progress_on(day),
            },
            xp: HomeStatView {
                value: format!("{:.0}", self.get_xp()),
                caption: yap_i18n::tr!("home-xp-earned"),
                note: None,
            },
            cards: HomeStatView {
                value: cards_studied.to_string(),
                caption: yap_i18n::tr!("home-cards-studied", count = cards_studied),
                note: Some(self.percent_known_label()),
            },
            dictionary: DictionaryCardView {
                title: yap_i18n::tr!("home-find-a-word"),
                search_placeholder: yap_i18n::tr!(
                    "dictionary-search-languages",
                    target =
                        (get_language_metadata(self.get_target_language()).common_name).to_string(),
                    native = (get_language_metadata(self.context.course.native_language)
                        .common_name)
                        .to_string()
                ),
            },
            books: HomeBooksView {
                title: crate::books::books_copy().title,
                caption: yap_i18n::tr!(
                    "books-home-caption",
                    language = get_language_metadata(self.get_target_language()).common_name
                ),
            },
            due_count,
        }
    }

    #[bridgerton::stable]
    pub fn stats_screen_view(
        &self,
        banned: Vec<ChallengeRequirements>,
        timestamp_ms: f64,
    ) -> StatsScreenView {
        let percent_known = self.get_percent_of_words_known();
        let review = self.get_review_info(banned, timestamp_ms);
        let leeches = self.get_leeches();
        let leech_count = leeches.len() as u64;
        let xp = self.get_xp();
        let total_reviews = self.get_total_reviews();
        let streak = self.streak_card_view(timestamp_ms);
        StatsScreenView {
            target_language: self.get_target_language(),
            title: yap_i18n::tr!("common-stats"),
            xp,
            total_reviews,
            tiles: vec![
                StatTileView {
                    eyebrow: yap_i18n::tr!("stats-xp"),
                    value: format!("{xp:.0}"),
                    caption: None,
                },
                StatTileView {
                    eyebrow: yap_i18n::tr!("common-reviews"),
                    value: total_reviews.to_string(),
                    caption: None,
                },
                StatTileView {
                    eyebrow: streak.title,
                    value: streak.days_label,
                    caption: Some(streak.today_label),
                },
                StatTileView {
                    eyebrow: yap_i18n::tr!("stats-vocabulary"),
                    value: format!("{:.1}%", percent_known * 100.0),
                    caption: Some(yap_i18n::tr!(
                        "stats-everyday-language",
                        language = (get_language_metadata(self.get_target_language()).common_name)
                            .to_string()
                    )),
                },
            ],
            percent_known,
            due: self.due_summary_view(review.due_count() as u64),
            your_cards: self.your_cards_summary(self.added_card_count()),
            assumed_known: self.assumed_known_summary(self.assumed_known_senses().len()),
            leeches,
            leech_count,
            leeches_label: yap_i18n::tr!("stats-leeches", count = leech_count),
            frequency_knowledge_chart_data: self.get_frequency_knowledge_chart_data(),
            frequency_knowledge_chart_title: yap_i18n::tr!("stats-word-knowledge-by-frequency"),
        }
    }

    #[bridgerton::stable]
    pub fn due_words_view(
        &self,
        banned: Vec<ChallengeRequirements>,
        timestamp_ms: f64,
    ) -> DueWordsScreenView {
        let review = self.get_review_info(banned, timestamp_ms);
        let summary = self.due_summary_view(review.due_count() as u64);
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
    /// Typing guidance under a dictation, for learners still early on; hosts
    /// remember the dismissal locally.
    pub keyboard_tip: Option<String>,
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
            keyboard_tip: (review.total_count() < 60)
                .then(|| crate::learning_metadata::keyboard_tip(self.get_target_language()))
                .flatten(),
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
        "Monday" => yap_i18n::tr!("accomplishment-monday-can-t-stop-you"),
        "Tuesday" => yap_i18n::tr!("accomplishment-solid-tuesday-session"),
        "Wednesday" => yap_i18n::tr!("accomplishment-midweek-momentum"),
        "Thursday" => yap_i18n::tr!("accomplishment-thursday-well-spent"),
        "Friday" => yap_i18n::tr!("accomplishment-happy-friday"),
        "Saturday" => yap_i18n::tr!("accomplishment-weekend-warrior"),
        "Sunday" => yap_i18n::tr!("accomplishment-so-much-for-the-day-of-rest"),
        _ => yap_i18n::tr!("accomplishment-nice"),
    };
    yap_i18n::tr!(
        "accomplishment-goal-reached-message",
        message = (message).to_string()
    )
}

fn streak_message(streak: u32) -> String {
    match streak {
        ..=1 => yap_i18n::tr!("accomplishment-you-re-just-getting-started"),
        2..=3 => yap_i18n::tr!("accomplishment-keep-it-up"),
        4..=7 => yap_i18n::tr!("accomplishment-you-re-on-a-roll"),
        8..=14 => yap_i18n::tr!("accomplishment-impressive-dedication"),
        15..=30 => yap_i18n::tr!("accomplishment-unstoppable"),
        31..=60 => yap_i18n::tr!("accomplishment-what-a-habit"),
        61..=100 => yap_i18n::tr!("accomplishment-legendary-consistency"),
        _ => yap_i18n::tr!("accomplishment-absolutely-incredible"),
    }
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
            false,
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
        assert_eq!(idle.manual_add_heading, "Choose cards to add");
    }

    fn with_due_cards() -> Deck {
        let deck = Deck::default();
        let event = deck
            .get_no_cards_ready_info(vec![], None)
            .smart_add_event
            .unwrap();
        apply(deck, event)
    }

    fn apply(deck: Deck, event: DeckEvent) -> Deck {
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

    #[test]
    fn home_counts_already_known_cards_as_studied() {
        let deck = with_due_cards();
        let scheduled = deck.get_all_cards_summary();
        assert_eq!(deck.home_screen_view(inputs()).cards.value, "0");
        let event = deck
            .review_cards(vec![crate::CardReview {
                card: scheduled[0].card_indicator.clone(),
                rating: current::Rating::Remembered,
            }])
            .unwrap();
        let deck = apply(deck, event);
        // Already-known cards leave the schedule but still count as studied.
        assert_eq!(deck.get_all_cards_summary().len(), scheduled.len() - 1);
        assert_eq!(deck.home_screen_view(inputs()).cards.value, "1");
    }

    fn json(value: impl Serialize) -> serde_json::Value {
        serde_json::to_value(value).unwrap()
    }

    #[test]
    fn categories_reopen_on_the_last_list_chosen_in_them() {
        let movie = SentenceListSelection::Movie {
            id: "tt0097904".into(),
        };
        let deck = Deck::default();
        let event = deck.change_sentence_list(Some(movie.clone()));
        let deck = apply(deck, event);
        let event = deck.choose_curriculum_category(SentenceListCategory::Essential);
        let deck = apply(deck, event);
        assert_eq!(deck.get_sentence_list(), None);
        let home = deck.home_screen_view(inputs());
        assert!(home.curriculum.is_none() || home.curriculum.unwrap().item.is_none());
        let event = deck.choose_curriculum_category(SentenceListCategory::Movie);
        let deck = apply(deck, event);
        assert_eq!(deck.get_sentence_list(), Some(movie));
    }

    #[test]
    fn idle_curriculum_commits_only_a_real_switch() {
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
                let IdleScreenView::Idle(idle) = home.up_next.idle.unwrap() else {
                    panic!("new deck should be idle");
                };
                assert_eq!(home.up_next.headline, idle.title);
                assert_eq!(home.curriculum.is_none(), idle.show_sentence_list);
                assert_eq!(
                    idle.commit_curriculum.is_some(),
                    idle.navigation.selection != deck.get_sentence_list()
                );
                assert_eq!(
                    json(deck.get_manual_add_options(idle.navigation.selection.clone(), signed_in)),
                    json(idle.manual_add_options)
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
        assert_eq!(home.course_name, "French");
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
        let curriculum = home
            .curriculum
            .expect("cards are ready, so Home shows the row");
        assert_eq!(curriculum.category, SentenceListCategory::Essential);
        assert_eq!(curriculum.options[0].label, curriculum.menu_label);
        assert_eq!(home.week.today.emphasis, "10 min");
        assert_eq!(home.week.goal_options[0].minutes_label, "5 min");
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
        let week = &home.week.today;
        assert_eq!(
            format!("{}{}{}", week.before, week.emphasis, week.after),
            tomorrow.today_label
        );
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
        let goal = deck.home_curriculum_view(
            &TierInfo {
                tier: 1,
                name: "Elementary".into(),
                level: 5,
                total_levels: 7,
                percent_known: 70.0,
                percent_of_usage: 24.8,
            },
            deck.curriculum_navigation(None),
        );
        assert_eq!(goal.menu_label, "Elementary French");
        assert_eq!(goal.percent, 70.0);
        assert_eq!(goal.percent_label, "70%");
        assert_eq!(goal.level_label.as_deref(), Some("Level 5 of 7"));
        assert_eq!(
            goal.subtitle.as_deref(),
            Some("Unlocks 24.8% of everyday French")
        );
        assert!(goal.item.is_none());
        let home = deck.home_screen_view(inputs());
        assert_eq!(home.course_icon, "fra.eiffel");
        assert_eq!(
            format!(
                "{} {}{}",
                home.greeting_lead, home.course_name, home.greeting_tail
            ),
            "A little French, every day."
        );
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
        assert_eq!(home.week.today.before, "0 / ");
        assert_eq!(
            home.week.today.emphasis,
            format!("{} min", deck.get_daily_review_target() / 60)
        );
    }
}
