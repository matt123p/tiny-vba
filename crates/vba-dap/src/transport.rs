//! Standard Content-Length framing; bounded UTF-8 JSON, no filesystem/network access.
use crate::Json;
use std::io::{self, BufRead, Read, Write};
pub const MAX_MESSAGE: usize = 1024 * 1024;
pub fn read(reader: &mut impl BufRead) -> io::Result<Option<Json>> {
    let mut length = None;
    let mut total = 0usize;
    loop {
        let mut line = String::new();
        let count = reader.take(4097).read_line(&mut line)?;
        if count == 0 && total == 0 {
            return Ok(None);
        }
        total = total.saturating_add(count);
        if count == 0 || total > 4096 || !line.is_ascii() || !line.ends_with("\r\n") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid/big DAP header",
            ));
        }
        if line == "\r\n" {
            break;
        }
        let (key, value) = line
            .trim_end()
            .split_once(':')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid DAP header"))?;
        if !key.eq_ignore_ascii_case("Content-Length") || length.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported/duplicate DAP header",
            ));
        }
        length = Some(
            value
                .trim()
                .parse::<usize>()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid DAP length"))?,
        );
    }
    let length = length
        .filter(|n| *n > 0 && *n <= MAX_MESSAGE)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "DAP message limit"))?;
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
pub fn write(writer: &mut impl Write, value: &Json) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    if bytes.len() > MAX_MESSAGE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "DAP output limit",
        ));
    }
    write!(writer, "Content-Length: {}\r\n\r\n", bytes.len())?;
    writer.write_all(&bytes)?;
    writer.flush()
}
