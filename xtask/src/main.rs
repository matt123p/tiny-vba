//! Repository quality commands; never builds or executes external corpora.
#![forbid(unsafe_code)]

use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

fn command(program: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
    let status = Command::new(program).args(args).status()?;
    if !status.success() {
        return Err(io::Error::other(format!("{program} {args:?} failed: {status}")).into());
    }
    Ok(())
}

fn workspace_root(start: &Path) -> io::Result<PathBuf> {
    start
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file()
                && path.join("xtask/Cargo.toml").is_file()
                && path.join("tools/architecture.json").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("run xtask from within a tiny-vba checkout"))
}
fn run() -> Result<(), Box<dyn Error>> {
    // The executable may be shared by different checkouts through CARGO_TARGET_DIR.
    // Resolve the workspace at invocation time; a compiled-in manifest path can be stale.
    let cwd = std::env::current_dir()?;
    let root = workspace_root(&cwd)?;
    std::env::set_current_dir(&root)?;
    eprintln!("xtask workspace: {}", root.display());
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
            let target = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("target"));
            let artifact = target.join("wasm32-wasip1/debug/examples/m3_smoke.wasm");
            let artifact = artifact
                .to_str()
                .ok_or_else(|| io::Error::other("non-UTF-8 WASM artifact path"))?;
            command("node", &["tools/run_wasi.cjs", artifact])?;
            // Cargo runs each test from its crate directory; use the current checkout's runner.
            let import_runner = format!(
                "target.wasm32-wasip1.runner=[\"node\", {:?}]",
                root.join("tools/run_wasi.cjs")
            );
            // Execute the same controlled project-import suite on the WASI target.
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-project",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                    "--config",
                    &import_runner,
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
