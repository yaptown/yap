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
Use an iPhone portrait simulator; the default is the parity runner's iPhone.
All existing parity prerequisites and its throwaway test account apply.

To change only the layout or copy without recapturing:

```sh
pnpm --dir scripts/app-store-screenshots generate --render-only
```

Edit the `slides` list and SVG layout in `generate.cjs`. Output is gitignored
under `.cache/app-store/screenshots/`: four numbered 1284 × 2778 opaque PNGs
for upload, editable SVGs, a combined `preview.png`, and `copy.json`.
Upload only the four numbered PNGs. Review the screenshots against the build
being submitted, particularly any film content and completed content audit.
