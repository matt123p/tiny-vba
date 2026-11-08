//! Dependency-free, deterministic mutation driver shared by the inert fuzz targets.
use std::{
    error::Error,
    io::{self, Read},
};
pub type R<T = ()> = Result<T, Box<dyn Error>>;
pub fn ensure(condition: bool, message: &str) -> R {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message).into())
    }
}
pub fn run(name: &str, seeds: &[Vec<u8>], mut target: impl FnMut(&[u8]) -> R) -> R {
    if std::env::args().nth(1).as_deref() == Some("--stdin") {
        let mut bytes = Vec::new();
        io::stdin().take(65537).read_to_end(&mut bytes)?;
        ensure(bytes.len() <= 65536, "fuzz input exceeds 64 KiB")?;
        return target(&bytes);
    }
    let cases = std::env::var("M8_FUZZ_CASES")
        .ok()
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(256);
    ensure(
        (1..=4096).contains(&cases),
        "campaign count must be 1..4096",
    )?;
    let seed = std::env::var("M8_FUZZ_SEED")
        .ok()
        .map(|s| s.parse::<u64>())
        .transpose()?
        .unwrap_or(0x4d38);
    let mut random = seed;
    ensure(!seeds.is_empty(), "missing authored seeds")?;
    for bytes in seeds {
        target(bytes)?;
    }
    for index in 0..cases {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut bytes = seeds.get(index % seeds.len()).ok_or("seed")?.clone();
        let at = (random >> 32) as usize % bytes.len().saturating_add(1);
        match index % 4 {
            0 => {
                if let Some(byte) = bytes.get_mut(at) {
                    *byte ^= (random as u8) | 1;
                }
            }
            1 => {
                bytes.truncate(at);
            }
            2 => {
                if bytes.len() < 65536 {
                    bytes.insert(at, random as u8);
                }
            }
            _ => {
                if at < bytes.len() {
                    bytes.remove(at);
                }
            }
        }
        target(&bytes)?;
    }
    println!(
        "M8 fuzz {name}: seed={seed} mutations={cases} authored_seeds={} PASS (robustness, not Office parity)",
        seeds.len()
    );
    Ok(())
}
