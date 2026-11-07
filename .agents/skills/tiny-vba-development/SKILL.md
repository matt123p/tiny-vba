---
name: tiny-vba-development
description: Implement or review tiny-vba changes using the project design contracts. Use as the entry point for repository development and route to the focused architecture, semantics, debugger, UI, host or conformance skill.
---

# tiny-vba development

Licensing is MIT OR Apache-2.0 on explicit maintainer direction; see
[ADR 0035](../../../docs/decisions/0035-dual-license.md). Retain third-party notices.

M6A — VBA intrinsic-function coverage — was accepted on 2026-11-02. See the
[intrinsic scope](../../../docs/vba-intrinsic-functions.md),
[acceptance record](../../../docs/m6a-acceptance.md), and
[ADR 0033](../../../docs/decisions/0033-intrinsics-before-package-acceptance.md).
M7 is complete for [m7-unchanged-v1](../../../docs/m7-acceptance.md).
The next milestone is M7A (native Office forms and removal of `.tvform`), then M8; fresh Office comparisons remain M8.5.

The [M6A framework](../../../docs/m6a-implementation-framework.md) and
[ledger](../../../docs/m6a-work-ledger.md) retain design and evidence for follow-up
intrinsic work. M6A's IR 8 and descriptor contracts are implemented; inspect code
and the current compatibility matrix before extending them.

Read [AGENTS.md](../../../AGENTS.md), [progress](../../../docs/implementation-progress.md)
and the relevant [implementation gate](../../../docs/implementation-plan.md). Inspect actual code
before choosing a change; planned crates and APIs are not implemented capabilities.

## Select the relevant design skill

Load only the skills needed for the task; a change crossing boundaries may need more than one.

| Task | Skill |
|---|---|
| Crate ownership, dependencies, public contracts, design proposals | [architecture](../tiny-vba-architecture/SKILL.md) |
| Parser/binder, values, bytecode, runtime and language behaviour | [language semantics](../tiny-vba-language-semantics/SKILL.md) |
| Pause/resume, stepping, watch evaluation, debugger/DAP lifecycle | [debugger](../tiny-vba-debugger/SKILL.md) |
| Editor panels, theme, interaction, standalone/embedded IDE | [editor UI](../tiny-vba-editor-ui/SKILL.md) |
| Foreign calls, host lifetime/capabilities, project import | [host integration](../tiny-vba-host-integration/SKILL.md) |
| Fixtures, oracle evidence, corpora, compatibility reports | [conformance](../tiny-vba-conformance/SKILL.md) |

For each material design choice, identify its owning layer and applicable contract. Follow current
user direction and adopted decisions over historical supplied design snapshots. Resolve routine
implementation choices within that scope; record material contract changes in docs/decisions and
update affected architecture, plan, compatibility and progress documents. Explain any tradeoff
against an existing invariant rather than silently changing it.

Product code is Rust. Follow the root safety/layering rules and keep failures explicit. Preserve
headless native/WASM use and the required editor/debugger UI owned by this repository.

Run the relevant checks plus the repository gates `cargo xtask ci` and `cargo xtask wasm`.
Report exact verification, unavailable checks and remaining gaps; compilation is not proof of
VBA compatibility. Leave the next smallest implementable task recorded in progress.

[ADR 0010](../../../docs/decisions/0010-m4-profile-and-verification-hardening.md) publishes
m4-scalars-v1; do not silently expand m3-core-v1. Keep failure classifications and nonzero
CRASH/TIMEOUT/MISMATCH exits intact, compare exact payloads, and check narrowing bounds before casts.
Confirm xtask's logged workspace matches the intended checkout; shared binaries may originate elsewhere.

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

M10's implemented classic tools, advanced debugging and optional DAP are defined by
[ADR 0018](../../../docs/decisions/0018-m10-classic-tools-and-debugger-policy.md) and the
[acceptance map](../../../docs/ui/m10-implementation.md). Preserve shared bounded source history,
literal scoped search, view selection/focus and explicit browser transfer. Set Next Statement
supports verified straight-line checkpoints only; watch evaluation stays read-only; Immediate
calls require explicit isolated-evaluation policy. Metadata does not imply library/type binding.
The [remaining plan](../../../docs/implementation-plan.md) now uses new M1–M8
numbering. Milestone IDs in the implemented contracts above are historical; see
the [mapping](../../../docs/implementation-progress.md#plan-renumbering--2026-10-28).
New M1 owns host type binding, M2 shared classes/standard Office forms, M5 Excel
events/UDFs, and M6/M7 remaining host/package acceptance. Third-party ActiveX/OLE
controls not supplied with Office are out of scope. Native extensions/translation
and AI assistance are removed from planned scope; do not schedule them from the
earlier supplied specification. Preserve fork ownership and existing UI evidence.

The 2026-10-27 user correction includes working scoped word/member completion, source signature help
and live red/underlined parser diagnostics in historical M8 (UI specification section 16).
Declarative reference metadata/List Constants belong to the historical M10 implementation;
executable external type/reference binding is new M1. Preserve original CR/CRLF source
bytes through the editor view mapping and Static state across compatible same-host reruns until Reset.

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

M7A.4 has started under [ADR 0042](../../../docs/decisions/0042-userform-instance-routing.md).
The [first instance packet](../../../docs/m7a4-userform-instances.md) removes the
predeclared-only restriction in shared tiny-vba. FormInstance keys separate native
runtime objects from designer/source IDs; UI input/window scopes use those keys.
Read-only VM class queries never instantiate objects; active and parked receivers
remain lifetime roots. Keep Reset/source/resources inert. Broader lifecycle,
control/property packets, fork integration and fresh Office evidence remain open.

M7A.4 now also includes the [native runtime increment](../../../docs/m7a4-native-runtime.md)
under ADR 0043: six flat control families, selected property/input semantics,
Controls/UserForms lookup and bounded transient project inspection. Canonical
VM state remains authoritative; queries never load forms or invoke getters.
Keep intrinsic providers/wait routing intact and do not extend .tvform. The
milestone remains in progress with its remaining requirements recorded there.

[ADR 0044](../../../docs/decisions/0044-userform-window-and-code-lifetime.md) separates
UserForm display/default ownership from referenced code lifetime. Named control
getters reload; inert inspection does not. Generic root release validates class/
object and protects document code names. Internal release waits remain bounded,
report Running and preserve slice quotas; Terminate runs only at normal finalization.

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

Native container focus/validation follows [ADR 0048](../../../docs/decisions/0048-native-container-focus-and-validation.md).
Keep ActiveControl and dirty/cancel state in ordinary generated classes. Focus
commands execute VBA and require the same authorization and modal ownership as
input; widget focus follows canonical state after cancellation/Stop/resume.
Preserve typed ByVal return identity, inert source/resources and leaf event
exposure. Broader M7A.4 control/platform semantics remain open.

Native TextBox behavior follows [ADR 0049](../../../docs/decisions/0049-native-textbox-selection-and-input.md).
Keep UTF-16 text/selection canonical in ordinary classes, input/cursor commits
atomic and executable input authorized. Preserve CRLF and scalar cursor mapping,
native flags/resource identity, password masking and ordinary callback/debugger
boundaries. Rendering must not replace host fonts/global policy or alter data on
missing glyphs. No .tvform expansion or full M7A.4 acceptance is implied.

Native numeric controls follow [ADR 0050](../../../docs/decisions/0050-native-spinbutton-and-scrollbar.md).
Keep Long state/events in ordinary classes, distinct binary records bounded and
resource retention inert. Numeric input executes VBA and needs the same policy
as other form commands. Preserve reversed bounds, error/coercion semantics and
Change/spin/Scroll continuations. No .tvform expansion or full milestone sign-off.

Native unbound ListBox follows [ADR 0051](../../../docs/decisions/0051-native-unbound-listbox.md).
Keep list/selection state in ordinary VM fields, bounded indexed row operations,
executable input authorization, focus/validation and callback continuations. Do
not expand .tvform or claim full list/binding/ComboBox/M7A.4 parity.

Native ComboBox follows [ADR 0052](../../../docs/decisions/0052-native-combobox.md).
Reuse ordinary list rows, canonical UTF-16 text/selection/drop state and existing
locale services/refusals. Preserve authorized input, typed validation, source/
resource identity and nonreplayed Change/Click/drop continuations. No new codec,
COM or second mutable store; broader M7A.4 work stays explicit.

Native ListBox/ComboBox bulk loading follows [ADR 0053](../../../docs/decisions/0053-native-list-array-loading.md).
Keep source arrays independent, validate all cells before canonical replacement,
and use ordinary control refresh/event continuations. No error-handler rank probes,
new intrinsic or mutable host store; array reads and broader M7A.4 remain open.

Native array readback follows [ADR 0054](../../../docs/decisions/0054-native-list-array-readback.md):
copy canonical cells with explicit bounds, preserving Err, selection and events.
[ADR 0055](../../../docs/decisions/0055-native-form-metadata-and-control-tips.md) keeps native
metadata in ordinary fields and tooltip hover inert, bounded and theme-injected.
Help/F1 routing, native persistence/removal and the broader M7A scope stay open.

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

ADR 0074 adds selected TextBox pointer selection after ordinary mouse callbacks.
Keep text/selection canonical and retain only immutable layout parameters/raw
anchors. Re-read values after callbacks, preserve cancelled/redirected focus and
Stop/Reset, and authorize InstanceText like other executable form commands.
Native AutoWordSelect bit 27 is inverted relative to its VBA property; do not
confuse persisted defaults with new dynamic controls. Full M01/M7A.4 remains open.

ADR 0075 adds selected composite/list/numeric/tab-header pointer defaults. Retain
only region geometry/identities and raw anchors; ordinary VBA fields own values.
Popup rows inherit the input barrier; numeric repeats use injected time and stop
on release even during Stop. Inert tab queries preserve identity after reorder
and reject detached members. Retained modal-display metadata never authorizes
input. Keep the complete M7A.4 checklist open for remaining acceptance.

Retained native modal presentation follows parked Show order, independent of
object creation, and never grants executable input permission. Selected nested modal input follows ADR 0076; the broader L02 checklist stays open.

ADR 0076 parks raw input scopes for modal callers (at most eight existing waits)
and keeps a shared 64-event queue. Preserve original capture release, owner-local
keyboard/mouse barriers, canonical receipts, normal authorization and full Reset
invalidation. Selected nested Stop/unload/Reset cases do not finish L02/M7A.4.

ADR 0077 adds canonical native window activation and separate keyboard receipt/
destination ownership. Activation commands execute VBA and retain existing trust
checks. External viewport focus and passive hover stay inert. Embedders must
apply document activation before document input; automatic raw document barriers
remain open. Preserve reentrant transition guards, modal ownership and bounded
raw input; L01/L02 and full M7A.4 remain partial.

ADR 0078 orders document activation at the raw-input boundary. Use the opaque
non-reused SessionInstance, existing host authorization, bounded raw batches and
completed document-pass acknowledgment. Keep debugger input available at Stop;
repaint before replay after skipped/hidden document passes. Reset/source/document/
trust/geometry changes and authored activation invalidate retained input. No new
VM/foreign behavior or increased budget is implied; full M7A.4 remains partial.

ADR 0080 keeps native OLE_COLOR bits in ordinary BackColor/ForeColor fields and
native metadata, never the legacy codec or a second mutable colour store. UI
resolution uses injected theme/palette policy and reports unresolved client
entries. Preserve typed errors, inert inspection, instance/lifetime/Stop/Reset and
original resources. Existing collections own runtime MultiPage/Page lookup.
Font/border/transparency, ambient/platform appearance and full M7A.4 stay open.


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
