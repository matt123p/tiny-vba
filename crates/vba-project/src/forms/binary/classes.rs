//! Inert MS-OFORMS SiteClassInfo identities. No registry, activation or executable metadata.
use super::*;
const REFEDIT: [u8; 16] = [0x12, 0x45, 2, 0, 0, 0, 0, 0, 0xc0, 0, 0, 0, 0, 0, 0, 0x46];
pub(super) fn read(r: &mut Reader<'_>) -> Result<Vec<[u8; 16]>> {
    let count = usize::from(r.u16()?);
    if count > MAX_CONTROLS {
        return Err(budget());
    }
    let mut classes = Vec::new();
    for _ in 0..count {
        let mut b = block(r, 0)?;
        let m = u64::from(mask(&mut b, 0x7ffb, 0)?);
        if m & (1 << 4) != 0 {
            b.u16()?;
            b.u16()?;
        }
        for (bit, size) in [
            (5, 4),
            (6, 4),
            (7, 2),
            (8, 2),
            (9, 2),
            (10, 2),
            (11, 2),
            (12, 2),
            (13, 4),
            (14, 2),
        ] {
            field(&mut b, m, bit, size, 0)?;
        }
        align(&mut b, 4)?;
        let mut clsid = [0; 16];
        if m & 1 != 0 {
            clsid.copy_from_slice(b.take(16)?);
        }
        for bit in [1, 3] {
            if m & (1 << bit) != 0 {
                b.take(16)?;
            }
        }
        b.end()?;
        classes.push(clsid);
    }
    Ok(classes)
}
pub(super) fn kind(classes: &[[u8; 16]], index: u32) -> Result<Kind> {
    let clsid = classes
        .get((index - 0x8000) as usize)
        .ok_or_else(|| corrupt("native control class-table index"))?;
    if *clsid == REFEDIT {
        Ok(Kind::RefEdit)
    } else {
        Err(unsupported("non-cached native control CLSID"))
    }
}
