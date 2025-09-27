# Weapon - A Local-First Event Sourcing & Cross-Device Sync Engine

Weapon is a Rust library that enables local-first applications with cross-device synchronization. It implements event sourcing patterns with support for offline usage, real-time sync, and multi-device collaboration. It is designed primarily to be compiled to WASM and used with React applications. That said, isn't react-specific in any way and would probably work in a Dioxus app (or similar) as well. I made it for [Yap.Town](https://yap.town), a language learning app I work on sometimes.

## Core Concept

Weapon uses an event-sourcing architecture where:
- User actions generate "events" with unique IDs and timestamps
- Application state is derived from "replaying" the chronological sequence of events
- Events are stored locally first (using OPFS in browsers)
- Synchronization simply merges events from all devices

## Key Features

1. **Local-First Architecture**
   - Users can use your app without logging in
   - All data is stored locally using browser storage (OPFS)
   - Works fully offline with zero network dependency (great for PWAs!)

2. **Seamless Authentication Transition**
   - When users log in, their local data automatically syncs to the cloud
   - Logged-out user data gets imported into their account
   - No data loss during authentication state changes

3. **Real-Time Cross-Device Sync**
   - Changes sync instantly across all devices
   - Any postgres server can be used for cloud persistence 
   - With supabase, supports real-time subscriptions for lower-latency sync
   - Supports both push (real-time) and pull (periodic) synchronization

4. **Event Sourcing**
   - Complete audit trail of all changes
   - Time-travel debugging capabilities
   - Conflict-free merging of concurrent edits
   - Ability to replay events to rebuild state

Event sourcing enables fixing bugs retroactively. When you fix a bug in your state computation logic, users will replay all historical events through the corrected code to regenerate a bug-free state. This effectively "rewrites history" as if the bug never existed.

For example, in a budgeting app, if you discover floating-point rounding errors and switch to fixed-precision arithmetic, replaying all events will recalculate every transaction with the correct precision, fixing all historical calculation errors automatically.

## Architecture

### Event Model

Events are the atomic units of change in Weapon. Each event:
- Has a unique timestamp and device-specific index
- Is immutable once created
- Can be versioned for backward compatibility
- Is serializable to JSON for storage/transmission

```rust
pub trait Event: Sized + PartialOrd + Ord + Clone + Eq {
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error>;
    fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error>;
}
```

### State Management

Application state is computed by applying events in chronological order:

```rust
pub trait PartialAppState: Sized {
    type Event: Event;
    type Partial: Sized;
    
    // Process events incrementally
    fn process_event(partial: Self::Partial, event: &Timestamped<Self::Event>) -> Self::Partial;
    
    // Compute derived state once after all events
    fn finalize(partial: Self::Partial) -> Self;
}
```

### Storage Layers

Weapon supports multiple storage backends:
- **OPFS** (Origin Private File System) - Browser storage
- **Supabase** - Cloud persistence and sync
- **Memory** - For testing and temporary state

## Real-World Usage Example

Here's how Weapon is used in Yap.Town for managing language learning state:

### 1. Define Your Events

```rust
#[derive(Clone, Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum DeckEvent {
    CardReviewed { 
        card_id: String, 
        rating: u8 
    },
    CardAdded { 
        card_id: String, 
        content: CardContent 
    },
    SettingChanged { 
        key: String, 
        value: serde_json::Value 
    },
}

// Version your events for future compatibility
pub enum VersionedDeckEvent {
    V1(DeckEvent),
}

impl Event for DeckEvent {
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        let versioned = VersionedDeckEvent::V1(self.clone());
        serde_json::to_value(versioned)
    }
    
    fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        let versioned: VersionedDeckEvent = serde_json::from_value(json.clone())?;
        Ok(match versioned {
            VersionedDeckEvent::V1(event) => event,
        })
    }
}
```

### 2. Define Your State

```rust
pub struct DeckState {
    cards: HashMap<String, Card>,
    settings: HashMap<String, serde_json::Value>,
    // Derived state (computed in finalize)
    due_cards: Vec<String>,
    statistics: DeckStatistics,
}

impl PartialAppState for DeckState {
    type Event = DeckEvent;
    type Partial = PartialDeckState;
    
    fn process_event(mut partial: Self::Partial, event: &Timestamped<DeckEvent>) -> Self::Partial {
        match &event.event {
            DeckEvent::CardReviewed { card_id, rating } => {
                // Update card with review
                partial.update_card_review(card_id, *rating, event.timestamp);
            }
            DeckEvent::CardAdded { card_id, content } => {
                partial.cards.insert(card_id.clone(), Card::new(content.clone()));
            }
            DeckEvent::SettingChanged { key, value } => {
                partial.settings.insert(key.clone(), value.clone());
            }
        }
        partial
    }
    
    fn finalize(partial: Self::Partial) -> Self {
        // Compute derived state like due cards and statistics
        let due_cards = partial.compute_due_cards();
        let statistics = partial.compute_statistics();
        
        DeckState {
            cards: partial.cards,
            settings: partial.settings,
            due_cards,
            statistics,
        }
    }
}
```

### 3. Initialize Weapon (Rust/WASM)

```rust
use weapon::data_model::{EventStore, EventType};

pub struct WeaponInstance {
    store: RefCell<EventStore<String, String>>,
    device_id: String,
    user_id: Option<String>,
}

impl WeaponInstance {
    pub async fn new(user_id: Option<String>) -> Result<Self, Error> {
        // Get or create device ID
        let device_id = get_or_create_device_id(&user_id).await?;
        
        // Initialize event store
        let mut store = EventStore::default();
        
        // Register sync callback for when events change
        store.register_listener(move |listener_id, stream_id| {
            // Trigger sync with cloud
            sync_with_supabase(stream_id).await;
        });
        
        Ok(Self {
            store: RefCell::new(store),
            device_id,
            user_id,
        })
    }
    
    pub fn add_event(&self, stream_id: String, event: DeckEvent) {
        let mut store = self.store.borrow_mut();
        let stream = store.get_or_insert_default::<EventType<DeckEvent>>(
            stream_id, 
            None
        );
        stream.add_event(event);
    }
}
```

### 4. React Integration

```typescript
import { Weapon } from 'weapon-wasm';

function WeaponProvider({ userId, children }) {
    const [weapon, setWeapon] = useState(null);
    
    useEffect(() => {
        async function init() {
            // Initialize Weapon with sync callback
            const weaponInstance = await new Weapon(
                userId,
                async (listenerId, streamId) => {
                    // Sync when events change
                    await weaponInstance.sync(streamId, accessToken);
                }
            );
            setWeapon(weaponInstance);
        }
        init();
    }, [userId]);
    
    // Subscribe to stream changes
    useEffect(() => {
        if (!weapon) return;
        
        const unsubscribe = weapon.subscribe_to_stream('deck_events', () => {
            // React to changes
            setDeckState(weapon.get_deck_state());
        });
        
        return () => weapon.unsubscribe(unsubscribe);
    }, [weapon]);
    
    return (
        <WeaponContext.Provider value={weapon}>
            {children}
        </WeaponContext.Provider>
    );
}

// Usage in components
function DeckComponent() {
    const weapon = useWeapon();
    
    const handleCardReview = (cardId, rating) => {
        // Add event - automatically syncs
        weapon.add_deck_event({
            type: 'CardReviewed',
            card_id: cardId,
            rating: rating
        });
    };
    
    return <div>...</div>;
}
```

### 5. Cross-Tab Synchronization

Weapon supports synchronization between browser tabs using BroadcastChannel:

```javascript
// Automatically handled by Weapon - tabs notify each other of changes
const channel = new BroadcastChannel('weapon-opfs-sync');

channel.onmessage = (event) => {
    if (event.data?.type === 'opfs-written') {
        // Reload affected stream from local storage
        weapon.load_from_local_storage(event.data.stream_id);
    }
};
```

## Sync Strategy

Weapon implements a simple synchronization strategy:

1. **Event Generation**: User actions create timestamped events
2. **Device Identification**: Each device gets a unique ID
3. **Local Storage**: Events are immediately persisted locally
4. **Cloud Sync**: Events sync to cloud when online
5. **Conflict Resolution**: Events merge chronologically by timestamp
6. **Real-time Updates**: Changes propagate instantly via WebSockets

The sync protocol ensures:
- No data loss during offline periods
- Eventual consistency across all devices
- Minimal sync overhead (only new events transfer)
- Automatic conflict resolution via timestamps

## Benefits

- **Instant UI Response**: No network latency for user actions
- **Offline Capable**: Full functionality without internet
- **Cross-Device Sync**: Seamless experience across devices
- **Data Portability**: Export/import entire event history
- **Time Travel**: Replay events to any point in time
- **Audit Trail**: Complete history of all changes
- **Conflict-Free**: Automatic merging of concurrent edits

## Status

Weapon is currently in active development and used in production by Yap.Town. While functional, the API may evolve significantly.
