import initSqlJs from "sql.js";
import sqlWasmUrl from "sql.js/dist/sql-wasm.wasm?url";
import { strToU8, zipSync } from "fflate";
import type { AnkiDeckPlan, AnkiMediaSource, AnkiNote } from "../../../yap-frontend-rs/pkg";
import { languageToLangAttr } from "../lib/pure";

export type MediaProgress = { done: number; total: number };

const schema = `
CREATE TABLE col (id integer PRIMARY KEY, crt integer NOT NULL, mod integer NOT NULL, scm integer NOT NULL, ver integer NOT NULL, dty integer NOT NULL, usn integer NOT NULL, ls integer NOT NULL, conf text NOT NULL, models text NOT NULL, decks text NOT NULL, dconf text NOT NULL, tags text NOT NULL);
CREATE TABLE notes (id integer PRIMARY KEY, guid text NOT NULL, mid integer NOT NULL, mod integer NOT NULL, usn integer NOT NULL, tags text NOT NULL, flds text NOT NULL, sfld integer NOT NULL, csum integer NOT NULL, flags integer NOT NULL, data text NOT NULL);
CREATE TABLE cards (id integer PRIMARY KEY, nid integer NOT NULL, did integer NOT NULL, ord integer NOT NULL, mod integer NOT NULL, usn integer NOT NULL, type integer NOT NULL, queue integer NOT NULL, due integer NOT NULL, ivl integer NOT NULL, factor integer NOT NULL, reps integer NOT NULL, lapses integer NOT NULL, left integer NOT NULL, odue integer NOT NULL, odid integer NOT NULL, flags integer NOT NULL, data text NOT NULL);
CREATE TABLE revlog (id integer PRIMARY KEY, cid integer NOT NULL, usn integer NOT NULL, ease integer NOT NULL, ivl integer NOT NULL, lastIvl integer NOT NULL, factor integer NOT NULL, time integer NOT NULL, type integer NOT NULL);
CREATE TABLE graves (usn integer NOT NULL, oid integer NOT NULL, type integer NOT NULL);
CREATE INDEX ix_notes_usn ON notes (usn);
CREATE INDEX ix_cards_usn ON cards (usn);
CREATE INDEX ix_revlog_usn ON revlog (usn);
CREATE INDEX ix_cards_nid ON cards (nid);
CREATE INDEX ix_cards_sched ON cards (did, queue, due);
CREATE INDEX ix_revlog_cid ON revlog (cid);
CREATE INDEX ix_notes_csum ON notes (csum);
`;

const sentenceFields = [
  "Sentence", "Translation", "TargetWord", "TargetGloss", "Glosses", "Source",
  "ClipUrl", "TtsBundled", "IncludeReading", "IncludeListening",
];
const css = `
.card { font-family: sans-serif; text-align: center; line-height: 1.5; padding: 20px; }
.sentence { font-size: 28px; }
.eyebrow { color: #777; font-size: 12px; text-transform: uppercase; }
.hint { color: #777; }
.translation { font-size: 20px; margin: 16px 0; }
.glosses { color: #777; font-size: 16px; list-style: none; padding: 0; }
a { color: inherit; }
.source { color: #777; font-size: 14px; margin: 8px 0 16px; }
.source img { display: block; max-height: 90px; margin: 0 auto 4px; }
video { display: block; width: 100%; max-width: 480px; margin: 16px auto 0; background: #000; }
`;

function templates(lang: string): { name: string; ord: number; qfmt: string; afmt: string }[] {
  const sentence = `<div class="sentence"><span lang="${lang}">{{Sentence}}</span></div>`;
  const answer = `<div class="translation">{{Translation}}</div>
<div><strong lang="${lang}">{{TargetWord}}</strong> — {{TargetGloss}}</div>
<ul class="glosses">{{Glosses}}</ul>`;
  // The clip never autoplays: clips don't reliably start on or cut exactly to
  // the sentence, so the TTS is the prompt and the clip is context. The film
  // (poster + title) sits under it on both sides so the front says which movie.
  const side = (name: "Reading" | "Listening", back: boolean) => `
<div class="eyebrow">${name === "Reading" ? "Translate" : "Listening"}</div>
{{TtsBundled}}
${name === "Reading" ? sentence : ""}
<hr id="answer">
${back ? (name === "Listening" ? sentence : "") + answer : '<p class="hint">Tap to reveal the answer</p>'}
<video src="{{ClipUrl}}" controls preload="metadata" playsinline></video>
<div class="source">{{Source}}</div>`;
  return (["Reading", "Listening"] as const).map((name, ord) => ({
    name, ord,
    qfmt: `{{#Include${name}}}${side(name, false)}{{/Include${name}}}`,
    afmt: side(name, true),
  }));
}

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, (char) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  })[char]!);
}

// The endpoint can return a different container than its cache filename suggests.
function audioFilename(filename: string, bytes: Uint8Array): string {
  const magic = String.fromCharCode(...bytes.subarray(0, 12));
  const ext = magic.startsWith("RIFF") && magic.slice(8) === "WAVE" ? "wav"
    : magic.startsWith("OggS") ? "ogg"
    : magic.startsWith("fLaC") ? "flac"
    : magic.slice(4, 8) === "ftyp" ? "m4a" : "mp3";
  return filename.replace(/\.[^.]+$/, `.${ext}`);
}

type Fetched = { bytes: Uint8Array; filename: string } | undefined;

function fieldsFor(note: AnkiNote, audio: (filename: string) => string, lang: string): string[] {
  if (note.type === "Word") {
    const media = audio(note.audio);
    return [escapeHtml(note.word), escapeHtml(note.definition), media];
  }
  const media = audio(note.tts);
  const glosses = note.glosses.map(({ text, gloss, url }) => {
    const word = `<span lang="${lang}">${escapeHtml(text)}</span>`;
    return `<li>${url ? `<a href="${escapeHtml(url)}">${word}</a>` : word}${gloss ? ` — ${escapeHtml(gloss)}` : ""}</li>`;
  }).join("");
  const source = (note.source.poster_filename ? `<img src="${escapeHtml(note.source.poster_filename)}" alt="">` : "")
    + escapeHtml(note.source.title) + (note.source.year ? ` (${note.source.year})` : "");
  return [
    escapeHtml(note.sentence), escapeHtml(note.translation), escapeHtml(note.target_word),
    escapeHtml(note.target_gloss), glosses, source, escapeHtml(note.clip_url), media,
    note.include_reading ? "1" : "", note.include_listening && media ? "1" : "",
  ];
}

/** Worker-side package assembly; no learner state or sentence selection lives here. */
export async function buildApkg(
  plan: AnkiDeckPlan,
  fetchBundled: (source: AnkiMediaSource) => Uint8Array | undefined | Promise<Uint8Array | undefined>,
  onProgress: (progress: MediaProgress) => void,
  modified: number,
): Promise<Blob> {
  const files: Record<string, Uint8Array> = {};
  const media: Record<string, string> = {};
  const audioFiles = new Map<string, string>();
  let done = 0;
  let mediaIndex = 0;
  onProgress({ done, total: plan.bundled.length });
  // Eight at a time. The backend verifies deck audio with Cloudflare alone
  // (720 Whisper requests a minute), so the ceiling is ElevenLabs' own
  // concurrency limit, not the transcription gate.
  const concurrency = 8;
  for (let start = 0; start < plan.bundled.length; start += concurrency) {
    const batch = plan.bundled.slice(start, start + concurrency);
    const results = await Promise.all(batch.map(async ({ filename, source }): Promise<Fetched> => {
      if (source.type !== "Tts") {
        const bytes = await fetchBundled(source);
        if (!bytes) throw new Error(`Bundled media missing: ${filename}`);
        onProgress({ done: ++done, total: plan.bundled.length });
        return { bytes, filename };
      }
      try {
        const response = await fetch(source.url);
        if (!response.ok) throw new Error(`Audio request failed (${response.status})`);
        const bytes = new Uint8Array(await response.arrayBuffer());
        if (!bytes.length) throw new Error("Audio response was empty");
        return { bytes, filename: audioFilename(filename, bytes) };
      } catch {
        return undefined;
      } finally {
        onProgress({ done: ++done, total: plan.bundled.length });
      }
    }));
    batch.forEach((item, index) => {
      const result = results[index];
      if (result) {
        const key = String(mediaIndex++);
        media[key] = result.filename;
        files[key] = result.bytes;
        audioFiles.set(item.filename, `[sound:${result.filename}]`);
      }
    });
  }
  const audio = (filename: string): string => audioFiles.get(filename) ?? "";
  const SQL = await initSqlJs({ locateFile: () => sqlWasmUrl });
  const db = new SQL.Database();
  const now = Date.now();
  const deckId = Number(plan.deck_id);
  const lang = escapeHtml(languageToLangAttr(plan.language));
  const sentenceId = Number(plan.sentence_model_id);
  const wordId = Number(plan.word_model_id);
  const model = (id: number, name: string, fields: string[], tmpls: ReturnType<typeof templates>, req: [number, string, number[]][]) => ({
    id, name, type: 0, mod: modified, usn: -1, sortf: 0, did: deckId, css,
    flds: fields.map((name, ord) => ({ name, ord, sticky: false, rtl: false, font: "Arial", size: 20 })),
    tmpls: tmpls.map((template) => ({ ...template, did: null, bqfmt: "", bafmt: "" })),
    req, vers: [], tags: [], latexPre: "", latexPost: "",
  });
  const models = {
    [sentenceId]: model(sentenceId, `Yap ${plan.course_code} sentences`, sentenceFields, templates(lang), [[0, "all", [8]], [1, "all", [9]]]),
    [wordId]: model(wordId, `Yap ${plan.course_code} words`, ["Word", "Definition", "AudioBundled"], [{
      name: "Word", ord: 0,
      // Same shape as the sentence cards: the recording plays on both sides.
      qfmt: `<div class="eyebrow">Word</div>
{{AudioBundled}}
<div class="sentence" lang="${lang}">{{Word}}</div>
<hr id="answer">
<p class="hint">Tap to reveal the answer</p>`,
      afmt: `<div class="eyebrow">Word</div>
{{AudioBundled}}
<div class="sentence" lang="${lang}">{{Word}}</div>
<hr id="answer">
<div class="translation">{{Definition}}</div>`,
    }], [[0, "all", [0]]]),
  };
  const deck = (id: number, name: string, desc: string) => ({
    id, name, mod: modified, usn: -1, desc, collapsed: false, browserCollapsed: false,
    dyn: 0, conf: 1, extendNew: 0, extendRev: 0,
    lrnToday: [0, 0], revToday: [0, 0], newToday: [0, 0], timeToday: [0, 0],
  });
  const defaultConfig = {
    id: 1, mod: 0, name: "Default", usn: 0, maxTaken: 60, autoplay: true, timer: 0, replayq: true,
    new: { bury: false, delays: [1, 10], initialFactor: 2500, ints: [1, 4, 0], order: 1, perDay: 20 },
    rev: { bury: false, ease4: 1.3, ivlFct: 1, maxIvl: 36500, perDay: 200, hardFactor: 1.2 },
    lapse: { delays: [10], leechAction: 1, leechFails: 8, minInt: 1, mult: 0 }, dyn: false,
  };
  try {
    db.run(schema);
    db.run("INSERT INTO col VALUES (?, ?, ?, ?, 11, 0, -1, 0, ?, ?, ?, ?, '{}')", [
      1, modified, now, now,
      JSON.stringify({ nextPos: plan.notes.length + 1, curDeck: deckId, activeDecks: [deckId], curModel: sentenceId, schedVer: 2, newSpread: 0 }),
      JSON.stringify(models), JSON.stringify({ 1: deck(1, "Default", ""), [deckId]: deck(deckId, plan.deck_name, plan.deck_description) }),
      JSON.stringify({ 1: defaultConfig }),
    ]);
    db.run("BEGIN");
    for (const [index, note] of plan.notes.entries()) {
      const fields = fieldsFor(note, audio, lang);
      const text = note.type === "Word" ? note.word : note.sentence;
      const hash = await crypto.subtle.digest("SHA-1", new TextEncoder().encode(text));
      const checksum = new DataView(hash).getUint32(0);
      db.run("INSERT INTO notes VALUES (?, ?, ?, ?, -1, ?, ?, ?, ?, 0, '')", [
        Number(note.note_id), note.guid, note.type === "Word" ? wordId : sentenceId, modified,
        ` yap yap::${plan.course_code} `, fields.join("\x1f"), text, checksum,
      ]);
      const ordinals = note.type === "Word" ? [0]
        : [fields[8] ? 0 : undefined, fields[9] ? 1 : undefined].filter((ord) => ord !== undefined);
      for (const ordinal of ordinals) {
        db.run("INSERT INTO cards VALUES (?, ?, ?, ?, ?, -1, 0, 0, ?, 0, 0, 0, 0, 0, 0, 0, 0, '')", [
          Number(note.card_id) + ordinal, Number(note.note_id), deckId, ordinal, modified, index + 1,
        ]);
      }
    }
    db.run("COMMIT");
    files["collection.anki2"] = db.export();
  } finally {
    db.close();
  }
  files.media = strToU8(JSON.stringify(media));
  // Media is deliberately bounded by the planner; an in-memory ZIP is enough for v1.
  return new Blob([zipSync(files, { level: 0 })], { type: "application/zip" });
}
