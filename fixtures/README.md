# Challenge parity fixtures

`challenges/*.json` are real French challenges captured from the throwaway test
account, plus the live reducer state for dictation and translation (`null` for
the others). Both hosts restore the same state and grading from these captures.
The captured set covers empty/typed/perfect/wrong dictation and translation,
written/listening flashcards, and pronunciation.
They are intentional test inputs: commit them, not generated bindings or screenshots.

## Capture on iOS

Build with `python3 yap-ios/build.py --debug --simulator <UDID>`, then launch:

```sh
xcrun simctl launch --terminate-running-process <UDID> town.yap.ios \
  --test-credentials yap-mcp-test@popovit.ch "$YAP_TEST_USER_PASSWORD" --test-driver
CONTAINER=$(xcrun simctl get_app_container <UDID> town.yap.ios data)
printf 'dump-fixture my-name' > "$CONTAINER/tmp/yap-command"
```

The test deck rarely schedules a translation challenge; `force-translation` poses
one regardless of what is due, then `type`, `type-reference`, `submit`, and
`continue` drive it before each `dump-fixture`.

Wait for `fixture written` in `$CONTAINER/tmp/yap-test.log`, then copy
`$CONTAINER/tmp/fixtures/my-name.json` into `fixtures/challenges/` unchanged.
Use `status`, `reveal` / `grade`, `dismiss-step`, `add-listening`, and
`add-pronunciation` to reach challenges. For dictation, capture before typing,
after `type <partial word>`, or after `type-reference` / `submit` (wait for
`transcription graded`). A wrong word followed by `submit` captures wrong grading.
Send commands separately, at least 1.5 seconds apart; `continue` advances dictation.
Only use `yap-mcp-test@popovit.ch`; reviews append real events to that test account.

## Render

- **Web:** build production WASM with
  `CARGO_PROFILE_RELEASE_LTO=true cargo bridgerton web --package yap-frontend-rs --release`,
  then `cd yap-frontend && pnpm dev`. Select French and open `/fixture/<name>`.
  Fixture routes and `/__fixtures/index.json` exist only in development. The page
  parses JSON in TypeScript and converts `transcription.inputs` into a JavaScript
  `Map`; the production WASM does not include the fixture JSON codec.
- **iOS:** launch the debug app with the test credentials above and
  `--fixture <absolute-path-to-json>`. The deck still comes from the signed-in
  French test account. The driver log announces `fixture rendered <name>`.
  Dictation fixtures do not overwrite normal pending-review storage. iOS builds
  enable the Rust `fixtures` feature, which exposes `parse_challenge_fixture` and
  `challenge_fixture_json`. Those bridge functions are otherwise available only
  in Rust tests; the `ChallengeFixture` type is always available.

## Side-by-side screenshots

From the repo root (requires the built WASM package, pnpm dependencies, Xcode,
and the iPhone 17 Pro simulator):

```sh
# Once, if Chromium is missing:
(cd yap-frontend && pnpm exec playwright install chromium)
YAP_TEST_USER_PASSWORD=... python3 fixtures/parity.py --out /tmp/parity
open /tmp/parity/index.html
```

The script builds/installs iOS, runs Playwright, and writes `<name>-web.png`,
`<name>-ios.png`, and `index.html`. Options: `--only <name>`, `--web-only`,
`--ios-only`, `--no-build` (reuse the iOS build), `--simulator <UDID>`.
Web capture uses a seeded offline French deck; iOS uses the test account's deck.
Deck-dependent disclosures, chrome, keyboard, and media can therefore differ;
these are visual comparison fixtures, not pixel-equality assertions.
