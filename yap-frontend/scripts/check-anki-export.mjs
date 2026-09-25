#!/usr/bin/env node
// Runs the real browser APKG writer in Node, then imports its output with official Anki.
// Requires uv on PATH. No backend calls are made; all media responses are mocked.
import { execFileSync } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scripts = path.dirname(fileURLToPath(import.meta.url));
const frontend = path.resolve(scripts, "..");
const require = createRequire(path.join(frontend, "package.json"));
const { build } = createRequire(require.resolve("vite/package.json"))("esbuild");
const output = await mkdtemp(path.join(tmpdir(), "yap-anki-export-"));

try {
  const runner = path.join(output, "run.mjs");
  await build({
    stdin: {
      contents: `
        import assert from "node:assert/strict";
        import { writeFileSync } from "node:fs";
        import path from "node:path";
        import { unzipSync, strFromU8 } from "fflate";
        import { buildApkg } from ${JSON.stringify(path.join(frontend, "src/anki/apkg.ts"))};
        import { nextModificationTime } from ${JSON.stringify(path.join(frontend, "src/anki/apkg-client.ts"))};
        (${writeFixtures.toString()})((...args) => buildApkg(...args, nextModificationTime()), ${JSON.stringify(output)})
          .catch(error => { console.error(error); process.exitCode = 1; });
      `,
      resolveDir: frontend,
      sourcefile: "anki-export-fixtures.mjs",
      loader: "js",
    },
    bundle: true,
    platform: "node",
    format: "esm",
    outfile: runner,
    plugins: [{
      name: "node-sql-wasm",
      setup(builder) {
        builder.onResolve({ filter: /\.wasm\?url$/ }, args => ({
          path: args.path, namespace: "wasm-url",
        }));
        builder.onLoad({ filter: /.*/, namespace: "wasm-url" }, () => ({
          contents: `export default ${JSON.stringify(require.resolve("sql.js/dist/sql-wasm.wasm"))}`,
          loader: "js",
        }));
        // Emscripten keeps its own CommonJS runtime and absolute package location.
        builder.onResolve({ filter: /^sql\.js$/ }, () => ({
          path: require.resolve("sql.js"), external: true,
        }));
      },
    }],
  });
  execFileSync(process.execPath, [runner], { stdio: "inherit" });
  execFileSync("uv", [
    "run", "--no-project", "--with", "anki", "python",
    path.join(scripts, "check-anki-export.py"), output,
  ], { stdio: "inherit" });
} finally {
  await rm(output, { recursive: true, force: true });
}

// Serialized as the bundle entry so the test calls the production export directly.
async function writeFixtures(buildApkg, output) {
  const hostileText = '<img src=x onerror="alert(1)"> & "é"';
  // Container headers suffice: this test checks packaging, not audio decoding.
  const wav = new Uint8Array([82, 73, 70, 70, 0, 0, 0, 0, 87, 65, 86, 69, 1, 2, 3]);
  const bundled = new Map([
    ["human.ogg", new Uint8Array([79, 103, 103, 83, 1, 2, 3])],
    ["poster.jpg", new Uint8Array([255, 216, 255, 217])],
  ]);
  let fetched = [];
  globalThis.fetch = async url => {
    fetched.push(url);
    if (url === "https://mock.invalid/bundle" || url === "https://mock.invalid/word") return new Response(wav);
    if (url === "https://mock.invalid/fail") return new Response("", { status: 503 });
    throw new Error(`Unexpected network request: ${url}`);
  };
  const identity = index => ({
    guid: `yaptest${index}`,
    note_id: 9007199254740908 + index * 4,
    card_id: 9007199254740972 + index * 4,
  });
  const sentence = (index, tts) => ({
    type: "Sentence", ...identity(index),
    sentence: `Phrase ${index} ${hostileText}`,
    translation: hostileText, target_word: hostileText, target_gloss: hostileText,
    glosses: [
      { text: hostileText, gloss: hostileText, url: 'https://mock.invalid/d?q="<>&' },
      { text: "plain", gloss: null, url: null },
    ],
    source: { title: hostileText, year: 2001, imdb_id: "tt0001", poster_filename: "poster.jpg" },
    clip_url: "https://mock.invalid/video.mp4?d=fake&v=1", tts,
    include_reading: true, include_listening: true,
    tags: ["yap", "yap::fra-eng", "yap::sentence", "yap::film::Amélie_2001"],
  });
  const base = {
    language: "French", course_code: "fra-eng", deck_name: "Yap test", deck_description: "Made with Yap (https://yap.town/anki)",
    deck_id: 9007199254740988, sentence_model_id: 9007199254740984, word_model_id: 9007199254740980,
    notes: [
      { type: "Word", ...identity(0), word: `Mot ${hostileText}`, definition: hostileText,
        audio: "human.ogg", tags: ["yap", "yap::fra-eng", "yap::word", "yap::pos::noun", "yap::frequency::top-100"] },
      { type: "Word", ...identity(1), word: `Autre ${hostileText}`, definition: hostileText,
        audio: "word.mp3", tags: ["yap", "yap::fra-eng", "yap::word", "yap::pos::phrase", "yap::frequency::rare"] },
      sentence(2, "tts.mp3"),
      sentence(3, "tts.mp3"),
      sentence(4, "failed.mp3"),
    ],
    bundled: [
      { filename: "human.ogg", source: { type: "HumanAudio", text: `Mot ${hostileText}` } },
      { filename: "poster.jpg", source: { type: "Poster", imdb_id: "tt0001" } },
      { filename: "tts.mp3", source: { type: "Tts", url: "https://mock.invalid/bundle" } },
      { filename: "word.mp3", source: { type: "Tts", url: "https://mock.invalid/word" } },
      { filename: "failed.mp3", source: { type: "Tts", url: "https://mock.invalid/fail" } },
    ],
    stats: { sentence_count: 3, word_count: 2, card_count: 8 },
  };
  const realNow = Date.now;
  try {
    for (const [variant, reading, listening] of [
      ["reading", true, false],
      ["both", true, true],
      ["listening", false, true],
    ]) {
      // Even exports in the same second must update Anki’s conditional fields.
      Date.now = () => 1790167000000;
      const plan = structuredClone(base);
      for (const note of plan.notes) {
        if (note.type === "Sentence") {
          note.include_reading = reading;
          note.include_listening = listening;
        }
      }
      plan.stats.card_count = 2 + 3 * (Number(reading) + Number(listening));
      const progress = [];
      fetched = [];
      const blob = await buildApkg(plan, source => {
        const filename = source.type === "Poster" ? "poster.jpg" : "human.ogg";
        assert(source.type !== "Tts");
        return bundled.get(filename);
      }, value => progress.push(value));
      assert.deepEqual(fetched, ["https://mock.invalid/bundle", "https://mock.invalid/word", "https://mock.invalid/fail"]);
      assert.deepEqual(progress.map(value => value.done), [0, 1, 2, 3, 4, 5]);
      assert(progress.every(value => value.total === 5));
      writeFileSync(path.join(output, `${variant}.apkg`), Buffer.from(await blob.arrayBuffer()));
      writeFileSync(path.join(output, `${variant}.json`), JSON.stringify(plan));
      console.log(`Wrote ${variant}: ${plan.notes.length} notes, ${plan.stats.card_count - Number(listening)} cards`);
    }
  } finally {
    Date.now = realNow;
  }

  // A slow first request must not hold the other seven workers idle.
  const pending = new Map();
  const started = [];
  globalThis.fetch = url => new Promise(resolve => {
    started.push(url);
    pending.set(url, resolve);
  });
  const plan = { ...base, bundled: Array.from({ length: 10 }, (_, i) => ({
    filename: `pool-${i}.mp3`, source: { type: "Tts", url: `https://mock.invalid/${i}` },
  })) };
  const progress = [];
  const building = buildApkg(plan, () => assert.fail("only TTS"), value => progress.push(value));
  assert.equal(started.length, 8, "bounded initial concurrency");
  for (const i of [1, 2]) {
    pending.get(`https://mock.invalid/${i}`)(new Response(wav));
    await new Promise(resolve => setImmediate(resolve));
    assert.equal(started.length, 8 + i, "replace a finished request without waiting for request zero");
  }
  // Finish out of order, with one failure. Media indices still follow the plan.
  for (const i of [9, 8, 7, 6, 5, 4, 3, 0]) {
    pending.get(`https://mock.invalid/${i}`)(i === 7 ? new Response("", { status: 503 }) : new Response(wav));
  }
  const blob = await building;
  const files = unzipSync(new Uint8Array(await blob.arrayBuffer()));
  assert.deepEqual(Object.values(JSON.parse(strFromU8(files.media))),
    [0, 1, 2, 3, 4, 5, 6, 8, 9].map(i => `pool-${i}.wav`));
  assert.deepEqual(progress.map(value => value.done), Array.from({ length: 11 }, (_, i) => i));
  assert(progress.every(value => value.total === 10));
  assert.equal(new Set(started).size, 10, "each request runs once");
  await assert.rejects(buildApkg(base, () => undefined, () => {}), /Bundled media missing/);
  console.log("Media pool: bounded concurrency, straggler refill, ordered output and failure handling passed");
}
