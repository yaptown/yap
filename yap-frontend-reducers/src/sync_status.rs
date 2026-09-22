//! Sync presentation shared by both hosts. Hosts supply connectivity and their
//! manual-call lifecycle; the store supplies the rest of the facts.
use serde::{Deserialize, Serialize};

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SyncStatusInputs {
    pub online: bool,
    pub now_ms: f64,
    pub last_sync_started_ms: Option<f64>,
    pub last_sync_finished_ms: Option<f64>,
    pub last_sync_error: Option<String>,
    /// A failure the host caught around its own sync call (iOS); web passes None.
    pub host_sync_error: Option<String>,
    pub earliest_unsynced_ms: Option<f64>,
    /// The host started a manual sync and hasn't seen it finish yet.
    pub manual_sync_in_flight: bool,
    pub local_events: u64,
    pub server_events: u64,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SyncStatus {
    Offline,
    Error,
    Unsynced,
    Synced,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SyncSeverity {
    Neutral,
    Caution,
    Negative,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SyncStatusView {
    pub status: SyncStatus,
    pub label: String,
    pub severity: SyncSeverity,
    pub running: bool,
    pub error: Option<String>,
    pub error_title: String,
    pub offline_banner: Option<String>,
    pub sync_button_label: String,
    pub sync_button_enabled: bool,
    pub title: String,
    pub description: String,
    /// The host appends its localized time.
    pub last_sync_label: Option<String>,
    pub last_sync_finished_ms: Option<f64>,
    pub local_events_label: String,
    pub server_events_label: String,
    pub user_id_label: String,
    pub logged_out_label: String,
    pub device_id_label: String,
    pub version_label: String,
    pub local_events: u64,
    pub server_events: u64,
}

pub const UNSYNCED_THRESHOLD_MS: f64 = 5000.0;

#[bridgerton::bridge]
pub fn sync_status_view(inputs: SyncStatusInputs) -> SyncStatusView {
    let running = inputs.manual_sync_in_flight
        || inputs.last_sync_started_ms.is_some_and(|started| {
            inputs
                .last_sync_finished_ms
                .is_none_or(|finished| started > finished)
        });
    let error = inputs.last_sync_error.or(inputs.host_sync_error);
    let (status, label, severity) = if !inputs.online {
        (SyncStatus::Offline, "Offline", SyncSeverity::Neutral)
    } else if error.is_some() {
        (SyncStatus::Error, "Sync error", SyncSeverity::Negative)
    } else if inputs
        .earliest_unsynced_ms
        .is_some_and(|earliest| inputs.now_ms - earliest > UNSYNCED_THRESHOLD_MS)
    {
        (SyncStatus::Unsynced, "Unsynced", SyncSeverity::Caution)
    } else {
        (SyncStatus::Synced, "Synced", SyncSeverity::Neutral)
    };
    SyncStatusView {
        status,
        label: label.into(),
        severity,
        running,
        error,
        error_title: "Sync Error".into(),
        offline_banner: (!inputs.online)
            .then(|| "You're currently offline. Changes will sync when you reconnect.".into()),
        sync_button_label: if running { "Syncing..." } else { "Sync Now" }.into(),
        sync_button_enabled: inputs.online && !running,
        title: "Sync Status".into(),
        description: "Yap.town keeps your data synchronized across your devices.".into(),
        last_sync_label: inputs.last_sync_finished_ms.map(|_| "Last sync:".into()),
        last_sync_finished_ms: inputs.last_sync_finished_ms,
        local_events_label: "Local Events".into(),
        server_events_label: "Server Events".into(),
        user_id_label: "User ID".into(),
        logged_out_label: "Logged out".into(),
        device_id_label: "Device ID".into(),
        version_label: "Yap.Town version".into(),
        local_events: inputs.local_events,
        server_events: inputs.server_events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> SyncStatusInputs {
        SyncStatusInputs {
            online: true,
            now_ms: 10_000.0,
            last_sync_started_ms: None,
            last_sync_finished_ms: None,
            last_sync_error: None,
            host_sync_error: None,
            earliest_unsynced_ms: None,
            manual_sync_in_flight: false,
            local_events: 42,
            server_events: 40,
        }
    }

    #[test]
    fn precedence_table() {
        for online in [false, true] {
            for has_error in [false, true] {
                for stale in [false, true] {
                    for running in [false, true] {
                        let view = sync_status_view(SyncStatusInputs {
                            online,
                            last_sync_error: has_error.then(|| "failure".into()),
                            earliest_unsynced_ms: stale.then_some(0.0),
                            manual_sync_in_flight: running,
                            ..inputs()
                        });
                        let expected = if !online {
                            (SyncStatus::Offline, "Offline", SyncSeverity::Neutral)
                        } else if has_error {
                            (SyncStatus::Error, "Sync error", SyncSeverity::Negative)
                        } else if stale {
                            (SyncStatus::Unsynced, "Unsynced", SyncSeverity::Caution)
                        } else {
                            (SyncStatus::Synced, "Synced", SyncSeverity::Neutral)
                        };
                        assert_eq!((view.status, view.label.as_str(), view.severity), expected);
                        assert_eq!(view.running, running);
                        assert_eq!(view.sync_button_enabled, online && !running);
                        assert_eq!(
                            view.sync_button_label,
                            if running { "Syncing..." } else { "Sync Now" }
                        );
                        assert_eq!(view.offline_banner.is_some(), !online);
                        assert_eq!(view.error.is_some(), has_error);
                    }
                }
            }
        }
    }

    #[test]
    fn unsynced_threshold_is_strict() {
        for (age, expected) in [
            (-1.0, SyncStatus::Synced),
            (4999.0, SyncStatus::Synced),
            (5000.0, SyncStatus::Synced),
            (5001.0, SyncStatus::Unsynced),
        ] {
            assert_eq!(
                sync_status_view(SyncStatusInputs {
                    earliest_unsynced_ms: Some(10_000.0 - age),
                    ..inputs()
                })
                .status,
                expected
            );
        }
    }

    #[test]
    fn running_uses_store_timestamps_or_manual_call() {
        for (started, finished, expected) in [
            (None, None, false),
            (None, Some(0.0), false),
            (Some(0.0), None, true),
            (Some(2.0), Some(1.0), true),
            (Some(1.0), Some(1.0), false),
            (Some(1.0), Some(2.0), false),
        ] {
            for manual in [false, true] {
                let view = sync_status_view(SyncStatusInputs {
                    last_sync_started_ms: started,
                    last_sync_finished_ms: finished,
                    manual_sync_in_flight: manual,
                    ..inputs()
                });
                assert_eq!(view.running, expected || manual);
                assert_eq!(view.sync_button_enabled, !(expected || manual));
                assert_eq!(view.status, SyncStatus::Synced);
            }
        }
    }

    #[test]
    fn store_error_takes_precedence_over_host_fallback() {
        for (store, host, expected) in [
            (None, None, None),
            (None, Some("host"), Some("host")),
            (Some("store"), None, Some("store")),
            (Some("store"), Some("host"), Some("store")),
            (Some(""), Some("host"), Some("")),
        ] {
            let view = sync_status_view(SyncStatusInputs {
                last_sync_error: store.map(str::to_owned),
                host_sync_error: host.map(str::to_owned),
                ..inputs()
            });
            assert_eq!(view.error.as_deref(), expected);
            assert_eq!(
                view.status,
                if expected.is_some() {
                    SyncStatus::Error
                } else {
                    SyncStatus::Synced
                }
            );
        }
    }

    #[test]
    fn copy_and_store_facts() {
        let view = sync_status_view(inputs());
        assert_eq!(view.title, "Sync Status");
        assert_eq!(view.error_title, "Sync Error");
        assert_eq!(view.logged_out_label, "Logged out");
        assert_eq!(
            view.description,
            "Yap.town keeps your data synchronized across your devices."
        );
        assert_eq!(view.local_events_label, "Local Events");
        assert_eq!(view.server_events_label, "Server Events");
        assert_eq!(view.user_id_label, "User ID");
        assert_eq!(view.device_id_label, "Device ID");
        assert_eq!(view.version_label, "Yap.Town version");
        assert_eq!((view.local_events, view.server_events), (42, 40));
        assert_eq!(view.last_sync_label, None);
        assert_eq!(view.last_sync_finished_ms, None);
        let view = sync_status_view(SyncStatusInputs {
            online: false,
            last_sync_finished_ms: Some(0.0),
            ..inputs()
        });
        assert_eq!(view.last_sync_label.as_deref(), Some("Last sync:"));
        assert_eq!(view.last_sync_finished_ms, Some(0.0));
        assert_eq!(
            view.offline_banner.as_deref(),
            Some("You're currently offline. Changes will sync when you reconnect.")
        );
    }
}
