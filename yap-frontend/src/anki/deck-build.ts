import type { AnkiNote } from "../../../yap-frontend-rs/pkg";

export type SentenceNote = Extract<AnkiNote, { type: "Sentence" }>;

/** What the planner has put in the deck so far, in deck order. */
export type DeckBuild = {
  sentences: SentenceNote[];
  words: number;
  /** Films in order of their first sentence. */
  films: { id: string; title: string; poster: boolean }[];
};

export const emptyBuild: DeckBuild = { sentences: [], words: 0, films: [] };

export function addNotes(build: DeckBuild, notes: AnkiNote[]): DeckBuild {
  if (notes.length === 0) return build;
  const sentences = [...build.sentences];
  const films = [...build.films];
  let words = build.words;
  for (const note of notes) {
    if (note.type === "Word") {
      words += 1;
      continue;
    }
    sentences.push(note);
    if (!films.some((film) => film.id === note.source.imdb_id)) films.push({ id: note.source.imdb_id, title: note.source.title, poster: note.source.poster_filename != null });
  }
  return { sentences, words, films };
}
