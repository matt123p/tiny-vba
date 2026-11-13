# ADR 0038 — Native Office forms, removal of .tvform and compatibility-only scope

Date: 2026-11-04. Status: adopted on explicit owner direction.

## Decision

Insert **M7A — Native Office forms and removal of .tvform** between accepted M7
and M8. Activate the [native forms contract](../forms.md)
under M7A, retaining its FC task IDs. Native forms are now scheduled work and a
prerequisite for the declared M8 release profile, rather than a deferred backlog.
M7 and earlier acceptance records remain evidence of their historical profiles.

Tiny-vba's product contract is to mirror the applicable Excel/Word VBA behavior,
formats and workflows. Implement missing Office compatibility; do not introduce
new user-facing language features, control behavior, file formats or extensions
that have no Office counterpart. Internal Rust representations, resource limits,
headless testing and host-neutral APIs may support that implementation without
becoming an alternative user-facing dialect or document format. Unknown behavior
requires investigation and explicit support gaps, never invented semantics.
Existing clean-room, licensing, trust and no-arbitrary-COM/P-code constraints stay
in force; a refusal is not a claim of Office compatibility.

The custom `.tvform` exchange format is to be **removed**, including its codec,
public import/export paths, file filters, examples and current product guidance.
Do not retain a parallel legacy importer, compatibility mode or new conversion
product. Reuse sound internal designer/runtime/debugger work behind native Office
forms and preserve useful regressions with native fixtures. Current `.tvform`
support remains a legacy implementation until the removal actually lands; this
planning decision does not claim it has already been removed.

## Required native workflow

M7A must cover native VBA UserForms from `.frm`/`.frx` exports and embedded project
designer resources in Excel/Word documents, plus the worksheet forms/control work
in the activated breakdown. Inventory Office-supplied controls and their applicable
hosts/platforms; implement their actual properties, events, instance/lifecycle and
binding rules. Preserve exact authored VBA and attached native resources.
No third-party OCX loading or arbitrary native library activation is authorized.

Native import alone is insufficient: design/code views, editing, execution/debugging
and native save/export/reopen must work for the milestone's declared inventory.
Where native form edits change `vbaProject.bin`, implement the necessary reviewed
source/resource encoder and fork-owned package commit path under
[ADR 0037](0037-m7-package-preservation-and-writeback.md). Original-byte preservation
cannot stand in for saving edits. Follow its signature invalidation, cache/source
consistency, bounded encoding, revision/identity checks and atomic commit contracts.
The earlier M7 decision that an encoder was outside M7 does not defer this M7A
prerequisite. The read-only importer remains inert; writeback is a separate explicit
operation. No P-code execution or protection bypass is introduced.

## Acceptance and ownership

Tiny-vba owns bounded reusable form codecs/model/runtime/editor/debugger contracts.
GridCraft owns worksheet controls and Excel bindings; both forks own their document
attachment, trust, package persistence, events and undo/recalculation integration.
Inspect actual extension points before selecting any new dependency or crate;
this decision changes no crate graph, IR version or implemented capability.

Use licensed reviewed native fixtures and preserve separate load/compile/display/
input/run/save results, exact source/resource identities and error/event outcomes.
Run shared native/WASM/WASI/UI/browser and real GridCraft/WordCraft workflows.
Record public specifications, authored expectations, inherited evidence and fresh
permitted Office observations separately. Plan native-form comparison cases now;
fresh Office comparison remains the mandatory M8.5 release gate, never a guessed
M7A PASS. Publish control-level gaps rather than claiming all Office forms work.

M7A acceptance also requires that `.tvform` is absent from shipping format support
and supported user workflows. Historical evidence may retain the name with an
explicit superseded-status note. New examples and regression fixtures use native
Office formats. No runtime feature is added or removed by this planning change.

## Supersession

This direction supersedes ADR 0024's custom exchange-format choice, the former
forms-plan deferral and current guidance to extend/preserve `.tvform` as a product
format. It preserves the useful shared runtime/design/debugger invariants and the
historical M2/M7 results. The [testing and evidence](../testing.md) now starts
at M7A; M8 task IDs stay unchanged.
