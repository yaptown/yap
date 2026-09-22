//! Deck loading decisions shared by both hosts. Hosts own IO and cancellation;
//! a completed task must still belong to the current course and input snapshot.
#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum PackStage {
    None,
    Core,
    Full,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct DeckLoadProgress {
    pub message: String,
    pub percent: f32,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct DeckLoadState {
    pub course_key: Option<String>,
    pub streams_ready: bool,
    pub language_selected: bool,
    pub pack: PackStage,
    pub pack_error: Option<String>,
    pub progress: Option<DeckLoadProgress>,
    pub built_inputs: Option<String>,
    pub deck_present: bool,
    pub build_error: Option<String>,
    pub retry_count: u32,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum DeckLoadEvent {
    CourseChanged { key: Option<String> },
    StreamsReady { language_selected: bool },
    PackProgress { message: String, percent: f32 },
    CoreLoaded,
    FullLoaded,
    PackFailed { message: String, network: bool },
    InputsChanged { key: String },
    DeckBuilt { present: bool, inputs: String },
    DeckBuildFailed { message: String },
    Retry,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum DeckLoadEffect {
    ClearDeck,
    LoadPack {
        course_key: String,
        from: PackStage,
    },
    RefreshInputs,
    BuildDeck {
        inputs: String,
    },
    ReportError {
        phase: String,
        message: String,
        network: bool,
    },
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct DeckLoadStep {
    pub state: DeckLoadState,
    pub effects: Vec<DeckLoadEffect>,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum DeckLoadPhase {
    Loading {
        message: String,
        percent: f32,
    },
    NoLanguageSelected,
    Ready,
    Error {
        heading: String,
        body: String,
        title: String,
        message: String,
        retry_label: String,
    },
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct PackBanner {
    pub message: String,
    pub retry_label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct DeckLoadView {
    pub phase: DeckLoadPhase,
    pub pack_banner: Option<PackBanner>,
}

#[bridgerton::bridge]
pub fn deck_load_start() -> DeckLoadState {
    DeckLoadState {
        course_key: None,
        streams_ready: false,
        language_selected: false,
        pack: PackStage::None,
        pack_error: None,
        progress: None,
        built_inputs: None,
        deck_present: false,
        build_error: None,
        retry_count: 0,
    }
}

#[bridgerton::bridge]
pub fn deck_load_transition(mut state: DeckLoadState, event: DeckLoadEvent) -> DeckLoadStep {
    use DeckLoadEffect::*;
    let mut effects = vec![];
    match event {
        DeckLoadEvent::CourseChanged { key } => {
            if key != state.course_key {
                state.course_key = key.clone();
                state.pack = PackStage::None;
                state.pack_error = None;
                state.progress = None;
                state.built_inputs = None;
                state.deck_present = false;
                state.build_error = None;
                effects.push(ClearDeck);
                if let Some(course_key) = key {
                    effects.push(LoadPack {
                        course_key,
                        from: PackStage::None,
                    });
                }
            }
        }
        DeckLoadEvent::StreamsReady { language_selected } => {
            state.streams_ready = true;
            state.language_selected = language_selected;
            if !language_selected {
                state.built_inputs = None;
                state.deck_present = false;
                effects.push(ClearDeck);
            }
        }
        DeckLoadEvent::PackProgress { message, percent } => {
            state.progress = Some(DeckLoadProgress { message, percent });
        }
        DeckLoadEvent::CoreLoaded => {
            state.pack = PackStage::Core;
            effects.push(RefreshInputs);
        }
        DeckLoadEvent::FullLoaded => {
            state.pack = PackStage::Full;
            state.progress = None;
            state.pack_error = None;
            effects.push(RefreshInputs);
        }
        DeckLoadEvent::PackFailed { message, network } => {
            state.pack_error = Some(message.clone());
            state.progress = None;
            effects.push(ReportError {
                phase: "pack".into(),
                message,
                network,
            });
        }
        DeckLoadEvent::InputsChanged { key } => {
            if state.pack != PackStage::None
                && state.streams_ready
                && state.language_selected
                && state.built_inputs.as_ref() != Some(&key)
            {
                effects.push(BuildDeck { inputs: key });
            }
        }
        DeckLoadEvent::DeckBuilt { present, inputs } => {
            state.built_inputs = Some(inputs);
            state.deck_present = present;
            state.build_error = None;
        }
        DeckLoadEvent::DeckBuildFailed { message } => {
            state.build_error = Some(message.clone());
            state.deck_present = false;
            state.built_inputs = None;
            effects.extend([
                ClearDeck,
                ReportError {
                    phase: "deck-state".into(),
                    message,
                    network: false,
                },
            ]);
        }
        DeckLoadEvent::Retry => {
            state.retry_count += 1;
            state.pack_error = None;
            state.build_error = None;
            let from = if state.deck_present && state.pack == PackStage::Core {
                PackStage::Core
            } else {
                state.pack = PackStage::None;
                state.progress = None;
                state.built_inputs = None;
                state.deck_present = false;
                effects.push(ClearDeck);
                PackStage::None
            };
            if let Some(course_key) = state.course_key.clone() {
                effects.push(LoadPack { course_key, from });
            }
        }
    }
    DeckLoadStep { state, effects }
}

#[bridgerton::bridge]
pub fn deck_load_view(state: DeckLoadState) -> DeckLoadView {
    let error = state.build_error.as_ref().or_else(|| {
        (!state.deck_present)
            .then_some(state.pack_error.as_ref())
            .flatten()
    });
    let phase = if state.streams_ready && !state.language_selected {
        DeckLoadPhase::NoLanguageSelected
    } else if let Some(message) = error {
        DeckLoadPhase::Error {
            heading: "Failed to Load Language Data".into(),
            body: "Unable to load the language pack right now. Try again to retry the download."
                .into(),
            title: "Failed to load language data".into(),
            message: message.clone(),
            retry_label: "Try Again".into(),
        }
    } else if state.deck_present {
        DeckLoadPhase::Ready
    } else {
        let progress = state.progress.unwrap_or(DeckLoadProgress {
            message: "Loading...".into(),
            percent: 0.0,
        });
        DeckLoadPhase::Loading {
            message: progress.message,
            percent: progress.percent,
        }
    };
    let pack_banner = state
        .pack_error
        .filter(|_| state.deck_present)
        .map(|e| PackBanner {
            message: format!("Couldn't finish downloading the language pack: {e}"),
            retry_label: "Retry".into(),
        });
    DeckLoadView { phase, pack_banner }
}

#[cfg(test)]
mod tests {
    use super::*;
    use DeckLoadEvent::*;
    fn apply(state: &mut DeckLoadState, event: DeckLoadEvent) -> Vec<DeckLoadEffect> {
        let step = deck_load_transition(state.clone(), event);
        *state = step.state;
        step.effects
    }
    fn core() -> DeckLoadState {
        let mut state = deck_load_start();
        apply(
            &mut state,
            CourseChanged {
                key: Some("fra:eng".into()),
            },
        );
        apply(
            &mut state,
            StreamsReady {
                language_selected: true,
            },
        );
        apply(&mut state, CoreLoaded);
        state
    }
    fn built() -> DeckLoadState {
        let mut state = core();
        apply(
            &mut state,
            DeckBuilt {
                present: true,
                inputs: "core".into(),
            },
        );
        state
    }
    #[test]
    fn happy_path() {
        let mut state = core();
        assert_eq!(
            apply(&mut state, InputsChanged { key: "core".into() }),
            vec![DeckLoadEffect::BuildDeck {
                inputs: "core".into()
            }]
        );
        apply(
            &mut state,
            DeckBuilt {
                present: false,
                inputs: "core".into(),
            },
        );
        assert_eq!(
            deck_load_view(state.clone()).phase,
            DeckLoadPhase::Loading {
                message: "Loading...".into(),
                percent: 0.0
            }
        );
        assert_eq!(
            apply(&mut state, FullLoaded),
            vec![DeckLoadEffect::RefreshInputs]
        );
        assert_eq!(
            apply(&mut state, InputsChanged { key: "full".into() }),
            vec![DeckLoadEffect::BuildDeck {
                inputs: "full".into()
            }]
        );
        apply(
            &mut state,
            DeckBuilt {
                present: true,
                inputs: "full".into(),
            },
        );
        assert_eq!(deck_load_view(state).phase, DeckLoadPhase::Ready);
    }
    #[test]
    fn upgrade_failure_and_retry_keep_deck() {
        let mut state = built();
        assert_eq!(
            apply(
                &mut state,
                PackFailed {
                    message: "offline".into(),
                    network: true
                }
            ),
            vec![DeckLoadEffect::ReportError {
                phase: "pack".into(),
                message: "offline".into(),
                network: true
            }]
        );
        assert_eq!(
            deck_load_view(state.clone()),
            DeckLoadView {
                phase: DeckLoadPhase::Ready,
                pack_banner: Some(PackBanner {
                    message: "Couldn't finish downloading the language pack: offline".into(),
                    retry_label: "Retry".into()
                })
            }
        );
        assert_eq!(
            apply(&mut state, Retry),
            vec![DeckLoadEffect::LoadPack {
                course_key: "fra:eng".into(),
                from: PackStage::Core
            }]
        );
        assert!(state.deck_present);
        assert_eq!(state.retry_count, 1);
        assert!(state.pack_error.is_none());
        apply(
            &mut state,
            PackProgress {
                message: "download".into(),
                percent: 50.0,
            },
        );
        assert_eq!(deck_load_view(state.clone()).phase, DeckLoadPhase::Ready);
        apply(&mut state, FullLoaded);
        assert!(state.progress.is_none());
    }
    #[test]
    fn retry_without_deck_resets() {
        let mut state = core();
        apply(
            &mut state,
            DeckBuilt {
                present: false,
                inputs: "core".into(),
            },
        );
        apply(
            &mut state,
            PackFailed {
                message: "failed".into(),
                network: false,
            },
        );
        assert!(matches!(
            deck_load_view(state.clone()).phase,
            DeckLoadPhase::Error { .. }
        ));
        assert_eq!(
            apply(&mut state, Retry),
            vec![
                DeckLoadEffect::ClearDeck,
                DeckLoadEffect::LoadPack {
                    course_key: "fra:eng".into(),
                    from: PackStage::None
                }
            ]
        );
        assert_eq!(state.pack, PackStage::None);
        assert!(state.built_inputs.is_none());
        assert!(!state.deck_present);
    }
    #[test]
    fn course_switch_clears_and_same_course_does_nothing() {
        let mut state = built();
        let key = state.course_key.clone();
        assert!(apply(&mut state, CourseChanged { key }).is_empty());
        assert_eq!(
            apply(
                &mut state,
                CourseChanged {
                    key: Some("spa:eng".into())
                }
            ),
            vec![
                DeckLoadEffect::ClearDeck,
                DeckLoadEffect::LoadPack {
                    course_key: "spa:eng".into(),
                    from: PackStage::None
                }
            ]
        );
        assert!(!state.deck_present);
        assert!(state.built_inputs.is_none());
        assert_eq!(state.pack, PackStage::None);
        assert_eq!(
            apply(&mut state, CourseChanged { key: None }),
            vec![DeckLoadEffect::ClearDeck]
        );
    }
    #[test]
    fn inputs_gate_and_cached_pack_before_streams() {
        let mut state = deck_load_start();
        apply(&mut state, CoreLoaded);
        assert!(apply(&mut state, InputsChanged { key: "core".into() }).is_empty());
        apply(
            &mut state,
            StreamsReady {
                language_selected: true,
            },
        );
        assert_eq!(
            apply(&mut state, InputsChanged { key: "core".into() }),
            vec![DeckLoadEffect::BuildDeck {
                inputs: "core".into()
            }]
        );
        apply(
            &mut state,
            DeckBuilt {
                present: true,
                inputs: "core".into(),
            },
        );
        assert!(apply(&mut state, InputsChanged { key: "core".into() }).is_empty());
    }
    #[test]
    fn build_failure_uses_web_copy() {
        let mut state = built();
        assert_eq!(
            apply(
                &mut state,
                DeckBuildFailed {
                    message: "broken".into()
                }
            ),
            vec![
                DeckLoadEffect::ClearDeck,
                DeckLoadEffect::ReportError {
                    phase: "deck-state".into(),
                    message: "broken".into(),
                    network: false
                }
            ]
        );
        assert_eq!(
            deck_load_view(state).phase,
            DeckLoadPhase::Error {
                heading: "Failed to Load Language Data".into(),
                body:
                    "Unable to load the language pack right now. Try again to retry the download."
                        .into(),
                title: "Failed to load language data".into(),
                message: "broken".into(),
                retry_label: "Try Again".into()
            }
        );
    }
    #[test]
    fn no_language_clears_snapshot_bookkeeping() {
        let mut state = built();
        assert_eq!(
            apply(
                &mut state,
                StreamsReady {
                    language_selected: false
                }
            ),
            vec![DeckLoadEffect::ClearDeck]
        );
        assert!(!state.deck_present);
        assert!(state.built_inputs.is_none());
        assert_eq!(
            deck_load_view(state.clone()).phase,
            DeckLoadPhase::NoLanguageSelected
        );
        assert!(
            apply(
                &mut state,
                InputsChanged {
                    key: "other".into()
                }
            )
            .is_empty()
        );
    }
}
