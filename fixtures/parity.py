#!/usr/bin/env python3
"""Capture the same saved challenges on web and iOS, with a side-by-side gallery."""
import argparse
import html
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent
BUNDLE = "town.yap.ios"
SIMULATOR = "8BC69AEC-1D85-4EE0-B979-74A15AE359DA"


def run(*args, **kwargs):
    return subprocess.run(list(map(str, args)), cwd=ROOT, check=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=Path("/tmp/parity"))
    parser.add_argument("--only", help="comma-separated fixture names or prefixes, e.g. translation")
    platforms = parser.add_mutually_exclusive_group()
    platforms.add_argument("--web-only", action="store_true")
    platforms.add_argument("--ios-only", action="store_true")
    parser.add_argument("--no-build", action="store_true")
    parser.add_argument("--simulator", default=SIMULATOR)
    parser.add_argument("--email", default="yap-mcp-test@popovit.ch")
    args = parser.parse_args()
    if args.email != "yap-mcp-test@popovit.ch":
        parser.error("Only the throwaway yap-mcp-test@popovit.ch account may be used")
    password = os.environ.get("YAP_TEST_USER_PASSWORD")
    if not args.web_only and not password:
        parser.error("Set YAP_TEST_USER_PASSWORD for the iOS test account")
    fixtures = sorted((ROOT / "fixtures/challenges").glob("*.json"))
    only = args.only.split(",") if args.only else []
    fixtures = [p for p in fixtures if not only or any(p.stem.startswith(o) for o in only)]
    if not fixtures:
        parser.error("No matching fixtures")
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    if not args.ios_only:
        env = dict(os.environ)
        if args.only:
            env["YAP_FIXTURE"] = args.only
        subprocess.run(["pnpm", "exec", "playwright", "test", "e2e/fixtures.spec.ts"],
                       cwd=ROOT / "yap-frontend", env=env, check=True)
        for fixture in fixtures:
            name = fixture.stem + "-web.png"
            shutil.copyfile(ROOT / "yap-frontend/screenshots-out/fixtures" / name, out / name)
    if not args.web_only:
        if not args.no_build:
            run("python3", "yap-ios/build.py", "--debug", "--simulator-build-only")
        run("xcrun", "simctl", "bootstatus", args.simulator, "-b")
        run("xcrun", "simctl", "install", args.simulator,
            ROOT / "yap-ios/DerivedData/Build/Products/Debug-iphonesimulator/Yap.app")
        container = Path(run("xcrun", "simctl", "get_app_container", args.simulator,
                             BUNDLE, "data", capture_output=True, text=True).stdout.strip())
        log = container / "tmp/yap-test.log"
        for fixture in fixtures:
            log.write_text("")
            run("xcrun", "simctl", "launch", "--terminate-running-process", args.simulator,
                BUNDLE, "--test-credentials", args.email, password, "--fixture", fixture)
            deadline = time.monotonic() + 60
            while f"fixture rendered {fixture.stem}\n" not in log.read_text():
                if time.monotonic() > deadline:
                    raise TimeoutError(f"{fixture.stem} did not render:\n{log.read_text()}")
                time.sleep(0.25)
            time.sleep(2.5)  # let images, layout, and keyboard settle
            run("xcrun", "simctl", "io", args.simulator, "screenshot", out / f"{fixture.stem}-ios.png")
    platforms = [p for p, enabled in [("web", not args.ios_only), ("ios", not args.web_only)] if enabled]
    rows = []
    for fixture in fixtures:
        name = html.escape(fixture.stem)
        images = "".join(f'<figure><figcaption>{p}</figcaption><img src="{name}-{p}.png"></figure>' for p in platforms)
        rows.append(f"<section><h2>{name}</h2><div>{images}</div></section>")
    (out / "index.html").write_text("<!doctype html><meta charset='utf-8'><title>Challenge parity</title>"
        "<style>body{font-family:system-ui;margin:2rem}section>div{display:flex;gap:1rem}"
        "figure{margin:0;max-width:45%}img{width:100%;max-width:430px}</style>"
        + "".join(rows))
    print(f"Gallery: {out / 'index.html'}")


if __name__ == "__main__":
    main()
