# ADR 0030: owned VBA metadata and a lossless code view

Status: adopted for M6.9–M6.12 implementation, 2026-11-01.

Canonical module source remains the original raw UTF-8 text, including exact line
endings, module/member attributes and recognized class export envelopes. Import,
plain-source save/export, source revisions and history use that canonical text.
The normal code pane projects recognized metadata records out and normalizes
newlines for the editor. It never filters arbitrary lines by prefix or hides
ordinary declarations, comments, strings or unsupported language statements.

The syntax layer owns module/member attribute records, their target identities,
literal values and raw spans. Procedure-local records occur immediately after
the signature and before executable code. Recognized class export envelopes have
separate nonexecutable spans. Attributes produce no statement/checkpoint IDs.
Opaque metadata is retained without implementing its semantics. Unsupported
semantic requirements, including project enumerator member IDs, fail explicitly.

The IDE owns a bounded projection with raw/view byte conversions. Visible edits
preserve hidden records and untouched mixed line endings. At a hidden boundary,
caret insertion attaches after the metadata. Member renames update associated
attribute targets. Deleting a complete member removes its owned attributes as
part of the same undoable edit; partial/incomplete edits retain metadata. Module
headers survive first insertion into an attribute-only document. Read-only raw
source remains available in a separate metadata inspector.

Search, completion, selections, diagnostics, gutters, breakpoints, stepping and
full/procedure/split views share this mapping; runtime/debugger identities remain
raw source coordinates. Displayed component names use VBA identity/code names,
with source/export filenames separate. Host-owned document code-name changes
require host validation and revision-checked acknowledgement before shared state
changes. Unsupported host renames refuse without mutating source or identity.

This does not enable binary project writeback, COM activation, cross-project
execution, Excel Macro Options, arbitrary export designer resources or third-party
controls. Forks own document identity and macro shortcut/category policy. The
public-documentation investigation remains distinct from Office observations.
