# 0014 — Bounded read-only project source decoding

Date: 2026-10-27. Status: adopted and implemented for M6.

`vba-project` owns the `decode(&[u8], &Limits) -> Result<Project, Diagnostic>` boundary.
The input is a standalone `vbaProject.bin` CFB root, already located/extracted by the host.
The milestone in implementation-plan.md governs this repository; the supplied combined
specification's ZIP/OOXML preservation, consent UX and workbook execution gates remain fork-owned.

## Choice and dependency review

Implement the small MS-OVBA record/compression reader here from the public format contract.
The suggested pre-1.0 ovba decoder is not adopted or copied: this boundary needs explicit
protection refusals, identity consistency checks and decompression/work budgets under our control.
Use pinned `cfb = 0.14.0` for strict compound-file structure and `encoding_rs = 0.8.42` for
strict decoding without replacement characters. No interpreter implementation was reused.

Reviewed installed registry manifests/source: cfb is MIT, Rust 1.74+, encoding_rs is
`(Apache-2.0 OR MIT) AND BSD-3-Clause`, Rust 1.88+. Both fit this workspace's Rust 1.95 floor.
Cargo.lock pins their transitive closure. cfb uses uuid/fnv/web-time (including conditional
WASM support); encoding_rs has no Office/UI dependency. Both compile for wasm32-unknown-unknown;
the same importer tests execute on native and wasm32-wasip1. No separate MSRV run is claimed.
`tools/architecture.json` now reviews external dependency requirements separately from workspace
nodes; the checker still enforces the exact owning crate edges and acyclic workspace DAG.

cfb's strict directory/FAT/DIFAT cycle checks are supplemented with complete-sector validation,
a bounded Read/Seek wrapper (16× input length in read bytes, 2,000,000 operations), bounded
entry traversal/depth and stream-length checks. Container validation occurs before the exposed
entry count check; its allocations are bounded by the hard input/work ceilings, not by the
caller's lowered entry limit. No file paths from the input are opened on the ambient filesystem.

## Data and error contract

See [the import contract](../project-import.md) for supported records/encodings and limits.
Project-local ModuleId is the directory ordinal; the host supplies globally unique SourceId
values to Module::source_file. Module names, stream names and document code names remain distinct.
Attributes and CR/LF/CRLF are preserved in decoded UTF-8 source; attribute ranges use those UTF-8
bytes. Original encoding is recorded; original binary preservation remains the caller's job.

Decode CMG/DPB/GC's public obfuscation envelopes solely to refuse protection, passwords or hidden
projects. No password recovery or protection bypass. PROJECTlk is ActiveX licence metadata,
not a lock flag; its presence returns UNSUPPORTED. Designer/UserForm modules, unknown encodings
and unsupported records refuse extraction explicitly. Corruption, unsupported structure,
protection and quota failures have distinct diagnostic codes.

Source decoding does not assert that cached code matches source. CacheConsistency::NotChecked
is explicit; the documented _VBA_PROJECT header is checked and its implementation cache is ignored.
No P-code interpretation, binary writeback, trust decision, host dispatch or automatic compilation
or execution is reachable through this crate. Its only workspace dependency is diagnostics.
The execution profile remains m5-host-v1 and the IR/foreign protocols are unchanged.

## Acceptance evidence

Fourteen independently authored integration tests construct controlled CFB/OVBA bytes in memory.
They prove source/reference/attribute extraction, three module kinds, code-name/stream identity,
Unicode names and diagnostic locations, representative encodings, optional records and empty projects;
negative cases include protection flags/envelopes, designers, identity mismatches, malformed Unicode,
missing streams, bad offsets/cache headers, every directory truncation, header mutations, targeted
CFB tree/FAT cycles and each exposed limit. Hand-encoded compression vectors cover overlapping
copy tokens, offset-width transitions, raw/multiple chunks, corrupt tokens and output budgets.

These tests run through cargo xtask ci and again on WASI through cargo xtask wasm-run.
No upstream workbook, Office binary or downloaded macro is executed. This establishes the
standalone M6 gate; it does not establish the forks' package preservation or trust UX gates.
