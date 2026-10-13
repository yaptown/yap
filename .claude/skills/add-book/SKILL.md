---
name: add-book
description: Add a book (or a batch, e.g. "one more per language") to Yap's Books reader end to end. Covers choosing learner-friendly public-domain books and checking copyright; importing into books/<slug>/source.json; modernizing spelling (luna proposes, a Sonnet reviewer per book checks and applies); teasers; a riso cover via the yap-icons harness; publishing to Supabase; verifying the catalog; committing. Use whenever the user wants more books, a specific book, covers, teasers or spelling fixes for books, or asks which public-domain works suit a course language.
---

# Add a book to Yap's reader

This records how the first two batches (Oct 2026, 29 books) were done. **What Andre asked for** is marked as such. Everything else is what we chose or found, a starting point rather than a rule; check with Andre when a case doesn't fit.

## Where things live

- `books/<slug>/source.json` is a `language_utils::book::Book`, plus an optional `cover.webp`. `books/*/build/` is generated and gitignored.
- *Mistress of the Waves* is the exception: a git-crypt'd `source.doc` plus `metadata.json` `{"translate": true}`, machine-translated into every course language. Public-domain (PD) books are not translated.
- Pipeline: `cargo run -p generate-data --bin build_book` (editions and teasers) → `--bin publish_book` (Supabase tables, cover to the packs R2 bucket) → the apps call `book_catalog(requested_language, native_language)`. A new book needs no app change.

## What Andre asked for

- Books in their original language, no translation; books not available in a learner's language must not appear for them.
- At least one book per course language besides Mistress (then "one more per language").
- The most modern, easy, engaging PD works for learners: "I would hate to be an English learner trying to read Moby Dick". He suggested standardebooks.org as a source.
- A teaser per book from the first couple of chapters, written by luna, translated so it's shown in the reader's native language.
- Spelling modernization, spelling only. luna proposes; Claude Code Sonnet subagents review luna's changes and do their own pass.
- Covers: codex may draw them using the yap-icons harness.

## 1. Choosing books: what we did

- One research agent per language, asking for 2–3 ranked candidates with the author's death year, publication year, copyright reasoning and a verified full-text URL.
- We required PD in both the US and the author's home country. Findings that recur:
  - US: published 1930 or earlier is PD. Later foreign works (e.g. 1936 Korean stories, 1930s Premchand) were treated as PD because the home-country term had already expired by 1996-01-01, so the URAA didn't restore them. Codex flagged this reasoning as less certain than the pre-1931 case.
  - Home-country extensions in India (1992), Korea (1987, 2013) and Japan (2018) didn't revive expired works.
  - Uruguay and Argentina have "paying public domain" (a fee for commercial use). We dropped Quiroga for that. INDECOPI lists Palma as royalty-free in Peru.
  - Source sites' copyright notices (Vajirayana, LiberLiber, Wikisource) cover their layout and commentary; we took only the original text.
- We matched the author's region to the course where there was a choice (`spa` = Latin America, `spa-es` = Spain, `por` = Brazil, `por-pt` = Portugal).

## 2. Importing: what worked

The importer was one-off code, deleted after each batch.

| Language | Source that worked | Didn't work |
|---|---|---|
| eng | Standard Ebooks EPUB (`?source=download`; the cover art is CC0) | |
| fra, deu, ita, spa, spa-es, por-pt | Gutenberg HTML (some legacy encodings), LiberLiber (Italian) | Gutenberg *Doña Perfecta* (English glosses); German files spelling umlauts as ue/oe |
| rus | ru.wikisource (modern orthography) | Gutenberg (English Chekhov only) |
| jpn | Aozora (Shift_JIS; we dropped ruby and annotation markers, kept gaiji) | |
| kor | ko.wikisource (a stable older modern transcription beat fresh unproofread OCR) | |
| zho-hans | zh.wikisource, `zh-hans` variant | |
| hin | Hindwi, Hindi Samay | Hindi Wikisource (OCR errors) |
| tha | vajirayana.org | Thai Gutenberg (nothing) |
| por | pt.wikisource 1920 transcription (usable because spelling gets modernized) | raw OCR; Dinis's only clean text followed a 1984 Brazilian edition |

What we did: one story per chapter for collections; titles in the original language; paragraphs and scene breaks kept, apparatus dropped; every title and paragraph run through `cleanup_sentence` (as build_book does for all book text); first and last paragraphs spot-checked. Copyright lines read like "1878. Public domain in the United States and Spain." We left the Project Gutenberg name out, because its license reserves the trademark on modified copies. Obvious transcription typos were fixed by hand against scans and listed.

## 3. Spelling (Andre's protocol)

1. luna writes unapplied proposals to `books/<slug>/spelling_proposals.json`: `{chapter, paragraph (null = title), old_word, new_word, context, check}`.
2. A Workflow runs one Sonnet agent per book (`model: 'sonnet'`; books don't share files). Each reviews every proposal, adds what luna missed, writes `spelling_reviewed.json` entries `{"accepted": true, chapter, paragraph, old_word, new_word, context}`, and runs the apply tool itself until it passes. Note: in workflow scripts `phase('Review')` is only a label, so call it, then `await parallel(...)`.
3. The apply tool was a temporary `generate-data` example. It applied only accepted entries, atomically, and refused non-unique contexts, overlapping spans, whitespace changes, word-count changes and edits beyond about two-thirds of the word. That last screen is only a heuristic: *Nympha→Ninfa* (4 edits) is real. Overlaps came from listing both 聞え and 聞えました; keep the longer one.

How the reviewers interpreted "spelling only" (their choices, which looked right): leave names, foreign quotations, mottos and deliberate dialogue dialect alone; keep each book's regional standard (British English, European Portuguese); skip optional French 1990 rectifications; leave changes that would move a hyphen (Portuguese *vel-o*).

What review caught that luna missed or got wrong: systematic misses (*Grethel→Gretel* ×37, *Rothkäppchen→Rotkäppchen* ×26); meaning changes (*d'óra* "henceforth" → *d'hora* "hour"; *caes* "quay" read as *cães* "dogs"); a dropped crasis (*àquele*); edits inside names and Latin quotes. The reports are worth a skim.

## 4. Teasers

`build_book` does this: luna writes an English teaser from the first two chapters (about 6,000 words), then translates it into every native language in `language_utils::COURSES`, cached. Rebuilding after spelling changes refreshes them (cents).

## 5. Covers

In `/Volumes/T7/coding/yap-icons`: add `covers/<slug>.mjs` (paper colour, title lines (the main title, which must prefix the source title), and one motif in the icon element format). `node build.mjs --covers /Volumes/T7/coding/yap` writes `books/<slug>/cover.webp`, plus sheets at `out/covers-sheet.png` and `out/covers-thumbs.png` (list size, 80×120). It refuses to overwrite the original covers (Mistress, Sherlock, Oz).

The first round of covers had thin, small titles and floating motifs. Bolder titles, motifs filling the space, and the icons' soft patch behind the motif made them read as book covers. Look at both sheets before shipping.

## 6. Publishing and verifying

```bash
YAP_CACHE_BUCKET= cargo run --release -p generate-data --bin build_book
YAP_CACHE_BUCKET= cargo run --release -p generate-data --bin publish_book   # loads .env, which is PRODUCTION
```
- `.env` holds the production Supabase key and R2 credentials. Variables already set in the environment win over `.env`. For a local test, point `SUPABASE_URL` and `SUPABASE_SERVICE_ROLE_KEY` at `pnpm dlx supabase status -o env` values.
- Verify each language with the public anon key (in `yap-frontend-rs/src/supabase.rs`): `POST https://eearwzqotpfoderpfrqx.supabase.co/rest/v1/rpc/book_catalog` with `{"requested_language":"fra","native_language":"eng"}`. Each book should come back with a teaser and a cover, and the cover URL on packs.yap.town should return 200.
- Migrations: `pnpm dlx supabase db push --dry-run`, then `--yes`. It works non-interactively through the keychain login, but the first linked command can take minutes.

## 7. Committing

We staged only `books/<slug>/source.json` and `cover.webp` (never `build/`), deleted the proposal and review files and the one-off tools first, and committed yap-icons separately. Model cost was about $0.05–0.10 per batch of 14 books.
