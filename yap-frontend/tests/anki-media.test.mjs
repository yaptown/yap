import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";

const source = ts.transpileModule(
  readFileSync(new URL("../src/anki/media.ts", import.meta.url), "utf8"),
  { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } },
).outputText;
const flush = () => new Promise(resolve => setImmediate(resolve));
function harness(fetch, count) {
  const timers = new Map();
  let nextTimer = 0;
  const exports = {};
  vm.runInNewContext(source, {
    exports, Error, AbortController, Uint8Array, fetch,
    setTimeout: (fn, ms) => { assert.equal(ms, 60000); timers.set(++nextTimer, fn); return nextTimer; },
    clearTimeout: id => timers.delete(id),
  });
  const plan = { bundled: Array.from({ length: count }, (_, i) => ({ filename: `${i}.mp3`, source: { type: "Tts", url: String(i) } })) };
  return { timers, run: () => exports.fetchMedia(plan, () => assert.fail("unexpected pack media"), () => {}) };
}

test("a stalled response body is aborted and retried only once", async () => {
  let attempts = 0;
  const h = harness(async (_, { signal }) => {
    attempts++;
    return { ok: true, status: 200, arrayBuffer: () => new Promise((_, reject) => {
      signal.addEventListener("abort", () => reject(signal.reason), { once: true });
    }) };
  }, 1);
  const pending = h.run();
  await flush();
  assert.equal(attempts, 1);
  [...h.timers.values()][0]();
  await flush();
  assert.equal(attempts, 2);
  [...h.timers.values()][0]();
  assert.equal((await pending)[0], undefined);
  assert.equal(attempts, 2);
  assert.equal(h.timers.size, 0);
});

test("a 429 is retried once like any other failure", async () => {
  let attempts = 0;
  const h = harness(async () => ++attempts === 1 ? new Response("", { status: 429 }) : new Response(new Uint8Array([1])), 1);
  assert.deepEqual((await h.run())[0], new Uint8Array([1]));
  assert.equal(attempts, 2);
});

test("32 exhausted recordings stop a large failing queue", async () => {
  const attempts = new Map();
  const h = harness(async url => {
    attempts.set(url, (attempts.get(url) ?? 0) + 1);
    return new Response("", { status: 503 });
  }, 100);
  await assert.rejects(h.run(), error => error.code === "audio_unavailable");
  await flush();
  assert([...attempts.values()].filter(count => count === 2).length >= 32);
  assert([...attempts.values()].every(count => count <= 2));
  assert(attempts.size <= 55, "only 23 already-running requests may remain at the cutoff");
  assert.equal(h.timers.size, 0);
});

test("missing subtitles are optional but pack media must exist", async () => {
  const exports = {};
  vm.runInNewContext(source, { exports, Error, AbortController });
  const run = type => exports.fetchMedia({ bundled: [{ filename: "media", source: { type } }] }, async () => undefined, () => {});
  assert.equal((await run("Subtitles"))[0], undefined);
  await assert.rejects(run("Poster"), /Bundled media missing: media/);
  await assert.rejects(run("HumanAudio"), /Bundled media missing: media/);
});
