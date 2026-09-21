use crate::{
    Result, command,
    packs::{PackServer, Stats},
    root, run as exec, text,
};
use clap::Args as ClapArgs;
use serde_json::Value;
use std::{collections::HashSet, fs, path::PathBuf};

#[derive(ClapArgs)]
pub struct Args {
    /// Directory containing fra_for_eng split language-pack files (default: workspace out/).
    #[arg(long)]
    packs: Option<PathBuf>,
}

pub fn run(args: Args) -> Result<()> {
    let packs = args.packs.unwrap_or_else(|| root().join("out"));
    for part in ["core", "sentences"] {
        if !packs
            .join(format!("fra_for_eng/language_data_{part}.rkyv"))
            .is_file()
        {
            return Err(format!(
                "missing French {part} language pack under {}/fra_for_eng",
                packs.display()
            )
            .into());
        }
    }
    let packs = packs.canonicalize()?;
    exec(command("cargo").args([
        "test",
        "-p",
        "yap-frontend-rs",
        "language_pack::native_tests",
        "--lib",
        "--locked",
    ]))?;
    exec(command("cargo").args(["test", "-p", "yap-ios-host", "--lib", "--locked"]))?;
    let generated = root().join("yap-ios/.build/Bindings");
    exec(
        command("cargo")
            .args([
                "bridgerton",
                "swift",
                "--package",
                "yap-ios-host",
                "--out-dir",
            ])
            .arg(&generated)
            .arg("--locked"),
    )?;
    let metadata: Value = serde_json::from_slice(&fs::read(generated.join("build.json"))?)?;
    let triple = format!(
        "{}-apple-macosx{}",
        text(command("uname").arg("-m"))?,
        text(command("sw_vers").arg("-productVersion"))?
    );
    let here = root().join("yap-ios/smoke");
    let mut swift = command("swiftc");
    swift
        .args([
            "-target",
            &triple,
            "-swift-version",
            "6",
            "-strict-concurrency=complete",
            "-warnings-as-errors",
            "-parse-as-library",
            "-I",
        ])
        .arg(&generated)
        .arg(generated.join("Bridge.swift"))
        .arg(here.join("Smoke.swift"));
    for key in ["archives", "native_static_libraries"] {
        for flag in metadata[key]
            .as_array()
            .ok_or_else(|| format!("missing {key} in build.json"))?
        {
            swift.arg(flag.as_str().ok_or("non-string link flag in build.json")?);
        }
    }
    exec(swift.arg("-o").arg(generated.join("smoke")))?;
    let data = tempfile::Builder::new()
        .prefix("yap-ios-smoke-")
        .tempdir()?;
    let server = PackServer::start(&packs)?;
    exec(
        command(generated.join("smoke"))
            .arg(here)
            .env("YAP_DATA_DIR", data.path())
            .env("YAP_PACKS_URL", &server.url),
    )?;
    let stats = server.finish()?;
    check_downloads(&stats)?;
    println!(
        "PASS: {} HTTP chunks; cache reopen needed no downloads",
        stats.downloads.len()
    );
    println!("PASS: real Yap Swift integration");
    Ok(())
}

fn check_downloads(stats: &Stats) -> Result<()> {
    if stats
        .downloads
        .iter()
        .map(|(part, _)| *part)
        .collect::<HashSet<_>>()
        != HashSet::from(["core", "sentences"])
    {
        return Err("expected downloads of both core and sentences packs".into());
    }
    if stats.downloads.len() != stats.downloads.iter().collect::<HashSet<_>>().len() {
        return Err("a cached chunk was downloaded again".into());
    }
    if !stats.offline || stats.offline_downloads != 0 {
        return Err("reopening attempted an HTTP download".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_three_cache_invariants() {
        let mut stats = Stats {
            downloads: vec![("core", 0), ("sentences", 0)],
            offline: true,
            offline_downloads: 0,
        };
        assert!(check_downloads(&stats).is_ok());
        stats.downloads.push(("core", 0));
        assert!(check_downloads(&stats).is_err());
        stats.downloads.pop();
        stats.offline_downloads = 1;
        assert!(check_downloads(&stats).is_err());
        stats.offline_downloads = 0;
        stats.offline = false;
        assert!(check_downloads(&stats).is_err());
        stats.offline = true;
        stats.downloads.pop();
        assert!(check_downloads(&stats).is_err());
    }
}
