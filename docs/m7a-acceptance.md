# M7A — Common native forms acceptance

Updated 2026-11-08. Profile: `m7a-common-native-v1`, authored-contract evidence.
Status: **COMPLETE** for the finite common profile. The reviewed scope is
[native-forms-common-v1](m7a4-common-profile.md), the
[worksheet controls profile](m7a-worksheet-controls.md),
[native persistence](m7a5-acceptance.md) and
[custom-format removal](m7a6-acceptance.md), under
[ADR 0100](decisions/0100-common-native-forms-acceptance.md).
Uncommon/exhaustive P9 behavior is not part of this acceptance.

## Reviewed selection and executable gate

[Selection](../tests/fixtures/m7a/selection.json) records the two original XLSM/DOCM
workflows, shared format/lifetime/IO/browser inputs, exact artifact hashes,
license/review and independent expected results. All fixtures/builders are
repository-authored Rust; no Office-produced binaries, downloaded macros or other
interpreters' code are admitted. The acceptance label is not a new VBA feature,
file format or runtime dialect. Fresh Office observations remain **NOT_RUN**.

`cargo xtask m7a` validates the selection and runs the selected shared, native
file, both host, native worksheet and retained M7 package regressions. Any failed
hash/entrypoint/retirement check or nonzero test status fails the gate. It is
optional and requires sibling forks; ordinary shared CI checks local selection
integrity without requiring siblings. Integrity output says VALID_FIXTURE and
NOT_RUN, independently of actual execution results. The shared production and
dependency tree digest also protects reuse of unchanged runtime/renderer evidence.

## Acceptance matrix

| Requirement | Executable evidence and expected outcome |
|---|---|
| Load and compile independently | Both `native_forms_attach_from_*_inertly_run_after_trust_and_retain_original_resources` cases assert attachment/output/trust before run. The edited-copy cases compile idle source before encoding without running it; reopened source/resources remain inert |
| Display and input in both hosts | Grid `ui-egui/tests/vba_forms.rs` six cases; Word native document canvas/hidden-editor cases and retained VBA UI tests; actual shared renderer with caller themes. Browser native CustomerForm imports two files, displays the designer, receives Submit input, pauses/resumes and exports/reimports |
| Common properties/lifetime/events | Shared 99 `form_instances` cases plus keyboard/mouse/window/collection and property suites; both native package suites retain common state, exact types, native resources and original project bytes. Scope and per-control details remain those of M7A.4 |
| Persist actual source/resource edits | `edited_xlsm_native_project_copy_reopens_runs_and_drops_signatures` and its DOCM counterpart change source plus embedded Entry.Text, encode/write/reopen actual macro-enabled packages, then execute the saved code against the existing workbook/document host |
| Stop/Continue, effects and undo | Reopened Apply_Click stops before mutation. Continue produces exactly `saved initialize`, `saved embedded`, writes A1 as Text or the Word main story as String `saved embedded`, and returns normally. Undo restores the prior host state; Reset leaves no form instances |
| Source/resource ownership and save failures | Shared `native_save`, `writeback`, IDE `native_save` and app `source_io`: exact source/design, original bytes separate, compiled-cache removal, stale/foreign/closed-project snapshot refusal, immutable companion collision/atomic primary failure and private permissions. Invalidated VBA signatures are stripped by the forks before package commit |
| Malformed/unsupported/bounded inputs | Shared native failure suites cover truncation, unsupported records/controls and authoritative limits. Both `malformed_native_package_resources_refuse_without_document_mutation` cases test truncated f/o, unknown cached class and hostile site lengths inside real packages, preserving document state and raw bytes |
| Trust loss, cancellation and shutdown | Both native package suites retain denied execution and revocation across input/debugger continuations. Shared execution/window/keyboard/mouse cases preserve QueryClose/validation cancellation, exactly-once continuation, inert Reset/close and token cleanup; both host canvas cases discard stale document input |
| Worksheet controls and recalculation | Grid `form_controls` and XLSX `form_controls` tests cover native units/types, linked/list state, edited persistence, malformed bounds and unknown data preservation. Host worksheet widget tests cover consent, paused linked edits and click routing; native binding cases cover recalc/epoch refresh and protected-cell refusal |
| M7 regression preservation | Grid's five-case and Word's six-case `package_acceptance` suites and the two Grid XLSX `m7_vba` graph cases retain module/document context, live globals/events, original project/related data and signature policy; form-free M7 fixtures alone are not the M7A acceptance basis |
| Custom format absent | Repository source check rejects the removed codec/product references/custom example. The optional sibling check also audits host product sources. No legacy importer, converter or renamed custom format is shipped |

## Gate receipts and evidence boundaries

Final commands, counts, environment, repository/source revisions and selection
identity are recorded in [execution evidence](../tests/fixtures/m7a/execution.json)
and [implementation progress](implementation-progress.md). The changed host cases
run natively and on Node 22 WASM. Both full host CI gates, strict optional VBA/UI
Clippy and VBA-enabled web checks pass. The reproducible integration gate passes
130 cases; native/WASM package suites pass 35/35 (Grid) and 30/30 (Word).

The shared runtime, renderer, dependencies and native/browser input files are
unchanged from M7A.6. Its full WASM/WASI, UI, Xvfb native and 15-flow Chromium
receipts remain valid for their exact pinned implementation/input hashes; the
selection verifies those hashes and the production tree digest. M7A.7 reruns
shared native CI and the selected integration gate, and runs affected host gates.
Evidence reuse is not a claim that unchanged commands ran again. Authored tests,
specification contracts and fresh Office observations remain distinct.

## Supported persistence and remaining limits

Form Save/Export uses native `.frm`/`.frx`. Confirmed native Save may acknowledge
only its current source/design snapshot; Export and browser download requests do
not infer a disk commit. The owning forks offer explicit edited XLSM/DOCM copies
with invalidated signature removal. Ordinary document Save retains original VBA
bytes; copied/downloaded packages do not clear the original editor's dirty state.
Open the edited copy to use its persisted changes.

Unsupported resource/group edits and unrepresentable code pages refuse explicitly.
Fixed component/code-name/reference identity during embedded writeback remains
required; component insertion/renaming, protection bypass, cryptographic signature
verification/resigning and arbitrary COM/OCX/DLL activation are unsupported. No
VM/host authority, memory or execution limit is raised. The existing permission
budget remains 4,096; this work changes no runtime budget.

P9 remains after M8 for uncommon/exhaustive extensions. Fresh Excel/Word comparison
is still M8.5 NOT_RUN. M8 conformance/hardening/release validation is the next
milestone after this finite common-profile acceptance; no universal Office parity
or release sign-off follows from these authored results alone.
