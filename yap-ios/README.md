# Yap for iOS

The native app links `yap-frontend-rs::Weapon` and `Deck` directly through
[Bridgerton](../libraries/bridgerton). The SwiftUI app provides email/password authentication, course selection,
offline deck loading, authenticated sync, audio, vocabulary/listening flashcards,
pronunciation practice, translation, and dictation challenges. Native onboarding
covers referral, motivation, experience, spaced repetition, and daily goals.
The review ladder includes placement, study plans, display-name setup, daily
accomplishments, and an inline essential/movie/Pimsleur sentence-list selector.
Once a course is loaded, five native tabs each own a navigation stack:

- **Learn** keeps the existing review ladder and challenge state. Switching tabs
  stops playback, but the tab container owns the review model, so its due timers
  and audio prefetch keep running, including after offscreen Deck replacements.
- **Stats** shows today's summary, week progress, an editable daily goal, XP,
  cards, word knowledge, streak, tier, review totals, the interactive frequency
  knowledge chart (with a data table), movie posters/progress, and Pimsleur progress.
- **Dictionary** searches words and meanings in 200-entry pages. Definitions,
  morphology, and audio use the review components; adding a word appends an event.
- **Lists** shares its snapshot model and progress view with the inline selector;
  essential, movie, and Pimsleur choices append an event and return to Learn.
- **Settings** contains sync diagnostics, account/display-name editing, course
  switching, sign-out, version, privacy, and terms. “Push pending events” enables
  upload of missing events; it never overwrites history. Normal sync also uploads.

Human recordings credit their actual `AudioResult.voice_actor` in a non-blocking
banner. `voice-actor-toast:<name>` in UserDefaults throttles each actor to once
per 24 hours, across relaunches and courses. The banner also works in Dictionary,
so an audio preview cannot consume a credit invisibly.
Placement survives core-to-full pack and sync-driven deck replacements. Wizard
answers are ephemeral until completion (referral is saved immediately); closing
an unfinished wizard returns to course selection. A new course is prefetched
immediately, but `SelectBothLanguages` is emitted only after the wizard's
`SetOnboardingSelections`, matching the web: the shared fold treats selection as
onboarded, so emitting it early would incorrectly turn abandoned wizards into
Resume entries.

## Build and run

Requires macOS, Xcode with Swift 6.2+, XcodeGen (`brew install xcodegen`), Rust,
and the `aarch64-apple-ios` and `aarch64-apple-ios-sim` Rust targets. Run from the
repository root:

```sh
python3 yap-ios/build.py                         # libraries, bindings, Xcode project
python3 yap-ios/build.py --simulator             # install/launch on the booted simulator
python3 yap-ios/build.py --simulator SIMULATOR_UUID
python3 yap-ios/build.py --simulator-build-only  # ad-hoc signed build; no booted simulator needed
python3 yap-ios/build.py --device IPHONE_UDID
```

Release is the default (`--release`); `--debug` selects debug Rust and Xcode
builds. Device builds use automatic signing with team `AF2CJ3G3ZU`; configure the
account in Xcode and enable Developer Mode on a paired iPhone. `--device` accepts
either its CoreDevice UUID or hardware UDID; the script resolves the Xcode destination. Simulator builds use ad-hoc signing with Keychain entitlements (no Apple account
required). Disabling signing prevents the auth SDK from persisting its session.
The app supports iPhone on iOS 18+. Supabase SwiftPM handles Keychain session persistence and token refresh.

The script builds both iOS static libraries, reads their actual paths and system
link flags from Cargo, generates `Generated/Link.xcconfig`, and runs XcodeGen.
`Yap.xcodeproj`, `DerivedData`, and `Generated` are disposable, ignored outputs.
Regenerate after Rust API changes before building in Xcode.

Bindings are generated **once on the macOS host**, by loading its cdylib:

```sh
cargo bridgerton swift --package yap-ios-host --release --out-dir yap-ios/Generated/Bindings
```

The app links the iOS static library, not the host library. Bridgerton's interface
fingerprint hashes the generated header and Swift text, not the binary. Host and
iOS simulator bindings have been verified byte-identical; the generated Swift
checks `bridgerton_abi_v1_matches` before the first application call to detect
drift. No XCFramework or on-device generation bootstrap is needed.

## Integration shape

- `YapHost.initialize()` owns a process-lifetime two-worker Tokio runtime and
  configures OPFS. The app does not set `YAP_DATA_DIR`: OPFS resolves the iOS
  sandbox's Application Support directory, preserving events between launches.
  Tests override the root with a temporary directory.
- `Weapon.create`, subscriptions, event ingestion, and `get_deck_state` retain
  the shared application flow. Generated Swift events use the existing immutable
  event serialization; there is no parallel native application model.
- SwiftUI Observation refreshes a `Deck` snapshot after listener notifications.
  Confined objects, futures, and Swift callbacks stay on the main actor; Tokio
  handles independent filesystem and decoding work.
- The shared progressive loader downloads immutable, hash-addressed packs from
  `https://packs.yap.town`, verifies and caches chunks, and reopens them offline.
  The app shows “French · for English speakers” when loading completes.
- `AuthStore` owns authentication; a user-keyed `YapSession` owns the Weapon,
  subscriptions, network monitoring, progressive pack loads, and 30-second sync.
  Leaving the session cancels tasks and consumes each subscription key once.
- A `ReviewModel` owns each immutable Deck snapshot. It holds the visible
  challenge until events change the Deck or the user changes restrictions;
  due timers and audio/manifest polling update readiness without swapping answers.
- `AudioPlayer` is the single playback registry. Cached Ogg Opus is losslessly
  remuxed to CAF by the shared, C-free Rust codec before native playback; cached
  bytes stay unchanged. Flashcards use shared Rust
  disclosure rules and the web's Again/Remembered row with Hard/Good/Easy in a menu.
- Sentence challenges use the same Rust grading, feedback and override functions
  as the web. Pending answers and grades live in account/course/challenge/version-
  scoped UserDefaults entries, including the original submission timestamp.
  Continue appends an immutable event at that timestamp and clears the draft.
  Offline grading stays local and exposes manual overrides.

## Smoke tests

With the French core and sentence packs present under `out/fra_for_eng`:

```sh
python3 yap-ios/smoke/check.py
# Or: python3 yap-ios/smoke/check.py --packs /path/to/packs
```

The harness builds the host Rust libraries, generates bindings, compiles
`smoke/Smoke.swift` with strict Swift 6 concurrency checks, and runs against a
loopback pack server and temporary event store. Its debug bindings stay in
`.build/Bindings`, separate from the app outputs. It verifies real event folding,
callbacks, object ownership, error transport, review APIs, persistence, chunked
loading, and an offline cache reopen. Ladder regressions also cover placement
across core/full-pack snapshots, onboarding/referral event folding, sentence-list
round trips, and a real backlog's lockup/release previews and events. Dictionary
regressions check two 200-entry pages against the existing search order, accent-insensitive queries, empty/overflow offsets, and the immutable
add-word event. No macOS app is built or opened.

Optional browser regression (requires the frontend's Playwright install):

```sh
CARGO_PROFILE_RELEASE_LTO=true cargo bridgerton web --package yap-frontend-rs --release --target web --out-dir ../yap-ios/Generated/web --features local-backend
node yap-ios/smoke/check-web.cjs
```

## Simulator interaction checks

Build with `--debug`, then opt into the debug-only driver:

```sh
xcrun simctl launch --terminate-running-process SIMULATOR_UUID town.yap.ios \
  --test-credentials yap-mcp-test@popovit.ch "$YAP_TEST_USER_PASSWORD" --test-driver
container=$(xcrun simctl get_app_container SIMULATOR_UUID town.yap.ios data)
printf 'status' > "$container/tmp/yap-command"
cat "$container/tmp/yap-test.log"
```

Commands: `switch-course`, `choose INDEX` (course/answer/placement word),
`next` (wizard/placement/plan/profile/accomplishment), `back` (wizard/placement),
`start-fresh` (wizard ready screen), `force-display-name`, `edit-goal`, `goal INDEX`,
`list essential`, `list movie`, `list pimsleur`, `acknowledge-pimsleur`, `skip` (display-name only),
and `add` (recommended cards),
`add-listening`, `add-pronunciation` (the manual add choices), `cant-listen`,
`cant-speak`, `undo`, `reveal`, `audio`, `grade` (Remembered, only after reveal
when required), `grade-again` (Forgot), `sync`, `status`, and `signout`. Sentence commands are `type TEXT`,
`type-reference` (the accepted answer), `submit`, `continue`, `tap-word INDEX`,
`toggle-phrase INDEX`, `focus PART_INDEX`, `accent CHARACTER`,
`toggle-word PART_INDEX WORD_INDEX` (mark Perfect), and `dismiss-keyboard`.
Pronunciation accepts `audio` and `pron-grade`. Additional browsing commands:

- `tab learn|stats|dictionary|lists|settings`
- `stats-today`, `stats-chart`, `stats-movies` scroll to those Stats sections
- `search TEXT`, `open INDEX`, `page` (next 200 results), `add-word`
- `select-list essential`, `select-list movie` (best suggestion),
  `select-list movie MOVIE_ID`, `select-list pimsleur` (best suggestion),
  `select-list pimsleur LEVEL LESSON`; Pimsleur requires `acknowledge-pimsleur`
- `sync`, `force-push` (both upload only missing events)
- `audio-text TEXT` previews an actual audio request in Learn without grading or
  replacing the challenge; useful for checking bundled actor credits and stopping
  playback across tabs. `status` includes tab/playback, dictionary, stats, and
  the finished sync timestamp. Review commands are ignored outside Learn.

Indexes are zero-based. `status`
also reports challenge kind and readiness counts; sentence completions report
submission timestamps. Write commands to a temporary file and rename it to
`yap-command` atomically to avoid the driver reading a partial write. They invoke the same view/model
handlers as taps. Review/add commands are restricted to the throwaway account;
no credentials are logged or persisted by the driver. All driver code is absent
from release builds. Use the repository's `setup_test_user.py` only for that
account, as described in the stage verification brief.

For an offline cache check, relaunch without credentials with `--test-offline`
and `SIMCTL_CHILD_YAP_PACKS_URL=http://127.0.0.1:9`. This disables event sync and
review network work, fails SDK HTTP requests, and points pack downloads at an
unreachable endpoint; the persisted session and cached deck must still open.

## Extension points

Push opt-in and production hardening remain. Dictionary paging is the one new
shared export for these tabs: `Deck.get_gram_dictionary_page(search_query, offset,
limit)`, preserving existing search relevance/frequency order while constructing
only the requested page's opaque entries. The bridge also needs a broader correctness
review before production; see its README for signature and ownership limits.
The foundation does not change stored events or build a second service layer.

## Movie clips

Sentence challenges fetch cached movie bytes through Rust's `get_clip`; the
native player writes a SHA-256(movie id + sentence)-keyed MP4 in the temporary
folder, reuses it while present, and removes it when the view disappears.
Captions follow the playhead; dictation hides blanked words until submission.
As on the web, translation clips own autoplay on submission; dictation retains
TTS autoplay and offers the clip for manual playback. The audio button remains
available in both. The critical start supplies a poster frame, **not** a loop:
replay starts at zero and plays the complete clip. All playback shares the
AudioPlayer interruption registry. Decode failures invalidate Rust's cache and
restore TTS. Missing clips are retried when the manifest version changes.

Debug-only driver commands `clip-text SENTENCE`, `clip-play`, `clip-fail` (inject playback failure), and `clip-close`
open/replay/close a preview using the same player and real `get_clip` path. This
is useful when the current review does not have a published clip; `status`
reports availability, playback, and the current subtitle.

## Sentry

Sentry Cocoa is initialized only if `SentryDSN` in the generated Info.plist is
nonempty. The default is empty. To enable it locally, create the gitignored
`yap-ios/Local.xcconfig`:

```xcconfig
// xcconfig treats // as a comment; the empty expansion preserves the URL.
SENTRY_DSN = https:/$()/YOUR_PUBLIC_DSN
```

`Generated/Link.xcconfig` includes that file optionally. A command-line Xcode
`SENTRY_DSN=...` setting also overrides the default. Release uses environment
`production`, Debug uses `development`, release is Rust's `get_app_version()`,
tracing samples 20%, and default PII is enabled to match the web. Authentication
sets/clears the Sentry user id; pack loading and sync failures leave breadcrumbs.
**No iOS session replay.** dSYM upload is not wired yet: configure a Sentry auth
token and symbol-upload CI step before relying on readable release crash stacks.

## Distribution resources and archive

`Assets.xcassets` contains the universal 1024px AppIcon (upscaled from the web's
512px maskable artwork), dynamic AccentColor matching the Swift theme, and the
system-grouped launch background. `PrivacyInfo.xcprivacy` declares UserDefaults
(CA92.1), sandbox file metadata (C617.1), no tracking, and account/product interaction/crash/performance data.
OPFS's native backend uses `metadata`/`stat` for file type and size (Apple's
file-timestamp category includes these metadata accesses even without reading
modification dates). No disk-space or boot-time API is used by the app. SDKs
supply their own privacy manifests. HTTPS-only encryption is declared exempt.

```sh
python3 yap-ios/build.py --archive
```

This makes a Release archive at `DerivedData/Yap.xcarchive`, then exports with
`method=app-store-connect`, automatic signing, team `AF2CJ3G3ZU`, and symbols
included to `DerivedData/export/`. It **never uploads**. `--archive --debug` is
rejected. Marketing version is 1.0, build number 1; increment the build number
before uploading another build of the same version.

### Release checklist

- In App Store Connect, create the `town.yap.ios` app under the signing team;
  accept agreements and complete metadata, age rating, screenshots, privacy
  disclosures/policy URL, export compliance, and review account/instructions.
- Ensure distribution signing/provisioning is available; archive/export, then
  manually upload with Xcode Organizer or Transporter. Configure TestFlight
  testers and complete beta review where required. Nothing here uploads for you.
- Exercise sign-in, sync, audio/video, offline reopening, and sign-out on a real
  device. Review account-deletion requirements before App Store submission.
- Create/configure the iOS Sentry project and production DSN; configure dSYM
  upload with a protected auth token, then verify a test event and readable stack.
  Check PII retention/access and keep App Store privacy answers consistent.
