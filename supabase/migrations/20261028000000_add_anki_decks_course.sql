-- The course a deck was generated for, as `Course::code()` ("jpn-eng").
-- Null for decks minted before this column existed.
ALTER TABLE public.anki_decks ADD COLUMN course TEXT;
