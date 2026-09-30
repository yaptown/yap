use std::{
    cell::RefCell,
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

use opfs::{
    DirectoryEntry, DirectoryHandle as _, FileHandle as _, WritableFileStream as _,
    persistent::{self, DirectoryHandle, FileHandle},
};

use crate::data_model::{
    Clock, EventStoreWithListeners, IndexedEvent, ListenerKey, Listeners, SyncTarget, Timestamped,
};
use futures::{Stream, StreamExt};

const EVENTS_FILE_NAME: &str = "events.blob";
const EVENT_LOG_MAGIC: &[u8] = b"WEAPONLG";
const EVENT_LOG_VERSION: u32 = 1;
const EVENT_LOG_HEADER_LEN: usize = EVENT_LOG_MAGIC.len() + 4;

impl<L: Listeners<String>> EventStoreWithListeners<String, String, L> {
    /// Full OPFS sync wrapper: marks lifecycle, runs inner sync, records result.
    pub async fn sync_with_opfs(
        store: &RefCell<Self>,
        user_directory: &UserDirectory,
        stream_id_to_sync: Option<String>,
        modifier: Option<ListenerKey>,
    ) -> Result<(), persistent::Error> {
        store.borrow_mut().mark_sync_started(SyncTarget::Opfs);

        let result =
            Self::sync_with_opfs_inner(store, user_directory, stream_id_to_sync.clone(), modifier)
                .await;

        match &result {
            Ok(()) => store
                .borrow_mut()
                .mark_sync_finished(SyncTarget::Opfs, None),
            Err(e) => store
                .borrow_mut()
                .mark_sync_finished(SyncTarget::Opfs, Some(format!("{e:?}"))),
        }

        result
    }

    /// Performs OPFS load/save for either a specific stream or all streams, then
    /// refreshes and records the OPFS clock in the sync state.
    async fn sync_with_opfs_inner(
        store: &RefCell<Self>,
        user_directory: &UserDirectory,
        stream_id_to_sync: Option<String>,
        modifier: Option<ListenerKey>,
    ) -> Result<(), persistent::Error> {
        // 1) Load fresh events from OPFS into memory
        if let Some(stream_id) = stream_id_to_sync.clone() {
            Self::load_from_local_storage(store, user_directory, stream_id.clone(), modifier)
                .await?;
        } else {
            let mut streams = user_directory.event_stream_directories().await?;
            while let Some((stream_id, _)) = streams.next().await {
                Self::load_from_local_storage(store, user_directory, stream_id.clone(), modifier)
                    .await?;
            }
        }

        // 2) Save any in-memory events to OPFS
        if let Some(stream_id) = stream_id_to_sync.clone() {
            let _ = Self::save_to_local_storage(store, user_directory, stream_id.clone()).await?;
        } else {
            // Persist all streams present in the store
            let stream_ids: Vec<String> =
                store.borrow().iter().map(|(sid, _)| sid.clone()).collect();
            for stream_id in stream_ids {
                let _ =
                    Self::save_to_local_storage(store, user_directory, stream_id.clone()).await?;
            }
        }

        // 3) Refresh OPFS remote clock and record it in sync state
        let final_clock = get_opfs_clock(user_directory, stream_id_to_sync.as_deref()).await?;
        store
            .borrow_mut()
            .update_sync_clock(SyncTarget::Opfs, final_clock);

        Ok(())
    }
    /// Reload events from local storage and merge with current state
    pub async fn load_from_local_storage(
        store: &RefCell<Self>,
        user_directory: &UserDirectory,
        stream_id: String,
        modifier: Option<ListenerKey>,
    ) -> Result<(), persistent::Error> {
        let stream_directory = user_directory.get_stream_directory(&stream_id).await?;
        let event_log_file = stream_directory.get_event_log_file().await?;

        let mut counts: BTreeMap<String, usize> = {
            let store_ref = store.borrow();
            store_ref
                .get_raw(stream_id.clone())
                .map(|s| {
                    s.num_events_per_device()
                        .into_iter()
                        .map(|(device, count)| (device.clone(), count))
                        .collect()
                })
                .unwrap_or_default()
        };

        let stored_events = event_log_file
            .read_records(&counts)
            .await
            .inspect_err(|e| log::error!("Failed to reload from local storage: {e:?}"))?;

        let mut events_to_add: BTreeMap<String, Vec<Timestamped<crate::data_model::RawJson>>> =
            BTreeMap::new();

        for record in stored_events {
            let device_id = record.device_id;
            let event_index = record.within_device_events_index;
            let event = record.event;

            let entry = counts.entry(device_id.clone()).or_insert(0);
            let expected_index = *entry;
            if event_index < expected_index {
                log::error!(
                    "OPFS log backtrack detected for stream {stream_id} device {device_id}: expected index {expected_index}, found {event_index}",
                );
                continue;
            }

            if event_index > expected_index {
                log::error!(
                    "OPFS log gap detected for stream {stream_id} device {device_id}: expected index {expected_index}, found {event_index}",
                );
            }

            events_to_add
                .entry(device_id.clone())
                .or_default()
                .push(event);
            *entry = event_index + 1;
        }

        if events_to_add.is_empty() {
            return Ok(());
        }

        let mut store_mut = store.borrow_mut();
        for (device_id, events) in events_to_add {
            store_mut.add_device_events_jsons(stream_id.clone(), device_id, events, modifier);
        }

        Ok(())
    }

    /// Save events to local storage
    pub async fn save_to_local_storage(
        store: &RefCell<Self>,
        user_directory: &UserDirectory,
        stream_id: String,
    ) -> Result<usize, persistent::Error> {
        let _guard = weblocks::acquire(
            &format!("opfs-save-to-local-storage-{stream_id}"),
            weblocks::AcquireOptions::exclusive(),
        )
        .await?;
        let mut total_written: usize = 0;

        // Local desired counts per device for this stream
        let Some(device_events) = store.borrow().vector_clock().remove(&stream_id) else {
            log::warn!("Stream {stream_id} not found in store, skipping save");
            return Ok(0);
        };

        let stream_directory = user_directory.get_stream_directory(&stream_id).await?;
        let event_log_file = stream_directory.get_event_log_file().await?;
        let device_counts_on_disk = event_log_file.prepare_for_append().await?;

        let mut records_to_append: Vec<EventLogRecord> = Vec::new();

        for (device_id, _num_events_in_memory) in device_events {
            let device_events_on_disk = device_counts_on_disk.get(&device_id).copied().unwrap_or(0);

            let events_to_write: Vec<Timestamped<crate::data_model::RawJson>> = {
                let store_ref = store.borrow();
                let Some(stream) = store_ref.get_raw(stream_id.clone()) else {
                    log::error!(
                        "Stream {stream_id} not found in store, which should be impossible as we already checked for it"
                    );
                    continue;
                };
                stream.jsons(&device_id, device_events_on_disk)
            };

            for event in events_to_write {
                records_to_append.push(EventLogRecord {
                    device_id: device_id.clone(),
                    within_device_events_index: event.within_device_events_index(),
                    event,
                });
            }
        }

        if !records_to_append.is_empty() {
            event_log_file.append_records(&records_to_append).await?;
            total_written += records_to_append.len();
        }

        // Tell the other instances of the app (browser tabs) to reload this stream.
        if total_written > 0 {
            #[derive(serde::Serialize)]
            struct Written<'a> {
                r#type: &'static str,
                stream_id: &'a str,
            }
            log::info!("Broadcasting opfs-written message for stream: {stream_id}");
            if let Err(error) = bridgerton::platform::broadcast(
                "weapon-opfs-sync",
                &Written {
                    r#type: "opfs-written",
                    stream_id: &stream_id,
                },
            ) {
                log::error!("Failed to broadcast: {error}");
            }
        }

        Ok(total_written)
    }

    /// Import events from the logged-out user directory into the current user's directory.
    /// This is used when a user first logs in so their offline data is preserved.
    pub async fn import_logged_out_user_data(
        weapon_directory: DirectoryHandle,
        mut user_events_directory: DirectoryHandle,
        current_user_directory: &UserDirectory,
    ) -> Result<(), persistent::Error> {
        // One import at a time across tabs; a tab that waited finds the
        // logged-out directory already gone and does nothing.
        let _import = weblocks::acquire(
            "opfs-import-logged-out-user-data",
            weblocks::AcquireOptions::exclusive(),
        )
        .await?;
        // Attempt to get the logged-out directory. If it doesn't exist, there's nothing to do.
        let logged_out_directory = match user_events_directory
            .get_directory_handle_with_options(
                "user__logged-out-unknown-user",
                &opfs::GetDirectoryHandleOptions { create: false },
            )
            .await
        {
            Ok(dir) => UserDirectory {
                directory_handle: dir,
                user_id: "logged-out-unknown-user".into(),
            },
            Err(_) => return Ok(()),
        };

        // Only import into an account with no history on this device. The
        // import exists so creating an account doesn't lose the progress made
        // before it. Signing into an account that already lives here is
        // different: the anonymous session may have been someone else's (the
        // owner signed out to hand the device over), so it isn't assumed to be
        // theirs and is left where it is.
        // A directory marker claims the import without a potentially torn file write.
        let mut entries = logged_out_directory.directory_handle.entries().await?;
        let mut resuming = false;
        while let Some(entry) = entries.next().await {
            let (name, _) = entry?;
            if let Some(user_id) = name.strip_prefix("import__") {
                if user_id != current_user_directory.user_id {
                    return Ok(());
                }
                resuming = true;
            }
        }
        if !resuming {
            let mut existing_streams = current_user_directory.event_stream_directories().await?;
            if existing_streams.next().await.is_some() {
                return Ok(());
            }
            logged_out_directory
                .directory_handle
                .get_directory_handle_with_options(
                    &format!("import__{}", current_user_directory.user_id),
                    &opfs::GetDirectoryHandleOptions { create: true },
                )
                .await?;
        }

        // Retire the anonymous identity before deleting streams, so a restart logged
        // out cannot reuse their old device/index pairs for new events.
        let mut entries = weapon_directory.entries().await?;
        while let Some(entry) = entries.next().await {
            if entry?.0 == "device-id-logged-out" {
                weapon_directory
                    .clone()
                    .remove_entry("device-id-logged-out")
                    .await?;
                break;
            }
        }

        let streams: Vec<_> = logged_out_directory
            .event_stream_directories()
            .await?
            .collect()
            .await;
        for (stream_id, stream_dir) in streams {
            // Same lock as `save_to_local_storage`, so the append can't
            // interleave with a save from another tab.
            let _save = weblocks::acquire(
                &format!("opfs-save-to-local-storage-{stream_id}"),
                weblocks::AcquireOptions::exclusive(),
            )
            .await?;
            let target_log = current_user_directory
                .get_stream_directory(&stream_id)
                .await?
                .get_event_log_file()
                .await?;
            let copied_counts = target_log.prepare_for_append().await?;
            let events = stream_dir
                .get_event_log_file()
                .await?
                .read_records(&copied_counts)
                .await
                .inspect_err(|e| log::error!("Failed to reload from local storage: {e:?}"))?;
            target_log.append_records(&events).await?;
            // Keep the claim until every source stream is gone, including during cleanup.
            logged_out_directory
                .directory_handle
                .clone()
                .remove_entry_with_options(
                    &format!("stream__{stream_id}"),
                    &opfs::FileSystemRemoveOptions { recursive: true },
                )
                .await?;
        }

        // Remove the logged-out user directory itself now that everything is moved.
        let _ = user_events_directory
            .remove_entry_with_options(
                "user__logged-out-unknown-user",
                &opfs::FileSystemRemoveOptions { recursive: true },
            )
            .await
            .inspect_err(|e| log::error!("Failed to remove logged-out user directory: {e:?}"));

        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct UserDirectory {
    directory_handle: DirectoryHandle,
    user_id: String,
}

#[derive(Debug, Clone)]
pub struct StreamDirectory {
    directory_handle: DirectoryHandle,
}

#[derive(Debug, Clone)]
pub struct EventLogFile {
    file_handle: FileHandle,
}

#[derive(Debug, Clone)]
pub struct EventLogRecord {
    pub device_id: String,
    pub within_device_events_index: usize,
    pub event: Timestamped<crate::data_model::RawJson>,
}

impl UserDirectory {
    pub async fn new(parent: &DirectoryHandle, user_id: &str) -> Result<Self, persistent::Error> {
        Ok(Self {
            user_id: user_id.into(),
            directory_handle: parent
                .get_directory_handle_with_options(
                    &format!("user__{user_id}"),
                    &opfs::GetDirectoryHandleOptions { create: true },
                )
                .await?,
        })
    }

    #[allow(dead_code)]
    async fn event_stream_directories(
        &self,
    ) -> Result<impl Stream<Item = (String, StreamDirectory)>, persistent::Error> {
        Ok(self.directory_handle.entries().await?.filter_map(|entry| {
            let (directory_name, stream_directory) = match entry {
                Ok(res) => res,
                Err(e) => {
                    log::error!("Failed to get stream directory: {e:?}");
                    return futures::future::ready(None);
                }
            };
            let Some(stream_id) = directory_name.strip_prefix("stream__") else {
                return futures::future::ready(None);
            };
            let DirectoryEntry::Directory(stream_directory) = stream_directory else {
                return futures::future::ready(None);
            };
            let stream_directory = StreamDirectory {
                directory_handle: stream_directory,
            };
            futures::future::ready(Some((stream_id.to_string(), stream_directory)))
        }))
    }

    async fn get_stream_directory(
        &self,
        stream_id: &str,
    ) -> Result<StreamDirectory, persistent::Error> {
        Ok(StreamDirectory {
            directory_handle: self
                .directory_handle
                .get_directory_handle_with_options(
                    &format!("stream__{stream_id}"),
                    &opfs::GetDirectoryHandleOptions { create: true },
                )
                .await?,
        })
    }
}

impl StreamDirectory {
    async fn get_event_log_file(&self) -> Result<EventLogFile, persistent::Error> {
        Ok(EventLogFile {
            file_handle: self
                .directory_handle
                .get_file_handle_with_options(
                    EVENTS_FILE_NAME,
                    &opfs::GetFileHandleOptions { create: true },
                )
                .await?,
        })
    }
}

impl EventLogFile {
    /// Called under the stream's save lock before counting or appending events.
    async fn prepare_for_append(&self) -> Result<BTreeMap<String, usize>, persistent::Error> {
        let bytes = self.file_handle.read().await?;
        let valid_len = if bytes.len() < EVENT_LOG_HEADER_LEN {
            0
        } else if bytes.starts_with(&event_log_header_bytes()) {
            event_log_records_iter(&bytes)
                .last()
                .map_or(EVENT_LOG_HEADER_LEN, |record| record.end_offset)
        } else {
            // An unknown format isn't a torn tail; leave it alone.
            bytes.len()
        };
        if valid_len < bytes.len() {
            log::warn!(
                "Truncating torn OPFS log from {} to {valid_len} bytes",
                bytes.len()
            );
            let mut file = self.file_handle.clone();
            let mut writable = file
                .create_writable_with_options(&opfs::CreateWritableOptions {
                    keep_existing_data: true,
                })
                .await?;
            writable.truncate(valid_len).await?;
            writable.close().await?;
        }
        Ok(parse_device_counts(&bytes[..valid_len]))
    }

    async fn read_records(
        &self,
        skip_counts: &BTreeMap<String, usize>,
    ) -> Result<Vec<EventLogRecord>, persistent::Error> {
        let bytes = self.file_handle.read().await?;
        Ok(parse_event_log_records_with_skip(&bytes, skip_counts))
    }

    async fn append_records(&self, records: &[EventLogRecord]) -> Result<(), persistent::Error> {
        if records.is_empty() {
            return Ok(());
        }

        let existing_size = self.file_handle.size().await?;
        let mut file_handle = self.file_handle.clone();
        let mut writable = file_handle
            .create_writable_with_options(&opfs::CreateWritableOptions {
                keep_existing_data: true,
            })
            .await?;

        if existing_size < EVENT_LOG_HEADER_LEN {
            writable.truncate(0).await?;
            let header = event_log_header_bytes();
            writable.write_at_cursor_pos(&header).await?;
            writable.seek(EVENT_LOG_HEADER_LEN).await?;
        } else {
            writable.seek(existing_size).await?;
        }

        for record in records {
            if let Some(bytes) = encode_event_log_record(record) {
                writable.write_at_cursor_pos(&bytes).await?;
            }
        }

        writable.close().await?;

        Ok(())
    }

    async fn device_counts(&self) -> Result<BTreeMap<String, usize>, persistent::Error> {
        let bytes = self.file_handle.read().await?;
        Ok(parse_device_counts(&bytes))
    }
}

/// Build a clock of on-disk counts per stream/device in OPFS.
async fn get_opfs_clock(
    user_directory: &UserDirectory,
    only_stream: Option<&str>,
) -> Result<Clock<String, String>, persistent::Error> {
    let mut clock: Clock<String, String> = BTreeMap::new();

    if let Some(stream_id) = only_stream {
        let stream_dir = user_directory.get_stream_directory(stream_id).await?;
        let device_counts = stream_dir
            .get_event_log_file()
            .await?
            .device_counts()
            .await?;
        clock.insert(stream_id.to_string(), device_counts);
        return Ok(clock);
    }

    let mut streams = user_directory.event_stream_directories().await?;
    while let Some((stream_id, stream_dir)) = streams.next().await {
        let device_counts = stream_dir
            .get_event_log_file()
            .await?
            .device_counts()
            .await?;
        clock.insert(stream_id, device_counts);
    }

    Ok(clock)
}

fn event_log_header_bytes() -> Vec<u8> {
    let mut header = Vec::with_capacity(EVENT_LOG_HEADER_LEN);
    header.extend_from_slice(EVENT_LOG_MAGIC);
    header.extend_from_slice(&EVENT_LOG_VERSION.to_le_bytes());
    header
}

fn encode_event_log_record(record: &EventLogRecord) -> Option<Vec<u8>> {
    let payload = match serde_json::to_vec(&record.event) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!(
                "Failed to serialize event for device {}: {e:?}",
                record.device_id
            );
            return None;
        }
    };

    let device_id_bytes = record.device_id.as_bytes();
    let device_id_len: u32 = match device_id_bytes.len().try_into() {
        Ok(len) => len,
        Err(_) => {
            log::error!(
                "Device ID too long to encode for device {} ({} bytes)",
                record.device_id,
                device_id_bytes.len()
            );
            return None;
        }
    };

    let payload_len: u32 = match payload.len().try_into() {
        Ok(len) => len,
        Err(_) => {
            log::error!(
                "Event payload too large to encode for device {} ({} bytes)",
                record.device_id,
                payload.len()
            );
            return None;
        }
    };

    let body_len = std::mem::size_of::<u64>()
        + std::mem::size_of::<u32>()
        + device_id_bytes.len()
        + std::mem::size_of::<u32>()
        + payload.len();

    let record_len: u32 = match body_len.try_into() {
        Ok(len) => len,
        Err(_) => {
            log::error!(
                "Record too large to encode for device {} ({} bytes)",
                record.device_id,
                body_len
            );
            return None;
        }
    };

    let mut buffer = Vec::with_capacity(std::mem::size_of::<u32>() + body_len);
    buffer.extend_from_slice(&record_len.to_le_bytes());
    buffer.extend_from_slice(&(record.within_device_events_index as u64).to_le_bytes());
    buffer.extend_from_slice(&device_id_len.to_le_bytes());
    buffer.extend_from_slice(device_id_bytes);
    buffer.extend_from_slice(&payload_len.to_le_bytes());
    buffer.extend_from_slice(&payload);

    Some(buffer)
}

struct RawEventLogRecord<'a> {
    end_offset: usize,
    within_device_events_index: u64,
    device_id_bytes: &'a [u8],
    payload_bytes: &'a [u8],
}

fn event_log_records_iter(bytes: &[u8]) -> impl Iterator<Item = RawEventLogRecord<'_>> {
    struct EventLogIterator<'a> {
        bytes: &'a [u8],
        offset: usize,
        validated: bool,
    }

    impl<'a> Iterator for EventLogIterator<'a> {
        type Item = RawEventLogRecord<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            if !self.validated {
                if self.bytes.is_empty() {
                    return None;
                }
                if self.bytes.len() < EVENT_LOG_HEADER_LEN {
                    log::warn!("Event log header too small ({} bytes)", self.bytes.len());
                    return None;
                }
                if !self.bytes.starts_with(EVENT_LOG_MAGIC) {
                    log::warn!("Event log magic bytes did not match");
                    return None;
                }
                let version_offset = EVENT_LOG_MAGIC.len();
                let version = u32::from_le_bytes(
                    self.bytes[version_offset..version_offset + 4]
                        .try_into()
                        .unwrap(),
                );
                if version != EVENT_LOG_VERSION {
                    log::warn!("Unsupported event log version {version}");
                    return None;
                }
                self.offset = EVENT_LOG_HEADER_LEN;
                self.validated = true;
            }

            if self.offset + std::mem::size_of::<u32>() > self.bytes.len() {
                return None;
            }

            let record_len = u32::from_le_bytes(
                self.bytes[self.offset..self.offset + std::mem::size_of::<u32>()]
                    .try_into()
                    .unwrap(),
            ) as usize;
            self.offset += std::mem::size_of::<u32>();

            if record_len > self.bytes.len() - self.offset {
                log::warn!(
                    "Event log record length {} exceeds remaining bytes {}",
                    record_len,
                    self.bytes.len() - self.offset
                );
                return None;
            }

            let record_end = self.offset + record_len;
            if record_len < std::mem::size_of::<u64>() + 2 * std::mem::size_of::<u32>() {
                log::warn!("Event log record too small ({record_len} bytes)");
                self.offset = record_end;
                return self.next();
            }

            let within_device = u64::from_le_bytes(
                self.bytes[self.offset..self.offset + std::mem::size_of::<u64>()]
                    .try_into()
                    .unwrap(),
            );
            self.offset += std::mem::size_of::<u64>();

            let device_len = u32::from_le_bytes(
                self.bytes[self.offset..self.offset + std::mem::size_of::<u32>()]
                    .try_into()
                    .unwrap(),
            ) as usize;
            self.offset += std::mem::size_of::<u32>();

            if device_len > record_end - self.offset - std::mem::size_of::<u32>() {
                log::warn!("Device ID length {device_len} exceeds record bounds");
                self.offset = record_end;
                return self.next();
            }
            let device_id_bytes = &self.bytes[self.offset..self.offset + device_len];
            self.offset += device_len;

            let payload_len = u32::from_le_bytes(
                self.bytes[self.offset..self.offset + std::mem::size_of::<u32>()]
                    .try_into()
                    .unwrap(),
            ) as usize;
            self.offset += std::mem::size_of::<u32>();

            if payload_len > record_end - self.offset {
                log::warn!("Payload length {payload_len} exceeds record bounds");
                self.offset = record_end;
                return self.next();
            }
            let payload_bytes = &self.bytes[self.offset..self.offset + payload_len];
            self.offset = record_end;

            Some(RawEventLogRecord {
                end_offset: record_end,
                within_device_events_index: within_device,
                device_id_bytes,
                payload_bytes,
            })
        }
    }

    EventLogIterator {
        bytes,
        offset: 0,
        validated: false,
    }
}

pub fn parse_event_log_records(bytes: &[u8]) -> Vec<EventLogRecord> {
    parse_event_log_records_with_skip(bytes, &BTreeMap::new())
}

fn parse_event_log_records_with_skip(
    bytes: &[u8],
    skip_counts: &BTreeMap<String, usize>,
) -> Vec<EventLogRecord> {
    let mut records = Vec::new();

    for raw_record in event_log_records_iter(bytes) {
        let within_device_index = match usize::try_from(raw_record.within_device_events_index) {
            Ok(index) => index,
            Err(_) => {
                log::warn!(
                    "within_device_events_index {} overflowed usize",
                    raw_record.within_device_events_index
                );
                continue;
            }
        };

        let device_id = match String::from_utf8(raw_record.device_id_bytes.to_vec()) {
            Ok(id) => id,
            Err(e) => {
                log::warn!("Device ID was not valid UTF-8: {e:?}");
                continue;
            }
        };

        // Skip events we already have
        let skip_count = skip_counts.get(&device_id).copied().unwrap_or(0);
        if within_device_index < skip_count {
            // This event is already known, skip parsing the payload
            continue;
        }

        match serde_json::from_slice::<Timestamped<crate::data_model::RawJson>>(
            raw_record.payload_bytes,
        ) {
            Ok(event) => records.push(EventLogRecord {
                device_id,
                within_device_events_index: within_device_index,
                event,
            }),
            Err(e) => {
                log::warn!("Failed to deserialize event payload: {e:?}");
            }
        }
    }

    records
}

pub fn parse_device_counts(bytes: &[u8]) -> BTreeMap<String, usize> {
    let start_time = web_time::Instant::now();
    let bytes_len = bytes.len();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut record_count = 0usize;

    for raw_record in event_log_records_iter(bytes) {
        record_count += 1;

        let within_device_index = match usize::try_from(raw_record.within_device_events_index) {
            Ok(index) => index,
            Err(_) => {
                log::warn!(
                    "within_device_events_index {} overflowed usize",
                    raw_record.within_device_events_index
                );
                continue;
            }
        };

        let device_id = match String::from_utf8(raw_record.device_id_bytes.to_vec()) {
            Ok(id) => id,
            Err(e) => {
                log::warn!("Device ID was not valid UTF-8: {e:?}");
                continue;
            }
        };

        let entry = counts.entry(device_id.clone()).or_insert(0);
        let expected = *entry;
        // Logs written before `jsons` selected by index can repeat an index after a backdated
        // event; skip the repeat exactly as `load_from_local_storage` does.
        if within_device_index < expected {
            log::error!(
                "OPFS duplicate index for device {device_id}: expected {expected}, found {within_device_index}"
            );
            continue;
        }
        if within_device_index != expected {
            log::error!(
                "OPFS index gap for device {device_id}: expected {expected}, found {within_device_index}"
            );
        }
        *entry = within_device_index + 1;
    }

    let duration = start_time.elapsed().as_secs_f64() * 1000.0;
    if duration > 0.0 {
        let throughput = bytes_len as f64 / duration / 1024.0; // KB/ms
        log::info!(
            "parse_device_counts: processed {} records from {} bytes in {:.2}ms ({:.2} KB/ms, {:.2} records/ms)",
            record_count,
            bytes_len,
            duration,
            throughput,
            record_count as f64 / duration
        );
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_payload_survives_opfs_without_normalization() {
        let json = r#"{ "User": {"version":"V99", "number":1e2, "text":"a"} }"#;
        let record = EventLogRecord {
            device_id: "future-device".into(),
            within_device_events_index: 0,
            event: Timestamped {
                timestamp: chrono::DateTime::from_timestamp(0, 0).unwrap(),
                timezone: chrono::FixedOffset::east_opt(0).unwrap(),
                within_device_events_index: 0,
                event: serde_json::from_str(json).unwrap(),
            },
        };
        let mut bytes = event_log_header_bytes();
        bytes.extend(encode_event_log_record(&record).unwrap());
        let records = parse_event_log_records(&bytes);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].event.event.get(), json);
        let old: crate::data_model::EventType<u32> =
            serde_json::from_str(records[0].event.event.get()).unwrap();
        assert!(matches!(old, crate::data_model::EventType::Unrecognized(_)));
        assert_eq!(serde_json::to_string(&old).unwrap(), json);
    }

    fn record(index: usize) -> EventLogRecord {
        EventLogRecord {
            device_id: "a".into(),
            within_device_events_index: index,
            event: Timestamped {
                timestamp: chrono::DateTime::from_timestamp(0, 0).unwrap(),
                timezone: chrono::FixedOffset::east_opt(0).unwrap(),
                within_device_events_index: index,
                event: crate::data_model::RawJson::from_serializable(&()).unwrap(),
            },
        }
    }

    #[test]
    fn malformed_lengths_do_not_panic() {
        let encoded = encode_event_log_record(&record(0)).unwrap();
        for (offset, length) in [
            (0, u32::MAX),
            (12, (encoded.len() - 16) as u32),
            (17, u32::MAX),
        ] {
            let mut damaged = encoded.clone();
            damaged[offset..offset + 4].copy_from_slice(&length.to_le_bytes());
            let mut bytes = event_log_header_bytes();
            bytes.extend(damaged);
            assert!(parse_event_log_records(&bytes).is_empty());
        }
    }

    #[test]
    fn device_counts_allow_gaps() {
        let mut bytes = event_log_header_bytes();
        for index in [0, 2, 3] {
            bytes.extend(encode_event_log_record(&record(index)).unwrap());
        }
        assert_eq!(
            parse_device_counts(&bytes),
            BTreeMap::from([("a".into(), 4)])
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn torn_tail_is_repaired_before_append() {
        let temp = tempfile::tempdir().unwrap();
        let stream = StreamDirectory {
            directory_handle: DirectoryHandle::from(temp.path().to_path_buf()),
        };
        // Cover a torn length prefix as well as every position in the record body.
        let encoded = encode_event_log_record(&record(1)).unwrap();
        for tail_len in 1..encoded.len() {
            let mut bytes = event_log_header_bytes();
            bytes.extend(encode_event_log_record(&record(0)).unwrap());
            bytes.extend(&encoded[..tail_len]);
            std::fs::write(temp.path().join(EVENTS_FILE_NAME), bytes).unwrap();
            let log = stream.get_event_log_file().await.unwrap();
            assert_eq!(
                log.prepare_for_append().await.unwrap(),
                BTreeMap::from([("a".into(), 1)])
            );
            log.append_records(&[record(1)]).await.unwrap();
            let reloaded = stream
                .get_event_log_file()
                .await
                .unwrap()
                .read_records(&BTreeMap::new())
                .await
                .unwrap();
            assert_eq!(
                reloaded
                    .iter()
                    .map(|r| r.within_device_events_index)
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn interrupted_import_resumes_only_for_its_owner() {
        use crate::data_model::EventStore;

        let temp = tempfile::tempdir().unwrap();
        let root = DirectoryHandle::from(temp.path().to_path_buf());
        let source = UserDirectory::new(&root, "logged-out-unknown-user")
            .await
            .unwrap();
        let owner = UserDirectory::new(&root, "owner").await.unwrap();
        let other = UserDirectory::new(&root, "other").await.unwrap();
        for stream_id in ["one", "two"] {
            source
                .get_stream_directory(stream_id)
                .await
                .unwrap()
                .get_event_log_file()
                .await
                .unwrap()
                .append_records(&[record(0), record(1)])
                .await
                .unwrap();
        }
        source
            .directory_handle
            .get_directory_handle_with_options(
                "import__owner",
                &opfs::GetDirectoryHandleOptions { create: true },
            )
            .await
            .unwrap();
        // A failed copy must already have retired the old anonymous identity.
        std::fs::write(temp.path().join("device-id-logged-out"), "a").unwrap();
        let target = owner.get_stream_directory("one").await.unwrap();
        let target_path = temp.path().join("user__owner/stream__one/events.blob");
        std::fs::create_dir(&target_path).unwrap();
        assert!(
            EventStore::<String, String>::import_logged_out_user_data(
                root.clone(),
                root.clone(),
                &owner,
            )
            .await
            .is_err()
        );
        assert!(!temp.path().join("device-id-logged-out").exists());
        std::fs::remove_dir(&target_path).unwrap();

        // Simulate a kill after one complete record and part of the next were copied.
        let mut bytes = event_log_header_bytes();
        bytes.extend(encode_event_log_record(&record(0)).unwrap());
        bytes.extend(&encode_event_log_record(&record(1)).unwrap()[..7]);
        let mut file = target.get_event_log_file().await.unwrap().file_handle;
        let mut writer = file
            .create_writable_with_options(&opfs::CreateWritableOptions {
                keep_existing_data: false,
            })
            .await
            .unwrap();
        writer.write_at_cursor_pos(&bytes).await.unwrap();
        writer.close().await.unwrap();

        EventStore::<String, String>::import_logged_out_user_data(
            root.clone(),
            root.clone(),
            &other,
        )
        .await
        .unwrap();
        assert!(
            other
                .event_stream_directories()
                .await
                .unwrap()
                .next()
                .await
                .is_none()
        );
        EventStore::<String, String>::import_logged_out_user_data(
            root.clone(),
            root.clone(),
            &owner,
        )
        .await
        .unwrap();
        // Retrying a completed import is also harmless.
        EventStore::<String, String>::import_logged_out_user_data(root.clone(), root, &owner)
            .await
            .unwrap();
        for stream_id in ["one", "two"] {
            let records = owner
                .get_stream_directory(stream_id)
                .await
                .unwrap()
                .get_event_log_file()
                .await
                .unwrap()
                .read_records(&BTreeMap::new())
                .await
                .unwrap();
            assert_eq!(
                records
                    .iter()
                    .map(|r| r.within_device_events_index)
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
        }
        assert!(!temp.path().join("user__logged-out-unknown-user").exists());
    }

    #[test]
    fn device_counts_skip_repeated_index_from_positional_export_bug() {
        let mut bytes = event_log_header_bytes();
        for index in [0, 0, 1] {
            bytes.extend(encode_event_log_record(&record(index)).unwrap());
        }
        assert_eq!(
            parse_device_counts(&bytes),
            BTreeMap::from([("a".into(), 2)])
        );
    }
}
