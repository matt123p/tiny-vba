---
name: tiny-vba-architecture
description: Design or review tiny-vba crate boundaries, public APIs, dependencies and cross-layer changes. Use when choosing where a feature belongs or changing the architecture; covers headless engine and repository-owned IDE layering.
---

# Architecture decisions

Current native-only contract (ADR 0099, 2026-11-08): the custom form codec,
`import_form`/`export_form` APIs, file routes/filters and example are removed.
New forms attach inert native templates; all form Save/Export/remove-export
requests use native snapshots. Use `.frm`/`.frx` fixtures in tests/examples and
keep retired extension tests negative only. No legacy importer/converter is
shipped. Historical codec-era instructions below are superseded for format
support. Preserve common runtime, source/resource ownership, trust and save
receipts; M7A.6 final gates are recorded in implementation progress.


Current owner scope under [ADR 0097](../../../docs/decisions/0097-common-forms-scope-and-p9.md):
M7A.4 is COMPLETE for [native-forms-common-v1](../../../docs/m7a4-common-profile.md).
Remaining uncommon/exhaustive parity belongs to [P9](../../../docs/p9-forms-compatibility.md),
after M8. This supersedes historical exhaustive-blocker language below. Next is
M7A.6 removal, then M7A.7 integration; M7A.5 explicit persistence is complete. Keep common
workflow bugs, existing support, safety/quality gates and M8.5 evidence intact.


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


## Lazy project-class array elements

Keep IR 14 array auto_new metadata and TouchElement in compiler/IR/VM. Reuse canonical Locations, element kind/nominal metadata and existing constructor frames; never add an object or construction registry. See [ADR 0068](../../../docs/decisions/0068-lazy-project-class-array-elements.md). Existing budgets remain unchanged; full M7A.4 remains open.


[ADR 0069](../../../docs/decisions/0069-userforms-without-form-components.md)
selects minimal ordinary UserForms support without form components via bounded
code tokens; normal binding resolves shadows. Runtime selection uses verified
class/source identity, never mutable source during ticks. Preserve Reset, inert
inspection, wrapped callback waits/identity, existing budgets and no extra object
registry. Fresh comparisons remain M8.5; use the M7A.4 completion checklist.

ADR 0070 adds native typed keyboard events. Keep mutable ReturnInteger and
completion receipts in ordinary VBA frames/fields; UI queues only bounded raw
input and waits before default widget processing. Inspection must not execute
getters. Preserve Stop/Continue/Reset, injected ANSI services, distinct paste/IME
input and existing host trust checks. Keyboard/platform and the full M7A.4
completion checklist remain partial until their executable acceptance passes.

ADR 0071 arbitrates native raw input before editor shortcuts. Preserve canonical
VBA focus and callbacks, bounded raw held-key ownership, root widget identities,
paused character/release continuation and non-text Space ordering. Debugger
F5/F8 releases must not become form KeyUp callbacks. Root input eligibility
requires more than a missing TabStop. Full M7A.4 remains in progress.


ADR 0081 adds selected native Font import and ordinary MSForms.Font/NewFont
properties with Currency sizes, typed attributes and independent container defaults.
Font inspection stays inert; UI uses local styles and caller-owned face selection.
The static nominal table ceiling is now 8,192 under unchanged metadata/work and VM
limits; host permissions stay 4,096. The ADR 0073 event-compaction regression must
still stay below 4,096 and retain its no-unused-field-alias checks. Do not use the
new ceiling to conceal Event metadata regression or omit public events. Full
M7A.4 and fresh Office font/platform acceptance remain open.

ADR 0082 adds ordinary typed Font property setters. Preserve explicit reference
identity, old aliases, root lazy reload and independent copied container defaults;
inspection stays inert and no budget/VM/permission change is made. Font-specific
Nothing/external/other-assignment and ambient/platform comparisons and full M7A.4
remain open; reference-derived tests are not fresh Office observations.

ADRs 0083–0085 correct Page appearance ownership (no Font/BackColor/ForeColor),
retain native borders/effects/backgrounds, add typed ordinary property interactions
and published Font aliases, and paint local clipped edges/transparency. Preserve
MultiPage-owned defaults, canonical objects, file versus insertion defaults, root
lazy loading, exact types/errors and inert inspection/resources. Do not add disabled
or non-applicable members or raise any budget. Full M7A.4/platform acceptance and
fresh Office observations remain open.

ADR 0086 adds selected Label/TextBox/ListBox/ComboBox TextAlign. Translate native
PARAFORMAT centre/right values to the different public fmTextAlign values; keep
ordinary Long state independent of Font, text, selection and events. Queries stay
inert and preserve authored form fields. Local text and caret geometry must agree;
ComboBox popup rows do not inherit edit alignment. No new VM/protocol/format,
canonical store or budget is added. Other applicability, wrap/AutoSize/platform
cases and full M7A.4 remain open; fresh Office observations remain M8.5 NOT_RUN.

Native caption WordWrap follows ADR 0087: ordinary Boolean state, distinct native
file/runtime defaults and fixed-bound local rendering. Keep complete text/cursor
rows when measuring unwrapped multiline TextBox viewports; do not mutate font
caches or canonical values to size widgets. Dynamic TextBox defaults to True.
Other flags, Boolean-control wrap, AutoSize/scrolling and broader M7A.4 acceptance
remain open; no VM/host budget, .tvform or native persistence change is implied.

ADR 0088 extends native Boolean-control WordWrap through ordinary canonical fields
and validated flags. Preserve Value/Null/groups/events and inert source/resources.
Keep real widget styling/id, checked/mixed accessibility and bounded caption layout;
ToggleButton centring must be checked with actual pixels, not only row counts.
Other Alignment/TextAlign, AutoSize/scrolling and full M7A.4 remain open. No VM,
foreign, budget, persistence or legacy-format change is implied.

ADR 0089 forwards native WASI imports through plain JS functions in the test
runner. Preserve exact values/errors/exit status, args/env/preopens isolation and
all guest/VM/stack budgets. Run the five independent runner guests plus full WASI
suites; never substitute a larger diagnostic stack or claim a confirmed V8 cause.

ADR 0090 adds native CheckBox/OptionButton Alignment. Translate bit 13 inversely
into ordinary Long state/defaults; do not enable ToggleButton's disabled member.
Keep caption text left aligned and clipped independently of the indicator using
local paint atoms, real widgets, full accessible labels/state and runtime-name
fallback for empty captions. Preserve inert inspection, canonical Value/Null/
groups/events/resources, global style and all budgets. Full M7A.4 stays open.

ADR 0091 adds native TextBox/ComboBox HideSelection and True defaults. Preserve
canonical text/ranges/events and inert resources/inspection. Paint inactive ranges
only in local clipped galley copies; query focus/style and decorate outside font
locks. Retain caller style/font caches, edit permission, non-applicable flag
refusals and all budgets. Full M7A.4/fresh Office acceptance remains separate.

ADR 0092 adds selected native AutoTab through ordinary focus/validation frames.
Preserve actual edit cause separately from caret-only input: autocomplete can
keep text unchanged. Programmatic assignment is passive; current policy, window,
focus serial and eligibility guard reentrant callbacks. Keep existing trust,
continuations, resources and all budgets; the full M7A.4 checklist stays open.

ADR 0093 adds native SelectionMargin flags/defaults and deferred displayed-line
selection/drag. Preserve theme-local padding, canonical UTF-16/CRLF ranges and
post-callback geometry/policy/focus; selection-only commands stay passive. Empty
native fields render blank with accessible names. Preserve all trust, resources,
continuations and budgets; the full M7A.4 checklist stays open.

ADR 0094 adds selected TextBox ScrollBars and UI-only tagged viewport offsets.
Keep text/ranges in VBA, inspection/resources inert, theme-local chrome, current
geometry/policy/focus after mouse callbacks, reset and caret following. ListBox
stored Both does not expose a public member. Preserve all budgets/authority;
AutoSize/hidden-bar/repeat/platform and full M7A.4 remain open.

ADR 0095 separates TextBox caret scrolling from bar visibility. Preserve fixed
bounds, complete glyph/cursor rows, canonical UTF-16/CRLF state, tagged UI offsets,
scrolled hit geometry, authorized mouse defaults and Reset. Accessible chrome uses
caller styles; no API, format, foreign capability or budget changes. Full M7A.4,
AutoSize/wheel/repeat/platform and geometry/font-change cases remain open.

ADR 0096 implements native CommandButton TakeFocusOnClick with ordinary Boolean
state and existing authorized input paths. Preserve text focus/ranges and dirty
validation for False, normal Tab/explicit focus, callback/Reset guards and inert
resources. Validate queued control membership after ordinary writes settle and
by object identity after index shifts; do not grant input permission or retain
VM roots in UI state. Tick with the active session's lower slice when configured.
No budget is raised; full M7A.4 and fresh Office acceptance remain open.

ADR 0098 activates explicit inert native form/source-only MS-OVBA encoding for
M7A.5. vba-project owns bounded format encoding and reopen checks; the IDE owns
compile-checked, project/source/design revision snapshots; callers own IO and
package/signature/trust policy. No new VM/foreign authority or budget is granted.
Retain imported originals separately, never write stale caches/signatures, and
never acknowledge failed/stale/foreign saves. Both host package workflows must
pass before marking M7A.5 complete. Browser download requests alone do not clear
dirty state. See docs/decisions/0098-native-form-encoding-and-source-only-writeback.md.

M7A.5 host persistence uses explicit edited XLSM/DOCM copy export. Format crates
strip invalidated signatures on document clones; engine IO serializes/reopens;
UI adapters revalidate document/project/source/design identity after selection and
commit atomically. Ordinary document Save preserves original VBA bytes. Exported
copies/browser downloads do not clear the original editor's dirty state. Native
`.frm` Save acknowledgment follows confirmed file IO only. Preserve these distinct
receipts; see docs/m7a5-acceptance.md and ADR 0098. Final gates are recorded in
implementation progress; M7A.6 removal remains separate.

M7A.5 is COMPLETE for the explicit common-profile native form Save/Export and
edited XLSM/DOCM copy workflows in docs/m7a5-acceptance.md. Both fork gates pass;
receipts are in implementation progress. Ordinary document Save retains original
VBA bytes, copy export retains editor dirty state, and browser download requests
never acknowledge a disk commit. Next is M7A.6 legacy `.tvform` removal, then
M7A.7 integration. Preserve the P9 and M8.5 boundaries.
