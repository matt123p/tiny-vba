# Architecture

Status: M0–M4 complete for the documented [M4 language profile](m4-language.md), with
verified IR 1/2/3/4/5 and explicit-frame execution; the M5 native host contract is implemented
([ADR 0013](decisions/0013-m5-host-contract.md)). M6 read-only project decoding is implemented
([ADR 0014](decisions/0014-m6-project-decoding.md)); M8's reusable graphical IDE and native/web shells are implemented under
[ADR 0016](decisions/0016-m8-embeddable-editor.md). Fork-owned workbook integration remains a separate gate.

The engine and toolkit-independent IDE model compile and test without Office, UI toolkits,
application forks or corpus downloads. This repository also owns an optional editor/debugger UI
and standalone IDE; those upper layers may depend on UI toolkits.
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
| vba-runtime | Shared coercion/operators/builtins/error rules | diagnostics, values |
| vba-foreign | Generic host call/result/wait protocol; protocol version 1 and validated source imports | diagnostics, values |
| vba-vm | Frames, instruction pointer, budgets, inspection hooks; host-root binding and protocol checks | diagnostics, ir, values, runtime, foreign |
| vba-debug | Breakpoint/step/inspection controller | diagnostics, vm |
| vba-dap | Optional bounded M10 Debug Adapter Protocol frontend | ide, serde_json (=1.0.154) |
| vba-project | Bounded read-only MS-OVBA source/project decoder | diagnostics, cfb (=0.14.0), encoding_rs (=0.8.42) |
| vba-testkit | Deterministic generic foreign host, trace support and bounded embedding driver | debug, diagnostics, foreign, ir, values, vm |
| vba-conformance | Scenario validation, typed expectations and M4 outcome judgment; comparison/reports later | diagnostics, testkit, values, vm |
| vba-cli | Source parsing, compilation/disassembly/execution, fixture validation and optional DAP stdio | conformance, diagnostics, syntax, compiler, ir, values, vm; dap optional |
| vba-ide | Toolkit-independent source buffers, commands, compiler/debug-session orchestration and safe watches | compiler, debug, diagnostics, foreign, ir, runtime, syntax, values, vm |
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
| vba-ide | Toolkit-independent editor/project/session state and actions | compiler, debug, diagnostics, foreign, IR, runtime, syntax, values, VM; no toolkit/IO |
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

## M2 standard forms (ADR 0024)

`vba-ide` owns shared form design/source, the bounded `.tvform` codec, generated
project-class support and form event orchestration. `vba-ui-egui` renders the
designer and runtime; the shell owns explicit native/browser IO. The VM supplies
generic parked host continuations and explicitly opted-in suspended callbacks,
with no UI/control dependency. Existing terminal-only callbacks are unchanged.
The reviewed 19-crate dependency graph is unchanged. See
[contracts and acceptance](m2-acceptance.md) and
[ADR 0024](decisions/0024-standard-form-execution-and-exchange.md).
