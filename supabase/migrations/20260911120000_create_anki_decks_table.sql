-- One row per exported Anki deck minted by the backend (POST /anki/deck).
--
-- The deck's media URLs carry a signed token naming this id, so a deck that
-- turns up somewhere it shouldn't can be traced back to its mint and later
-- revoked individually. `revoked_at` is reserved for that: nothing reads it
-- yet (the clip domain is still a plain public bucket), but recording the
-- mint from day one is what makes revocation possible without touching
-- decks already in the wild.
--
-- `user_id` is null for decks minted without signing in — the page is public.
-- `options` is the generation request as the client sent it, stored opaque
-- so the deck feature can grow options without a migration each time.
CREATE TABLE public.anki_decks (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES auth.users(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  options JSONB NOT NULL DEFAULT '{}'::jsonb,
  revoked_at TIMESTAMPTZ
);

CREATE INDEX idx_anki_decks_user ON public.anki_decks(user_id);
CREATE INDEX idx_anki_decks_created_at ON public.anki_decks(created_at);

-- Only the backend (service role, which bypasses RLS) writes rows. Users may
-- see their own decks; anonymous mints are visible to nobody but us.
ALTER TABLE public.anki_decks ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view their own decks"
  ON public.anki_decks
  FOR SELECT
  USING (auth.uid() = user_id);
