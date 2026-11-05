---
name: tiny-vba-host-integration
description: Design or review tiny-vba generic foreign calls, host references, ByRef/wait ownership, source import and application integration contracts. Use when bridging the engine or IDE to external document applications.
---

# Host and import boundaries

New M1.1 reference binding uses the immutable, validated `ReferenceSet` contract in
[ADR 0020](../../../docs/decisions/0020-explicit-host-reference-snapshots.md).
Pass explicit snapshots through compiler/IDE configuration; never load a library
or activate COM implicitly. Enum types bind to Long and constants to typed IR.
Nominal descriptors use IR 5 and an explicit read-only host type resolver under
[ADR 0021](../../../docs/decisions/0021-nominal-host-object-types.md). Query the host's
existing object store; do not introduce a parallel registry or Office names here.
Retain nominal array/ByRef/temporary/return identity, precommit writeback checks,
query budgets and lifetime validation. Metadata grants no default-member support.
See the staged M1 log and retain existing host identity/cancel checks.

Read [architecture ownership](../../../docs/architecture.md),
[scope ADR](../../../docs/decisions/0001-repository-scope.md) and
[UI ownership amendment](../../../docs/decisions/0002-editor-debugger-ui.md).
Inspect real host APIs before proposing an adapter; do not implement imaginary Gridcraft methods.

M0's current owned snapshot/writeback and explicit cancel contract is in
[ADR 0003](../../../docs/decisions/0003-m0-source-scenarios-foreign.md). Inspect vba-foreign and
vba-testkit tests before extending it; final alias/type/error writeback behavior is not frozen.
Session callers must cancel on close/disconnect; Drop cannot access a host. Host/location owner
identities must be unique during their session.

The [IR 1 specification](../../../docs/bytecode-vm-specification.md) requires an exactly-once VM-owned
host continuation and an M3 typed-writeback adapter. The restricted snapshot feature rejects unsupported
foreign aliases/error writebacks; it does not define full VBA ByRef. Close/stop/disconnect must cancel
pending host tokens, and a vanished host must not be reported as successfully cleaned up.

## Keep the boundary generic

vba-foreign carries invocation kind, reference/target, named/positional/omitted arguments, ByRef
obligations and explicit result/error/wait state. Preserve Missing versus Empty, Nothing versus Null,
and property Let versus Set. Prove ownership/error/suspension with a deterministic generic-host double.

Applications own Office names, member IDs/defaults/catalogues, document/object identity, Office-value
conversions, undo/recalc, OOXML packaging and trust/lifecycle decisions. Office-shaped contract tests
belong there. tiny-vba owns VBA values/classes and source decoding, plus the reusable editor/debugger;
Gridcraft embeds those panels instead of maintaining a second editor.

Ordinary host calls use native in-process Rust dispatch. JSON/DAP must not become the hot-path member
ABI. No ambient filesystem/network/shell/COM or native dialogs in the VM. Supply time, random, locale,
prompts and allowed capabilities explicitly. Capability checks cover late binding, callbacks, watches
and user functions, not just the initial run request.

Before pause/yield/callback, release mutable document borrows. Define stale-reference, reentrancy,
close/cancel/disconnect and cleanup outcomes. Preserve host identity; do not add a parallel registry
or document model. Do not impose whole-macro rollback to simplify failure handling; the host's chosen
compatibility policy determines what happens after a successful write followed by a runtime error.

## Decode without executing

vba-project reads bytes/source into module metadata. Preserve source identities, code names, encoding
and attributes. Bound decompression/allocation/nesting; reject unsupported/protected/corrupt inputs
explicitly. Import never executes macros. Project attachment, consent and Office package preservation
are host tasks. P-code, protection bypass and VBA binary project writeback remain outside the initial scope.

Verify generic invocation and pause/resume locally, including omitted/named/ByRef arguments, errors,
suspension and stale handles. Real workbook mutation, recalc/undo and package round trips require
fork integration evidence; do not claim those from a generic double or source decoder alone.

## Implemented M3 contract

Read [runtime](../../../docs/runtime.md) and [ADR 0007](../../../docs/decisions/0007-m3-execution-and-debugger.md).
The m3-core-v1 runtime is intentionally narrower than structural IR. Preserve explicit frame/alias
storage, one-visit checkpoint permits, saved fault PCs, generation checks and nonreplayed continuations.
Use vba-runtime for operations/edits. VM host dispatch must use checked precommit return/writeback
validation and explicit per-site permissions; foreign aliases remain unsupported. Run cargo xtask ci,
wasm and wasm-run (native plus actual WASI execution). Runtime example results do not mark unrun M0
or external scenario assertions PASS. Graphical UI, DAP and VBA error-handler semantics remain later work.

## Implemented M5 contract

Read [ADR 0013](../../../docs/decisions/0013-m5-host-contract.md). The current consumer profile is
`m5-host-v1` over verified IR 1/2/3/4. `vba-foreign` owns protocol version 1 (`API_VERSION`,
`ProtocolVersion`, `check_protocol`); hosts report their version through `Host::protocol` and the
foreign session and VM refuse incompatible majors/minors before any dispatch. Embeddings supply
generic source imports (`vba_foreign::Import`, Call/Get only, validated by `validate_imports`) through
`CompileOptions::imports`; each import lowers to a hidden IR 4 `HostRoot` slot plus a fixed member
call/get and can never be written or captured. `HostBinding` carries case-insensitive named roots
(host identity must match, cap 64); missing or mismatched roots refuse launch, and
`HostBinding::for_program` derives the full site allowlist from a verified program. Reuse
`vba_testkit::run_to_boundary` and `vba_testkit::statement` for bounded bridge drivers;
vba-vm/tests/embedding.rs is the break/edit/resume proof with exactly one dispatch. Office-shaped
names, catalogues, workbook FillCells evidence, undo/recalc and permission gates remain fork-owned
(Gridcraft); do not add them here.

## Implemented M7 contract

Root-object imports (`member: None`) preserve actual property/method dispatch under
[ADR 0017](../../../docs/decisions/0017-imported-root-members.md). Only fixed-member
aliases consume the whole member application. Preserve the exact dispatch regression
in vba-vm/tests/embedding.rs; do not silently discard an imported root's member call.

GridCraft baseline integration is tracked in
[the staged work log](../../../docs/gridcraft-integration.md); fork-owned tests supply
workbook/undo/recalc evidence. Neither fork becomes a tiny-vba dependency.

Read [ADR 0015](../../../docs/decisions/0015-m7-callbacks-and-lifecycle.md). The VM stays
non-reentrant: schedule VBA callbacks with `Session::schedule_callback` (FIFO, `Limits.callbacks`
budget, name/arity validated) and deliver each with `Session::run_next_callback` once the current
program is terminal; a delivery runs to a boundary, shares static state and bindings, applies
breakpoints, and waits through `resume_host` exactly-once. `reset_state` (terminal-only) and
`rebind_roots` cover static reset and the host's own object-lifetime model; a stale root at
delivery surfaces the host's reference error 91. Cancel/disconnect/host loss drain queued
callbacks without executing them and permanently close the lifecycle even after completion.
Reset requires a terminal/open/current session, clears output/static/Err state and prepares the
original entry with original bound arguments; queued callbacks, instance/debugger identities and
lifetime budgets survive. Invalid delivery quotas/host/source/lifecycle preserve the queue and
outcome; argument types are checked before enqueue and queued metadata/payload share the cumulative
session budget. Top-level ByRef arguments are owned locations. Run the callback suite natively and
through cargo xtask wasm-run. Event ordering, `EnableEvents`, document events and UDF policy
remain fork-owned over this queue.

M6 is implemented under [ADR 0014](../../../docs/decisions/0014-m6-project-decoding.md).
Read [the import contract](../../../docs/project-import.md) before changing vba-project.
Keep extraction read-only/inert, project-local ModuleId separate from caller-owned SourceId,
UTF-8 attribute spans and code names distinct from stream/display names. Preserve explicit
corrupt/unsupported/protected/limit refusals and container/decompression work caps. cfb/encoding_rs
are reviewed pinned external dependencies; package preservation and trust UX remain fork-owned.
The 14 controlled importer tests run natively and under WASI through cargo xtask wasm-run.

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

ADR 0025 distinguishes fixed getter aliases from method aliases. Getter aliases
resolve before chained member dispatch; captured results retain identity. Keep
method shortcuts and root-object imports unchanged. Dynamic Office roots remain
fork-owned; do not substitute mutable proxy handles for captured objects.

M6 host-object WithEvents delivery uses [ADR 0029](../../../docs/decisions/0029-host-event-subscriptions.md)
and the explicit `host-events-v1` capability. Discover only live matching publisher
connections, validate every handler before any effect, and deliver snapshot members
sequentially through ordinary callbacks. Host-supplied signatures must include
scalar kind, ByRef mode and nominal type. Carry completed ByRef outputs to the next
sink; preserve pending delivery through pause/wait and discard it on Reset/close/trust
loss. Office names, event signatures and save/close ordering stay in forks.

## Waiting project calls

Follow [ADR 0036](../../../docs/decisions/0036-suspended-host-project-calls.md).
Use `Ide::start_suspended_call` with the current wait token and normal debugger ticks;
read `take_host_call_result`, then answer the original host wait. Callback ByRef outputs
are separate from original foreign-call writes. Preserve shared state/budgets, host/source
identity, catchable runtime failures and fail-closed resource/protocol cancellation. Do not
start a substitute interpreter or commit a document while callbacks are paused. Idle events
can use `prepare_host_events` without executing code. Office signatures/policy remain fork-owned.
Use exact-token `abort_host_wait` when an embedding needs a fatal nested error policy;
normal typed result/error transport remains available. It cancels modal waits through
the form gate as well as foreign host waits. Do not implement Office policy in the VM.

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

M7A.4 now also includes the [native runtime increment](../../../docs/m7a4-native-runtime.md)
under ADR 0043: six flat control families, selected property/input semantics,
Controls/UserForms lookup and bounded transient project inspection. Canonical
VM state remains authoritative; queries never load forms or invoke getters.
Keep intrinsic providers/wait routing intact and do not extend .tvform. The
milestone remains in progress with its remaining requirements recorded there.

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

Native Frame containers follow [ADR 0056](../../../docs/decisions/0056-native-frame-containers.md):
bounded native parent storages, immediate child Controls/Parent identity, scoped
TabIndex/option groups and nested ActiveControl/focus cancellation. Canonical state
remains ordinary classes; effective ancestor flags, live relative geometry,
clipping and accessible groups use the shared renderer. Original source/resources
stay inert and unchanged. Further container/page/image/binding/platform semantics,
native edited persistence and .tvform removal remain in M7A; fresh Office traces
remain NOT_RUN under M8.5.
