---
name: tiny-vba-debugger
description: Design, implement or review tiny-vba breakpoints, stepping, inspection, watches, debugger actions, DAP and pause/disconnect lifecycle. Use for execution-control correctness, including editor-to-debugger behaviour.
---

# Debugger execution contract

Read [debugger contract](../../../docs/debugger.md) and the state/lifecycle sections of
[editor/debugger UI](../../../docs/editor-debugger-ui.md). Inspect VM checkpoints and controller
state before adding controls. Use the editor UI skill as well when changing visual interaction.

Follow [IR 1 checkpoint/continuation semantics](../../../docs/bytecode-vm-specification.md): distinguish
normal checkpoint pauses, post-Stop pauses, fault snapshots, quota yields and host waits. Continue grants
one permit only for the paused checkpoint visit; step targets use frame identities and dynamic visits.
The M3 headless VM/controller subset is now implemented; the runtime contract records its limits.

## Required invariants

- Pause at source statement boundaries with explicit frames and instruction state. Keep separate
  statement identities on a shared line; preserve compiled-source versions in breakpoint mapping.
- Continue/step must resume the paused state without restarting or repeating earlier host writes.
  Step into/over/out must respect call depth, loops, breakpoints and error-handler jumps.
- Release mutable host borrows before pause/yield/callback. Stop, cancel, disconnect and host close
  must have explicit outcomes and release retained references/resources under the host policy.
- Inspection handles belong to a pause/session generation; reject stale frames, values and objects.
  Validate supported local edits using shared language rules before changing execution state.
- Watches default to read-only inspection. Property Get, default members and functions may cause
  effects; report unsupported/unsafe evaluation rather than silently executing them. Evaluation
  must obey the same capabilities as macros. M10 Immediate calls require explicit policy and an
  isolated bounded child VM; real host effects must remain visible and permission-checked.
- Editing paused source marks compiled code stale. Require stop/recompile/restart for changed code;
  do not imply live code replacement. Keep diagnostics and execution highlights version-correct.
- Bound execution slices and output batches so UI repaint, pause and cancellation remain responsive.
  Controller/VM APIs stay toolkit independent; native UI calls Rust directly. DAP is optional and
  advertises only capabilities actually implemented.

Prove break/inspect/edit/resume with deterministic cases that count host writes, plus nested calls,
loops, runtime errors and cancellation. Exercise stale handles and disconnect/close during pause.
Keep UI state/actions consistent with the controller; unsupported actions are disabled or return
explicit diagnostics. A CLI or DAP test does not satisfy the required visual debugger milestone.

## Implemented M3 contract

Read [runtime](../../../docs/runtime.md) and [ADR 0007](../../../docs/decisions/0007-m3-execution-and-debugger.md).
The m3-core-v1 runtime is intentionally narrower than structural IR. Preserve explicit frame/alias
storage, one-visit checkpoint permits, saved fault PCs, generation checks and nonreplayed continuations.
Use vba-runtime for operations/edits. VM host dispatch must use checked precommit return/writeback
validation and explicit per-site permissions; foreign aliases remain unsupported. Run cargo xtask ci,
wasm and wasm-run (native plus actual WASI execution). Runtime example results do not mark unrun M0
or external scenario assertions PASS. M8/M10 implement the graphical UI and optional DAP;
current handler semantics are defined by the M4 runtime profile.

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

M10 is implemented under [ADR 0018](../../../docs/decisions/0018-m10-classic-tools-and-debugger-policy.md).
Use the [acceptance map](../../../docs/ui/m10-implementation.md) for exact restrictions.
Checkpoint inspection must reject foreign/stale/permitted visits; conditional watches are read-only
and visibly disable on budget exhaustion. Relocation rejects branches/handlers/faults/ancestor frames
and invalidates the old pause without execution. DAP wraps vba-ide, not a second session controller;
JSON is optional transport only. Run native/WASI protocol tests and CLI subprocess lifecycle tests.

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

M6 metadata/code projection is defined by [ADR 0030](../../../docs/decisions/0030-attribute-metadata-and-code-projection.md).
Keep raw source canonical; use CodeProjection for visible edits/coordinates and
preserve hidden metadata/newline bytes through history and export. Attribute
records never create executable checkpoints. Document rename requires host
validation and revision-checked acknowledgement; unsupported hosts refuse.

M7A.4 has started under [ADR 0042](../../../docs/decisions/0042-userform-instance-routing.md).
The [first instance packet](../../../docs/m7a4-userform-instances.md) removes the
predeclared-only restriction in shared tiny-vba. FormInstance keys separate native
runtime objects from designer/source IDs; UI input/window scopes use those keys.
Read-only VM class queries never instantiate objects; active and parked receivers
remain lifetime roots. Keep Reset/source/resources inert. Broader lifecycle,
control/property packets, fork integration and fresh Office evidence remain open.

[ADR 0044](../../../docs/decisions/0044-userform-window-and-code-lifetime.md) separates
UserForm display/default ownership from referenced code lifetime. Named control
getters reload; inert inspection does not. Generic root release validates class/
object and protects document code names. Internal release waits remain bounded,
report Running and preserve slice quotas; Terminate runs only at normal finalization.

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


[ADR 0067](../../../docs/decisions/0067-vartype-default-value-inspection.md) adds
IR 13 optional default inspection for VarType. Preserve source-order evaluation,
normal class frames/authorized host calls, getter errors and one-shot Stop/wait
continuations. Resolver presence metadata is inert and identity checked; actual
getters remain executable. Older IR consumers refuse this capability. Watches
stay inert and all existing budgets remain. Full M7A.4/M8.5 are separate gates.


## Lazy project-class array elements

TouchElement may construct only during authorized program execution. Local/array snapshots and safe watches remain inert. Preserve element references and caller instruction state across constructor Stop/failure; do not replay indices or earlier arguments. See [ADR 0068](../../../docs/decisions/0068-lazy-project-class-array-elements.md). Existing budgets remain unchanged; full M7A.4 remains open.


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

ADR 0078 orders document activation at the raw-input boundary. Use the opaque
non-reused SessionInstance, existing host authorization, bounded raw batches and
completed document-pass acknowledgment. Keep debugger input available at Stop;
repaint before replay after skipped/hidden document passes. Reset/source/document/
trust/geometry changes and authored activation invalidate retained input. No new
VM/foreign behavior or increased budget is implied; full M7A.4 remains partial.

ADR 0079 orders earlier native key/text/paste/IME prefixes through the existing
bounded keyboard barrier before document Deactivate. Wait for receipts, defaults,
ordinary callbacks and any composition started before the press. Preserve IME
continuation through Stop and debugger shortcut ownership. Context invalidation
clears unapplied prefixes; modal redirection cancels only the later document
gesture, preserving its executing keyboard receipt. The host API and all budgets
stay unchanged; full M7A.4/platform acceptance remains open.

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
