#[path = "../driver.rs"]
mod driver;
fn target(bytes: &[u8]) -> driver::R {
    let limits = vba_project::Limits {
        input_bytes: 65536,
        stream_bytes: 16384,
        directory_bytes: 16384,
        source_bytes: 8192,
        total_source_bytes: 16384,
        modules: 8,
        references: 8,
        entries: 64,
    };
    if let Ok(project) = vba_project::decode(bytes, &limits) {
        driver::ensure(project.modules.len() <= limits.modules, "module budget")?;
        driver::ensure(
            project
                .modules
                .iter()
                .map(|m| m.source.len())
                .sum::<usize>()
                <= limits.total_source_bytes,
            "source budget",
        )?;
    }
    if let Ok(output) = vba_project::decompress(bytes, 16384) {
        driver::ensure(output.len() <= 16384, "decompression budget")?;
    }
    Ok(())
}
fn main() -> driver::R {
    driver::run(
        "importer",
        &[
            include_bytes!("../seeds/importer.cfb").to_vec(),
            vba_project::compress(b"Option Explicit\r\nSub Main()\r\nEnd Sub\r\n", 16384)?,
            vec![1, 0x03, 0xb0, 1, 255, 255],
            vec![0; 4096],
            vec![255; 4096],
            vec![],
        ],
        target,
    )
}
