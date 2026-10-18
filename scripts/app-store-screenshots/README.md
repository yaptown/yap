# App Store screenshots

This uses `cargo xtask parity --ios-only` to render the existing translation,
home, flashcard, and stats fixtures in the native app, then wraps those captures
in phone frames with App Store copy. Screens are not recreated or generated.

From the repository root:

```sh
pnpm --dir scripts/app-store-screenshots install
pnpm --dir scripts/app-store-screenshots generate
```

The normal command builds the iOS app first. Pass `--no-build` to reuse an
existing simulator build, or `--simulator UUID` to choose another simulator.
`--device ipad` captures on the iPad Pro 13-inch simulator instead and lays the
same slides out at the 13-inch iPad size (the App Store requires one once the
app runs on iPad); the default is the parity runner's iPhone. All existing
parity prerequisites and its throwaway test account apply. The captures use
the test account's live course on that simulator, so make sure it is French
there (film titles follow the course language).

To change only the layout or copy without recapturing:

```sh
pnpm --dir scripts/app-store-screenshots generate --render-only
```

Edit the `slides` list and the per-device layouts in `generate.cjs`. Output is
gitignored under `.cache/app-store/screenshots/<device>/`: four numbered opaque
PNGs for upload (1284 × 2778 on iPhone, 2064 × 2752 on iPad), editable SVGs, a
combined `preview.png`, and `copy.json`.
Upload only the four numbered PNGs. Review the screenshots against the build
being submitted, particularly any film content and completed content audit.
