# Credits

Yap.Town is built on the work of many people and projects. This file lists the
third-party data, media and code we distribute, with the licence each comes
under. Anything derived from a share-alike source stays under that source's
licence, not Yap.Town's.

## Sentences and vocabulary

- **[Tatoeba](https://tatoeba.org)**: example sentences and their
  translations, by the Tatoeba contributors, under
  [CC BY 2.0 FR](https://creativecommons.org/licenses/by/2.0/fr/). We filter
  them to each course's language pair and annotate them with tokens, lemmas
  and grammar.
- **neri's frequency decks**: French and Spanish Anki decks built from
  Tatoeba sentences (same licence as above).
- **Movie and TV subtitles**, found through
  [OpenSubtitles](https://www.opensubtitles.org). Dialogue belongs to its
  respective rights holders.
- **[Wiktionary](https://www.wiktionary.org)**: pronunciations (collected
  with [WikiPron](https://github.com/CUNY-CL/wikipron)), inflection tables and
  multiword expressions, by the Wiktionary contributors, under
  [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). We convert
  and filter this data for each course; the converted data remains
  CC BY-SA 4.0.

## Films

- **[TMDB](https://www.themoviedb.org)**: film posters and metadata.
- **[OMDb](https://www.omdbapi.com)**: Rotten Tomatoes scores, under
  [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/).

## Handwriting

Stroke-order data comes from KanjiVG (CC BY-SA 3.0), Make Me a Hanzi and
AnimCJK (Arphic Public License), and Scribing (MIT). The full attribution,
including how each source was converted, is in
[libraries/stroke-order/licenses/NOTICE.md](libraries/stroke-order/licenses/NOTICE.md),
alongside the licence texts.

## Voices

- French recordings by **Jean-Cavard**.
- Other speech is synthesised with Google Cloud Text-to-Speech, Gemini and
  ElevenLabs.

## Fonts and images

- [Nunito](https://github.com/googlefonts/nunito) (copyright 2014 The Nunito
  Project Authors) and [Nunito Sans](https://github.com/Fonthausen/NunitoSans)
  (copyright 2016 The Nunito Sans Project Authors), under the
  [SIL Open Font License 1.1](https://openfontlicense.org).
- Background textures from photos by
  [Michael Oeser](https://unsplash.com/photos/black-and-gray-textile-in-close-up-photography-X7jvviscg8o)
  and
  [Corina Rainer](https://unsplash.com/photos/white-cotton-on-white-textile-jZc5eTXnYLU)
  on Unsplash.

## Code

- **[rs-fsrs](https://github.com/open-spaced-repetition/rs-fsrs)** by the
  Open Spaced Repetition group, the scheduler behind every review (MIT).
- The many other open-source Rust, JavaScript and Swift libraries Yap.Town
  depends on, each under its own licence (listed in `Cargo.lock`,
  `yap-frontend/pnpm-lock.yaml` and the iOS package manifest).
