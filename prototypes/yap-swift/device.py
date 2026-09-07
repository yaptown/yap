#!/usr/bin/env python3
"""Build, sign, and run Yap on a paired iPhone using its real target-generated bindings."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shlex
import subprocess
import time
import uuid

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
OUT = HERE / "generated" / "iphone"
TARGET = "aarch64-apple-ios"
MINIMUM = "18.0"


def run(*args, **kwargs):
    print("+ " + shlex.join(map(str, args)), flush=True)
    return subprocess.run(list(map(str, args)), cwd=ROOT, check=True, **kwargs)


def project(name, bundle, sources, archive, libraries, team, bindings=None, header=None):
    """A disposable Xcode project supplies Apple's automatic development signing."""
    directory = OUT / f"{name}.xcodeproj"
    directory.mkdir(parents=True, exist_ok=True)
    objects = {}

    def obj(isa, **fields):
        key = f"{len(objects) + 1:024X}"
        objects[key] = {"isa": isa, **fields}
        return key

    files, builds = [], []
    for source in sources:
        reference = obj("PBXFileReference", lastKnownFileType="sourcecode.swift", path=str(source), sourceTree="<absolute>")
        files.append(reference)
        builds.append(obj("PBXBuildFile", fileRef=reference))
    product = obj("PBXFileReference", explicitFileType="wrapper.application", path=f"{name}.app", sourceTree="BUILT_PRODUCTS_DIR")
    group = obj("PBXGroup", children=[*files, product], sourceTree="<group>")
    sources_phase = obj("PBXSourcesBuildPhase", buildActionMask=2147483647, files=builds, runOnlyForDeploymentPostprocessing=0)
    settings = {
        "PRODUCT_NAME": name, "PRODUCT_BUNDLE_IDENTIFIER": bundle,
        "DEVELOPMENT_TEAM": team, "CODE_SIGN_STYLE": "Automatic",
        "SDKROOT": "iphoneos", "SUPPORTED_PLATFORMS": "iphoneos", "ARCHS": "arm64",
        "IPHONEOS_DEPLOYMENT_TARGET": MINIMUM, "TARGETED_DEVICE_FAMILY": "1",
        "SWIFT_VERSION": "6.0", "SWIFT_STRICT_CONCURRENCY": "complete",
        "SWIFT_OPTIMIZATION_LEVEL": "-O", "GENERATE_INFOPLIST_FILE": "YES",
        "INFOPLIST_KEY_CFBundleDisplayName": "Yap" if bindings else "Yap Bindings",
        "INFOPLIST_KEY_UILaunchScreen_Generation": "YES",
        "INFOPLIST_KEY_UIApplicationSceneManifest_Generation": "YES",
        "MARKETING_VERSION": "0.1", "CURRENT_PROJECT_VERSION": "1",
        "OTHER_LDFLAGS": [f"-Wl,-force_load,{archive}", *libraries],
        "ENABLE_USER_SCRIPT_SANDBOXING": "YES",
        "ALWAYS_SEARCH_USER_PATHS": "NO",
    }
    if bindings:
        settings["SWIFT_INCLUDE_PATHS"] = [str(bindings)]
    if header:
        settings["SWIFT_OBJC_BRIDGING_HEADER"] = str(header)

    def configs(settings):
        config = obj("XCBuildConfiguration", name="Release", buildSettings=settings)
        return obj("XCConfigurationList", buildConfigurations=[config], defaultConfigurationIsVisible=0, defaultConfigurationName="Release")

    target = obj("PBXNativeTarget", name=name, productName=name, productReference=product,
                 productType="com.apple.product-type.application", buildConfigurationList=configs(settings),
                 buildPhases=[sources_phase], buildRules=[], dependencies=[])
    root = obj("PBXProject", compatibilityVersion="Xcode 14.0", developmentRegion="en",
               knownRegions=["en", "Base"], mainGroup=group, projectDirPath="", projectRoot="",
               buildConfigurationList=configs({}), targets=[target],
               attributes={"LastUpgradeCheck": "2620"})
    (directory / "project.pbxproj").write_bytes(plistlib.dumps({
        "archiveVersion": "1", "objectVersion": "56", "classes": {}, "objects": objects, "rootObject": root,
    }))
    return directory


def build_app(project_path, name, device):
    derived = OUT / f"build-{name}"
    run("xcodebuild", "-project", project_path, "-scheme", name, "-configuration", "Release",
        "-destination", f"id={device}", "-derivedDataPath", derived,
        "-allowProvisioningUpdates", "-allowProvisioningDeviceRegistration", "build")
    return derived / "Build/Products/Release-iphoneos" / f"{name}.app"


def install_launch(app, bundle, device, *arguments):
    run("xcrun", "devicectl", "device", "install", "app", "--device", device, app)
    run("xcrun", "devicectl", "device", "process", "launch", "--device", device,
        "--terminate-existing", bundle, *arguments)


def copy_from(device, bundle, source, destination):
    return subprocess.run([
        "xcrun", "devicectl", "device", "copy", "from", "--device", device,
        "--domain-type", "appDataContainer", "--domain-identifier", bundle,
        "--source", source, "--destination", str(destination),
    ], capture_output=True, text=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--device", required=True, help="paired iPhone UDID")
    parser.add_argument("--team", required=True, help="Apple development team ID")
    parser.add_argument("--reuse-bindings", action="store_true", help="reuse verified bindings from an earlier device run")
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    built = run("cargo", "rustc", "-p", "yap-swift-prototype", "--lib", "--release", "--locked",
                "--target", TARGET, "--message-format=json-render-diagnostics", "--", "--print=native-static-libs",
                env={**os.environ, "IPHONEOS_DEPLOYMENT_TARGET": MINIMUM}, capture_output=True, text=True)
    print(built.stderr, flush=True)
    artifacts = [json.loads(line) for line in built.stdout.splitlines() if line.startswith("{")]
    archives = [Path(name) for artifact in artifacts if artifact.get("reason") == "compiler-artifact"
                and artifact["target"]["name"] == "yap_swift_prototype"
                for name in artifact["filenames"] if name.endswith(".a")]
    [archive] = archives
    [native] = [line.removeprefix("note: native-static-libs: ") for line in built.stderr.splitlines()
                if line.startswith("note: native-static-libs: ")]
    libraries = shlex.split(native)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    bindings = OUT / "Bindings"
    stamp = OUT / "device-generation.json"
    if args.reuse_bindings:
        if json.loads(stamp.read_text())["archive_sha256"] != digest:
            parser.error("Rust archive changed; generate the bindings on the device again")
    else:
        header = OUT / "Metadata.h"
        header.write_text(f'#include "{ROOT / "libraries/bridgerton/src/runtime.h"}"\n'
                          'BridgeResult bridgerton_generate_v1(const uint8_t *, size_t);\n')
        bundle = "town.yap.prototype.bindings"
        path = project("YapBindings", bundle, [HERE / "swift/DeviceMetadata.swift"], archive, libraries, args.team, header=header)
        app = build_app(path, "YapBindings", args.device)
        generation_id = uuid.uuid4().hex
        install_launch(app, bundle, args.device, "--generation-id", generation_id)
        for attempt in range(30):
            copied = copy_from(args.device, bundle, f"Documents/Bindings-{generation_id}", bindings)
            if copied.returncode == 0:
                break
            time.sleep(1)
        else:
            raise RuntimeError(f"Could not retrieve device-generated bindings: {copied.stderr}")
        for file in ("Bridge.swift", "BridgeFFI.h", "module.modulemap"):
            if not (bindings / file).is_file():
                raise RuntimeError(f"Device did not produce {file}")
        stamp.write_text(json.dumps({"target": TARGET, "device": args.device, "archive_sha256": digest}, indent=2) + "\n")
        print("PASS: bindings generated by the actual iPhone library", flush=True)
    bundle = "town.yap.prototype.iphone"
    path = project("Yap", bundle, [bindings / "Bridge.swift", HERE / "swift/App.swift", HERE / "swift/DeviceChecks.swift"], archive, libraries, args.team, bindings=bindings)
    app = build_app(path, "Yap", args.device)
    run_id = uuid.uuid4().hex
    install_launch(app, bundle, args.device, "--device-check", "--device-check-id", run_id)
    report = OUT / "device-check.json"
    for attempt in range(180):
        copied = copy_from(args.device, bundle, "Documents/device-check.json", report)
        if copied.returncode == 0:
            result = json.loads(report.read_text())
            if result.get("run_id") == run_id:
                if not result["passed"]:
                    raise RuntimeError(f"Device integration failed: {result}")
                print("PASS: physical iPhone integration: " + ", ".join(result["checks"]), flush=True)
                break
        if attempt % 15 == 0:
            print("Waiting for the iPhone to download the language pack and finish its checks…", flush=True)
        time.sleep(2)
    else:
        raise RuntimeError("Timed out waiting for the iPhone test report; inspect the app's error message")
    # Leave the interactive app open using its own persistent sandbox.
    run("xcrun", "devicectl", "device", "process", "launch", "--device", args.device,
        "--terminate-existing", bundle)
    if not args.reuse_bindings:
        run("xcrun", "devicectl", "device", "uninstall", "app", "--device", args.device,
            "town.yap.prototype.bindings")
    print(f"Installed and launched Yap on {args.device}. Build: {app}", flush=True)


if __name__ == "__main__":
    main()
