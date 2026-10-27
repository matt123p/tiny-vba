#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod support;
use support::{Fixture, Spec, envelope, fixture, packed, record, replace};
use vba_diagnostics::SourceId;
use vba_project::{
    CacheConsistency, Limits, ModuleId, ModuleKind, ReferenceKind, decode, decompress,
};

fn code(f: &Fixture) -> &'static str {
    decode(&f.bytes(), &Limits::default()).unwrap_err().code
}
#[test]
fn extracts_modules_references_attributes_and_code_names_without_execution() {
    let f = fixture();
    let bytes = f.bytes();
    let original = bytes.clone();
    let p = decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(bytes, original);
    assert_eq!(p.information.name, "ControlledProject");
    assert_eq!(p.information.code_page, 1252);
    assert_eq!(p.information.compatibility_version, Some(1));
    assert_eq!(
        (p.information.version_major, p.information.version_minor),
        (7, 2)
    );
    assert_eq!(p.information.constants.as_deref(), Some("Test = 1"));
    assert_eq!(p.cache_consistency, CacheConsistency::NotChecked);
    assert_eq!(p.modules.len(), 3);
    for (i, m) in p.modules.iter().enumerate() {
        assert_eq!(m.id, ModuleId(i as u32));
        assert!(m.read_only && m.private);
        assert_eq!(m.source_offset, 3);
        assert_eq!(m.doc_string, "Module description");
        assert!(
            m.source
                .contains("Public Sub Auto_Open()\r\n    UnknownHostCall\r\n")
        );
        assert_eq!(m.attributes.len(), 2);
        let attr = &m.attributes[1];
        assert_eq!(attr.name, "VB_Description");
        assert_eq!(attr.value, "\"kept\"");
        assert_eq!(
            &m.source[attr.byte_start..attr.byte_end],
            "Attribute VB_Description = \"kept\""
        );
        let file = m.source_file(SourceId(100 + i as u64)).unwrap();
        assert_eq!(file.text(), m.source);
        assert_eq!(file.id(), SourceId(100 + i as u64));
    }
    assert_eq!(p.modules[0].kind, ModuleKind::Standard);
    assert_eq!(p.modules[1].kind, ModuleKind::Class);
    assert_eq!(p.modules[2].kind, ModuleKind::Document);
    assert_eq!(p.modules[2].code_name.as_deref(), Some("BookCode"));
    assert_eq!(p.modules[0].code_name, None);
    assert_eq!(p.modules[0].stream_name, "CodeStream");
    assert_eq!(p.references.len(), 3);
    assert_eq!(p.references[0].name.as_deref(), Some("Library"));
    assert!(
        matches!(&p.references[0].kind,ReferenceKind::Registered{libid} if libid.contains("inert.tlb"))
    );
    assert!(
        matches!(&p.references[1].kind,ReferenceKind::Project{absolute,relative,major:4,minor:5} if absolute=="*\\C/tmp/inert.xlsm" && relative=="*\\Cinert.xlsm")
    );
    assert!(
        matches!(&p.references[2].kind,ReferenceKind::Control{original:Some(original),twiddled,extended,extended_name:Some(name),original_type_lib,cookie:9} if original=="original-control" && twiddled=="twiddled-control" && extended=="extended-control" && name=="Control" && original_type_lib==&[0x42;16])
    );
    let again = decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(again.modules[2].id, p.modules[2].id);
}
#[test]
fn encoding_and_decoded_source_offsets_are_preserved() {
    for (cp, encoded, expected) in [
        (1252, vec![0xe9], "é"),
        (65001, "日本語".as_bytes().to_vec(), "日本語"),
        (932, vec![0x82, 0xa0], "あ"),
        (1251, vec![0xc6], "Ж"),
    ] {
        let mut s = Spec::new("Encoded", "Encoded", "Module");
        s.source.extend(b"' ");
        s.source.extend(encoded);
        s.source.extend(b"\r\nAttribute Extra = \"tail\"\r\n");
        let p = decode(&Fixture::new(cp, &[s]).bytes(), &Limits::default()).unwrap();
        let m = &p.modules[0];
        assert!(m.source.contains(expected));
        let a = &m.attributes[2];
        assert_eq!(
            &m.source[a.byte_start..a.byte_end],
            "Attribute Extra = \"tail\""
        );
    }
    let mut s = Spec::new("Encoded", "Encoded", "Module");
    s.source.push(0xff);
    assert_eq!(code(&Fixture::new(65001, &[s])), "VBA_PROJECT_CORRUPT");
    assert_eq!(code(&Fixture::new(42, &[])), "VBA_PROJECT_UNSUPPORTED");
}
#[test]
fn optional_records_empty_projects_and_no_name_map_are_supported() {
    let mut f = Fixture::new(1252, &[]);
    f.wm = None;
    let mut optional = vec![];
    record(&mut optional, 0x4a, &1u32.to_le_bytes());
    replace(&mut f.dir, &optional, &[]);
    optional.clear();
    record(&mut optional, 0xc, b"Test = 1");
    record(&mut optional, 0x3c, &support::wide("Test = 1"));
    replace(&mut f.dir, &optional, &[]);
    let p = decode(&f.bytes(), &Limits::default()).unwrap();
    assert!(p.modules.is_empty());
    assert!(p.information.constants.is_none());
    assert_eq!(p.information.compatibility_version, None);
}
#[test]
fn protection_envelopes_refuse_user_host_editor_password_and_visibility_restrictions() {
    for seed in 0..=7 {
        for bits in [1u32, 2, 4] {
            let mut f = fixture();
            replace(
                &mut f.project,
                envelope(&[0; 4], 0x16).as_bytes(),
                envelope(&bits.to_le_bytes(), seed).as_bytes(),
            );
            assert_eq!(code(&f), "VBA_PROJECT_PROTECTED");
        }
        for (old, new) in [
            (envelope(&[0], 0x23), envelope(&[1, 2, 3], seed)),
            (envelope(&[255], 0x32), envelope(&[0], seed)),
        ] {
            let mut f = fixture();
            replace(&mut f.project, old.as_bytes(), new.as_bytes());
            assert_eq!(code(&f), "VBA_PROJECT_PROTECTED");
        }
    }
    for bad in ["\"\"", "\"GG\"", "\"0\"", "\"000000\"", "unquoted"] {
        let mut f = fixture();
        replace(
            &mut f.project,
            envelope(&[0; 4], 0x16).as_bytes(),
            bad.as_bytes(),
        );
        assert!(matches!(
            code(&f),
            "VBA_PROJECT_CORRUPT" | "VBA_PROJECT_UNSUPPORTED"
        ));
    }
    let mut f = fixture();
    f.project
        .extend(format!("CMG={}\r\n", envelope(&[0; 4], 0x16)).bytes());
    // A second pre-section protection property is always rejected.
    replace(
        &mut f.project,
        b"[Host Extender Info]",
        b"CMG=\"duplicate\"",
    );
    assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
}
#[test]
fn unsupported_designers_activex_and_unknown_records_are_explicit() {
    let mut f = fixture();
    f.streams.push(("/PROJECTlk".into(), vec![]));
    assert_eq!(code(&f), "VBA_PROJECT_UNSUPPORTED");
    let f = Fixture::new(1252, &[Spec::new("Form", "Form", "BaseClass")]);
    assert_eq!(code(&f), "VBA_PROJECT_UNSUPPORTED");
    let mut f = fixture();
    replace(
        &mut f.dir,
        &[0x0d, 0, 0, 0, 0, 0],
        &[0xff, 0x7f, 0, 0, 0, 0],
    );
    assert_eq!(code(&f), "VBA_PROJECT_UNSUPPORTED");
}
#[test]
fn module_identity_type_stream_mapping_and_attributes_must_agree() {
    for (from, to) in [
        (
            b"Module=Utilities".as_slice(),
            b"Class=Utilities".as_slice(),
        ),
        (b"Module=Utilities", b"Module=Other"),
        (b"/00000000", b"/nothex!!"),
    ] {
        let mut f = fixture();
        replace(&mut f.project, from, to);
        assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
    }
    let mut f = fixture();
    replace(f.wm.as_mut().unwrap(), b"Utilities", b"OtherName");
    assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
    for stream in ["dir", "_VBA_PROJECT", "__SRP_0", "../escape", "a/b", "."] {
        let f = Fixture::new(1252, &[Spec::new("Utilities", stream, "Module")]);
        // Directory validation occurs before trying to read the referenced stream.
        // Avoid asking the CFB fixture writer to construct an invalid path.
        let mut base = fixture();
        base.dir = f.dir;
        assert_eq!(code(&base), "VBA_PROJECT_CORRUPT");
    }
    let mut s = Spec::new("Utilities", "Code", "Module");
    replace(&mut s.source, b"\"Utilities\"", b"\"Other\"");
    assert_eq!(code(&Fixture::new(1252, &[s])), "VBA_PROJECT_CORRUPT");
    let s = Spec::new("Utilities", "Code", "Module");
    let mut other = s.clone();
    other.stream = "Code2".into();
    let mut f = Fixture::new(1252, &[s.clone(), other]);
    assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
    let mut other = s.clone();
    other.name = "Other".into();
    f = Fixture::new(1252, &[s, other]);
    assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
}
#[test]
fn missing_streams_invalid_offsets_and_cache_headers_are_corrupt() {
    for name in ["/VBA/_VBA_PROJECT", "/VBA/CodeStream"] {
        let mut f = fixture();
        f.streams.retain(|(n, _)| n != name);
        assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
    }
    let mut f = fixture();
    f.streams[0].1 = vec![0; 7];
    assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
    let mut f = fixture();
    let mut from = vec![];
    record(&mut from, 0x31, &3u32.to_le_bytes());
    let mut to = vec![];
    record(&mut to, 0x31, &u32::MAX.to_le_bytes());
    replace(&mut f.dir, &from, &to);
    assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
    let mut f = fixture();
    f.dir.extend([1, 2]);
    assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
}
#[test]
fn every_directory_truncation_and_selected_container_mutations_fail_without_panicking() {
    let mut f = fixture();
    let dir = f.dir.clone();
    for end in 0..dir.len() {
        f.dir = dir[..end].to_vec();
        assert!(
            decode(&f.bytes(), &Limits::default()).is_err(),
            "dir length {end}"
        );
    }
    let bytes = fixture().bytes();
    for end in [
        0,
        1,
        7,
        64,
        511,
        512,
        1024,
        bytes.len() / 2,
        bytes.len() - 1,
    ] {
        assert!(
            decode(&bytes[..end], &Limits::default()).is_err(),
            "container length {end}"
        );
    }
    for offset in 0..512 {
        let mut mutated = bytes.clone();
        mutated[offset] ^= 0xff;
        // Some reserved/header fields are allowed to change; every outcome must be bounded.
        let _ = decode(&mutated, &Limits::default());
    }
}
#[test]
fn all_decoding_limits_are_enforced_and_cannot_be_raised() {
    let bytes = fixture().bytes();
    for limits in [
        Limits {
            input_bytes: bytes.len() - 1,
            ..Default::default()
        },
        Limits {
            stream_bytes: 10,
            ..Default::default()
        },
        Limits {
            directory_bytes: 10,
            ..Default::default()
        },
        Limits {
            source_bytes: 10,
            ..Default::default()
        },
        Limits {
            total_source_bytes: 10,
            ..Default::default()
        },
        Limits {
            modules: 2,
            ..Default::default()
        },
        Limits {
            references: 2,
            ..Default::default()
        },
        Limits {
            entries: 2,
            ..Default::default()
        },
        Limits {
            input_bytes: usize::MAX,
            ..Default::default()
        },
    ] {
        assert_eq!(
            decode(&bytes, &limits).unwrap_err().code,
            "VBA_PROJECT_LIMIT"
        );
    }
    let mut s = Spec::new("Expansion", "Expansion", "Module");
    s.source.extend(vec![0xe9; 1024]);
    let f = Fixture::new(1252, &[s]);
    assert_eq!(
        decode(
            &f.bytes(),
            &Limits {
                source_bytes: 1600,
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        "VBA_PROJECT_LIMIT"
    );
    let p = decode(&bytes, &Limits::default()).unwrap();
    let size = p.modules.iter().map(|m| m.source.len()).sum();
    assert!(
        decode(
            &bytes,
            &Limits {
                total_source_bytes: size,
                ..Default::default()
            }
        )
        .is_ok()
    );
    assert_eq!(
        decode(
            &bytes,
            &Limits {
                total_source_bytes: size - 1,
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        "VBA_PROJECT_LIMIT"
    );
}
#[test]
fn literal_raw_multiple_chunks_and_overlapping_copy_tokens_decode_exactly() {
    assert_eq!(decompress(&[1], 0).unwrap(), vec![]);
    for bytes in [b"hello".to_vec(), vec![0x42; 4096], vec![0x42; 8193]] {
        assert_eq!(decompress(&packed(&bytes), bytes.len()).unwrap(), bytes);
    }
    // One literal 'a', then offset 1 / length 4095 (overlapping repetition).
    let encoded = [1, 3, 0xb0, 2, b'a', 0xfc, 0x0f];
    assert_eq!(decompress(&encoded, 4096).unwrap(), vec![b'a'; 4096]);
    assert_eq!(
        decompress(&encoded, 4095).unwrap_err().code,
        "VBA_PROJECT_LIMIT"
    );
    // Exercise the offset-bit boundary: 17 literals require five offset bits.
    let mut tokens = vec![0];
    tokens.extend(0..8);
    tokens.push(0);
    tokens.extend(8..16);
    tokens.extend([2, 16, 0, 0x80]); // offset 17, length 3
    let mut encoded = vec![1];
    encoded.extend((0xb000u16 | (tokens.len() as u16 - 1)).to_le_bytes());
    encoded.extend(tokens);
    let mut expected = (0..17).collect::<Vec<_>>();
    expected.extend([0, 1, 2]);
    assert_eq!(decompress(&encoded, 20).unwrap(), expected);
}
#[test]
fn malformed_compression_and_work_budgets_fail_explicitly() {
    for input in [
        vec![],
        vec![0],
        vec![1, 0],
        vec![1, 0, 0],
        vec![1, 0, 0xb0, 0],
        vec![1, 1, 0xb0, 1, 0],
        vec![1, 2, 0xb0, 1, 0, 0],
        vec![1, 0, 0x30, 0],
        vec![1, 1, 0xb0, 0, b'a', 1, 0xb0, 0, b'b'],
        vec![1, 3, 0xb0, 2, b'a', 0xff, 0x0f],
    ] {
        assert_eq!(
            decompress(&input, 8192).unwrap_err().code,
            "VBA_PROJECT_CORRUPT",
            "{input:?}"
        );
    }
    assert_eq!(
        decompress(&[1], usize::MAX).unwrap_err().code,
        "VBA_PROJECT_LIMIT"
    );
    assert_eq!(
        decompress(&vec![0; 4 * 1024 * 1024 + 1], 1)
            .unwrap_err()
            .code,
        "VBA_PROJECT_LIMIT"
    );
}

#[test]
fn unicode_identities_and_source_diagnostic_locations_survive_import() {
    let f = Fixture::new(65001, &[Spec::new("日本語", "UnicodeStream", "Document")]);
    let p = decode(&f.bytes(), &Limits::default()).unwrap();
    let m = &p.modules[0];
    assert_eq!(m.name, "日本語");
    assert_eq!(m.code_name.as_deref(), Some("日本語"));
    let file = m.source_file(SourceId(700)).unwrap();
    let attr = &m.attributes[1];
    let span = file
        .span(attr.byte_start as u32, attr.byte_end as u32)
        .unwrap();
    assert_eq!(span.source(), SourceId(700));
    assert_eq!(file.position(span).unwrap(), (2, 1));
    assert_eq!(
        file.slice(span).unwrap(),
        "Attribute VB_Description = \"kept\""
    );
}
#[test]
fn malformed_unicode_attributes_and_name_maps_are_rejected() {
    for replacement in [
        vec![0],
        vec![0x00, 0xd8],
        vec![0, 0],
        support::wide("Other"),
    ] {
        let mut f = fixture();
        let mut old = vec![];
        record(&mut old, 0x47, &support::wide("Utilities"));
        let mut new = vec![];
        record(&mut new, 0x47, &replacement);
        replace(&mut f.dir, &old, &new);
        assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
    }
    for extra in [
        b"Attribute VB_Name = \"Utilities\"\r\n".as_slice(),
        b"Attribute Bad\r\n",
        b"\0",
        b"Attribute Empty =\r\n",
    ] {
        let mut s = Spec::new("Utilities", "Code", "Module");
        s.source.extend(extra);
        assert_eq!(code(&Fixture::new(1252, &[s])), "VBA_PROJECT_CORRUPT");
    }
    let mut f = fixture();
    f.wm.as_mut().unwrap().pop();
    assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
    let mut f = fixture();
    replace(&mut f.project, b"CMG=", b"Absent=");
    assert_eq!(code(&f), "VBA_PROJECT_CORRUPT");
}
#[test]
fn cyclic_container_metadata_and_impossible_header_counts_are_rejected() {
    let bytes = fixture().bytes();
    let sector_size = 1usize << u16::from_le_bytes(bytes[30..32].try_into().unwrap());
    let dir_sector = u32::from_le_bytes(bytes[48..52].try_into().unwrap()) as usize;
    let fat_sector = u32::from_le_bytes(bytes[76..80].try_into().unwrap()) as usize;
    // Root directory's child points to itself.
    let mut cyclic = bytes.clone();
    let child = (dir_sector + 1) * sector_size + 76;
    cyclic[child..child + 4].copy_from_slice(&0u32.to_le_bytes());
    assert_eq!(
        decode(&cyclic, &Limits::default()).unwrap_err().code,
        "VBA_PROJECT_CORRUPT"
    );
    // Directory sector's FAT link points to itself.
    let mut cyclic = bytes.clone();
    let link = (fat_sector + 1) * sector_size + dir_sector * 4;
    cyclic[link..link + 4].copy_from_slice(&(dir_sector as u32).to_le_bytes());
    assert_eq!(
        decode(&cyclic, &Limits::default()).unwrap_err().code,
        "VBA_PROJECT_CORRUPT"
    );
    for offset in [44, 64, 72] {
        let mut invalid = bytes.clone();
        invalid[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            decode(&invalid, &Limits::default()).unwrap_err().code,
            "VBA_PROJECT_CORRUPT"
        );
    }
}
