# Screen parity fixtures

`challenges/*.json` and `screens/*.json` are real screens captured from the
throwaway test account. Each file is a tagged Rust `Fixture` (`type` plus `view`):
`Challenge`, `Idle`, or `Accomplishment`. Challenge captures include the live
reducer state for dictation and translation. Screen captures contain the entire
Rust view, including preview cards, progress, copy, and button events. They are
intentional test inputs: commit them, not generated bindings or screenshots.

Screen coverage: French caught-up and review-plan views, Italian first-run, and
German needs-more-cards. Audio-pending and accomplishment variants are supported
but do not yet have live captures.

## Capture on iOS

Build with `cargo xtask ios --debug --simulator <UDID>`, then launch:

```sh
xcrun simctl launch --terminate-running-process <UDID> town.yap.ios \
  --test-credentials yap-mcp-test@popovit.ch "$YAP_TEST_USER_PASSWORD" --test-driver
CONTAINER=$(xcrun simctl get_app_container <UDID> town.yap.ios data)
printf 'dump-fixture my-name' > "$CONTAINER/tmp/yap-command"
```

`dump-fixture` captures the visible challenge, idle screen, review plan (including
an expanded "Study more" plan), or accomplishment. Wait for `fixture written` in
`$CONTAINER/tmp/yap-test.log`, then copy `$CONTAINER/tmp/fixtures/my-name.json`
unchanged into the appropriate directory. Re-query the container after installing
a new build; its path can change.

Use `status` to inspect the current screen, due counts, and today's goal progress.
Use `reveal` / `grade` (or `grade-again`), `type-reference` / `submit` / `continue`,
`dismiss-step`, `add`, `add-listening`, and `add-pronunciation` to reach screens.
`goal 0` selects the smallest daily goal; it does not retroactively award a goal
already passed. `switch-course`, `status`, then `choose N` selects another course.
The test deck rarely schedules translation; `force-translation` poses one,
then `type`, `type-reference`, `submit`, and `continue` drive it.
Send commands separately, at least 1.5 seconds apart, and wait for grading or
new status output before advancing. Only use `yap-mcp-test@popovit.ch`; normal
driver reviews append real, immutable events to that test account.

## Render

- **Web:** build WASM with
  `CARGO_PROFILE_RELEASE_LTO=true cargo bridgerton web --package yap-frontend-rs --release`,
  then `cd yap-frontend && pnpm dev`. Select French and open `/fixture/<name>`.
  The route and `/__fixtures/index.json` exist only in development. TypeScript
  parses the fixture and converts dictation's numeric input keys into a `Map`;
  production WASM does not include the fixture JSON codec. Screen text and data
  come from the capture, not the seeded web deck, including the target language.
- **iOS:** launch the debug app with test credentials and
  `--fixture <absolute-path-to-json>`. The driver logs `fixture rendered <name>`.
  Screen actions are no-ops, and challenge fixtures cannot append review events
  or overwrite normal pending-review storage. iOS enables Rust's `fixtures`
  feature for `parse_fixture` / `fixture_json`; the transparent view and fixture
  types are always available. Fixture rendering still uses the live app shell.

## Side-by-side screenshots

From the repo root (requires pnpm dependencies, Xcode, and the
simulator):

```sh
(cd yap-frontend && pnpm exec playwright install chromium)
YAP_TEST_USER_PASSWORD=... cargo xtask parity --out /tmp/parity
YAP_TEST_USER_PASSWORD=... cargo xtask parity --out /tmp/parity-screens \
  --only idle,accomplishment --no-build
open /tmp/parity/index.html
```

The task discovers both directories, builds WASM and iOS, installs the iOS app,
runs Playwright, and writes `<name>-web.png`, `<name>-ios.png`, and `index.html`. Options: `--only`
(comma-separated names/prefixes), `--web-only`, `--ios-only`, `--no-build`,
`--simulator <UDID>`, and `--email` (restricted to the throwaway test account).
`--no-build` skips both WASM and iOS builds. Names must be unique across both fixture directories.

These are visual comparisons, not pixel-equality assertions. Platform-local
pickers, posters, acknowledgements, confetti, engagement prompts, chrome, and
keyboards can differ. Relative due dates also advance with the clock.
