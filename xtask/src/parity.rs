use crate::{Result, command, ios, root, run as exec, text};
use clap::Args as ClapArgs;
use std::{
    env, fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

const TEST_EMAIL: &str = "yap-mcp-test@popovit.ch";

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, default_value = "/tmp/parity")]
    out: PathBuf,
    /// Comma-separated fixture names or prefixes, e.g. translation.
    #[arg(long)]
    only: Option<String>,
    #[arg(long, conflicts_with = "ios_only")]
    web_only: bool,
    #[arg(long)]
    ios_only: bool,
    /// Skip both WASM and iOS builds.
    #[arg(long)]
    no_build: bool,
    #[arg(long, default_value = "8BC69AEC-1D85-4EE0-B979-74A15AE359DA")]
    simulator: String,
    #[arg(long, default_value = TEST_EMAIL)]
    email: String,
}

pub fn run(args: Args) -> Result<()> {
    if args.email != TEST_EMAIL {
        return Err("Only the throwaway yap-mcp-test@popovit.ch account may be used".into());
    }
    let password = env::var("YAP_TEST_USER_PASSWORD").unwrap_or_default();
    if !args.web_only && password.is_empty() {
        return Err("Set YAP_TEST_USER_PASSWORD for the iOS test account".into());
    }
    let mut fixtures = Vec::new();
    for directory in fs::read_dir(root().join("fixtures"))? {
        let directory = directory?.path();
        if !directory.is_dir() {
            continue;
        }
        for fixture in fs::read_dir(directory)? {
            let fixture = fixture?.path();
            if fixture.extension().is_some_and(|ext| ext == "json")
                && fixture.is_file()
                && matches(
                    fixture
                        .file_stem()
                        .unwrap()
                        .to_str()
                        .ok_or("non-UTF8 fixture name")?,
                    args.only.as_deref(),
                )
            {
                fixtures.push(fixture);
            }
        }
    }
    fixtures.sort();
    if fixtures.is_empty() {
        return Err("No matching fixtures".into());
    }
    fs::create_dir_all(&args.out)?;
    let out = args.out.canonicalize()?;
    if !args.ios_only {
        if !args.no_build {
            exec(
                command("cargo")
                    .args([
                        "bridgerton",
                        "web",
                        "--package",
                        "yap-frontend-rs",
                        "--release",
                    ])
                    .env("CARGO_PROFILE_RELEASE_LTO", "true"),
            )?;
        }
        let mut playwright = command("pnpm");
        playwright.current_dir(root().join("yap-frontend")).args([
            "exec",
            "playwright",
            "test",
            "e2e/fixtures.spec.ts",
        ]);
        if let Some(only) = &args.only {
            playwright.env("YAP_FIXTURE", only);
        }
        exec(&mut playwright)?;
        for fixture in &fixtures {
            let name = format!("{}-web.png", fixture.file_stem().unwrap().to_string_lossy());
            fs::copy(
                root()
                    .join("yap-frontend/screenshots-out/fixtures")
                    .join(&name),
                out.join(name),
            )?;
        }
    }
    if !args.web_only {
        if !args.no_build {
            ios::run(ios::Args {
                debug: true,
                simulator_build_only: true,
                ..Default::default()
            })?;
        }
        exec(command("xcrun").args(["simctl", "bootstatus", &args.simulator, "-b"]))?;
        exec(
            command("xcrun")
                .args(["simctl", "install", &args.simulator])
                .arg(
                    root().join("yap-ios/DerivedData/Build/Products/Debug-iphonesimulator/Yap.app"),
                ),
        )?;
        let container = text(command("xcrun").args([
            "simctl",
            "get_app_container",
            &args.simulator,
            ios::BUNDLE,
            "data",
        ]))?;
        let log = PathBuf::from(container).join("tmp/yap-test.log");
        for fixture in &fixtures {
            let name = fixture.file_stem().unwrap().to_string_lossy();
            fs::write(&log, "")?;
            exec(
                command("xcrun")
                    .args([
                        "simctl",
                        "launch",
                        "--terminate-running-process",
                        &args.simulator,
                        ios::BUNDLE,
                        "--test-credentials",
                        &args.email,
                        &password,
                        "--fixture",
                    ])
                    .arg(fixture),
            )?;
            let deadline = Instant::now() + Duration::from_secs(60);
            loop {
                let contents = fs::read_to_string(&log)?;
                if contents.contains(&format!("fixture rendered {name}\n")) {
                    break;
                }
                if Instant::now() > deadline {
                    return Err(format!("{name} did not render:\n{contents}").into());
                }
                thread::sleep(Duration::from_millis(250));
            }
            thread::sleep(Duration::from_millis(2500));
            exec(
                command("xcrun")
                    .args(["simctl", "io", &args.simulator, "screenshot"])
                    .arg(out.join(format!("{name}-ios.png"))),
            )?;
        }
    }
    let platforms: Vec<_> = [("web", !args.ios_only), ("ios", !args.web_only)]
        .into_iter()
        .filter_map(|(platform, enabled)| enabled.then_some(platform))
        .collect();
    let names: Vec<_> = fixtures
        .iter()
        .map(|fixture| fixture.file_stem().unwrap().to_string_lossy())
        .collect();
    fs::write(
        out.join("index.html"),
        gallery(names.iter().map(|name| name.as_ref()), &platforms),
    )?;
    println!("Gallery: {}", out.join("index.html").display());
    Ok(())
}

fn matches(name: &str, only: Option<&str>) -> bool {
    only.is_none_or(|only| only.split(',').any(|prefix| name.starts_with(prefix)))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn gallery<'a>(names: impl Iterator<Item = &'a str>, platforms: &[&str]) -> String {
    let mut html = "<!doctype html><meta charset='utf-8'><title>Screen parity</title><style>body{font-family:system-ui;margin:2rem}section>div{display:flex;gap:1rem}figure{margin:0;max-width:45%}img{width:100%;max-width:430px}</style>".to_owned();
    for name in names {
        let name = escape(name);
        html.push_str(&format!("<section><h2>{name}</h2><div>"));
        for platform in platforms {
            html.push_str(&format!("<figure><figcaption>{platform}</figcaption><img src=\"{name}-{platform}.png\"></figure>"));
        }
        html.push_str("</div></section>");
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filtering_and_escaped_gallery() {
        assert!(matches("translation-ready", Some("idle,translation")));
        assert!(!matches("dictation", Some("idle,translation")));
        assert!(matches("anything", None));
        let html = gallery(["a<&\"'"].into_iter(), &["web"]);
        assert!(html.contains("a&lt;&amp;&quot;&#x27;-web.png"));
        assert!(!html.contains("-ios.png"));
    }
}
