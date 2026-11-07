# Architecture

Status: M0–M4 complete for the documented [M4 language profile](m4-language.md), with
verified IR 1/2/3/4/5 and explicit-frame execution; the M5 native host contract is implemented
([ADR 0013](decisions/0013-m5-host-contract.md)). M6 read-only project decoding is implemented
([ADR 0014](decisions/0014-m6-project-decoding.md)); M8's reusable graphical IDE and native/web shells are implemented under
[ADR 0016](decisions/0016-m8-embeddable-editor.md). Fork-owned workbook integration remains a separate gate.

The engine and toolkit-independent IDE model compile and test without Office, UI toolkits,
application forks or corpus downloads. This repository also owns an optional editor/debugger UI
and standalone IDE; those upper layers may depend on UI toolkits.

Accepted 2026-11-02: [M6A intrinsic coverage](vba-intrinsic-functions.md) precedes M7
under [ADR 0033](decisions/0033-intrinsics-before-package-acceptance.md). It uses
the existing syntax/binding, values/runtime, verified IR/VM and shared IDE layers;
environmental services remain explicit host injections. This planning decision
added no crate, dependency edge or ambient IO authority.

The [M6A implementation framework](m6a-implementation-framework.md), adopted in
[ADR 0034](decisions/0034-intrinsic-implementation-framework.md), specifies a
shared descriptor catalogue, argument call plan and IR 8 extended-call capability.
The legacy instruction remains supported; environmental calls use explicit
context/host services. See [M6A acceptance](m6a-acceptance.md) for evidence and
profile limits.

M7 package acceptance is implemented for the [authored profile](m7-acceptance.md).
[ADR 0037](decisions/0037-m7-package-preservation-and-writeback.md) keeps source decoding
read-only in vba-project and package/trust/signature policy in the forks. An optional
xtask gate consumes sibling workflow tests; ordinary core CI has no sibling dependency.
No shared crate, dependency edge, IR version or ambient authority was added.

[M7A](implementation-plan.md) now precedes M8 under [ADR 0038](decisions/0038-native-office-forms-and-compatibility-only.md).
The target is native Office forms and compatible behavior, with `.tvform` removed
from shipping workflows. Reuse internal design/runtime layers; reusable bounded
form codecs stay shared, while forks own document/package commits and host objects.
Native edited persistence requires the reviewed encoder/signature contract. This
planning change adds no crate, dependency, IR capability or implemented support.

VBA source flows through syntax → semantics → verified IR → explicit-state VM.
The dependency graph is separate from this data flow: IR does not need a dependency on the binder.
The adopted [IR 1 specification](bytecode-vm-specification.md) defines instruction/stack contracts,
verified immutable programs, calling conventions, continuations and budgets. M2 implements the structural IR/verifier; the runtime subset is implemented at M3; the M4 extension is adopted in [ADR 0011](decisions/0011-m4-language-extension.md).
[ADR 0004](decisions/0004-bytecode-vm-contract.md) and
[ADR 0006](decisions/0006-m2-binding-and-verification.md) place compilation/lowering in vba-compiler,
which depends on diagnostics/syntax/semantics/IR/values. IR remains independent of the binder.
See the [M2 compiler contract](compiler.md) for supported source, API and structural capability target.


| Crate | Responsibility | Direct normal dependencies |
|---|---|---|
| vba-diagnostics | Source identities, spans, diagnostic codes | none |
| vba-values | Typed Variant, arrays, references and l-values | diagnostics |
| vba-syntax | Lexer/parser and source-preserving AST | diagnostics |
| vba-semantics | Scope/type/call binding | diagnostics, runtime, syntax, values |
| vba-ir | Bytecode, verifier, constants and source maps | diagnostics, values |
| vba-compiler | Parse/bind coordination and deterministic verified IR lowering | diagnostics, syntax, semantics, ir, values |
| vba-runtime | Shared coercion/operators/builtins/error rules; bounded encoding_rs code-page conversion for string/byte intrinsics | diagnostics, values, encoding_rs (=0.8.42) |
| vba-foreign | Generic host call/result/wait protocol; protocol version 1 and validated source imports | diagnostics, values |
| vba-vm | Frames, instruction pointer, budgets, inspection hooks; host-root binding and protocol checks | diagnostics, ir, values, runtime, foreign |
| vba-debug | Breakpoint/step/inspection controller | diagnostics, vm |
| vba-dap | Optional bounded M10 Debug Adapter Protocol frontend | ide, serde_json (=1.0.154) |
| vba-project | Bounded read-only MS-OVBA source/project, native UserForm and inert raster decoding | diagnostics, cfb (=0.14.0), encoding_rs (=0.8.42), image (=0.25.10; explicit raster features, ADR 0059) |
| vba-testkit | Deterministic generic foreign host, trace support and bounded embedding driver | debug, diagnostics, foreign, ir, values, vm |
| vba-conformance | Scenario validation, typed expectations and M4 outcome judgment; comparison/reports later | diagnostics, testkit, values, vm |
| vba-cli | Source parsing, compilation/disassembly/execution, fixture validation and optional DAP stdio | conformance, diagnostics, syntax, compiler, ir, values, vm; dap optional |
| vba-ide | Toolkit-independent source buffers, commands, compiler/debug-session orchestration and safe watches | compiler, debug, diagnostics, foreign, ir, project, runtime, syntax, values, vm |
| vba-ui-egui | Reusable Classic editor/debugger rendering; explicit commands and scoped theme/IDs | ide, egui |
| vba-ide-app | Native/browser shell, source IO, preferences and capture harness | ide, ui-egui, eframe; image native-only, js-sys/wasm-bindgen/futures/web-sys browser-only |
| xtask | Maintenance/build/render commands | none |

DAP implements the [M10 policy](decisions/0018-m10-classic-tools-and-debugger-policy.md) above the headless IDE. Project implements [M6 source decoding](project-import.md). Runtime, VM and debug implement the [M3 contract](runtime.md); semantics/compiler/IR implement M2 with generic member lowering added at M3.
Syntax implements the bounded lexer/parser described in [frontend](frontend.md) and
[ADR 0005](decisions/0005-m1-frontend.md); its only dependency remains diagnostics.
Diagnostics, values, foreign, testkit and conformance implement the bounded M0 contracts in
[ADR 0003](decisions/0003-m0-source-scenarios-foreign.md).
`tools/architecture.json` is the enforced graph. New external dependencies require explicit graph review.
`vba-testkit` and `vba-conformance` are development packages; shipping libraries can use them only as
dev-dependencies. The development CLI has a reviewed normal conformance dependency for validation.
All packages are currently unpublished. No crate automatically loads corpus code.

M3 runtime instances now use an opaque, non-reused identity independent of the caller's routing
label for location ownership and debugger/inspection binding. [ADR 0008](decisions/0008-vm-instance-isolation.md)
records this runtime-only refinement; it changes no IR operands, packages or dependency edges.

## Ownership boundary

The generic foreign protocol carries invocation kind, target/reference, named/positional/omitted arguments,
ByRef obligations and explicit result/error/wait state. Its owned M0 Rust prototype is proven against a deterministic host; VM wiring remains M3 work.
It must not know Excel/Word names, Range behavior, Office enums or document storage.
The applications implement a bridge to their own Office compatibility layers.
Root-object imports preserve member Get/Call/Let/Set dispatch; fixed-member aliases
retain their M5 shortcut semantics for methods ([ADR 0017](decisions/0017-imported-root-members.md)).
Property getter aliases resolve before chaining under [ADR 0025](decisions/0025-getter-alias-member-dispatch.md); the returned object retains its identity.
The VM owns VBA class instances;
external object references preserve host identity/lifetime without defining a parallel Office registry.

No `vba-office-bridge`, `vba-excel-host`, `office-*` or `automation-*` crate belongs in this workspace.
The forks own Office dispatch/catalogues, object handles, document trust/lifecycle, OOXML,
undo/recalc and document UI. tiny-vba owns the reusable VBA editor/debugger UI and standalone IDE.
The host supplies document integration and prompts through explicit interfaces.
The interpreter owns VbaValue and source decoding; conversions to Office AutomationValue occur in fork bridges.
A generic foreign-host double here proves interpreter mechanics; an Office-shaped fake belongs with the Office layer.

Use synchronous native Rust calls for ordinary host work. Release mutable host borrows before callbacks,
pause or yield. Budget execution and allocations. The VM never calls DAP or the UI and never performs ambient IO.
Inspection hooks and statement safe points belong to VM; debug controls them. DAP wraps the IDE orchestration, which uses debug and VM through native APIs.

## Files and portability

`vba-project` consumes project bytes/source and returns module metadata. It does not own OOXML import/export,
project trust or macro attachment. P-code, protected-project bypass and binary source writeback are excluded.
Platform facilities stay in vba-ide-app/eframe. `cargo xtask wasm` compiles every library for
wasm32-unknown-unknown; `wasm-run` executes language, import, callback and headless IDE acceptance
under WASI. `web` packages the actual browser UI; Chromium exercises its debugger keyboard path.

## Implemented M8 editor/debugger layers

Adopted scope: [ADR 0002](decisions/0002-editor-debugger-ui.md). Product contract:
[editor/debugger UI](editor-debugger-ui.md). Implemented under [ADR 0016](decisions/0016-m8-embeddable-editor.md):

| Package | Responsibility | Allowed direction |
|---|---|---|
| vba-ide | Toolkit-independent editor/project/session state and actions | compiler, debug, diagnostics, foreign, IR, project, runtime, syntax, values, VM; no toolkit/IO |
| vba-ui-egui | Embeddable editor/debugger panels | ide, egui and compatible UI support; no Gridcraft/Wordcraft dependency |
| vba-ide-app | Standalone native/web application shell | ui-egui, ide; eframe/platform facilities at this layer |

Gridcraft embeds vba-ui-egui in its UI composition layer and supplies a host bridge.
The core, VM and debugger controller never depend on these packages. The embedded UI accepts
a caller-owned egui context and style; it must not replace global fonts, theme or window settings.
Native debugging uses in-process Rust APIs; DAP remains an optional adapter for external editors.

The 19-node graph includes these real packages and reviewed external dependencies. Layers checks
reject upward GUI/platform edges from engine/IDE model. An EditorUi owns an independent widget
namespace so multiple documents can share a context. Headless and rendered/browser checks remain
separate. [Embedding instructions](ui/embedding.md) and [acceptance status](ui/m8-acceptance.md)
record actual support, commands and remaining broader-spec/fork-owned gates.

M4's execution profile was m4-conformance-v1 (ADR 0012), distinct from the historical M3 matrix;
the current consumer is m5-host-v1 (ADR 0013), which preserves the M4 scalar/array/handler matrix.
Typed scenario judgment and diagnostic/result classification stay in vba-conformance; CLI owns
compile/run orchestration. Runtime owns narrowing bounds and scalar comparison rules. Infrastructure
xtask resolves its checkout at runtime and prints that identity before gates. No dependency edge or
IR operand changes were needed for this hardening.

Semantics now depends on runtime for constant/default conversion, keeping one coercion implementation.
IR stays independent of syntax, semantics and execution. Values owns bounded arrays and canonical
element locations; the VM owns per-frame handlers and explicit project-state transfer. See
[the IR 2 extension](m4-language.md) and ADR 0011 for versions, ownership and limits.

[M4 acceptance](m4-oracle-report.md) now includes 153 reviewed inherited Microsoft Excel oracle
cases under [ADR 0012](decisions/0012-m4-oracle-acceptance.md). IR 3 owns array-layout/parameter
metadata and same-block canonical reference captures; IR 1/2 targets remain closed. No Cargo DAG
edge changed. Schema 2 provenance and evidence checks remain above shipping engine layers.

M7 ([ADR 0015](decisions/0015-m7-callbacks-and-lifecycle.md)) keeps the VM non-reentrant and adds
the bounded generic callback queue, terminal-only delivery, static reset and host root rebinding
inside vba-vm; no new crate or DAG edge. Forks implement event sinks over this queue and own event
ordering and `Application.EnableEvents` policy above it. Lifecycle closure is independent of the
last invocation outcome; reset prepares the original entry again while retaining the instance,
queued callbacks and lifetime budgets. Callback validation precedes queue mutation, and queued
payload/metadata share the session allocation budget.

M5 ([ADR 0013](decisions/0013-m5-host-contract.md)) adds IR 4 with hidden `HostRoot` slots and the
`host-roots-v4` feature; `Capabilities::ir4_structural()` is the compiler's structural default while
IR 1/2/3 stay selectable and verified. Semantics and compiler gained reviewed foreign edges for
import validation/binding, and the testkit gained debug/ir/vm edges for its reusable embedding
driver (`run_to_boundary`, `statement`). `vba-foreign` owns the native protocol version; the VM
checks `Host::protocol` before dispatch and binds named roots through `HostBinding`. No Office
names or catalogue enter the engine; workbook-level FillCells evidence stays fork-owned.

M6 adds reviewed external dependencies only to vba-project. Its bounded in-memory CFB adapter
returns source/module/reference metadata and explicit corrupt/unsupported/protected/limit errors;
no compiler, VM or host edge exists. [ADR 0014](decisions/0014-m6-project-decoding.md) records
licences, portability, source identity semantics and work limits. External requirements are
pinned in tools/architecture.json separately from the unchanged 16-node workspace DAG.

## Remaining scope ownership

The [remaining plan](implementation-plan.md) uses new M1–M8 numbering; earlier
architecture and ADR milestone IDs above are historical. M1/M2 own generic host
types and shared class/form contracts; Office objects, event ordering, trust and
document persistence remain fork-owned in M3–M7. M8 owns final conformance/release.
Third-party ActiveX/OLE controls not supplied with Office applications are excluded.
Native extensions/translation and AI assistance are removed from planned scope.
[ADR 0019](decisions/0019-remaining-plan-scope.md) changes no implemented DAG/API.

New M1.1's first increment adds validated `vba-foreign::ReferenceSet` snapshots
to compiler/binder options. Registered enums lower to Long; constants become
typed IR constants. IDE discovery and compilation share an explicitly configured
snapshot. Nominal host object identity and storage checks now use IR 5 and an
explicit read-only host resolver under [ADR 0021](decisions/0021-nominal-host-object-types.md).
Office definitions/object stores remain in the forks; no parallel registry or
application dependency enters tiny-vba.
[ADR 0020](decisions/0020-explicit-host-reference-snapshots.md) adds no dependency,
IR version or host invocation ABI; [the stage log](m1-host-references.md) records
remaining work and execution evidence.

## Default members and iteration (ADR 0022)

IR 6 adds generic unnamed invocation intent and verified enumeration instructions.
Syntax/semantics/compiler own source binding and value/reference contexts; the VM
owns bounded cursors, canonical native array reads and continuation cleanup.
vba-foreign carries native arguments and bounded snapshot replies. The existing
crate DAG is unchanged. Hosts exclusively own default-member selection, collection
objects, signatures, ordering, lifetimes and document effects. No Office object
definition, registry, package IO or application dependency enters tiny-vba.

## Project classes (M2 foundation / ADR 0023)

Project classes use a VM-owned heap whose fields are ordinary canonical locations.
Verified IR 7 descriptors define source ownership, nominal identity, fields and
methods; native calls run on the same frames/controller as other VBA. Native
state is separate from the foreign host's existing object store. Snapshots carry
owned native state and rebind receiving-session handles. Normal lifecycle code
is interpreted; reset/cancel cleanup is inert. No new shipping crate/dependency
edge is added. The selected form designer/runtime and exchange contract are implemented under ADR 0024.

M2.2's `vba-ide::forms` model is toolkit/IO-free and owns bounded standard-control
design data, selection and atomic layout/history commands. It contains no Office
application objects, runtime control activation or persistence IO. Its document
is independent of VM Reset. The shared egui designer/runtime and source/design exchange now use this same model under ADR 0024.

## M2 implemented form profile (ADR 0024; format direction superseded by M7A)

The existing `.tvform` codec is legacy code scheduled for removal under ADR 0038.
This section describes current implementation, not the M7A native target.

`vba-ide` owns shared form design/source, the bounded legacy `.tvform` codec, generated
project-class support and form event orchestration. `vba-ui-egui` renders the
designer and runtime; the shell owns explicit native/browser IO. The VM supplies
generic parked host continuations and explicitly opted-in suspended callbacks,
with no UI/control dependency. Existing terminal-only callbacks are unchanged.
The reviewed 19-crate dependency graph is unchanged. See
[contracts and acceptance](m2-acceptance.md) and
[ADR 0024](decisions/0024-standard-form-execution-and-exchange.md).

## Host-object event subscriptions (M6)

[ADR 0029](decisions/0029-host-event-subscriptions.md) adds the explicit
`host-events-v1` capability for nominal host-object WithEvents fields. The VM
snapshots and validates live subscriptions against host-supplied generic parameter
signatures, then delivers each through existing bounded instance callbacks.
The toolkit-independent IDE exposes that path. Applications own event names,
trust, sequential ByRef propagation, cancellation and document lifecycle ordering;
no Office names or ambient subscriptions enter the shared crates.

[ADR 0031](decisions/0031-variant-for-counters.md) extends shared numeric For
controls to scalar Variant, using binder validation and existing verified
instructions/runtime arithmetic; host-specific execution remains outside the engine.

## Waiting host project calls (2026-11-03)

[ADR 0036](decisions/0036-suspended-host-project-calls.md) extends the existing VM frame
suspension and toolkit-independent IDE orchestration. The IDE validates public procedure
visibility, exact pending token, source and host identity; the VM owns frames, typed values,
ByRef outputs and cumulative limits. No dependency edge or foreign protocol variant changes.
Application forks own macro resolution policy, lifecycle events, prompts and native commits.

## M7A native control metadata (2026-11-04)

[ADR 0039](decisions/0039-m7a-native-control-metadata.md) and the
[inventory](m7a-inventory.md) record the first GridCraft-owned native worksheet
Button metadata decoder. It reads the fork's preserved graph into a bounded
read-only snapshot, with no new shared crate/edge or VM authority. Full DrawingML
identity reconciliation and worksheet UI/runtime activation remain open.

## M7A native UserForm import/attachment (2026-11-04)

[ADR 0040](decisions/0040-native-userform-import-and-attachment.md) adds one
reviewed IDE -> project edge. The headless project decoder owns native resource
results; the IDE maps them into validated design/runtime data and retains original
source/resources independently. Standalone and forks own IO, package identity,
trust and atomic publication. No new runtime registry, VM instruction, production
external dependency or library activation is added. See the
[property/event/format/host contracts](m7a-control-contracts.md). M7A.1/M7A.2
are complete for the selected three-control flat import profile; broader native
semantics, native edited persistence and removal of `.tvform` remain open.

M7A.3 (2026-11-04): GridCraft implements the nine standard native worksheet Form
Controls for [worksheet-form-controls-v1](m7a-worksheet-controls.md).
Model/engine/XLSX own state, commands and packaging; the optional adapter/shell own
VBA bindings, trusted launches and input. No shared crate/dependency edge changed.
Native UserForm semantics/persistence and .tvform removal remain M7A.4–M7A.7.

M7A.4 has started under [ADR 0042](decisions/0042-userform-instance-routing.md).
The [first instance packet](m7a4-userform-instances.md) removes the
predeclared-only restriction in shared tiny-vba. FormInstance keys separate native
runtime objects from designer/source IDs; UI input/window scopes use those keys.
Read-only VM class queries never instantiate objects; active and parked receivers
remain lifetime roots. Keep Reset/source/resources inert. Broader lifecycle,
control/property packets, fork integration and fresh Office evidence remain open.

M7A.4 now also includes the [native runtime increment](m7a4-native-runtime.md)
under ADR 0043: six flat control families, selected property/input semantics,
Controls/UserForms lookup and bounded transient project inspection. Canonical
VM state remains authoritative; queries never load forms or invoke getters.
Keep intrinsic providers/wait routing intact and do not extend .tvform. The
milestone remains in progress with its remaining requirements recorded there.

[ADR 0044](decisions/0044-userform-window-and-code-lifetime.md) separates
UserForm display/default ownership from referenced code lifetime. Named control
getters reload; inert inspection does not. Generic root release validates class/
object and protects document code names. Internal release waits remain bounded,
report Running and preserve slice quotas; Terminate runs only at normal finalization.

IR 9 class enumeration is defined by
[ADR 0045](decisions/0045-class-and-native-collection-enumerators.md):
verified DISPID_NEWENUM metadata invokes ordinary Object-returning getters before
the existing bounded foreign enumeration path. Native forwarding receivers and
active/parked enumeration snapshots remain lifetime roots. No Office class names,
object registry, dependency edge or new opcode enter the VM.

[ADR 0046](decisions/0046-lazy-project-class-variables.md) adds scalar project-class
and native-form As New under IR 10 `auto-instancing-v10`, including module/static/
instance storage, canonical ByRef preparation and constructor Stop/failure/retry.
Reads construct only when Nothing; declaration/Set targets and inspection remain
inert. Arrays and external activation refuse explicitly. Native designer launch
constructs non-predeclared forms explicitly. Broader M7A.4 requirements remain open.

[ADR 0047](decisions/0047-native-forms-nominal-types-and-library-access.md) adds
IR 11 nominal class interfaces with bounded event exposure. The native Forms
provider binds MSForms form/control/collection/return types and qualified
VBA.UserForms through explicit source-function aliases; scope shadows and missing
references remain authoritative. Ordinary canonical values/frames are retained.
This is not full native lifecycle/event/control parity or edited persistence.

[ADR 0048](decisions/0048-native-container-focus-and-validation.md) adds selected
flat native ActiveControl/SetFocus and container Enter/Exit/BeforeUpdate/AfterUpdate
with typed mutable cancellation, ordinary frames and authorized UI focus/input.
Programmatic writes do not schedule update events. Tab/Shift+Tab and pointer focus
follow canonical VBA state through cancellation and debugger waits. The broader
[M7A.4 scope](m7a4-native-runtime.md) remains in progress.

[ADR 0049](decisions/0049-native-textbox-selection-and-input.md) implements native
TextBox UTF-16 selection, multiline/wrap, Enter/Tab/field behavior and custom
password masking through ordinary classes and authorized atomic input. Native
metadata decoding remains inert; original resources and canonical text are
preserved. The broader [M7A.4 scope](m7a4-native-runtime.md) remains in progress.

[ADR 0050](decisions/0050-native-spinbutton-and-scrollbar.md) adds selected native
SpinButton/ScrollBar decoding, Long properties, nominal interfaces and ordinary
Change/spin/drag-only Scroll delivery. Shared arrows/track/thumb/keyboard input
uses authorized callbacks and canonical state, including reversed ranges and
pause/resume. Full [M7A.4](m7a4-native-runtime.md) remains in progress.

[ADR 0051](decisions/0051-native-unbound-listbox.md) adds selected native unbound
ListBox decoding, ten-column indexed rows, selection/default Value and ordinary
Change/Click frames. Authorized shared list input uses canonical state and
preserves focus/validation and Stop/resume. Whole-array APIs, further list/binding
semantics and ComboBox remain open with the unchanged broader M7A.4 scope.

[ADR 0052](decisions/0052-native-combobox.md) adds the selected native editable/
drop-list ComboBox packet, shared ordinary list rows, text/selection/matching and
VM-owned drop state. Authorized input preserves focus/validation and debugger
continuations; contextual F4 leaves the editor shortcut intact elsewhere. Locale
limits, further properties/bindings/resources and broader M7A.4 remain open.

[ADR 0053](decisions/0053-native-list-array-loading.md) adds shared native ListBox/
ComboBox two-dimensional List loading, transposed Column loading and indexed/
current-row Column access through ordinary class procedures. Source bounds/types
remain independent; complete validation precedes canonical row/selection changes
and ordinary callbacks. Whole-array reads, additional list/binding semantics and
the broader M7A.4 scope remain open.

[ADR 0054](decisions/0054-native-list-array-readback.md) adds detached zero-based
List/Column array snapshots to the existing unbound ten-column native packet,
preserving selection, Err and ordinary debugger lifetime. [ADR 0055](decisions/0055-native-form-metadata-and-control-tips.md)
promotes native Tag/ControlTipText/HelpContextID into ordinary instance metadata
and renders canonical control tips without executing code or changing focus.
Original resources stay independent; actual Help-file/F1 routing and the broader
M7A control/persistence/removal requirements remain open.

Native Frame containers follow [ADR 0056](decisions/0056-native-frame-containers.md):
bounded native parent storages, immediate child Controls/Parent identity, scoped
TabIndex/option groups and nested ActiveControl/focus cancellation. Canonical state
remains ordinary classes; effective ancestor flags, live relative geometry,
clipping and accessible groups use the shared renderer. Original source/resources
stay inert and unchanged. Further container/page/image/binding/platform semantics,
native edited persistence and .tvform removal remain in M7A; fresh Office traces
remain NOT_RUN under M8.5.

[ADR 0057](decisions/0057-native-tabstrip.md) adds selected native TabStrip
records and separate ordinary Tab/Tabs identity, indexed/named lookup,
enumeration/default selection and authorized widget input. Canonical state stays
in ordinary classes; inert queries/rendering do not instantiate or execute VBA.
Dynamic page/tab mutation/index/name semantics and the remaining M7A.4
ledger stay open. Native edited persistence and .tvform removal remain separate.

[ADR 0058](decisions/0058-native-multipage.md) adds selected native MultiPage
f/o/x reconciliation, distinct Page/Pages identity, immediate child collections,
selection-gated input/focus and actual page rendering. Internal tabs stay private;
canonical state remains ordinary classes and source/resources stay inert. Indexed
Click and repeated-selection behavior also correct the prior TabStrip contract.
Dynamic page/tab mutation, further persistence/property/event/lifecycle cases and
the full remaining M7A.4 ledger remain open. Native edited saving/removal stay
M7A.5/.6; fresh Office traces remain NOT_RUN under M8.5.

[ADR 0059](decisions/0059-native-image-pictures.md) adds selected native Image,
bounded inert BMP/GIF/JPEG/ICO decoding, canonical stdole picture identity and
assignment, display properties and nonfocusable Click. Immutable raster assets
and capped derivative texture caches preserve source/resources and existing VM
budgets. Further picture metadata/platform/format and event/presentation cases,
RefEdit, bindings, dynamic collections and remaining M7A.4 semantics stay open.
Native edited saving/.tvform removal remain M7A.5/.6; Office traces remain NOT_RUN
under M8.5.

[ADR 0060](decisions/0060-native-refedit.md) adds bounded inert RefEdit class-table
resolution, ordinary typed RefEdit text/drop state and selected focus/validation/
Change/DropButtonClick behavior. The shared renderer accepts host range-selection
availability and collapses/restores modal RefEdit; GridCraft owns worksheet
selection and authorization. Original source/resources stay separate. Broader
RefEdit/platform/property/event/binding semantics and the unchanged M7A.4 ledger
remain open. Native edited saving/removal stay M7A.5/.6; fresh Office traces stay
NOT_RUN under M8.5.

[ADR 0061](decisions/0061-native-tab-page-mutation.md) adds native Tabs/Pages
Add/Remove/Clear, writable Integer Index and runtime member Name. Scalar position
cells preserve member/selection identity without owning cycles; inert queries and
page input/rendering follow canonical ordering. Generic VM reference-scan metadata
keeps primitive slots out of object lifetime scans under unchanged budgets.
Dynamic Controls, bindings/multiple selection and the broader M7A.4 ledger remain
open; native edited saving/removal remain M7A.5/.6. Fresh Office traces are NOT_RUN.

[ADR 0062](decisions/0062-native-dynamic-controls.md) implements selected native
Controls.Add/Remove/Clear, runtime names, scoped collections/notifications and
WithEvents focus/validation. An inert runtime projection drives widgets and
input from ordinary canonical objects; imported source/design/resources stay
unchanged. Weak context requests avoid ownership cycles. Shared typed helpers,
indexed verifier metadata and generic dispatch optimizations preserve all existing
VM limits. Full M7A.4, native edited persistence/removal and M8.5 comparisons remain
separate requirements; CI/WASM/WASI validation of ADR 0062 passed.

[ADR 0063](decisions/0063-native-listbox-multiple-selection.md) adds native ListBox
multiple selection, independent focus/selection, row mutation and authorized
pointer/keyboard input. [ADR 0064](decisions/0064-native-runtime-type-names.md) adds
IR 12 inert runtime class names and native TypeName through canonical aliases.
State stays in ordinary classes, rendering stays inert, and budgets are unchanged.
Bindings and the remaining M7A.4 ledger remain open; M7A.5/.6 and M8.5 are separate.

[ADR 0065](decisions/0065-native-cell-and-row-bindings.md) defines native
ControlSource/RowSource through ordinary classes and the existing explicit host
range import. Import and rendering remain inert; GridCraft owns cell commands,
trust and revision-driven callbacks. The owner approved 4,096 static permission
entries on 2026-11-05; per-site authorization and executed-call/work/memory limits
remain in force. Further binding timing/types and the broader M7A.4 ledger remain open.


[ADR 0066](decisions/0066-foreign-runtime-type-names.md) extends existing read-only
host type resolution with bounded native TypeName metadata. Identity/lifetime,
no-getter/no-activation rules, lookup/allocation budgets and generic Object fallback
are retained. TypeName/VarType/IsArray named varname binding matches their documented
signatures. Shared and pinned fork validation passed; the remaining M7A.4 ledger stays open.


[ADR 0067](decisions/0067-vartype-default-value-inspection.md) adds IR 13
inspection-defaults-v13: VarType resolves one compatible public default value
through normal project frames or authorized host calls, preserving errors and
Stop/wait continuations. Public default functions and New value contexts use
ordinary binding. Inert watches, host ownership and existing budgets are retained.
Shared CI, WASM and actual WASI validation pass; the full M7A.4 ledger stays open.


[ADR 0068](decisions/0068-lazy-project-class-array-elements.md) adds IR 14 lazy project-class/native-form array elements.
Fixed/dynamic arrays, Preserve/Erase, private/static storage, canonical ByRef,
source-order capture and constructor/For Each Stop continuations reuse existing
locations, classes and budgets. Whole-array reads and safe inspection stay inert;
external activation and object-module Public arrays remain refused. Shared CI,
WASM and actual WASI pass; the broader M7A.4 ledger and M8.5 comparisons remain open.


[ADR 0069](decisions/0069-userforms-without-form-components.md) enables the
existing UserForms collection in projects without form components. A bounded
code-token hint selects only the minimal ordinary collection/enumerator support;
semantic shadowing, compiled identity, inert inspection, host ownership and
existing budgets remain authoritative. Shared/host acceptance is tracked in
progress; [the completion checklist](m7a4-completion-checklist.md) retains the
full remaining M7A.4 scope. No new IR, format or control library is introduced.

Native typed keyboard callbacks and the raw-input/default-processing barrier are
implemented under [ADR 0070](decisions/0070-native-keyboard-events.md). Keyboard
completion is still partial pending the remaining K01/K02 acceptance cases.

[ADR 0071](decisions/0071-native-keyboard-focus-and-default-order.md) adds
root-window keyboard focus, input arbitration before editor shortcuts and
canonical Tab/SetFocus/Space default ordering. Full M7A.4 remains in progress.


[ADR 0072](decisions/0072-native-form-accelerators.md) adds native persisted
accelerators and canonical Accelerator properties, selected eligible Alt actions
for Label/buttons/booleans/Tab/Page, validation/callback continuations and inert
caption underlines. Duplicate routing explicitly refuses; non-ASCII/platform and
further ordering cases remain open. Checklist K03 is PARTIAL; M7A.4 remains in
progress, edited persistence/removal remain M7A.5/.6 and Office observations remain
M8.5 NOT_RUN.


[ADR 0073](decisions/0073-native-mouse-callbacks.md) adds typed native mouse events,
indexed Tab/MultiPage arguments, mutable DblClick cancellation and canonical
completion receipts. Selected shared pointer capture/default paths preserve
Stop/Continue, Reset, identity and keyboard focus; remaining text/list/numeric/
indexed-tab/platform defaults and full event ordering keep M01 PARTIAL. M7A.4
remains incomplete and fresh Office observations remain M8.5 NOT_RUN.

ADR 0073's event-signature metadata refinement omits unused instance-field
aliases from Event declarations. Declared argument types, receivers, canonical
class fields and every executable method's field aliases remain verified under
the existing nominal metadata and runtime limits. No IR change is needed.

[ADR 0074](decisions/0074-native-text-pointer-selection.md) adds selected native
TextBox pointer caret/drag/word selection after mouse callbacks, cancellation and
focus redirects, with ordinary authorized selection commits. Canonical UTF-16,
CRLF, Stop/Reset and source/resource identity are preserved. Native AutoWordSelect
bit 27 is the inverse of the Boolean property; imported defaults and new dynamic
controls are distinguished. ComboBox/RefEdit pointer defaults, further platform/
word/selection cases and the full M7A.4 checklist remain open.

[ADR 0075](decisions/0075-native-composite-pointer-defaults.md) adds selected
ComboBox/RefEdit text and drop/picker, list/popup row, numeric step/repeat/drag and
indexed tab/page-header pointer defaults after ordinary mouse callbacks. Inert
modal-display metadata is separate from input ownership; popup input inherits the
execution barrier and tab identity is revalidated after mutation. ComboBox native
AutoWordSelect uses its inverse file bit. Canonical state and existing executable
command authorization remain authoritative. Full M01/M7A.4 acceptance stays open.

[ADR 0076](decisions/0076-native-modal-input-continuations.md) parks bounded raw
mouse input by modal caller, so child callbacks can run while their parent waits.
Physical release stays with its original capture, keyboard barriers use the
current callback owner, and Reset/unload discard stale defaults. The shared raw
queue stays at 64, modal scope depth stays bounded by eight existing waits, and
canonical state/receipts remain ordinary VBA. Selected nested lifecycle/input
cases are covered; L02 and the full M7A.4 checklist remain partial.

[ADR 0079](decisions/0079-form-input-before-document-activation.md) orders selected
native key/text/paste/IME prefixes before document deactivation. The raw hook
transfers earlier input to the existing bounded keyboard barrier and waits for
receipts, defaults and ordinary callbacks. Spanning IME composition retains only
owner/lifetime metadata; its raw continuation stays with the form through Stop.
Reset/context invalidation discards unapplied input; a modal opened by the prefix
cancels the later document gesture while preserving the parked keyboard receipt.
No VM/foreign behavior or budget changes. Full M7A.4 and platform/Office comparison
requirements remain open.

[Native colours (ADR 0080)](decisions/0080-native-form-colours.md) retain signed
OLE_COLOR metadata and ordinary BackColor/ForeColor fields, with RGB/system-theme
rendering and an injected UI palette resolver. Invalid assignments preserve prior
state; import/source/resources and inert inspection remain separate. Runtime Page
lookup searches the existing MultiPage collections. Font/border/transparency,
ambient/platform appearance and the full M7A.4 checklist remain open.


ADR 0081 adds selected native font metadata, ordinary typed Font objects, copied
container defaults and inert UI snapshots. Font sizes retain Currency precision;
local typography and optional caller-owned face selection preserve global host
styles/fonts. The static nominal metadata ceiling is 8,192, with the existing
64 MiB metadata/work bounds and VM/4,096-permission limits unchanged. The separate
ADR 0073 event regression still stays below 4,096. Replacement/ambient/platform
font behavior and the unchanged full M7A.4 checklist remain open; fresh Office
comparisons remain M8.5 NOT_RUN. See [ADR 0081](decisions/0081-native-form-fonts.md).

Native Font assignment follows [ADR 0082](decisions/0082-native-font-assignment.md):
typed ordinary setters preserve explicit reference identity, old aliases and root
lazy reload; new controls still copy independent defaults. No VM, permission or
budget change is needed. Nothing/external/other-assignment and ambient/platform
font cases and the full remaining M7A.4 checklist stay open.

[ADR 0083](decisions/0083-page-appearance-ownership.md) corrects Page appearance
ownership: no Page Font/BackColor/ForeColor APIs; MultiPage owns defaults/painting.
[ADR 0084](decisions/0084-native-control-appearance.md) retains native border/effect/
background data and adds applicable typed properties, local effects/transparency
and fixed-bound clipping. [ADR 0085](decisions/0085-native-font-attribute-aliases.md)
adds published Font aliases forwarding to canonical objects. Full member, platform
and M7A.4 acceptance remains open; no VM/permission/budget increase is made.

[ADR 0086](decisions/0086-native-text-alignment.md) adds selected native TextAlign
on Label/TextBox/ListBox/ComboBox. Native ParagraphAlign swaps centre/right values
relative to the public enum; ordinary Long state is independent of Font/text/
selection. Inert inspection preserves authored form fields, local layout matches
pointer/caret geometry, and ComboBox popup alignment stays independent. Source/
resources, global styles/fonts and all budgets are unchanged. Broader member,
wrap/AutoSize, Boolean/RefEdit and platform cases and full M7A.4 remain open.

[ADR 0087](decisions/0087-native-caption-word-wrap.md) adds selected native
Label/CommandButton WordWrap flags, typed Boolean state, distinct file/runtime
defaults and fixed-bound captions. Dynamic TextBox defaults to True; multiline
unwrapped text retains fixed horizontal interaction bounds with complete UTF-16
cursor rows. Canonical state, original source/resources, global font/style policy
and all VM/permission budgets remain unchanged. Broader Boolean-control wrap,
AutoSize/scrolling and full M7A.4 acceptance remain open.

[ADR 0088](decisions/0088-native-boolean-caption-wrapping.md) extends selected
native WordWrap flags/defaults and ordinary Boolean state to CheckBox, OptionButton
and ToggleButton. Rendering keeps widget input/accessibility, local bounded
captions and physical ToggleButton centring. Value/Null/groups/events, original
source/resources, global font/style policy and all VM/host budgets are unchanged.
Other properties, Alignment/TextAlign, AutoSize/scrolling and full M7A.4 stay open.

[ADR 0089](decisions/0089-restricted-wasi-callback-forwarding.md) keeps a plain JS
forwarding boundary around native WASI callbacks to avoid a reproduced Node 22
runner crash. Guest bytes, stack, budgets, args/env/preopens and error/exit behavior
remain unchanged; five independent runner guests accompany full native/WASI
acceptance. This does not extend Office APIs or complete M7A.4.
