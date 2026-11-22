//! The service Dockerfiles build from a workspace trimmed to one binary's
//! path-dependency closure, so cargo-chef's planner never sees the whole repo.
//! The lists that trim it (manifests to copy, workspace members, stub target
//! files, source directories) are generated here from `cargo metadata` into
//! marked blocks, so adding a path dependency can't silently break a deploy.

use crate::{Result, command, root};
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// Each Dockerfile and the package its image builds.
const IMAGES: &[(&str, &str)] = &[
    ("yap-ai-backend/Dockerfile", "ai-backend"),
    ("yap-mcp/Dockerfile", "yap-mcp"),
];

#[derive(clap::Args)]
pub struct Args {
    /// Fail if a Dockerfile is out of date instead of rewriting it.
    #[arg(long)]
    check: bool,
}

pub fn run(args: Args) -> Result<()> {
    let workspace = Workspace::load()?;
    let mut stale = Vec::new();
    for &(dockerfile, package) in IMAGES {
        let path = root().join(dockerfile);
        let current = std::fs::read_to_string(&path)?;
        let updated = render(&current, &workspace.blocks(package))?;
        if updated != current {
            if args.check {
                stale.push(dockerfile);
            } else {
                std::fs::write(&path, updated)?;
                println!("updated {dockerfile}");
            }
        }
    }
    if !stale.is_empty() {
        return Err(format!(
            "{} out of date; run `cargo xtask dockerfiles`",
            stale.join(", ")
        )
        .into());
    }
    Ok(())
}

struct Package {
    /// Directory relative to the repo root.
    dir: String,
    /// Path dependencies (normal, build and dev), as package names.
    path_deps: Vec<String>,
    /// Target source files relative to the repo root, and whether each is a library.
    targets: Vec<(String, bool)>,
}

struct Workspace(HashMap<String, Package>);

impl Workspace {
    fn load() -> Result<Self> {
        let output = command("cargo")
            .args(["metadata", "--format-version", "1", "--no-deps"])
            .output()?;
        assert!(output.status.success(), "cargo metadata failed");
        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let root = root();
        let relative = |path: &str| {
            Path::new(path)
                .strip_prefix(&root)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        };
        let mut by_manifest_dir = HashMap::new();
        for package in metadata["packages"].as_array().unwrap() {
            let manifest = relative(package["manifest_path"].as_str().unwrap());
            let dir = Path::new(&manifest).parent().unwrap().to_str().unwrap();
            by_manifest_dir.insert(dir.to_owned(), package["name"].as_str().unwrap().to_owned());
        }
        let mut packages = HashMap::new();
        for package in metadata["packages"].as_array().unwrap() {
            let manifest = relative(package["manifest_path"].as_str().unwrap());
            let path_deps = package["dependencies"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|dep| dep["path"].as_str())
                .map(|path| by_manifest_dir[&relative(path)].clone())
                .collect();
            let targets = package["targets"]
                .as_array()
                .unwrap()
                .iter()
                .map(|target| {
                    let is_lib = target["kind"].as_array().unwrap().iter().any(|kind| {
                        matches!(
                            kind.as_str().unwrap(),
                            "lib" | "rlib" | "cdylib" | "staticlib" | "dylib" | "proc-macro"
                        )
                    });
                    (relative(target["src_path"].as_str().unwrap()), is_lib)
                })
                .collect();
            packages.insert(
                package["name"].as_str().unwrap().to_owned(),
                Package {
                    dir: Path::new(&manifest)
                        .parent()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                    path_deps,
                    targets,
                },
            );
        }
        Ok(Self(packages))
    }

    /// The package and everything it reaches through path dependencies,
    /// dev-dependencies included: cargo-chef loads every member's manifest.
    /// The image's own package comes first, the rest sorted by directory.
    fn closure(&self, package: &str) -> Vec<&Package> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![package.to_owned()];
        while let Some(name) = stack.pop() {
            if seen.insert(name.clone()) {
                stack.extend(self.0[&name].path_deps.iter().cloned());
            }
        }
        seen.remove(package);
        let mut rest: Vec<_> = seen.iter().map(|name| &self.0[name]).collect();
        rest.sort_by(|a, b| a.dir.cmp(&b.dir));
        std::iter::once(&self.0[package]).chain(rest).collect()
    }

    fn blocks(&self, package: &str) -> HashMap<&'static str, String> {
        let closure = self.closure(package);
        let manifests = closure
            .iter()
            .map(|p| format!("COPY {0}/Cargo.toml {0}/Cargo.toml\n", p.dir))
            .collect();
        let members: String = closure
            .iter()
            .map(|p| format!("\\n    \"{}\",", p.dir))
            .collect();
        let members =
            format!("RUN sed -i '/^members = \\[/,/^\\]/c\\members = [{members}\\n]' Cargo.toml\n");
        let stubs = closure
            .iter()
            .map(|p| {
                let dirs: BTreeSet<_> = p
                    .targets
                    .iter()
                    .map(|(path, _)| Path::new(path).parent().unwrap().to_str().unwrap())
                    .collect();
                let mut run = format!(
                    "RUN mkdir -p {}",
                    dirs.into_iter().collect::<Vec<_>>().join(" ")
                );
                for (path, is_lib) in &p.targets {
                    if *is_lib {
                        run.push_str(&format!(" \\\n    && touch {path}"));
                    } else {
                        run.push_str(&format!(" \\\n    && echo \"fn main() {{}}\" > {path}"));
                    }
                }
                run + "\n"
            })
            .collect();
        // A member nested inside another member's directory arrives with it.
        let sources = closure
            .iter()
            .filter(|p| {
                !closure
                    .iter()
                    .any(|outer| p.dir.starts_with(&format!("{}/", outer.dir)))
            })
            .map(|p| format!("COPY {0} {0}\n", p.dir))
            .collect();
        HashMap::from([
            ("manifests", manifests),
            ("members", members),
            ("stubs", stubs),
            ("sources", sources),
        ])
    }
}

/// Replace the body of every `# >>> cargo xtask dockerfiles: <name>` …
/// `# <<< cargo xtask dockerfiles` block with the generated text.
fn render(dockerfile: &str, blocks: &HashMap<&str, String>) -> Result<String> {
    const BEGIN: &str = "# >>> cargo xtask dockerfiles: ";
    const END: &str = "# <<< cargo xtask dockerfiles";
    let mut out = String::new();
    let mut found = BTreeSet::new();
    let mut lines = dockerfile.lines();
    while let Some(line) = lines.next() {
        out.push_str(line);
        out.push('\n');
        if let Some(name) = line.strip_prefix(BEGIN) {
            let body = blocks
                .get(name)
                .ok_or_else(|| format!("unknown generated block `{name}`"))?;
            out.push_str(body);
            for line in lines.by_ref() {
                if line == END {
                    out.push_str(line);
                    out.push('\n');
                    break;
                }
            }
            found.insert(name);
        }
    }
    let missing: Vec<_> = blocks
        .keys()
        .filter(|name| !found.contains(**name))
        .collect();
    if !missing.is_empty() {
        return Err(format!("Dockerfile has no generated block for {missing:?}").into());
    }
    Ok(out)
}
