import initSqlJs from "sql.js";
import sqlWasmUrl from "sql.js/dist/sql-wasm.wasm?url";
import { strToU8, zipSync } from "fflate";
import type { AnkiDeckPlan, AnkiMediaSource, AnkiNote } from "../../../yap-frontend-rs/pkg";
import { languageToLangAttr } from "../lib/pure";

import { fetchMedia, type MediaProgress } from "./media";

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

// Field order is the note type's schema; everything else refers to fields by name.
const sentenceFields = [
  "Sentence", "Translation", "Glosses", "ProperNouns", "Source",
  "ClipUrl", "ClipWebmUrl", "ClipPosterUrl", "ClipAspect", "TtsBundled", "IncludeReading", "IncludeListening", "Subtitles", "MaskedSubtitles", "MaskedSentence",
] as const;
const wordFields = ["Word", "Definition", "AudioBundled", "Source", "ClipUrl", "ClipWebmUrl", "ClipPosterUrl", "ClipAspect", "Subtitles"] as const;
type SentenceFields = Record<(typeof sentenceFields)[number], string>;
type WordFields = Record<(typeof wordFields)[number], string>;
const css = `
/* Translucent surfaces, so the cards sit on each Anki app's own background. */
.card {
  --yap-fg: #1c1b19; --yap-muted: #75716a; --yap-line: rgba(0,0,0,.12); --yap-chip: rgba(0,0,0,.05);
  --yap-accent: #d9480f; --yap-accent-soft: rgba(217,72,15,.1);
  font-family: system-ui, -apple-system, "Segoe UI", sans-serif; text-align: center; line-height: 1.45;
  padding: 24px 16px; color: var(--yap-fg);
}
.card.nightMode, .card.night_mode, .nightMode .card, .night_mode .card {
  --yap-fg: #f1efe9; --yap-muted: #a29e96; --yap-line: rgba(255,255,255,.18); --yap-chip: rgba(255,255,255,.07);
  --yap-accent: #ff8a3d; --yap-accent-soft: rgba(255,138,61,.14);
}
.yap { max-width: 520px; margin: 0 auto; display: flex; flex-direction: column; align-items: center; gap: 16px; }
.eyebrow {
  font-size: 11px; font-weight: 600; letter-spacing: .08em; text-transform: uppercase;
  color: var(--yap-accent); background: var(--yap-accent-soft); padding: 4px 10px; border-radius: 99px;
}
.sentence { font-size: 28px; font-weight: 600; line-height: 1.25; letter-spacing: -.01em; }
.word { font-size: 36px; }
.prompt .replay-button { margin: 0 0 0 6px; vertical-align: middle; }
.prompt .replay-button svg { width: 32px; height: 32px; }
.prompt .replay-button svg circle { fill: transparent; stroke: var(--yap-line); stroke-width: 3; }
.prompt .replay-button svg path { fill: var(--yap-fg); }
.yap #answer { width: 100%; height: 0; margin: -8px 0; border: 0; background: none; }
.answer { display: flex; flex-direction: column; align-items: center; gap: 12px; width: 100%; animation: yap-in .18s ease-out; }
@keyframes yap-in { from { opacity: 0; transform: translateY(4px); } }
.translation { font-size: 20px; }
.card ul.glosses {
  display: grid; grid-template-columns: auto auto; justify-content: center; gap: 4px 14px;
  width: 100%; max-width: 360px; margin: 0; padding: 12px 0 0; border-top: 1px solid var(--yap-line);
  list-style: none; font-size: 15px;
}
.card ul.glosses li { display: contents; }
.glosses .term { text-align: right; font-weight: 600; text-decoration: none; color: inherit; }
.glosses .gloss { text-align: left; color: var(--yap-muted); }
.names { margin: 0; padding: 0; list-style: none; font-size: 14px; color: var(--yap-muted); }
.names .name { color: var(--yap-fg); font-weight: 600; }
.scene {
  position: relative; display: block; width: 100%; aspect-ratio: 16 / 9; padding: 0; border: 0; border-radius: 14px; overflow: hidden;
  background: #000; cursor: pointer; font: inherit; box-shadow: 0 6px 24px rgba(0,0,0,.18); -webkit-tap-highlight-color: transparent;
}
/* Sized up front from the clip's own shape (16:9 when the manifest lacks it), so loading resizes nothing. */
.scene video { display: block; width: 100%; height: 100%; object-fit: contain; }
.subs { position: absolute; left: 8px; right: 8px; bottom: 10px; display: flex; flex-direction: column; align-items: center; gap: 3px; pointer-events: none; }
.subs span { font-size: 15px; line-height: 1.3; padding: 2px 8px; border-radius: 6px; background: rgba(0,0,0,.65); color: rgba(255,255,255,.75); }
.subs .sentence { color: #ffd166; font-weight: 600; letter-spacing: 0; }
.subs .gap { display: inline-block; width: 2.2em; height: .9em; border-bottom: 2px dashed currentColor; }
.prompt .gap { font-style: normal; color: transparent; border-bottom: 2px dashed var(--yap-muted); }
.play {
  position: absolute; top: 10px; right: 10px; width: 34px; height: 34px; border-radius: 50%;
  background: rgba(0,0,0,.5); display: grid; place-items: center; transition: opacity .15s;
}
.play svg { width: 12px; height: 14px; margin-left: 2px; fill: #fff; }
.scene.playing .play { opacity: 0; }
.source {
  display: flex; align-items: center; gap: 8px; align-self: flex-start; padding: 4px 12px 4px 4px;
  border-radius: 8px; background: var(--yap-chip); color: var(--yap-muted); font-size: 13px;
}
.source img { display: block; height: 32px; border-radius: 4px; }
`;

// MP4 first, so AnkiDroid and AnkiMobile play what they always have. Desktop
// Anki's Qt WebEngine has no H.264/AAC; the codecs string makes it skip the
// MP4 (a bare video/mp4 type answers "maybe" there) and take the WebM.
const videoSources = `<source src="{{ClipUrl}}" type='video/mp4; codecs="avc1.64001F, mp4a.40.2"'><source src="{{ClipWebmUrl}}" type='video/webm; codecs="vp9, opus"'>`;

// The clip never autoplays: clips don't reliably start on or cut exactly to
// the sentence, so the audio is the prompt and the clip is context, showing a
// still from itself until tapped (a tap plays or pauses). Captions are drawn
// by the script below rather than as a native track, which AnkiMobile doesn't
// render; the film (poster + title) sits under the clip on both sides so the
// front says which movie.
function scene(subtitles: string): string {
  return `{{#ClipUrl}}<button type="button" class="scene" style="aspect-ratio: {{ClipAspect}}"><video poster="{{ClipPosterUrl}}" preload="metadata" playsinline>${videoSources}</video>`
    + `<div class="subs">{{${subtitles}}}</div>`
    + `<span class="play"><svg viewBox="0 0 12 14"><path d="M0 0l12 7-12 7z"/></svg></span></button>{{/ClipUrl}}
{{#Source}}<div class="source">{{Source}}</div>{{/Source}}`;
}

// Anki keeps one page across cards, so this runs again on every side; `const`
// at the top level would throw the second time.
const script = `<script>
(() => {
  for (const scene of document.querySelectorAll(".scene")) {
    const video = scene.querySelector("video");
    const cues = [...scene.querySelectorAll(".subs > span")];
    // The still is the sentence's moment, so until playback it carries that caption.
    let started = false;
    video.addEventListener("play", () => (started = true));
    const sync = () => {
      const ms = video.currentTime * 1000;
      for (const cue of cues) {
        cue.hidden = started ? !(ms >= +cue.dataset.at && ms < +cue.dataset.until) : !cue.classList.contains("sentence");
      }
      scene.classList.toggle("playing", !video.paused);
      if (!video.paused) requestAnimationFrame(sync);
    };
    for (const event of ["play", "pause", "seeked", "loadedmetadata"]) video.addEventListener(event, sync);
    scene.addEventListener("click", () => (video.paused ? video.play() : video.pause()));
    sync();
  }
})();
</script>`;

// With readings, the target-language fields hold Anki's furigana markup:
// `{{kanji:…}}` strips the readings on the front, `{{furigana:…}}` shows them
// as ruby on the back.
function filtered(field: string, furigana: boolean, back: boolean): string {
  return furigana ? `{{${back ? "furigana" : "kanji"}:${field}}}` : `{{${field}}}`;
}

// The stylesheet rides in every template rather than the note type's CSS:
// AnkiDroid updates an existing note type's templates on import but keeps
// its old CSS.
//
// Everything above the answer is the same on both sides, so revealing it
// shifts nothing: the answer takes the place of the card-type badge below.
function card(prompt: string, subtitles: string, eyebrow: string, answer: string | undefined): string {
  return `<style>${css}</style>
<div class="yap">
${prompt}
${scene(subtitles)}
<hr id="answer">
${answer === undefined ? `<span class="eyebrow">${eyebrow}</span>` : `<div class="answer">${answer}</div>`}
</div>
${script}`;
}

function templates(lang: string, furigana: boolean): { name: string; ord: number; qfmt: string; afmt: string }[] {
  const sentence = (back: boolean) =>
    `<div class="sentence prompt"><span lang="${lang}">${filtered("Sentence", furigana, back)}</span>{{TtsBundled}}</div>`;
  // Names get no gloss rows, so they're spelled out under the prompt.
  const names = '{{#ProperNouns}}<ul class="names">{{ProperNouns}}</ul>{{/ProperNouns}}';
  const meaning = '<div class="translation">{{Translation}}</div><ul class="glosses">{{Glosses}}</ul>';
  // Listening asks for the words hidden in MaskedSentence; the back reveals
  // them in place.
  const side = (name: "Reading" | "Listening", back: boolean) => card(
    (name === "Listening" && !back
      ? `<div class="sentence prompt"><span lang="${lang}">{{MaskedSentence}}</span>{{TtsBundled}}</div>`
      : sentence(back)) + names,
    name === "Listening" && !back ? "MaskedSubtitles" : "Subtitles",
    name === "Reading" ? "Translate" : "Listen",
    back ? meaning : undefined,
  );
  return (["Reading", "Listening"] as const).map((name, ord) => ({
    name, ord,
    qfmt: `{{#Include${name}}}${side(name, false)}{{/Include${name}}}`,
    afmt: side(name, true),
  }));
}

function wordTemplate(lang: string, furigana: boolean, back: boolean): string {
  // Same shape as the sentence cards: the recording plays on both sides.
  return card(
    `<div class="sentence word prompt"><span lang="${lang}">${filtered("Word", furigana, back)}</span>{{AudioBundled}}</div>`,
    "Subtitles", "Word", back ? '<div class="translation">{{Definition}}</div>' : undefined,
  );
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

function sourceFor(note: AnkiNote): string {
  return note.source ? (note.source.poster_filename ? `<img src="${escapeHtml(note.source.poster_filename)}" alt="">` : "")
    + `<span>${escapeHtml(note.source.title)}${note.source.year ? ` · ${note.source.year}` : ""}</span>` : "";
}

function fieldsFor(
  note: AnkiNote, bundled: (filename: string | undefined) => string, captions: (key: string | undefined) => string, lang: string,
): SentenceFields | WordFields {
  const audio = (filename: string) => { const file = bundled(filename); return file ? `[sound:${file}]` : ""; };
  if (note.type === "Word") {
    return {
      Word: escapeHtml(note.word_furigana), Definition: escapeHtml(note.definition), AudioBundled: audio(note.audio),
      Source: sourceFor(note), ClipUrl: escapeHtml(note.clip?.mp4 ?? ""),
      Subtitles: captions(note.subtitles ?? undefined), ClipWebmUrl: escapeHtml(note.clip?.webm ?? ""),
      ClipPosterUrl: escapeHtml(note.clip?.poster ?? ""), ClipAspect: String(note.clip?.aspect_ratio ?? ""),
    };
  }
  const media = audio(note.tts);
  const glosses = note.glosses.map(({ text, gloss, url }) => {
    const term = url ? `<a class="term" lang="${lang}" href="${escapeHtml(url)}">` : `<span class="term" lang="${lang}">`;
    return `<li>${term}${escapeHtml(text)}${url ? "</a>" : "</span>"}<span class="gloss">${escapeHtml(gloss ?? "")}</span></li>`;
  }).join("");
  const properNouns = note.proper_nouns.map(({ spans }) => `<li>${spans.map(({ text, target_language }) =>
    target_language ? `<span class="name" lang="${lang}">${escapeHtml(text)}</span>` : escapeHtml(text)).join("")}</li>`).join("");
  return {
    Sentence: escapeHtml(note.sentence_furigana), Translation: escapeHtml(note.translation), Glosses: glosses, ProperNouns: properNouns,
    Source: sourceFor(note), ClipUrl: escapeHtml(note.clip.mp4), TtsBundled: media,
    IncludeReading: note.include_reading ? "1" : "", IncludeListening: note.include_listening && media ? "1" : "",
    Subtitles: captions(note.subtitles), MaskedSubtitles: captions(note.masked_subtitles ?? undefined),
    MaskedSentence: note.masked_sentence ?? "",
    ClipWebmUrl: escapeHtml(note.clip.webm), ClipPosterUrl: escapeHtml(note.clip.poster),
    ClipAspect: String(note.clip.aspect_ratio ?? ""),
  };
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
  const bundledFiles = new Map<string, string>();
  const captionMarkup = new Map<string, string>();
  let mediaIndex = 0;
  const results = await fetchMedia(plan, fetchBundled, onProgress);
  // Completion order must not change the package's media identities.
  results.forEach((bytes, index) => {
    const entry = plan.bundled[index];
    if (bytes && entry.source.type === "Subtitles") {
      captionMarkup.set(entry.filename, new TextDecoder().decode(bytes));
    } else if (bytes) {
      const filename = (entry.source.type === "Tts" || entry.source.type === "HumanAudio") ? audioFilename(entry.filename, bytes) : entry.filename;
      const key = String(mediaIndex++);
      media[key] = filename;
      files[key] = bytes;
      bundledFiles.set(entry.filename, filename);
    }
  });
  const bundled = (filename: string | undefined): string => filename ? bundledFiles.get(filename) ?? "" : "";
  const captions = (key: string | undefined): string => key ? captionMarkup.get(key) ?? "" : "";
  const SQL = await initSqlJs({ locateFile: () => sqlWasmUrl });
  const db = new SQL.Database();
  const now = Date.now();
  const deckId = Number(plan.deck_id);
  const lang = escapeHtml(languageToLangAttr(plan.language));
  const sentenceId = Number(plan.sentence_model_id);
  const wordId = Number(plan.word_model_id);
  const model = (id: number, name: string, fields: readonly string[], tmpls: ReturnType<typeof templates>, req: [number, string, number[]][]) => ({
    id, name, type: 0, mod: modified, usn: -1, sortf: 0, did: deckId, css: "",
    flds: fields.map((name, ord) => ({ name, ord, sticky: false, rtl: false, font: "Arial", size: 20 })),
    tmpls: tmpls.map((template) => ({ ...template, did: null, bqfmt: "", bafmt: "" })),
    req, vers: [], tags: [], latexPre: "", latexPost: "",
  });
  const models = {
    [sentenceId]: model(sentenceId, `Yap ${plan.course_code} sentences`, sentenceFields, templates(lang, plan.furigana), [
      [0, "all", [sentenceFields.indexOf("IncludeReading")]], [1, "all", [sentenceFields.indexOf("IncludeListening")]],
    ]),
    [wordId]: model(wordId, `Yap ${plan.course_code} words`, wordFields, [{
      name: "Word", ord: 0,
      qfmt: wordTemplate(lang, plan.furigana, false),
      afmt: wordTemplate(lang, plan.furigana, true),
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
      const fields = fieldsFor(note, bundled, captions, lang);
      const values = "Word" in fields ? wordFields.map((name) => fields[name]) : sentenceFields.map((name) => fields[name]);
      const text = note.type === "Word" ? note.word : note.sentence;
      const hash = await crypto.subtle.digest("SHA-1", new TextEncoder().encode(text));
      const checksum = new DataView(hash).getUint32(0);
      db.run("INSERT INTO notes VALUES (?, ?, ?, ?, -1, ?, ?, ?, ?, 0, '')", [
        Number(note.note_id), note.guid, note.type === "Word" ? wordId : sentenceId, modified,
        ` ${note.tags.join(" ")} `, values.join("\x1f"), text, checksum,
      ]);
      const ordinals = "Word" in fields ? [0]
        : [fields.IncludeReading ? 0 : undefined, fields.IncludeListening ? 1 : undefined].filter((ord) => ord !== undefined);
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
