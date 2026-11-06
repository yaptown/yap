use language_utils::{Compensation, Language, language_pack::LanguagePack};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

// A process-wide Mutex (not thread_local): on the native MCP server, lookups
// happen on arbitrary tokio worker threads and must all see the packs that
// were registered at load time. On wasm there is one thread and the lock is
// free.
static REGISTRY: Mutex<Registry> = Mutex::new(Registry {
    packs: BTreeMap::new(),
    next_actor_index: BTreeMap::new(),
});

struct Registry {
    /// Weak refs to packs owned by `Weapon.language_pack`. The audio fetch
    /// path can't reach the `Weapon`, so it looks packs up here — but holding
    /// `Weak` (not `Arc`) keeps the `Weapon`'s map the sole owner: if a pack
    /// is dropped there, `upgrade()` fails and `lookup` falls through to TTS
    /// rather than serving a stale clip. `register` overwrites on reload.
    packs: BTreeMap<Language, Weak<LanguagePack>>,
    next_actor_index: BTreeMap<(Language, String), usize>,
}

pub fn register(language: Language, pack: &Arc<LanguagePack>) {
    REGISTRY
        .lock()
        .expect("human audio registry poisoned")
        .packs
        .insert(language, Arc::downgrade(pack));
}

pub struct HumanAudio {
    pub bytes: Vec<u8>,
    pub actor_name: String,
    pub compensation: Compensation,
}

/// Whether any voice actor has a recording for `(language, text)` — the
/// existence half of `lookup`, without advancing the round-robin counter or
/// cloning clip bytes.
pub fn has_clip(language: Language, text: &str) -> bool {
    let registry = REGISTRY.lock().expect("human audio registry poisoned");
    let Some(pack) = registry.packs.get(&language).and_then(Weak::upgrade) else {
        return false;
    };
    pack.sentences
        .human_audio
        .values()
        .any(|clips| clips.contains_key(text))
}

pub fn has_pronunciation_audio(language: Language, spoken_text: &str) -> bool {
    let registry = REGISTRY.lock().expect("human audio registry poisoned");
    registry
        .packs
        .get(&language)
        .and_then(Weak::upgrade)
        .is_some_and(|pack| pack.sentences.pronunciation_audio.contains_key(spoken_text))
}

pub fn pronunciation_audio(language: Language, spoken_text: &str) -> Option<Vec<u8>> {
    let registry = REGISTRY.lock().expect("human audio registry poisoned");
    registry
        .packs
        .get(&language)?
        .upgrade()?
        .sentences
        .pronunciation_audio
        .get(spoken_text)
        .map(|clip| clip.audio.bytes.to_vec())
}

/// Return a human-recorded clip for `(language, text)` if any voice actor
/// has a recording for that exact phrase. When multiple actors have a recording,
/// rotates round-robin across successive calls (actors sorted by name).
pub fn lookup(language: Language, text: &str) -> Option<HumanAudio> {
    let mut registry = REGISTRY.lock().expect("human audio registry poisoned");
    {
        let registry = &mut *registry;
        let pack = registry.packs.get(&language)?.upgrade()?;

        let mut actors: Vec<&language_utils::ArchivedVoiceActor> = pack
            .sentences
            .human_audio
            .iter()
            .filter_map(|(actor, clips)| clips.contains_key(text).then_some(actor))
            .collect();
        if actors.is_empty() {
            return None;
        }
        actors.sort_by(|a, b| a.name.cmp(&b.name));

        let key = (language, text.to_string());
        let counter = registry.next_actor_index.entry(key).or_insert(0);
        let index = *counter % actors.len();
        *counter = counter.wrapping_add(1);

        let actor = actors[index];
        let bytes = pack
            .sentences
            .human_audio
            .get(actor)?
            .get(text)?
            .bytes
            .to_vec();
        Some(HumanAudio {
            bytes,
            actor_name: actor.name.to_string(),
            compensation: rkyv::deserialize::<Compensation, rkyv::rancor::Error>(
                &actor.compensation,
            )
            .unwrap(),
        })
    }
}
