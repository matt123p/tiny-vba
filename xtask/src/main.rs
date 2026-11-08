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

fn python() -> &'static str {
    if cfg!(windows) { "python" } else { "python3" }
}

fn m8_examples(wasm: bool) -> Result<(), Box<dyn Error>> {
    for (package, example) in [
        ("vba-syntax", "fuzz_parser"),
        ("vba-project", "fuzz_importer"),
        ("vba-vm", "fuzz_vm"),
        ("vba-cli", "m8_conformance"),
    ] {
        if wasm {
            command(
                "cargo",
                &[
                    "build",
                    "-p",
                    package,
                    "--example",
                    example,
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                ],
            )?;
            let target = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("target"));
            let path = target.join(format!("wasm32-wasip1/debug/examples/{example}.wasm"));
            command(
                "node",
                &[
                    "tools/run_wasi.cjs",
                    path.to_str().ok_or("non-UTF-8 WASI path")?,
                ],
            )?;
        } else {
            command(
                "cargo",
                &["run", "-p", package, "--example", example, "--locked"],
            )?;
        }
    }
    Ok(())
}

fn m8_host_tests(wasm: bool) -> Result<(), Box<dyn Error>> {
    if wasm {
        let runner = format!(
            "target.wasm32-wasip1.runner=[\"node\", {:?}]",
            std::env::current_dir()?.join("tools/run_wasi.cjs")
        );
        command(
            "cargo",
            &[
                "test",
                "-p",
                "vba-vm",
                "--test",
                "host",
                "--locked",
                "--target",
                "wasm32-wasip1",
                "--config",
                &runner,
            ],
        )?;
    } else {
        command(
            "cargo",
            &["test", "-p", "vba-vm", "--test", "host", "--locked"],
        )?;
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
        Some("m8") => {
            command(python(), &["tools/check_m8.py"])?;
            m8_host_tests(false)?;
            m8_examples(false)?;
            println!(
                "M8 local authored/inherited conformance and robustness PASS; fresh Office NOT_RUN; release comparison gate remains open"
            );
        }
        Some("m8-wasm") => {
            m8_examples(true)?;
            m8_host_tests(true)?;
        }
        Some("m8-hosts") => command(python(), &["tools/run_m8_hosts.py"])?,
        Some("m8-bench") => {
            command(
                "cargo",
                &[
                    "run",
                    "--release",
                    "-p",
                    "vba-cli",
                    "--example",
                    "m8_bench",
                    "--locked",
                ],
            )?;
            command(
                "cargo",
                &[
                    "build",
                    "--release",
                    "-p",
                    "vba-cli",
                    "--example",
                    "m8_bench",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                ],
            )?;
            let target = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("target"));
            let path = target.join("wasm32-wasip1/release/examples/m8_bench.wasm");
            command(
                "node",
                &[
                    "tools/run_wasi.cjs",
                    path.to_str().ok_or("non-UTF-8 benchmark path")?,
                ],
            )?;
        }
        Some("m7") => {
            // Optional real-fork integration gate. Ordinary core CI remains sibling-independent.
            command(python(), &["tools/check_m7.py"])?;
            let parent = root
                .parent()
                .ok_or_else(|| io::Error::other("M7 requires sibling fork checkouts"))?;
            for (repository, package, filter) in [
                ("gridcraft", "gridcraft-vba", "package_acceptance"),
                ("wordcraft", "wordcraft-vba", "package_acceptance"),
            ] {
                let status = Command::new("cargo")
                    .current_dir(parent.join(repository))
                    .args(["test", "--locked", "-p", package, "--test", filter])
                    .status()?;
                if !status.success() {
                    return Err(io::Error::other(format!(
                        "M7 {repository} acceptance failed: {status}"
                    ))
                    .into());
                }
            }
            let status = Command::new("cargo")
                .current_dir(parent.join("gridcraft"))
                .args(["test", "--locked", "-p", "gridcraft-xlsx", "m7_vba"])
                .status()?;
            if !status.success() {
                return Err(io::Error::other(format!(
                    "M7 package graph acceptance failed: {status}"
                ))
                .into());
            }
            command(python(), &["tools/check_m7.py"])?;
            println!(
                "M7 authored fork acceptance PASS; Office observations and real signature verification NOT_RUN"
            );
        }
        Some("m7a") => {
            // Reviewed authored integration only; core CI never needs the siblings.
            command(python(), &["tools/check_m7a.py"])?;
            command(python(), &["tools/check_m7.py"])?;
            command(python(), &["tools/check_repository.py", "native-forms"])?;
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-project",
                    "--test",
                    "native_form_failures",
                    "--test",
                    "native_save",
                    "--test",
                    "writeback",
                    "--locked",
                ],
            )?;
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-ide",
                    "--test",
                    "native_save",
                    "--test",
                    "form_execution",
                    "--locked",
                ],
            )?;
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-ide-app",
                    "--test",
                    "source_io",
                    "--locked",
                ],
            )?;
            let parent = root
                .parent()
                .ok_or_else(|| io::Error::other("M7A requires sibling fork checkouts"))?;
            for (repo, package, suites) in [
                (
                    "gridcraft",
                    "gridcraft-vba",
                    &["native_forms", "package_acceptance", "form_controls"][..],
                ),
                (
                    "wordcraft",
                    "wordcraft-vba",
                    &["native_forms", "package_acceptance"][..],
                ),
            ] {
                let mut cmd = Command::new("cargo");
                cmd.current_dir(parent.join(repo))
                    .args(["test", "--locked", "-p", package]);
                for suite in suites {
                    cmd.args(["--test", suite]);
                }
                let status = cmd.status()?;
                if !status.success() {
                    return Err(io::Error::other(format!(
                        "M7A {repo} acceptance failed: {status}"
                    ))
                    .into());
                }
            }
            for filter in ["m7_vba", "form_controls"] {
                let status = Command::new("cargo")
                    .current_dir(parent.join("gridcraft"))
                    .args(["test", "--locked", "-p", "gridcraft-xlsx", filter])
                    .status()?;
                if !status.success() {
                    return Err(io::Error::other(format!(
                        "M7A worksheet/package graph acceptance failed: {status}"
                    ))
                    .into());
                }
            }
            command(python(), &["tools/check_m7a.py"])?;
            println!(
                "M7A authored common-profile integration PASS; fresh Office comparisons NOT_RUN (M8.5); P9 remains deferred"
            );
        }
        Some("ci") => {
            command("cargo", &["fmt", "--all", "--check"])?;
            command(
                "cargo",
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--locked",
                    "--",
                    "-D",
                    "warnings",
                ],
            )?;
            command(
                "cargo",
                &["test", "--workspace", "--all-features", "--locked"],
            )?;
            command(python(), &["tools/check_repository.py"])?;
            m8_examples(false)?;
        }
        Some("wasm-run") => {
            command("node", &["--test", "tools/run_wasi.test.cjs"])?;
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
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-ir",
                    "--test",
                    "verification",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                    "--config",
                    &import_runner,
                ],
            )?;
            m8_examples(true)?;
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-values",
                    "--test",
                    "intrinsics",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                    "--config",
                    &import_runner,
                ],
            )?;
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-runtime",
                    "--test",
                    "intrinsic_context",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                    "--config",
                    &import_runner,
                ],
            )?;
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-runtime",
                    "--test",
                    "intrinsics_strings",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                    "--config",
                    &import_runner,
                ],
            )?;
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-syntax",
                    "--test",
                    "attributes",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                    "--config",
                    &import_runner,
                ],
            )?;
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
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-compiler",
                    "--test",
                    "compiler",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                    "--config",
                    &import_runner,
                ],
            )?;
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-vm",
                    "--test",
                    "callbacks",
                    "--test",
                    "host",
                    "--test",
                    "m4",
                    "--test",
                    "embedding",
                    "--test",
                    "inspection",
                    "--test",
                    "references",
                    "--test",
                    "nominal",
                    "--test",
                    "defaults_collections",
                    "--test",
                    "classes",
                    "--test",
                    "attribute_execution",
                    "--test",
                    "class_enumeration",
                    "--test",
                    "auto_instances",
                    "--test",
                    "class_interfaces",
                    "--test",
                    "procedure_aliases",
                    "--test",
                    "execution",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                    "--config",
                    &import_runner,
                ],
            )?;
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-ide",
                    "-p",
                    "vba-dap",
                    "--locked",
                    "--target",
                    "wasm32-wasip1",
                    "--config",
                    &import_runner,
                ],
            )?;
        }
        Some("ui") => {
            command(
                "cargo",
                &[
                    "test",
                    "-p",
                    "vba-ide",
                    "-p",
                    "vba-ui-egui",
                    "-p",
                    "vba-ide-app",
                    "--locked",
                ],
            )?;
            command(
                "cargo",
                &[
                    "run",
                    "-p",
                    "vba-ui-egui",
                    "--example",
                    "snapshots",
                    "--locked",
                ],
            )?;
            command(
                "cargo",
                &[
                    "run",
                    "-p",
                    "vba-ui-egui",
                    "--example",
                    "form_snapshots",
                    "--locked",
                ],
            )?;
        }
        Some("ui-native") => {
            command(
                "cargo",
                &[
                    "run",
                    "-p",
                    "vba-ide-app",
                    "--locked",
                    "--",
                    "--capture",
                    "reports/ui/native-paused.png",
                ],
            )?;
            command(
                "cargo",
                &[
                    "run",
                    "-p",
                    "vba-ide-app",
                    "--locked",
                    "--",
                    "--capture-form",
                    "reports/ui/native-form.png",
                ],
            )?;
        }
        Some("web") => {
            command(
                "cargo",
                &[
                    "build",
                    "-p",
                    "vba-ide-app",
                    "--lib",
                    "--locked",
                    "--target",
                    "wasm32-unknown-unknown",
                ],
            )?;
            let target = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("target"));
            let artifact = target.join("wasm32-unknown-unknown/debug/vba_ide_app.wasm");
            let artifact = artifact
                .to_str()
                .ok_or_else(|| io::Error::other("non-UTF-8 WASM artifact path"))?;
            std::fs::create_dir_all("dist/ui")?;
            std::fs::copy("apps/vba-ide-app/web/index.html", "dist/ui/index.html")?;
            command(
                "wasm-bindgen",
                &[
                    "--target",
                    "web",
                    "--out-dir",
                    "dist/ui/pkg",
                    "--no-typescript",
                    artifact,
                ],
            )?;
        }
        Some("layers") => command(python(), &["tools/check_repository.py", "layers"])?,
        Some("corpora") => command(python(), &["tools/check_repository.py", "corpora"])?,
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
            return Err(io::Error::other(
                "usage: cargo xtask <ci|layers|corpora|m7|m7a|m8|m8-wasm|m8-hosts|m8-bench|wasm|wasm-run|ui|ui-native|web>",
            )
            .into());
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
