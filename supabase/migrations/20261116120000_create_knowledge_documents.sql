-- Plain-text documents for the knowledge graph (yap-mcp's `knowledge` event
-- stream). Documents are mutable content, so they live here rather than in
-- the append-only events table; the graph's events link concepts to them by
-- (user_id, id).
create table knowledge_documents (
  user_id uuid not null references auth.users on delete cascade,
  id text not null check (id ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
  title text not null,
  body text not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (user_id, id)
);

alter table knowledge_documents enable row level security;

create policy "Users manage own knowledge documents" on knowledge_documents
  for all using (auth.uid() = user_id) with check (auth.uid() = user_id);
