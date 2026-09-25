mod ios;
mod packs;
mod parity;
mod shaders;
mod smoke;

use clap::{Parser, Subcommand};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Parser)]
#[command(about = "Yap development tasks")]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Regenerate web and SwiftUI shaders from the shared WGSL.
    Shaders,
    /// Build Rust libraries, host-generated bindings, and the XcodeGen iOS app.
    Ios(ios::Args),
    /// Run the real offline Swift integration test (macOS, Swift 6.2+).
    Smoke(smoke::Args),
    /// Capture saved screens on web and iOS with a side-by-side gallery.
    Parity(parity::Args),
}

fn main() {
    let result = match Cli::parse().task {
        Task::Shaders => shaders::run(),
        Task::Ios(args) => ios::run(args),
        Task::Smoke(args) => smoke::run(args),
        Task::Parity(args) => parity::run(args),
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned()
}

fn command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut cmd = Command::new(program);
    cmd.current_dir(root());
    cmd
}

fn echo(cmd: &Command) -> Result<String> {
    let display = std::iter::once(cmd.get_program())
        .chain(cmd.get_args())
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");
    println!("+ {display}");
    std::io::stdout().flush()?;
    Ok(display)
}

fn run(cmd: &mut Command) -> Result<()> {
    let display = echo(cmd)?;
    let status = cmd
        .status()
        .map_err(|error| format!("starting {display}: {error}"))?;
    if !status.success() {
        return Err(format!("{display} failed: {status}").into());
    }
    Ok(())
}

fn run_capture(cmd: &mut Command) -> Result<std::process::Output> {
    let display = echo(cmd)?;
    let output = cmd
        .output()
        .map_err(|error| format!("starting {display}: {error}"))?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        return Err(format!("{display} failed: {}", output.status).into());
    }
    Ok(output)
}

fn text(cmd: &mut Command) -> Result<String> {
    Ok(String::from_utf8(run_capture(cmd)?.stdout)?
        .trim()
        .to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_modes_and_profiles() {
        for args in [
            vec!["xtask", "ios", "--release", "--debug"],
            vec!["xtask", "ios", "--archive", "--debug"],
            vec!["xtask", "ios", "--simulator", "--device", "abc"],
            vec!["xtask", "parity", "--ios-only", "--web-only"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
        let cli = Cli::try_parse_from(["xtask", "ios", "--simulator"]).unwrap();
        let Task::Ios(args) = cli.task else {
            panic!("wrong task")
        };
        assert_eq!(args.simulator.as_deref(), Some("booted"));
        assert!(!args.debug);
    }
}
