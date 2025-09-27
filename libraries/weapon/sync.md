# Syncing with Supabase

This guide covers everything needed to set up Supabase for syncing with Weapon, including the database schema, RPC functions, and security policies.

## Prerequisites

- A Supabase project
- Authentication enabled (Supabase Auth)
- Real-time subscriptions enabled (for instant sync)

## Database Schema

### Events Table

The core table that stores all events from all devices:

```sql
-- Create the events table
create table events (
  id bigserial primary key,
  user_id uuid references auth.users,
  stream_id text not null,
  device_id text not null,
  within_device_events_index integer not null,
  event jsonb not null,
  created_at timestamptz default now()
);

-- Create indexes for efficient querying
create index idx_events_sync on events(device_id, id);
create index idx_events_stream_sync on events(stream_id, device_id, id);

-- Create unique constraint to prevent duplicate events
alter table events 
add constraint events_unique_stream_device_index 
unique (user_id, stream_id, device_id, within_device_events_index);

-- Enable Row Level Security
alter table events enable row level security;

-- Create RLS policies
create policy "Users can see own events" on events
  for select using (auth.uid() = user_id);

create policy "Users can insert own events" on events
  for insert with check (auth.uid() = user_id);

create policy "Users can update own events" on events
  for update using (auth.uid() = user_id);
```

### Column Descriptions

- `id`: Auto-incrementing primary key
- `user_id`: References the authenticated user
- `stream_id`: Identifies which data stream (e.g., "reviews", "deck_selection")
- `device_id`: Unique identifier for the device that created the event
- `within_device_events_index`: Sequential index for events from a specific device
- `event`: The actual event data in JSONB format
- `created_at`: Timestamp when the event was stored in Supabase

## RPC Functions

### 1. sync_events Function

This is the main synchronization function that handles bidirectional sync:

```sql
create or replace function sync_events(sync_request jsonb)
returns jsonb as $$
declare
  result jsonb = '{}'::jsonb;
  stream_record record;
  device_record record;
  stream_result jsonb;
  requested_devices jsonb;
begin
  -- Loop through each stream in the request
  for stream_record in select * from jsonb_each(sync_request)
  loop
    stream_result := '{}'::jsonb;
    requested_devices := stream_record.value->'last_synced_ids';
    
    -- First, get events for explicitly requested devices
    for device_record in select * from jsonb_each_text(requested_devices)
    loop
      stream_result := stream_result || jsonb_build_object(
        device_record.key,
        (
          select coalesce(jsonb_agg(
            row_to_json(e) order by e.within_device_events_index
          ), '[]'::jsonb)
          from events e
          where e.device_id = device_record.key
          and e.stream_id = stream_record.key
          and e.within_device_events_index > device_record.value::integer
          and e.user_id = auth.uid()
        )
      );
    end loop;
    
    -- Then, get ALL events for devices not in the request
    for device_record in 
      select distinct device_id 
      from events 
      where stream_id = stream_record.key 
      and user_id = auth.uid()
      and device_id not in (select jsonb_object_keys(requested_devices))
    loop
      stream_result := stream_result || jsonb_build_object(
        device_record.device_id,
        (
          select coalesce(jsonb_agg(
            row_to_json(e) order by e.within_device_events_index
          ), '[]'::jsonb)
          from events e
          where e.device_id = device_record.device_id
          and e.stream_id = stream_record.key
          and e.user_id = auth.uid()
        )
      );
    end loop;
    
    -- Add this stream's results to the main result
    result := result || jsonb_build_object(stream_record.key, stream_result);
  end loop;
  
  return result;
end;
$$ language plpgsql
set search_path = public, auth, extensions, pg_catalog;

-- Grant execution rights
grant execute on function sync_events(jsonb) to authenticated;
```

#### Input Format:
```json
{
  "stream_id": {
    "last_synced_ids": {
      "device_id": last_within_device_events_index
    }
  }
}
```

#### Output Format:
```json
{
  "stream_id": {
    "device_id": [array_of_events]
  }
}
```

### 2. get_clock Function

Returns the current event count per device per stream:

```sql
create or replace function public.get_clock(p_user_id uuid)
returns jsonb
language sql
stable
set search_path = public
as $$
  with counts as (
    select
      stream_id,
      device_id,
      count(*)::int as event_count
    from public.events
    where user_id = p_user_id
    group by stream_id, device_id
  ),
  device_map as (
    select
      stream_id,
      jsonb_object_agg(device_id::text, to_jsonb(event_count)) as devices
    from counts
    group by stream_id
  )
  select coalesce(
    jsonb_object_agg(stream_id::text, devices),
    '{}'::jsonb
  )
  from device_map;
$$;

-- Grant execution rights
grant execute on function public.get_clock(uuid) to authenticated, service_role;
```

#### Output Format:
```json
{
  "stream_id": {
    "device_id": event_count
  }
}
```

## Real-time Subscriptions

Enable real-time for instant cross-device sync:

```sql
-- Enable real-time for the events table
alter publication supabase_realtime add table events;
```

## Usage in Weapon

### JavaScript/TypeScript Client Setup

```typescript
import { createClient } from '@supabase/supabase-js'

const supabase = createClient(SUPABASE_URL, SUPABASE_ANON_KEY)

// Subscribe to real-time events
const channel = supabase
  .channel(`events:${userId}`)
  .on(
    'postgres_changes',
    {
      event: 'INSERT',
      schema: 'public',
      table: 'events',
      filter: `user_id=eq.${userId}`,
    },
    (payload) => {
      const { device_id, stream_id, event } = payload.new
      // Handle incoming event from another device
      weapon.add_remote_event(device_id, stream_id, event)
    }
  )
  .subscribe()
```

### Rust/WASM Integration

The Weapon library handles the sync protocol internally. When using the Supabase feature:

```rust
// In your Cargo.toml
[dependencies]
weapon = { features = ["supabase"] }

// In your code
use weapon::supabase::sync_with_supabase;

// Sync with Supabase
async fn sync(access_token: &str) {
    // Get local event state
    let clock = weapon.get_clock();
    
    // Call Supabase RPC
    let response = supabase_client
        .rpc("sync_events", clock)
        .execute()
        .await?;
    
    // Apply remote events
    weapon.apply_remote_events(response);
    
    // Upload local events
    let local_events = weapon.get_unsent_events();
    for event in local_events {
        supabase_client
            .from("events")
            .insert(event)
            .execute()
            .await?;
    }
}
```

## Security Considerations

1. **Row Level Security (RLS)**: Always enabled to ensure users can only access their own events
2. **Unique Constraints**: Prevent duplicate events with the composite unique constraint
3. **Authentication Required**: All operations require a valid authenticated user
