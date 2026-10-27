//! Independently authored MS-OVBA records. No Office output or external workbook material.
#![allow(dead_code)]
use std::io::{Cursor, Write};

pub fn record(out: &mut Vec<u8>, id: u16, bytes: &[u8]) {
    out.extend(id.to_le_bytes());
    out.extend((bytes.len() as u32).to_le_bytes());
    out.extend(bytes);
}
pub fn wide(text: &str) -> Vec<u8> {
    text.encode_utf16().flat_map(u16::to_le_bytes).collect()
}
fn pair(out: &mut Vec<u8>, id: u16, uid: u16, text: &str) {
    record(out, id, text.as_bytes());
    record(out, uid, &wide(text));
}
fn sized(out: &mut Vec<u8>, text: &str) {
    out.extend((text.len() as u32).to_le_bytes());
    out.extend(text.as_bytes());
}
/// Literal-only compressed final chunks; full blocks use the specified raw chunk form.
pub fn packed(bytes: &[u8]) -> Vec<u8> {
    let mut out = vec![1];
    for block in bytes.chunks(4096) {
        if block.len() == 4096 {
            out.extend(0x3fffu16.to_le_bytes());
            out.extend(block);
        } else {
            // Keep literal payload below a chunk's 4096-byte compressed limit.
            // The test fixtures use short final blocks (< 3600 bytes).
            let mut tokens = Vec::new();
            for group in block.chunks(8) {
                tokens.push(0);
                tokens.extend(group);
            }
            assert!(tokens.len() <= 4096);
            out.extend((0xb000u16 | (tokens.len() as u16 - 1)).to_le_bytes());
            out.extend(tokens);
        }
    }
    out
}
/// Public envelope format, with arbitrary deterministic seed/key and ignored bytes.
pub fn envelope(data: &[u8], seed: u8) -> String {
    let key = 0x35u8;
    let mut out = vec![seed, seed ^ 2, seed ^ key];
    let ignored = vec![0x19; ((seed & 6) / 2) as usize];
    let mut plain = key;
    let mut last = seed ^ key;
    let mut previous = seed ^ 2;
    for byte in ignored
        .into_iter()
        .chain((data.len() as u32).to_le_bytes())
        .chain(data.iter().copied())
    {
        let encrypted = byte ^ previous.wrapping_add(plain);
        out.push(encrypted);
        previous = last;
        last = encrypted;
        plain = byte;
    }
    format!(
        "\"{}\"",
        out.iter().map(|b| format!("{b:02X}")).collect::<String>()
    )
}
#[derive(Clone)]
pub struct Spec {
    pub name: String,
    pub stream: String,
    pub kind: &'static str,
    pub source: Vec<u8>,
    pub offset: u32,
}
impl Spec {
    pub fn new(name: &str, stream: &str, kind: &'static str) -> Self {
        Self {name: name.into(), stream: stream.into(), kind, offset: 3,
            source: format!("Attribute VB_Name = \"{name}\"\r\nAttribute VB_Description = \"kept\"\r\nOption Explicit\r\nPublic Sub Auto_Open()\r\n    UnknownHostCall\r\nEnd Sub\r\n").into_bytes()}
    }
}
pub struct Fixture {
    pub dir: Vec<u8>,
    pub project: Vec<u8>,
    pub wm: Option<Vec<u8>>,
    pub streams: Vec<(String, Vec<u8>)>,
}
impl Fixture {
    pub fn new(cp: u16, specs: &[Spec]) -> Self {
        let mut dir = Vec::new();
        record(&mut dir, 1, &1u32.to_le_bytes());
        record(&mut dir, 0x4a, &1u32.to_le_bytes());
        record(&mut dir, 2, &0x409u32.to_le_bytes());
        record(&mut dir, 0x14, &0x409u32.to_le_bytes());
        record(&mut dir, 3, &cp.to_le_bytes());
        record(&mut dir, 4, b"ControlledProject");
        pair(&mut dir, 5, 0x40, "Authored decoder fixture");
        pair(&mut dir, 6, 0x3d, ""); // Both help-file fields are MBCS, empty here.
        record(&mut dir, 7, &0u32.to_le_bytes());
        record(&mut dir, 8, &0u32.to_le_bytes());
        record(&mut dir, 9, &7u32.to_le_bytes());
        dir.extend(2u16.to_le_bytes());
        pair(&mut dir, 0xc, 0x3c, "Test = 1");
        pair(&mut dir, 0x16, 0x3e, "Library");
        dir.extend(0xdu16.to_le_bytes());
        dir.extend(0u32.to_le_bytes());
        sized(&mut dir, "*\\G{fixture}#1.0#0#inert.tlb#Library");
        dir.extend([0; 6]);
        dir.extend(0xeu16.to_le_bytes());
        dir.extend(0u32.to_le_bytes());
        sized(&mut dir, "*\\C/tmp/inert.xlsm");
        sized(&mut dir, "*\\Cinert.xlsm");
        dir.extend(4u32.to_le_bytes());
        dir.extend(5u16.to_le_bytes());
        dir.extend(0x33u16.to_le_bytes());
        sized(&mut dir, "original-control");
        dir.extend(0x2fu16.to_le_bytes());
        dir.extend(0u32.to_le_bytes());
        sized(&mut dir, "twiddled-control");
        dir.extend([0; 6]);
        pair(&mut dir, 0x16, 0x3e, "Control");
        dir.extend(0x30u16.to_le_bytes());
        dir.extend(0u32.to_le_bytes());
        sized(&mut dir, "extended-control");
        dir.extend([0; 6]);
        dir.extend([0x42; 16]);
        dir.extend(9u32.to_le_bytes());
        record(&mut dir, 0xf, &(specs.len() as u16).to_le_bytes());
        record(&mut dir, 0x13, &0u16.to_le_bytes());
        let mut project = String::from("ID=\"{00000000-0000-0000-0000-000000000000}\"\r\n");
        let mut wm = Vec::new();
        let mut streams = vec![(
            "/VBA/_VBA_PROJECT".into(),
            vec![0xcc, 0x61, 0xff, 0xff, 0, 0, 0],
        )];
        for spec in specs {
            record(&mut dir, 0x19, spec.name.as_bytes());
            record(&mut dir, 0x47, &wide(&spec.name));
            pair(&mut dir, 0x1a, 0x32, &spec.stream);
            pair(&mut dir, 0x1c, 0x48, "Module description");
            record(&mut dir, 0x31, &spec.offset.to_le_bytes());
            record(&mut dir, 0x1e, &0u32.to_le_bytes());
            record(&mut dir, 0x2c, &0u16.to_le_bytes());
            dir.extend(
                (if spec.kind == "Module" {
                    0x21u16
                } else {
                    0x22u16
                })
                .to_le_bytes(),
            );
            dir.extend(0u32.to_le_bytes());
            dir.extend(0x25u16.to_le_bytes());
            dir.extend(0u32.to_le_bytes());
            dir.extend(0x28u16.to_le_bytes());
            dir.extend(0u32.to_le_bytes());
            dir.extend(0x2bu16.to_le_bytes());
            dir.extend(0u32.to_le_bytes());
            project.push_str(&format!(
                "{}={}{}\r\n",
                spec.kind,
                spec.name,
                if spec.kind == "Document" {
                    "/00000000"
                } else {
                    ""
                }
            ));
            wm.extend(spec.name.as_bytes());
            wm.push(0);
            wm.extend(wide(&spec.name));
            wm.extend([0; 2]);
            let mut source = vec![0xa5; spec.offset as usize];
            source.extend(packed(&spec.source));
            streams.push((format!("/VBA/{}", spec.stream), source));
        }
        dir.extend(0x10u16.to_le_bytes());
        dir.extend(0u32.to_le_bytes());
        wm.extend([0; 2]);
        project.push_str(&format!("Name=\"ControlledProject\"\r\nCMG={}\r\nDPB={}\r\nGC={}\r\n\r\n[Host Extender Info]\r\n", envelope(&[0;4],0x16), envelope(&[0],0x23), envelope(&[255],0x32)));
        Self {
            dir,
            project: project.into_bytes(),
            wm: Some(wm),
            streams,
        }
    }
    pub fn bytes(&self) -> Vec<u8> {
        let mut cfb = cfb::CompoundFile::create(Cursor::new(Vec::new())).unwrap();
        cfb.create_storage("/VBA").unwrap();
        let mut streams = self.streams.clone();
        streams.push(("/VBA/dir".into(), packed(&self.dir)));
        streams.push(("/PROJECT".into(), self.project.clone()));
        if let Some(wm) = &self.wm {
            streams.push(("/PROJECTwm".into(), wm.clone()));
        }
        for (name, bytes) in streams {
            cfb.create_stream(&name).unwrap().write_all(&bytes).unwrap();
        }
        cfb.into_inner().into_inner()
    }
}
pub fn fixture() -> Fixture {
    Fixture::new(
        1252,
        &[
            Spec::new("Utilities", "CodeStream", "Module"),
            Spec::new("Counter", "ClassStream", "Class"),
            Spec::new("BookCode", "DocumentStream", "Document"),
        ],
    )
}
pub fn replace(bytes: &mut Vec<u8>, from: &[u8], to: &[u8]) {
    let start = bytes.windows(from.len()).position(|s| s == from).unwrap();
    bytes.splice(start..start + from.len(), to.iter().copied());
}
