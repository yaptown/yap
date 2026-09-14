-- One row per real synthesis an exported Anki deck caused (cache hits excluded).
--
-- This is the per-deck ElevenLabs spend and the input for a later per-deck
-- budget. A log rather than a counter also tells us which sentences a deck
-- synthesized, including best-effort clips that cost money but failed checks.
-- Only the backend, via the service role, reads or writes this append-only log.
CREATE TABLE public.anki_deck_tts_synths (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  deck_id UUID NOT NULL REFERENCES public.anki_decks(id) ON DELETE CASCADE,
  cache_filename TEXT NOT NULL,
  verified BOOLEAN NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_anki_deck_tts_synths_deck ON public.anki_deck_tts_synths(deck_id);
ALTER TABLE public.anki_deck_tts_synths ENABLE ROW LEVEL SECURITY;
