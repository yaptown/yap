-- Decks are no longer minted with signed tokens. The browser picks the deck
-- id and the package upload writes the row before storing anything; the
-- primary key is what stops an expired package's id from being reused.
DROP TABLE public.anki_deck_tts_synths;
ALTER TABLE public.anki_decks DROP COLUMN revoked_at;
