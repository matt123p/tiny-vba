use crate::{Reader, Result, budget, corrupt};

/// MS-OVBA 2.4.1 compressed container. `limit` bounds output before every append.
/// No recursion; overlapping copy tokens read only this chunk's produced bytes.
pub fn decompress(input: &[u8], limit: usize) -> Result<Vec<u8>> {
    if limit > 8 * 1024 * 1024 || input.len() > 4 * 1024 * 1024 {
        return Err(budget());
    }
    let mut r = Reader::new(input);
    if r.u8()? != 1 {
        return Err(corrupt("invalid compressed container signature"));
    }
    let mut output = Vec::new();
    while r.pos < input.len() {
        let header = r.u16()?;
        if header & 0x7000 != 0x3000 {
            return Err(corrupt("invalid chunk signature"));
        }
        let size = (header as usize & 0xfff) + 1; // chunk size minus its two-byte header
        let chunk = r.take(size)?;
        let start = output.len();
        if header & 0x8000 == 0 {
            if size != 4096 {
                return Err(corrupt("raw chunk must contain 4096 bytes"));
            }
            append(&mut output, chunk, limit)?;
        } else {
            let mut tokens = Reader::new(chunk);
            while tokens.pos < chunk.len() {
                let flags = tokens.u8()?;
                if tokens.pos == chunk.len() {
                    return Err(corrupt("empty token sequence"));
                }
                for bit in 0..8 {
                    if tokens.pos == chunk.len() {
                        break;
                    }
                    if flags & (1 << bit) == 0 {
                        let value = tokens.u8()?;
                        append(&mut output, &[value], limit)?;
                    } else {
                        let token = tokens.u16()? as usize;
                        let produced = output.len() - start;
                        let mut bits = 4;
                        while (1usize << bits) < produced {
                            bits += 1;
                        }
                        if produced == 0 || bits > 12 {
                            return Err(corrupt("copy token without chunk history"));
                        }
                        let length_mask = 0xffffusize >> bits;
                        let length = (token & length_mask) + 3;
                        let offset = (token >> (16 - bits)) + 1;
                        if offset > produced || produced + length > 4096 {
                            return Err(corrupt("copy token outside chunk"));
                        }
                        if output.len().checked_add(length).is_none_or(|n| n > limit) {
                            return Err(budget());
                        }
                        output.try_reserve(length).map_err(|_| budget())?;
                        for _ in 0..length {
                            let value = output
                                .get(output.len() - offset)
                                .copied()
                                .ok_or_else(|| corrupt("copy offset"))?;
                            output.push(value);
                        }
                    }
                    if output.len() - start > 4096 {
                        return Err(corrupt("oversized decompressed chunk"));
                    }
                }
            }
        }
        if r.pos < input.len() && output.len() - start != 4096 {
            return Err(corrupt("short nonfinal chunk"));
        }
    }
    Ok(output)
}
fn append(output: &mut Vec<u8>, bytes: &[u8], limit: usize) -> Result<()> {
    if output
        .len()
        .checked_add(bytes.len())
        .is_none_or(|n| n > limit)
    {
        return Err(budget());
    }
    output.try_reserve(bytes.len()).map_err(|_| budget())?;
    output.extend_from_slice(bytes);
    Ok(())
}
