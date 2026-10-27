# Read-only VBA project import

Implemented M6, 2026-10-27, under [ADR 0014](decisions/0014-m6-project-decoding.md).

`vba_project::decode(bytes, &Limits::default())` extracts an unprotected project from a CFB
`vbaProject.bin` root. The host owns OOXML relationship discovery, ZIP extraction, original
part preservation, document attachment and permission to run. This API only returns data.

The result contains project information (name, code page, platform/LCIDs, version, description,
help metadata, optional constants/compatibility version), inert PROJECT text, registered/project/
control reference metadata and modules in directory order. Reference paths and identifiers never
cause files, libraries or controls to load. Standard, class and document modules retain separate
module and stream names, source offsets, flags, descriptions, full source and attributes.
Document `code_name` is the VBA identity rather than a sheet tab/document display name.

ModuleId is a project-local directory ordinal, stable for repeat decoding of identical bytes.
It is not a cross-project identity. Assign a unique SourceId per embedding source set via
`module.source_file(id)`; this preserves text for parser/compiler diagnostics. Attribute ranges
are half-open UTF-8 byte offsets excluding line endings. Text, including hidden attribute lines,
line endings and whitespace, is preserved after strict encoding conversion; retain original
project bytes in the embedding when byte-for-byte preservation matters.

## Supported format profile

- MS-OVBA compressed directory/module containers: literal/raw chunks and bounded overlapping copy tokens.
- Project information, optional compatibility/constants records, all three reference forms and optional reference names/original-control metadata.
- Module names/optional Unicode names, Unicode/MBCS stream names/descriptions, offsets, kinds, cookies, read-only/private flags and terminators.
- PROJECT module declarations and protection metadata; optional PROJECTwm name maps are checked against directory order and identities.
- Code pages 1250–1258, 874, 932, 936, 949, 950, 10000 and 65001 via encoding_rs without replacement decoding. Metadata Unicode must be well-formed UTF-16 and agree with MBCS identities. Representative 1252/1251/932/UTF-8 source and UTF-8 Unicode names have executable evidence; the full encoding mapping space is not exhaustively tested.

Unknown encodings/records, platforms outside 0–3, nonstandard project LCIDs, nonzero library
flags, UserForms/designers and ActiveX licence streams return explicit unsupported errors.
The profile checks metadata pairs strictly and refuses lossy/inconsistent identities.
Source must contain exactly one matching VB_Name attribute. Malformed containers, missing required
streams/records, unsafe/reserved/duplicate stream names and identity/type disagreements are errors.

CMG/DPB/GC envelopes are decoded only to require unprotected/empty-password/visible states.
Protection is refused, never bypassed. The cache header is checked, but cached implementation
data and P-code are ignored. `cache_consistency` is always `NotChecked`: decoding source is not
proof that it matches code another application would execute.

## Limits and errors

Limits can be lowered, never raised above these hard defaults:

| Limit | Default |
|---|---:|
| Input CFB bytes | 16 MiB |
| Individual stored stream bytes | 4 MiB |
| Decompressed directory bytes | 1 MiB |
| Per-module decompressed bytes and resulting UTF-8 source bytes | 1 MiB |
| Total resulting UTF-8 source bytes | 8 MiB |
| Modules / references | 256 each |
| CFB entries including root | 2,048 |
| CFB path components | 8 |

CFB reading additionally caps read bytes at 16× input size and Read/Seek operations at 2,000,000.
Strict container validation allocates metadata before the lowered entry cap is checked; hard
input/work ceilings still bound that phase. Encoding conversion has a bounded transient expansion
before the UTF-8 result limit is checked. Decompression precharges output allocations and is
iterative with at most 4,096 produced bytes per chunk. The separately exposed `decompress` API
caps input at 4 MiB and requested output at 8 MiB.

| Diagnostic code | Meaning |
|---|---|
| VBA_PROJECT_CORRUPT | Invalid/truncated/inconsistent binary, text or source metadata |
| VBA_PROJECT_UNSUPPORTED | Recognized boundary outside this decoding profile |
| VBA_PROJECT_PROTECTED | Restricted project source extraction refused |
| VBA_PROJECT_LIMIT | Input, output, count, depth or container work cap exceeded |

`crates/vba-project/tests/import.rs` has 14 independently authored tests, shared unchanged between
native CI and WASI execution. No external workbook acceptance or Office observation is claimed.
Format authority: [MS-OVBA](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/575462ba-bf67-4190-9fac-c275523c75fc)
and [MS-CFB](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b).
