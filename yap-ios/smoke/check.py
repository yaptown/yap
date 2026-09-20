#!/usr/bin/env python3
"""Run the real Yap offline Swift integration test (macOS, Swift 6.2+)."""
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile

from pack_server import serve_packs

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
# Keep debug smoke bindings separate from the app's release build outputs.
GENERATED = HERE.parent / ".build/Bindings"


def run(*args, env=None):
    print("+ " + " ".join(map(str, args)), flush=True)
    subprocess.run(list(map(str, args)), cwd=ROOT, env=env, check=True, timeout=600)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packs", type=Path, default=ROOT / "out", help="directory containing fra_for_eng split language-pack files")
    args = parser.parse_args()
    packs = args.packs.resolve()
    for part in ("core", "sentences"):
        if not (packs / "fra_for_eng" / f"language_data_{part}.rkyv").is_file():
            parser.error(f"missing French {part} language pack under {packs}/fra_for_eng")
    run("cargo", "test", "-p", "yap-frontend-rs", "language_pack::native_tests", "--lib", "--locked")
    run("cargo", "test", "-p", "yap-ios-host", "--lib", "--locked")
    run("cargo", "bridgerton", "swift", "--package", "yap-ios-host", "--out-dir", GENERATED, "--locked")
    metadata = json.loads((GENERATED / "build.json").read_text())
    # Match the local macOS version: locally built C dependencies can target the
    # host SDK. A distributable build needs an explicit shared deployment target.
    triple = f"{platform.machine()}-apple-macosx{platform.mac_ver()[0]}"
    swift = ("swiftc", "-target", triple, "-swift-version", "6", "-strict-concurrency=complete", "-warnings-as-errors", "-parse-as-library", "-I", GENERATED)
    link = (*metadata["archives"], *metadata["native_static_libraries"])
    run(*swift, GENERATED / "Bridge.swift", HERE / "Smoke.swift", *link, "-o", GENERATED / "smoke")
    with tempfile.TemporaryDirectory(prefix="yap-ios-smoke-") as data, serve_packs(packs) as server:
        run(GENERATED / "smoke", HERE, env={**os.environ, "YAP_DATA_DIR": data, "YAP_PACKS_URL": server.url})
        assert {part for part, _ in server.downloads} == {"core", "sentences"}
        assert len(server.downloads) == len(set(server.downloads)), "a cached chunk was downloaded again"
        assert server.offline and server.offline_downloads == 0, "reopening attempted an HTTP download"
        print(f"PASS: {len(server.downloads)} HTTP chunks; cache reopen needed no downloads")
    print("PASS: real Yap Swift integration")


if __name__ == "__main__":
    main()
