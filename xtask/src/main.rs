//! Repository quality commands; never builds or executes external corpora.
#![forbid(unsafe_code)]

use std::{
    error::Error,
    io,
    path::Path,
    process::{Command, ExitCode},
};

fn command(program: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
    let status = Command::new(program).args(args).status()?;
    if !status.success() {
        return Err(io::Error::other(format!("{program} {args:?} failed: {status}")).into());
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("workspace root missing"))?;
    std::env::set_current_dir(root)?;
    match std::env::args().nth(1).as_deref() {
        Some("ci") => {
            command("cargo", &["fmt", "--all", "--check"])?;
            command(
                "cargo",
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--locked",
                    "--",
                    "-D",
                    "warnings",
                ],
            )?;
            command("cargo", &["test", "--workspace", "--locked"])?;
            command("python3", &["tools/check_repository.py"])?;
        }
        Some("wasm-run") => {
            command(
                "cargo",
                &["run", "-p", "vba-cli", "--example", "m3_smoke", "--locked"],
            )?;
            command(
                "cargo",
                &[
                    "build",
                    "-p",
                    "vba-cli",
                    "--example",
                    "m3_smoke",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                ],
            )?;
            command(
                "node",
                &[
                    "tools/run_wasi.cjs",
                    "target/wasm32-wasip1/debug/examples/m3_smoke.wasm",
                ],
            )?;
        }
        Some("layers") => command("python3", &["tools/check_repository.py", "layers"])?,
        Some("corpora") => command("python3", &["tools/check_repository.py", "corpora"])?,
        Some("wasm") => command(
            "cargo",
            &[
                "check",
                "--workspace",
                "--lib",
                "--locked",
                "--target",
                "wasm32-unknown-unknown",
            ],
        )?,
        _ => {
            return Err(
                io::Error::other("usage: cargo xtask <ci|layers|corpora|wasm|wasm-run>").into(),
            );
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
