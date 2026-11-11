# Usage inventory ledger

`<language>/usage_inventories.jsonl` contains one `UsageInventory` per exact
serialized gram. `key` is display text, not identity (different parts of speech
can display identically).

The `usages` array is an append-only identity ledger: position 0 is sense 1.
Never reorder, delete, compact, or repurpose a slot. Discovery updates its own
slots and retires missing usages by clearing their counts and anchors. Reading
hierarchies append their own slots. Neither may overwrite or retire a manual
slot. Every row edited by the manual CLI must end up with at least two live
senses, counting existing live slots and the additions. A lone sense gives the
occurrence labeler nothing to choose between.

To add a sense, append a usage with `"source":"manual"`. Keep existing slots
unchanged. A usage is live when `n_assigned > 0` **or** its source is `manual`;
manual entries do not need invented corpus counts. For example, append an
“again” sense to the exact French `encore` adverb gram:

```json
{"source":"manual","kind":"meaning","gloss":"de nouveau, une autre fois ; répétition","n_gold":0,"n_assigned":0,"loo_correct":0,"anchors":[{"sentence":"Dis-le encore.","spans":[[7,13]],"gold":true}]}
```

Anchors may be handwritten, outside the corpus. Spans are zero-based Unicode
character ranges, end-exclusive, of the gram's words (not UTF-8 byte offsets).
Use real sentences that unambiguously demonstrate this sense. Anchors are
shown in occurrence menus and used as dictionary examples; manual examples
are retained even when absent from the corpus. `kind` is `meaning`,
`construction`, or `formula`. Leave `readings` absent for a meaning-only slot.

For a new manual-only row, use the exact serialized `gram` from segmentation,
set `key` to its display text, `usages` to the manual entries, `absorbers` to
`[]`, `n` and `n_expression` to 0, `silhouette` to 0, and `source` to `manual`.
Do not merge rows merely because their display keys match.

## Hand-driven CLI

Run from the repository root with a built pack in `out/`:

```sh
cargo run --bin manual_senses -- --course fra list encore arriver
cargo run --bin manual_senses -- --course fra list --json encore arriver
cargo run --bin manual_senses -- --course fra add SELECTOR --kind meaning \
  --gloss 'de nouveau, une autre fois' --example 'Dis-le encore.'
```

Copy the short selector printed by `list`, not the display text. It is a
stable hash prefix of the complete gram identity, lengthened only when needed
to disambiguate. Listing includes multiword grams containing the lemma, packed
frequency (plus `direct`, the sentence-only count), and every existing usage
slot (including retired ones). `list --json` emits one JSON object per gram:
`selector`, `display`, `frequency`, `direct_count`, `lemmas` (objects with
`lemma` and `pos`), and `usages` (objects with one-based `id`, `kind`, `gloss`,
`live`, `manual`, `source`, and `n_assigned`). Several lemmas select their union.
You can
also pass a course slug such as `fra_for_eng`, or `--native` for another native
language. Glosses should be in the target language.

`add` finds the selected gram's exact surface token sequence in each example,
ignoring case and accepting Unicode whitespace. It derives character offsets;
it never expands the selected gram to other inflections. Missing or repeated
matches fail before writing. Examples are optional; use repeated `--example`
flags for more examples. `add SELECTOR SELECTOR... --kind meaning --gloss '...'`
adds the same sense to several grams, but examples are allowed only with one
selector. Each selected gram must have at least two live senses after the edit.
A new gloss must not equal any existing gloss (including retired slots), or
another new gloss for that gram, after trimming and case-insensitive comparison.

### Parallel proposals, single writer

Research agents write separate proposal JSONL files, never the inventory itself.
Each line targets one selector with one or more senses:

```json
{"gram":"SELECTOR","senses":[{"kind":"meaning","gloss":"de nouveau, une autre fois","examples":["Dis-le encore."]},{"kind":"meaning","gloss":"toujours, jusqu'au moment présent"}]}
```

`examples` may be omitted or empty. Kinds and example matching follow `add`.
New rows usually need at least two proposed senses; existing live senses count
toward the same minimum. Apply all proposal files in one command:

```sh
cargo run --bin manual_senses -- --course fra apply /tmp/batch-a.jsonl /tmp/batch-b.jsonl
```

`apply` validates every line of every file before writing and reports errors
with file, line, selector, and reason. Duplicate selectors across lines/files
(including different prefixes of the same gram) are errors. Any error leaves
the inventory untouched. Both commands append slots without changing existing
sense IDs, create missing rows, preserve untouched JSONL rows verbatim, and
replace the inventory once, atomically.

For a trial, copy the inventory to scratch and pass
`--inventory /absolute/path/to/scratch.jsonl` before `list`, `add`, or `apply`. Only that
file is written. `--out` can select another built-pack directory.

Non-reading languages label every occurrence of grams with at least two live
usages using GPT-6 Luna at medium reasoning. Menus show live usages and
expression absorbers, with up to four anchor examples per option. Answers rank
compatible sense IDs; assignment uses the first while the report retains all.
Only `none_of_these` escapes go to `out/<language>/missing_senses.jsonl`, with
the gram key and identity, sentence, marked character span, and missing meaning.
There is no automatic inventory repair. Escapes, absorbers, and invalid answers
explicitly bypass centroids and use the frozen frequency fallback.
`sense_labels.jsonl` records all requests, parsed responses and answer errors.
CJK reading requests remain unchanged; their sense labels feed the same
assignment abstraction.

`generate-data --readings-sample 10 fra` uses write-isolated preparation and
selects roughly 10% of whole grams by a stable hash. Requests are identical to
full-run requests, so the normal LLM cache is reusable. Reports are under
`target/readings-sample/fra/`; the inventory is never written by this sample.
