---
name: tiny-vba-architecture
description: Design or review tiny-vba crate boundaries, public APIs, dependencies and cross-layer changes. Use when choosing where a feature belongs or changing the architecture; covers headless engine and repository-owned IDE layering.
---

# Architecture decisions

M6A's accepted intrinsic extension follows
[the framework](../../../docs/m6a-implementation-framework.md) and
[ADR 0034](../../../docs/decisions/0034-intrinsic-implementation-framework.md):
descriptors in values, one semantic argument binder, verified IR 8 extended calls,
central runtime coercion and VM-owned context/effects. The design adds no crate
or ambient IO authority; implementation/versioned documentation must land
together. See the [acceptance record](../../../docs/m6a-acceptance.md) before
extending the supported profile. M7 authored-profile acceptance is complete; M7A native forms/removal precedes M8.

The remaining plan now uses new M1–M8 numbers under ADR 0019. Implemented contract
IDs in this skill and earlier ADRs are historical; use the progress mapping when
planning future work. Office-supplied standard controls remain in scope, third-party
ActiveX/OLE controls do not. Native extensions/translation and AI are not scheduled.

ADR 0021 adds generic nominal type identity in values, IR 5 local metadata and a
foreign read-only type resolver over the host's existing object store. Office
types/resolvers remain in forks. No reverse dependency, parallel object registry,
JSON dispatch or implicit library activation is permitted. M1.2 defaults and M1.3
collections do not follow automatically from nominal binding.

Read [architecture](../../../docs/architecture.md), the [actual dependency graph](../../../tools/architecture.json),
[ADR 0001](../../../docs/decisions/0001-repository-scope.md) and
[ADR 0002](../../../docs/decisions/0002-editor-debugger-ui.md).
Inspect the affected Cargo manifests and callers before designing an API.
Read the adopted [IR 1 contract](../../../docs/bytecode-vm-specification.md) and
[ADR 0004](../../../docs/decisions/0004-bytecode-vm-contract.md) for bytecode/VM work.
Compilation/lowering belongs in the implemented upper-layer vba-compiler;
do not add documentation-only graph nodes or put reusable lowering inside the CLI/VM.

## Place responsibility before adding code

- Syntax preserves source; semantics binds names/calls; IR verifies bytecode; runtime centralizes
  language rules; VM owns explicit execution state. Keep these separable and testable headlessly.
- VM inspection hooks support vba-debug; debug controls VM; optional DAP wraps the headless IDE. Never reverse
  those dependencies or put egui/DAP types into the VM.
- This repository owns reusable editor/debugger panels and the standalone IDE. Implemented vba-ide
  holds toolkit-independent state/actions, vba-ui-egui renders them and the app owns its shell.
  UI packages are optional for headless consumers; the visual product remains a required deliverable.
- Office catalogues, document storage, undo/recalc, OOXML and application bridges stay in the forks.
  vba-foreign is a minimal generic protocol. Neither Office types nor application sibling paths
  belong in core or reusable UI dependencies.
- Testkit/conformance packages are not shipping library dependencies. ADR 0003 permits the
  development CLI to depend on conformance for fixture validation; keep that exception out of engine/UI libraries.
  Keep native hot-path host calls
  in Rust; JSON belongs to manifests, reports and optional debugger transport.

For a new contract, describe the owner, data/identity semantics, errors, cancellation and resource
lifetime. Prefer the smallest compiling proof against a deterministic generic host before expansion.
Do not invent APIs for Gridcraft; inspect its actual contract when integration is in scope.

## Make a decision reviewable

Compare the proposal against the relevant milestone and existing interfaces. State the concrete
reason for a boundary change and its effect on headless builds, WASM, embedding and compatibility.
Record material changes in an ADR and update the architecture alongside the implementation.
Keep tools/architecture.json synchronized with actual packages/dependencies; do not insert planned
packages merely to reserve names. External dependency adoption includes licence and portability review.

Run `cargo xtask layers`, then repository CI/WASM gates. Demonstrate headless consumers do not
pull UI/platform dependencies when UI packages are introduced. Report planned versus working APIs.

M1 implements vba-syntax's lexer/parser/AST and adds CLI -> syntax for source parsing under ADR 0005.
Its only library dependency is diagnostics. Syntax success, binding validity and execution results
remain distinct. See [frontend](../../../docs/frontend.md) before consuming recovered trees.

## Implemented M2 contract

Read [compiler](../../../docs/compiler.md) and [ADR 0006](../../../docs/decisions/0006-m2-binding-and-verification.md).
Reuse vba-values ScalarKind, vba-semantics bound symbols and vba-compiler lowering. The compiler always
passes vba-ir verification; only vba-ir constructs VerifiedProgram. Keep IR independent of binding and
VM independent of compilation/debug/UI. M2 ir1-structural capabilities describe verification shapes,
not runtime support. Actual execution must check the M3 runtime matrix. Preserve exact constant tags,
ordered slots/sites, complete nonexecutable statement metadata and explicit unsupported diagnostics.

## Implemented M3 contract

Read [runtime](../../../docs/runtime.md) and [ADR 0007](../../../docs/decisions/0007-m3-execution-and-debugger.md).
The m3-core-v1 runtime is intentionally narrower than structural IR. Preserve explicit frame/alias
storage, one-visit checkpoint permits, saved fault PCs, generation checks and nonreplayed continuations.
Use vba-runtime for operations/edits. VM host dispatch must use checked precommit return/writeback
validation and explicit per-site permissions; foreign aliases remain unsupported. Run cargo xtask ci,
wasm and wasm-run (native plus actual WASI execution). Runtime example results do not mark unrun M0
or external scenario assertions PASS. M8/M10 graphical UI and optional DAP are implemented above
the headless engine; use ADR 0018 for their policy. Current handlers belong to the M4 profile.

[ADR 0008](../../../docs/decisions/0008-vm-instance-isolation.md) distinguishes the caller's routing
label from the non-reused SessionInstance. Key handles, locations, breakpoints and stepping by the
engine instance; caller-label/frame/generation equality alone cannot identify an execution. Include
the replacement-session regression in native/WASI evidence when changing these identities.

The current scalar consumer is m4-scalars-v1 under [ADR 0010](../../../docs/decisions/0010-m4-profile-and-verification-hardening.md).
Compile against current VM capabilities; legacy M3-tagged programs require recompilation. Inspect
the xtask workspace banner before trusting gate evidence from a shared target cache.

Current M4 support supersedes earlier first-slice limits: read [M4 language/IR 2](../../../docs/m4-language.md)
and [ADR 0011](../../../docs/decisions/0011-m4-language-extension.md). Compile against m4-language-v1
VM capabilities. Preserve the IR 1 structural target and explicit IR 2 feature/version gates.
The authored inventory is 20 M0 plus 24 M4 PASS fixtures; no Microsoft oracle observation is claimed.

M4 external acceptance is now implemented under [ADR 0012](../../../docs/decisions/0012-m4-oracle-acceptance.md).
Read [the report](../../../docs/m4-oracle-report.md) and current language contract: 44 authored
plus 153 inherited upstream-reported Excel Mac scenarios PASS. Schema 2 and the reviewed allowlist
preserve raw results/environment/hashes; no fresh Office run is claimed. Use m5-host-v1
and explicit IR 3/4 capabilities, retaining IR 1/2 refusal rules. Unselected external cases remain NOT_RUN.

M6 is implemented under [ADR 0014](../../../docs/decisions/0014-m6-project-decoding.md).
Read [the import contract](../../../docs/project-import.md) before changing vba-project.
Keep extraction read-only/inert, project-local ModuleId separate from caller-owned SourceId,
UTF-8 attribute spans and code names distinct from stream/display names. Preserve explicit
corrupt/unsupported/protected/limit refusals and container/decompression work caps. cfb/encoding_rs
are reviewed pinned external dependencies; package preservation and trust UX remain fork-owned.
The 14 controlled importer tests run natively and under WASI through cargo xtask wasm-run.

## Implemented M8 editor/debugger

Read [ADR 0016](../../../docs/decisions/0016-m8-embeddable-editor.md), the
[embedding/harness guide](../../../docs/ui/embedding.md) and
[acceptance matrix](../../../docs/ui/m8-acceptance.md). vba-ide is toolkit/IO-free; vba-ui-egui
mounts in caller Ui with injected tokens, independent per-view IDs and explicit commands;
vba-ide-app owns native/web IO and preferences. Run cargo xtask ui, ui-native and web plus ordinary
CI/WASM gates for affected UI changes. Keep accessibility checks enabled; render/inspect original
light/dark/narrow/scaled/host-theme/focus output. No downloaded reference evidence enters Git:
.cache/evidence/ is ignored and provenance/notices/pins are verified. Do not claim full C0/C1,
Office event/workbook, Set Next Statement or unsafe evaluation support from standalone UI evidence.

ADR 0022 adds IR 6 generic default invocation/value contexts and bounded For Each.
Read docs/m1-defaults-and-collections.md and the current compiler/runtime contract.
Office defaults, collections and signatures remain fork-owned. Preserve safe
watch evaluation, canonical array aliases, resize locks and wait/cancel semantics.

M2's selected class/standard-form profile is implemented under ADR 0023/0024.
Read docs/m2-acceptance.md for current scope, workflow, exchange format and gates;
docs/m2-classes-and-forms.md retains historical incomplete checkpoints. Preserve
IR7 class/event identity, canonical field ByRef, static visibility/signature checks,
property value ByVal semantics, inert safe inspection and all three error modes.

vba-ide owns the bounded shared design, authored source, .tvform codec and form
orchestration; vba-ui-egui renders the same designer/runtime in native/browser
hosts. Generated native control classes use ordinary Event/WithEvents frames.
Explicit suspended callbacks park a modal caller; existing callbacks remain
terminal-only. Retain parked locations, shared budgets, input ownership and host
tokens through pause/resume. Cancel every token on Reset/close; do not execute
user cleanup during abnormal shutdown. Design/history must survive VM Reset.

Form import is inert and validates before project mutation. Save/export preserve
exact authored source and design; revision acknowledgments must reject intervening
source/design edits. Unknown formats, controls and binary resources fail explicitly.
The selected Label/TextBox/CommandButton profile does not authorize COM/DLL or
third-party controls, binary Office form/resource compatibility or package writeback.
Office document bindings/events, storage, trust and package persistence stay in forks.
Run CI/WASM/WASI plus UI/native/web and both Chromium smoke/form acceptance gates
when changing these surfaces. M3 range/worksheet work is GridCraft-owned.

## M7 package evidence

Read [M7 acceptance](../../../docs/m7-acceptance.md) and
[ADR 0037](../../../docs/decisions/0037-m7-package-preservation-and-writeback.md).
The optional `cargo xtask m7` verifies reviewed sibling artifact hashes before
and after actual fork tests. Ordinary core CI remains sibling-independent.
Keep load/compile/run/save, canonical source/original project bytes and
authored/inherited/fresh evidence separate. Original binary preservation and
plain-source exports do not implement edited CFB writeback. Opaque signature
transport does not verify validity or grant trust. Native forms/controls and removal of `.tvform` are now scheduled in M7A;
fresh Office observations remain M8.5.

## Standing Office compatibility policy — M7A

[ADR 0038](../../../docs/decisions/0038-native-office-forms-and-compatibility-only.md)
supersedes the custom format direction and native-forms deferral. Mirror applicable
Excel/Word VBA behavior, formats and workflows; do not add custom user-facing
language features, formats or control semantics. Unknown behavior remains an
explicit gap pending evidence. M7A is in progress (see [inventory/ledger](../../../docs/m7a-inventory.md)); native `.frm`/`.frx`
and embedded form workflows, required native edited persistence, and removal of
`.tvform` precede M8 release acceptance. Existing codec references describe legacy
implementation, not permission to extend or retain that product format. Reuse
compatible internal layers and native-equivalent regressions. Original-byte
preservation alone does not satisfy saving native edits.

Initial FC1.1 native worksheet metadata lives in GridCraft XLSX, with no VM or
UI dependency and no second writable model. [ADR 0039](../../../docs/decisions/0039-m7a-native-control-metadata.md)
requires source DrawingML identity/assignment reconciliation before later native
rendering or activation. Decode tests are not execution evidence.

M7A.1/M7A.2 implement [native-userform-import-v1](../../../docs/m7a-control-contracts.md)
under [ADR 0040](../../../docs/decisions/0040-native-userform-import-and-attachment.md):
bounded native export/embedded Label/TextBox/CommandButton decoding and inert
shared/standalone/fork attachment. IDE -> project is a reviewed downward edge.
Keep original resources separate from authored source/design, validate handlers
before insertion, and never substitute .tvform or acknowledge edited native saves
without the M7A.5 encoder. Full semantics, broader controls and removal remain open.

M7A.3 completes [worksheet-form-controls-v1](../../../docs/m7a-worksheet-controls.md)
under ADR 0041 in GridCraft: native Shape identity/state, engine bindings/undo/recalc,
OOXML/VML edited persistence, optional ControlFormat APIs and trusted widget input.
Preserve original package data separately; failed joins stay inert and callbacks
use the existing bridge. This is not MSForms/ActiveX or UserForm edited-save support.
M7A.4 is in progress; fresh Office observations remain NOT_RUN under M8.5.

IR 9 `class-enumerators-v9` under
[ADR 0045](../../../docs/decisions/0045-class-and-native-collection-enumerators.md)
validates public zero-argument Object-returning class enumerators. Keep older IR
refusal, ordinary getter checkpoints, bounded forwarding roots, wait/cancel and
active/parked snapshot lifetimes. Native Controls/UserForms snapshot canonical
objects through the transient view; no second mutable registry or .tvform change.

IR 10 scalar As New is defined by
[ADR 0046](../../../docs/decisions/0046-lazy-project-class-variables.md). Preserve
exact nominal local/field/persistent metadata, inert declaration/inspection/Set
targets, source-order ByRef preparation, ordinary constructor frames and failed
initialization retry. No external activation or mutable registry is added.

IR 11 class interfaces and Forms library bindings follow
[ADR 0047](../../../docs/decisions/0047-native-forms-nominal-types-and-library-access.md).
Preserve exact nominal identity and event-source/exposure, bounded immutable
descriptors, restored-session validation and source-function alias shadowing.
MSForms names stay in the IDE provider; no new store, COM or unsafe evaluation.

Native Frame containers follow [ADR 0056](../../../docs/decisions/0056-native-frame-containers.md):
bounded native parent storages, immediate child Controls/Parent identity, scoped
TabIndex/option groups and nested ActiveControl/focus cancellation. Canonical state
remains ordinary classes; effective ancestor flags, live relative geometry,
clipping and accessible groups use the shared renderer. Original source/resources
stay inert and unchanged. Further container/page/image/binding/platform semantics,
native edited persistence and .tvform removal remain in M7A; fresh Office traces
remain NOT_RUN under M8.5.

[ADR 0057](../../../docs/decisions/0057-native-tabstrip.md) adds selected native TabStrip
records and separate ordinary Tab/Tabs identity, indexed/named lookup,
enumeration/default selection and authorized widget input. Canonical state stays
in ordinary classes; inert queries/rendering do not instantiate or execute VBA.
MultiPage, dynamic tab mutation/index/name semantics and the remaining M7A.4
ledger stay open. Native edited persistence and .tvform removal remain separate.

[ADR 0058](../../../docs/decisions/0058-native-multipage.md) adds selected native MultiPage
f/o/x reconciliation, distinct Page/Pages identity, immediate child collections,
selection-gated input/focus and actual page rendering. Internal tabs stay private;
canonical state remains ordinary classes and source/resources stay inert. Indexed
Click and repeated-selection behavior also correct the prior TabStrip contract.
Dynamic page/tab mutation, further persistence/property/event/lifecycle cases and
the full remaining M7A.4 ledger remain open. Native edited saving/removal stay
M7A.5/.6; fresh Office traces remain NOT_RUN under M8.5.

[ADR 0059](../../../docs/decisions/0059-native-image-pictures.md) adds selected native Image,
bounded inert BMP/GIF/JPEG/ICO decoding, canonical stdole picture identity and
assignment, display properties and nonfocusable Click. Immutable raster assets
and capped derivative texture caches preserve source/resources and existing VM
budgets. Further picture metadata/platform/format and event/presentation cases,
RefEdit, bindings, dynamic collections and remaining M7A.4 semantics stay open.
Native edited saving/.tvform removal remain M7A.5/.6; Office traces remain NOT_RUN
under M8.5.

[ADR 0060](../../../docs/decisions/0060-native-refedit.md) adds bounded inert RefEdit class-table
resolution, ordinary typed RefEdit text/drop state and selected focus/validation/
Change/DropButtonClick behavior. The shared renderer accepts host range-selection
availability and collapses/restores modal RefEdit; GridCraft owns worksheet
selection and authorization. Original source/resources stay separate. Broader
RefEdit/platform/property/event/binding semantics and the unchanged M7A.4 ledger
remain open. Native edited saving/removal stay M7A.5/.6; fresh Office traces stay
NOT_RUN under M8.5.

[ADR 0061](../../../docs/decisions/0061-native-tab-page-mutation.md) adds native Tabs/Pages
Add/Remove/Clear, writable Integer Index and runtime member Name. Scalar position
cells preserve member/selection identity without owning cycles; inert queries and
page input/rendering follow canonical ordering. Generic VM reference-scan metadata
keeps primitive slots out of object lifetime scans under unchanged budgets.
Dynamic Controls, bindings/multiple selection and the broader M7A.4 ledger remain
open; native edited saving/removal remain M7A.5/.6. Fresh Office traces are NOT_RUN.

[ADR 0062](../../../docs/decisions/0062-native-dynamic-controls.md) implements selected native
Controls.Add/Remove/Clear, runtime names, scoped collections/notifications and
WithEvents focus/validation. An inert runtime projection drives widgets and
input from ordinary canonical objects; imported source/design/resources stay
unchanged. Weak context requests avoid ownership cycles. Shared typed helpers,
indexed verifier metadata and generic dispatch optimizations preserve all existing
VM limits. Full M7A.4, native edited persistence/removal and M8.5 comparisons remain
separate requirements; CI/WASM/WASI validation of ADR 0062 passed.

[ADR 0063](../../../docs/decisions/0063-native-listbox-multiple-selection.md) adds native ListBox
multiple selection, independent focus/selection, row mutation and authorized
pointer/keyboard input. [ADR 0064](../../../docs/decisions/0064-native-runtime-type-names.md) adds
IR 12 inert runtime class names and native TypeName through canonical aliases.
State stays in ordinary classes, rendering stays inert, and budgets are unchanged.
Bindings and the remaining M7A.4 ledger remain open; M7A.5/.6 and M8.5 are separate.

[ADR 0065](../../../docs/decisions/0065-native-cell-and-row-bindings.md) defines native
ControlSource/RowSource through ordinary classes and the existing explicit host
range import. Import and rendering remain inert; GridCraft owns cell commands,
trust and revision-driven callbacks. The owner approved 4,096 static permission
entries on 2026-11-05; per-site authorization and executed-call/work/memory limits
remain in force. Further binding timing/types and the broader M7A.4 ledger remain open.


[ADR 0066](../../../docs/decisions/0066-foreign-runtime-type-names.md) adds optional
read-only runtime type names through the existing object resolver. Check full
identity/lifetime, bound names/work/allocation and preserve errors/fallback; never
invoke getters, host calls or activation. Forks own native names. No IR or budget
increase is implied. Broader M7A.4 and fresh M8.5 comparisons remain separate.


[ADR 0067](../../../docs/decisions/0067-vartype-default-value-inspection.md) adds
IR 13 optional default inspection for VarType. Preserve source-order evaluation,
normal class frames/authorized host calls, getter errors and one-shot Stop/wait
continuations. Resolver presence metadata is inert and identity checked; actual
getters remain executable. Older IR consumers refuse this capability. Watches
stay inert and all existing budgets remain. Full M7A.4/M8.5 are separate gates.
