//! Serializable snapshots of the complete review screen.
//! Hosts own presentation state, never the choice of screen or deck actions.
use crate::*;

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdleKind {
    NothingToDo,
    FirstRun,
    NeedsMoreCards,
    AllCaughtUp,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewPlanView {
    pub target_language: Language,
    pub cards: Vec<CardSummary>,
    pub event: DeckEvent,
    pub week: Vec<DayProgress>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SentenceListOptionView {
    pub category: SentenceListCategory,
    pub selection: Option<SentenceListSelection>,
    pub event: DeckEvent,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdleView {
    pub target_language: Language,
    pub target: DailyReviewTarget,
    pub goals: Vec<GoalOptionView>,
    pub kind: IdleKind,
    pub title: String,
    pub body: String,
    pub info: NoCardsReadyInfo,
    pub manual_add_options: Vec<ManualAddOption>,
    pub next_due: Option<CardSummary>,
    pub banned_notice: Option<String>,
    pub week: Vec<DayProgress>,
    pub navigation: SentenceListNavigation,
    pub sentence_list_options: Vec<SentenceListOptionView>,
    pub progress: SentenceListProgress,
    pub sentence_list_label: String,
    pub next_sentence_list: Option<SentenceListSelection>,
    pub next_sentence_list_event: Option<DeckEvent>,
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
    pub minutes: u32,
    pub event: DeckEvent,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccomplishmentView {
    pub target_language: Language,
    pub accomplishment: Accomplishment,
    pub today: TodaySummary,
    pub streak: u32,
    pub percent_known: f64,
    pub words_known: u32,
    pub target: DailyReviewTarget,
    pub goals: Vec<GoalOptionView>,
    pub days: Vec<DayProgress>,
}

impl Deck {
    /// Initial backlog plan, rendered by the same plan view as a release offer.
    pub fn lockup_screen_view(
        &self,
        banned: Vec<ChallengeRequirements>,
        timestamp_ms: f64,
    ) -> Option<ReviewPlanView> {
        let offer = self.get_lockup_offer(banned, timestamp_ms)?;
        let day = DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64)?
            .with_timezone(&self.context.timezone)
            .date_naive();
        Some(ReviewPlanView {
            target_language: self.get_target_language(),
            cards: offer.keep_preview(),
            event: offer.lock_event(),
            week: self.get_current_week_progress_on(day),
        })
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
        let day = DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64)
            .unwrap_or_else(Utc::now)
            .with_timezone(&self.context.timezone)
            .date_naive();
        let review = self.get_review_info(banned.clone(), timestamp_ms);
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
            let plan = ReviewPlanView {
                target_language: self.get_target_language(),
                cards: offer.release_preview(),
                event: offer.unlock_event(),
                week,
            };
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
        let movies = self.get_movie_stats();
        let navigation = get_sentence_list_navigation(
            sentence_list,
            !movies.is_empty(),
            !self.get_pimsleur_stats().is_empty(),
        );
        let info = self.get_no_cards_ready_info(banned, navigation.selection.clone());
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
                    self.get_sentence_list_for_category(
                        *category,
                        movies.first().map(|movie| movie.id.clone()),
                    )
                };
                SentenceListOptionView {
                    category: *category,
                    event: self.change_sentence_list(selection.clone()),
                    selection,
                }
            })
            .collect();
        let next_sentence_list_event = next_sentence_list
            .as_ref()
            .map(|selection| self.change_sentence_list(Some(selection.clone())));
        IdleScreenView::Idle(Box::new(IdleView {
            target: self.get_daily_review_target_setting(),
            goals: self.goal_options_view(),
            target_language: self.get_target_language(),
            kind,
            title: title.into(),
            body,
            manual_add_options: self
                .get_manual_add_options(navigation.selection.clone(), is_signed_in),
            info,
            next_due,
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
            next_sentence_list_event,
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
        Some(AccomplishmentView {
            target_language: self.get_target_language(),
            accomplishment: self.get_accomplishment()?,
            today: self.get_today_summary_on(day),
            streak: self.get_daily_streak_on(day),
            percent_known: self.get_percent_of_words_known(),
            words_known: (self.get_percent_of_words_known() * self.num_cards_added() as f64).round()
                as u32,
            target: self.get_daily_review_target_setting(),
            goals: self.goal_options_view(),
            days: self.get_current_week_progress_on(day),
        })
    }
}

impl Deck {
    fn goal_options_view(&self) -> Vec<GoalOptionView> {
        get_daily_goal_options()
            .into_iter()
            .map(|option| GoalOptionView {
                target: option.value.clone(),
                minutes: option.minutes,
                event: self.set_daily_review_target(option.value),
            })
            .collect()
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

/// The entire review screen, also the on-disk parity fixture format.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReviewScreenView {
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
    pub fn review_screen_view(&self, inputs: ReviewScreenInputs) -> ReviewScreenView {
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
        } else if let Some(plan) =
            self.lockup_screen_view(inputs.banned.clone(), inputs.timestamp_ms)
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
            ReviewStep::Idle(Box::new(self.idle_screen_view(
                inputs.banned,
                inputs.sentence_list,
                inputs.online,
                inputs.is_signed_in,
                inputs.timestamp_ms,
            )))
        };
        let day = DateTime::<Utc>::from_timestamp_millis(inputs.timestamp_ms as i64)
            .unwrap_or_else(Utc::now)
            .with_timezone(&self.context.timezone)
            .date_naive();
        ReviewScreenView {
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
