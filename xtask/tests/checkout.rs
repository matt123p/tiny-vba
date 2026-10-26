//! A cached xtask executable must operate on the checkout from which it is invoked.
use std::{error::Error, fs, process::Command};
#[test]
fn cached_binary_uses_the_invoking_checkout_instead_of_its_build_path() -> Result<(), Box<dyn Error>>
{
    let root = std::env::temp_dir().join(format!("tiny-vba-xtask-checkout-{}", std::process::id()));
    fs::create_dir(&root)?;
    fs::create_dir_all(root.join("tools"))?;
    fs::create_dir_all(root.join("xtask"))?;
    fs::create_dir_all(root.join("crates/nested/src"))?;
    fs::write(root.join("Cargo.toml"), "[workspace]\n")?;
    fs::write(root.join("xtask/Cargo.toml"), "[package]\nname='xtask'\n")?;
    fs::write(root.join("tools/architecture.json"), "{}\n")?;
    fs::write(
        root.join("tools/check_repository.py"),
        "from pathlib import Path\nassert Path.cwd() == Path(__file__).resolve().parent.parent\nprint('INVOKING_CHECKOUT_VERIFIED')\n",
    )?;
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("layers")
        .current_dir(root.join("crates/nested/src"))
        .output()?;
    fs::remove_dir_all(&root)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8(output.stdout)?.contains("INVOKING_CHECKOUT_VERIFIED"),
        "cached xtask ran a different checkout"
    );
    Ok(())
}
