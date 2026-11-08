# M3/M4 runtime and debugger

M7A.6 native-only update (2026-11-08): the custom form codec/APIs, source-file
routes, file filters and `.tvform` example are removed under
[ADR 0099](decisions/0099-native-only-form-workflows.md). Insert UserForm now
starts from an inert native template, and all form Save/Export/remove-export
requests use native `.frm`/`.frx` snapshots. The CustomerForm demo and retained
exchange/host tests use native files. No legacy importer or converter remains.
M7A.6 is COMPLETE; shared and pinned-host gates pass. See the
[acceptance record](m7a6-acceptance.md). M7A.7 combined acceptance remains next.
Older incremental format/support notes below are historical and superseded.

M7A.5 persistence update (2026-11-08): native `.frm`/`.frx` saving/export and
explicit edited XLSM/DOCM copy export are implemented under
[ADR 0098](decisions/0098-native-form-encoding-and-source-only-writeback.md).
The [workflow and acceptance record](m7a5-acceptance.md) distinguishes native Save,
Export, browser downloads and ordinary document Save, which preserves originals.
M7A.5 is COMPLETE for these explicit common-profile workflows. Historical
read-only descriptions below refer to original decoder/ordinary-save paths, not
the explicit native encoders.

Current scope (2026-11-08, [ADR 0097](decisions/0097-common-forms-scope-and-p9.md)):
**M7A.4 is COMPLETE for [native-forms-common-v1](m7a4-common-profile.md).**
The outstanding uncommon/exhaustive cases in earlier incremental notes now belong
to [P9](p9-forms-compatibility.md). Those notes retain their original support and
evidence limits; this sign-off does not implement deferred behavior. M7A.5 native
persistence is complete for the explicit workflows above. M7A.6
format removal is complete; M7A.7 integration and M8.5 fresh comparisons remain
unfinished. Next work is M7A.7.

Implemented 2026-10-26, extended by new M1.1. The current consumer is `m5-host-v1`, using verified IR 1/2/3/4/5/6/7/8/9/10/11/12/13/14.
The [M4 language contract](m4-language.md) and [ADR 0011](decisions/0011-m4-language-extension.md)
define scalar coercions, arrays, arguments, persistent state, builtins and VBA handlers.
[ADR 0013](decisions/0013-m5-host-contract.md) adds the versioned native host protocol and hidden
host-root imports. The headless debugger and generic host protocol retain their M3 lifecycle
contracts. The visual IDE and optional DAP are implemented; Office objects remain fork-owned.

```sh
cargo run -p vba-cli -- run examples/pure-vba/Arithmetic.bas
# COMPLETED Function(Long(42))
cargo run -p vba-cli -- run examples/pure-vba/M2Control.bas
# COMPLETED Function(Long(18))
cargo run -p vba-cli --example m3_smoke
rustup target add wasm32-unknown-unknown wasm32-wasip1
cargo xtask ci
cargo xtask wasm
cargo xtask wasm-run
cargo test --workspace --release --locked
```

`wasm-run` runs the same Rust acceptance example natively and under Node WASI, asserting Long(42),
If/For/direct calls/ByRef Long(18), a host wait/pause/edit/resume result of Long(41), exactly one
invocation and two polls, stale handles, pending-token cancellation and infinite-loop cancellation.
The Node runner has no preopened directories or inherited environment; the interpreter/host double
are Rust. WASI is the smoke executable transport, not a core VM filesystem API. Core libraries
still compile for wasm32-unknown-unknown. Node 18.19.1 and Rust 1.99.1 were used locally; CI uses
Node 22 for this gate. Node may print its WASI experimental-feature notice.

## Runtime capability matrix

The compiler defaults to `ir3-structural` for check/compile/disasm. To execute, compile with
`CompileOptions { capabilities: vba_vm::capabilities(), ..Default::default() }` and pass the resulting
VerifiedProgram to Session. Session rechecks immutable IR against its actual capabilities without
cloning the source payload. A caller's capability set cannot enable an unimplemented operation.

| Operation | Current support |
|---|---|
| Scalar declarations/conversion | All twelve ScalarKinds; centralized checked coercions and exact tags |
| Arithmetic | Integer/Long/LongLong, Single/Double, scaled Currency and Date serial rules; overflow 6 and zero divisor 11 |
| States/strings | Empty/Null rules, distinct Missing/Nothing/CVErr, UTF-16 concatenation/comparison/slicing |
| Calls | Positional/named/optional/omitted source arguments, exact canonical ByRef and typed temporaries |
| Arrays/storage | Fixed/dynamic bounded arrays, lower bounds, Preserve/Erase, ByRef cells; module/Static locals and explicit project state |
| Errors/builtins | On Error, Resume, Err/Erl and deterministic intrinsic registry; see M4 contract for exact boundaries |
| Host roots/protocol | IR 4 hidden `HostRoot` slots bound by `HostBinding.roots`; `vba_foreign::check_protocol` refuses other protocol majors/minors before any dispatch |
| Generic host/debugger | Existing permitted calls, waits, typed patches, checkpoint stops, stepping, inspection and edits |
| IR 8 extended intrinsics | Descriptor-verified stack mapping; named/omitted calls, StrComp, InStr, InStrRev, Replace, Split and Filter use IR 8; descriptor defaults resolve supported optional gaps. `IntrinsicContext` provides immutable mode/locale/calendar/code-page/target-width metadata and an identified comparison-service seam. Text-mode StrComp/InStr/InStrRev/Replace/Split/Filter and Option Compare Text relational operations require an injected provider; the no-service path returns Unsupported. InStr, Replace, Split and Filter use the caller's Option Compare by default; InStrRev defaults to Binary. Filter returns a zero-based String array. |

The M4 contract lists explicit unsupported forms and profile limits. Source literals retain the
reviewed decimal forms; additional suffix/radix, locale and class/default-member behavior is refused.

The shared vba-runtime owns these rules; VM, debugger edits and typed parameter binding do not
implement independent conversion paths. Tests are independently authored; no Microsoft oracle or
broad Office compatibility observation is claimed.

## Session and source control

`Session::new` takes VerifiedProgram, a caller-assigned routing label, ordered
launch Values, optional HostBinding and Limits. Each execution also receives a non-reused opaque
SessionInstance from the loaded engine, independent of that label. The label remains available via
identity(); instance() identifies the execution. Instance exhaustion returns a resource error; it
never wraps or reuses a cancelled/dropped execution's token. IR 1 launch parameters must be ByVal. Locals and
result storage are frame-owned locations; direct ByRef slots are aliases of the caller's canonical
location. Returning invalidates owned storage while retaining caller aliases. No Rust recursion
implements VBA calls. Caller operand prefixes survive nested calls and pauses without evaluation replay.
Embeddings using `vba-ide::Ide` may set bounded `Limits` with
`configure_execution_limits` before launching. The validated snapshot is used by
normal runs and idle callback sessions; an existing session keeps its captured
limits. Work counters remain VM/session-owned across slices, debugger pauses and
resume, so yielding does not replenish a host-call budget. The standalone IDE
keeps `Limits::default()` unless its embedding chooses another profile.

`run_slice(host, quota, predicate)` reports Yielded, Paused, Waiting, Completed, Failed or Cancelled.
The initial Yielded state is ready/runnable. Invalid/zero/oversized quotas are command errors.
Hard instruction limits precede yields and breakpoint offers. Terminal queries never rerun code.
The synchronous API is cooperatively controlled between bounded slices; callbacks/reentrant execution
and forcible interruption of a blocked synchronous host are not provided.

At each Checkpoint the VM records a fresh dynamic visit with instance/session/frame/site identity. A breakpoint
pause leaves that visit unpassed. Continue permits exactly that visit once; the same loop checkpoint
can break on a later iteration. Predicate errors retain the unpassed visit and PC. Hidden checkpoints
still consume budget. Stop advances past itself and pauses before its mandatory Jump. A user pause
request is served at the next visible checkpoint. `mark_source_changed` blocks edits/continuation until
cancel/recompile/restart; it never replaces the immutable source underneath paused execution.

M7 callbacks (ADR 0015): hosts schedule by name with `Session::schedule_callback` (FIFO, bounded
by `Limits.callbacks`, name/arity/payload validated, refused on a closed lifecycle) and deliver
each with `Session::run_next_callback` after the current program is terminal. Delivery runs as a
fresh top-level invocation on the same instance — sharing static state, bindings and output —
drives to a boundary in one call, applies breakpoints and the predicate, counts
`counters().callbacks_run`, and waits through the existing exactly-once `resume_host` path. A
stale host root at delivery surfaces the host's reference error 91 without a repeated dispatch.
`cancel`/`disconnect`/`host_lost` drain the queue and permanently close the lifecycle, including
when already completed/failed. Invalid quota, host, lifecycle or stale-source delivery requests
preserve queue and outcome. Callback argument types are checked before enqueue; queue metadata
and payload share the cumulative session budget. Top-level ByRef arguments use owned locations.
`Session::reset_state()` requires a terminal, open, current session, clears static/output/Err state,
and prepares the original entry with its original bound arguments for `run_slice`. Queued callbacks,
instance identity, debugger breakpoints and lifetime work budgets survive reset; an exhausted
budget produces a clean terminal failure. `Session::rebind_roots()` validates host identity and
required roots and refuses nonempty roots without a host binding. Nested/reentrant dispatch and mid-run injection stay
rejected; event ordering and Office-shaped event policy remain fork-owned.

M5 document callbacks may be queued with `Session::schedule_document_callback(code_name, member, arguments)`;
an embedding with live document objects may use `schedule_document_callback_with_owner` to attach a
host-owned object to the predeclared class instance. The VM resolves only verified document classes
by code name. Declared fields/procedures remain VM-owned; unresolved members on a document receiver
forward through the existing host permission/identity path to the attached owner. Owner references
are identity-checked and included in project-state snapshots. The host decides event authorization,
captures arguments and schedules delivery after the active invocation is terminal. The IDE can also
create a verified idle session for a host-authorized callback in Design mode: it compiles with the
temporary standard-module entry stub, then queues the handler through the same session/debugger
path without making a private document procedure a compiler entry or launching the project's normal
entry procedure. This does not permit automatic
`Workbook_Open` execution or provide event ordering by itself.

`vba-debug::Controller` binds breakpoints to the VM instance, compiled revision and StatementId. Multiple
statements on a line are separate stops. Step Into targets the next visible visit; Over stays in the
starting frame or surviving ancestors; Out targets surviving ancestors. Descendant calls finish their
remaining expression work before an editable caller stop. Explicit breakpoints remain active during steps.

Paused snapshots expose ordered frames, exact PCs/source spans and visible parameters/user locals.
Handles carry the caller label, opaque VM instance, frame and pause generation. A reused caller
label cannot revive historical handles or controller targets; location ownership also uses the
non-reused instance. See [ADR 0008](decisions/0008-vm-instance-isolation.md). Continue, host resume, terminal cleanup and new pauses
invalidate old handles. Read-only inspection can read stored locals in paused frames. Editing is restricted
to visible active-frame locals at ordinary checkpoint/Stop pauses; assignment validates before the write.
Scratch/result slots, other-frame edits, fault-pause edits and execution of watch expressions are rejected.
Editing a ByRef local writes its canonical caller location. Snapshot local payload is capped at 1 MiB
(or a smaller session payload cap); frames explicitly set `locals_truncated` when values are omitted.
This prevents repeated aliases of a large string from multiplying snapshot memory without bound.

Runtime failures retain bounded source/frame snapshots and error numbers. Infrastructure failures are
separate Unsupported/ResourceLimit/HostProtocol/HostDisconnected/InternalInvariant kinds. Optional
break-on-error pauses a saved runtime failure read-only; continue delivers it once and never retries.
Saved fault PCs identify Call/Invoke even after the caller PC advances to its continuation. Successful
host effects and direct ByRef writes remain observable after failure. VBA handlers follow the M4 contract; only numbered errors are catchable.

## Generic host lifecycle

HostBinding records the expected host identity, an allowlist of `(ProcedureId, ForeignSiteId)`
(cap 4,096 entries) and named host roots (cap 64). Roots are `ObjectRef` values whose `host` must
equal the binding identity; names are case-insensitive and unique. Compiled source cannot grant
permissions or roots. Every procedure local with an IR 4 `HostRoot` role must be bound at launch or
`Session::new` refuses; each root local initializes to its bound reference and stays hidden from
snapshots, edits and reference capture. `HostBinding::for_program` derives the full site allowlist
from a verified program. The compiler binds import names through `CompileOptions::imports`
(vba-foreign `Import`, ≤64, validated shape/kind); an import resolves to a hidden root local plus a
fixed member call/get, and cannot be written. At every run/resume/cancel/disconnect the VM checks
`vba_foreign::check_protocol(host.protocol())` before dispatch, so a future protocol major/minor
fails with `VBA0006` without touching host state. The VM validates each root's identity at launch;
object lifetime/generation remains the host's own validation, and a stale root surfaces the host's
reference error (91) instead of a successful write.

Validate member binding before dispatch; compiled source cannot grant permissions. Compiler
member binding is restricted to an explicitly declared Object local/parameter. Call statements and
member applications use Call, plain member reads use Get, assignments use Let/Set. Indexed/default
property ambiguity and implicit Office globals are rejected. Source foreign arguments are owned ByVal
snapshots or omitted; foreign Reference descriptors are exercised separately through verified IR.

A host Invoke is dispatched once. The VM removes its argument suffix, retains the caller prefix,
saves its fault PC, and advances to the continuation. Waiting run_slice calls never poll. Only an
explicit resume_host on the correct host polls; repeated Wait replies replace the pending token.
Success consumes the continuation and pushes one result without repeating Invoke. Wrong-host and
duplicate resumes are command errors. Invocations, polls, waits and typed location lifetimes are bounded.

vba-foreign remains the sole writeback committer. Its new checked APIs validate the return Value and
**all** patches before the existing commit loop. The VM validates declared kinds and payload budget
in that precommit hook. Typed locations require exact tags; Variant accepts validated scalar values.
Aliased foreign ByRef locations are refused before host dispatch. This is the restricted snapshot ABI,
not direct VBA's live alias model. No writebacks accompany a host error; live/partial-error foreign
alias semantics remain M4. Existing M0 unchecked-hook entry points remain available to their original
untyped prototype callers; the VM always uses the checked entry points.

Cancel/disconnect cancels the current pending token before releasing VM frames/owned locations.
Wrong-host cancellation leaves state intact. If the host is already gone, host_lost reports that external
token cleanup was unavailable; dropping VM memory does not claim host resource release. Hosts own
object validity/generations and lifecycle; ObjectRef remains a non-owning opaque token. No document
borrow, Office registry, filesystem, network or COM object is retained by the VM.

## Resource accounting and evidence

Defaults: 1,000,000 instructions, slices up to 4,096, 128 explicit frames, 4,096 cumulative owned
locations, 65,536 live reserved stack entries, 1,024 host invocations and 4,096 host polls. The foreign
controller separately caps 1,024 Wait replies. Hard counters survive soft yields. String lengths retain
the 32,768 UTF-16-unit ceiling. Locations are never reused within the session. Frames reserve their
verified operand-stack maxima before executing calls; function-result copy space is charged before
callee effects. Generic results/patches validate payload before writeback or result commitment.

The 64 MiB session payload budget uses conservative cumulative copy charges (64 bytes per scalar
copy plus UTF-16 buffers, 256 per local descriptor/binding and 64 per reserved stack entry). It also
reserves 64 KiB per host dispatch/poll and maximum declared result-copy space before direct calls.
Transient copying can exhaust this conservative budget before the instruction ceiling. Snapshots have
an independent bounded local-copy budget; returned immutable snapshots are caller-owned. Rust tree/
source/value clones may use the standard allocator; fallible reservations guard operand/frame vectors.
This is logical accounting, not an RSS or process-OOM guarantee. Cancellation is available between
synchronous slices/host calls; M3 makes no background-thread or Send/Sync promise.

Tests: `vba-runtime/tests/scalars.rs`, `vba-vm/tests/execution.rs`, `vba-vm/tests/host.rs`, CLI tests
and the native/WASM m3_smoke example. They cover the advertised operator matrix, exact types/errors,
If/For/calls, stepping, alias writes/edits, source/handle staleness, quotas/recursion/storage/payload/
stack limits, infinite-loop cancellation, bounded alias snapshots, all host kinds, named/omitted args,
wrong host, repeated waits, duplicate resume, atomic typed patches, permission/alias refusal and
cancel/disconnect/lost-host cleanup. Existing M0 fixture manifests and external corpus execution
statuses remain separate from these smoke results. The M4 assertion runner now judges the self-authored
M0 set as 17 PASS / 3 UNSUPPORTED; external corpus execution remains NOT_RUN.

## M4 execution and assertion hardening

The native/WASI smoke now also checks Double division (3.5), negative/mixed numeric comparisons
(Boolean true) and out-of-range Double-to-Long failure (6) before destination mutation. Narrowing
rounds ties to even and then checks Integer/Long bounds before casting; no i64-to-i32 wrapping is
permitted. Floating arithmetic preserves signed-zero bits. Conformance judgment compares exact
float/Date bits, while language numeric equality retains its own runtime comparison semantics.

`conformance run` uses explicit diagnostic categories and fails CI for MISMATCH/CRASH/TIMEOUT
(exit 1) or INVALID_FIXTURE (exit 2); explicit unsupported capabilities remain exit 0. UTF-16 value
reports are escaped per unit. `conformance validate` executes nothing and keeps NOT_RUN.
`cargo xtask` prints the runtime-selected checkout root; `wasm-run` honors CARGO_TARGET_DIR so shared
build artifacts cannot silently redirect tests or select an older WASM executable.

Current M4 execution evidence and boundaries supersede the historical first-slice notes above.
All 44 authored and 153 inherited oracle scenario assertions pass. See [the M4 acceptance report](m4-oracle-report.md). `Session::output`, `project_state` and `restore_project`
are described in the M4 contract. Fault snapshots include unavailable_locals for expired array cells.

The current profile includes required typed whole-array ByRef parameters, positional ParamArray,
Option Base and qualified VBA.Array, and source-order named array-reference capture under IR 3.
Host writeback checks all scalar representation tags and declared array element kinds before commit.
Historical matrices below/above do not override [the current contract](m4-language.md).
# Nominal host types (new M1.1)

[ADR 0021](decisions/0021-nominal-host-object-types.md) adds explicitly gated IR 5
nominal metadata to the existing runtime profile. Embeddings supply a read-only
`ObjectTypeResolver` over their existing object model via
`Session::new_with_type_resolver`. Plain `Session::new` retains empty-resolver
behavior; typed Nothing values work, non-Nothing nominal values require a provider.
Typed scalar/array storage, aliases, temporaries, results, host writebacks and
debugger edits share checks. Handles include host/object/generation; wrong types
fail with 13 and stale handles with 91. Query work/payload is budgeted; no Office
object definitions, COM loader or duplicate registry are introduced. See
[M1 acceptance](m1-host-references.md) for exact tests and remaining stages.

## New M1.2/M1.3 execution contract (ADR 0022)

Explicit unnamed Get/Let/Set and Enumerate calls use an empty member string.
Named calls still require a bounded nonempty name. Primitive value resolution,
dynamic native array access and primitive Variant assignment bypass the host.
Effectful defaults require an explicit permitted site and consume ordinary host,
instruction, polling and payload budgets. Recursive object defaults revisit only
the resolution instruction, never the completed target/index expressions.
Wait/resume/cancel uses the existing foreign continuation. Safe Watch evaluation
does not invoke a default property or enumerate a host object.

IR 6 permits identical aliased foreign snapshots with last-write order after
precommit validation; earlier IR preserves its refusal. This snapshot ABI does
not pretend to provide a live shared document borrow. Assignment intent and
Missing/named values are preserved; `$value` identifies a property assignment
operand independently of positional/named index operands.

Enumerate returns an owned VbaArray (at most 65,536 elements and its existing
payload bound). Native arrays use their original canonical location for live
reads, including writes made through aliases and paused element edits. Whole
array replacement, ReDim and Erase while iterating fail with error 10; loop
bookkeeping work shares a bounded enumeration-work counter. Each frame has at
most 64 cursors. Next without initialized iteration fails with error 92.
Source-region bounds release cursors when jumping outside the loop; active
error handlers retain them until Resume. Exit/return/unwind/reset/cancel drops
owned snapshots; no additional external iterator tokens or document locks exist.
Paused element edits use canonical element locations and retain nominal checks.

## M2 native class runtime foundation

Class-enabled IR 7 programs own a bounded instance heap (up to 1,024 instances,
sharing canonical location/payload/instruction limits) and reserve host identity
zero for native instances. Non-class programs preserve legacy host-zero behavior.
Native nominal checks query this heap directly; external resolvers cannot supply
project instances. Fields alias ordinary locations, and methods/accessors use
ordinary frames, checkpoints, handlers and source inspection. No Office store
or additional interpreter is introduced.

Initialize and normal last-reference Terminate execute VBA. References in live
frames, native fields, persistent slots, queued callbacks and pending results
retain instances; cycles remain retained. Work is bounded by instance_work.
Abnormal cancel/disconnect/reset releases data without invoking user finalizers.
Project snapshots validate and stage state, then rebind handles to the receiving
session generation. Predeclared/document instances retain lazy project state;
external Office base objects/events remain the embedding's responsibility.

Explicit instance callbacks reuse the existing queue. start_next_callback leaves
execution to normal bounded inspected slices; run_next_callback remains a headless
convenience. The IDE event route retains watches, stepping and edit/resume.
set_break_in_class_modules pauses before an unhandled fault leaves its class.
Safe evaluation cannot construct objects or invoke properties. Full M2 remains
in progress; the [implementation record](m2-classes-and-forms.md) lists gaps.

M2's IR 7 source-event increment supports public/default Event declarations and
scalar project-typed WithEvents fields in class/document modules. RaiseEvent uses
verified required signatures and synchronous ordinary frames, preserves connection
order and canonical ByRef across sinks, and retains queued receivers through
pause/resume. State transfer remaps objects and preserves subscription ordering.
Unknown events, mismatched handler signatures, Optional/ParamArray/named/omitted
event arguments and foreign WithEvents mutation are refused. Editor Options
exposes all-error, class-module and unhandled-error trapping. Forms remain pending;
see docs/m2-classes-and-forms.md for exact checkpoint and remaining work.

## M2 completion: form events and modal continuations

ADR 0024 adds explicitly opted-in `start_suspended_callback` and
`finish_suspended_callback`; existing callbacks remain terminal-only. Waiting
frames, foreign tokens, locations and error state survive ordinary event frames
and source break/inspect/edit/resume. Depth is bounded at eight and work/payload
budgets are shared. Cancel/disconnect release every pending token; failed events
abort parked continuations. Snapshot/reset refuse unfinished parked callbacks.
`read_instance_field` is inert and validates native handle/member identity.
Named IR7 field captures preserve canonical ByRef storage. The VM has no form
control definitions, toolkit or IO dependency. The upper IDE implements selected
form lifecycle/control events; see [the exact profile](m2-acceptance.md).

Getter aliases resolve before chained member Get/Call/Let/Set under
[ADR 0025](decisions/0025-getter-alias-member-dispatch.md). Method aliases retain
the existing fixed-call shortcut. Hosts can therefore expose dynamic roots while
objects captured by source retain their identity. No IR version changes.

## Live host-object subscriptions (M6)

`Session::host_event_handlers` resolves actual live foreign WithEvents fields by
host/object identity and validates every matching Sub signature before effects.
Connection order is stable, enumeration is budgeted, and opaque callback handles
remain session-bound. Scheduling uses the existing callback/debugger machinery;
reset, disconnect and stale receivers cannot revive a subscription. The host owns
trusted event routing, sequential ByRef propagation and its document undo batch.
[ADR 0029](decisions/0029-host-event-subscriptions.md) defines the capability.

Numeric For loops now accept scalar Variant counters, including implicit locals
without Option Explicit. Initialization, Next, Exit For and pause/resume retain
the normal checkpoint/alias/budget machinery. Variant bounds are validated before
the counter is written; existing numeric operations own coercion and overflow.

## Host calls while waiting (2026-11-03)

[ADR 0036](decisions/0036-suspended-host-project-calls.md) defines explicit host-mediated
calls into the existing waiting session. Use `Ide::start_suspended_call` only for its exact
pending token and read `take_host_call_result` after normal ticks. Callback outputs belong
to the callback; never copy them into unrelated original foreign argument slots. VBA runtime
errors return to the caller, while resource/protocol/cancellation failures cancel the stack.
Persistent state, parked locations and cumulative budgets are shared. Reset/close cancel all
waits. Hosts own precommit/cancellation policy and asynchronous prompt delivery.

`Ide::abort_host_wait(token, failure, host)` selects an embedding's fatal nested-error
policy. It validates the active token, cancels the whole parked/modal stack and exposes
a runtime diagnostic; it does not inject a catchable error into the original caller.

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

The generic VM avoids repeating instance-lifetime scans across scalar-only
checkpoints. Reference-bearing stack/storage changes, frame changes, event
deliveries and enumeration-root removal invalidate the cached scan; every
execution slice starts conservatively invalidated. Normal finalization still
repeats to a fixed point before proceeding. Execution, allocation and host
permission limits remain unchanged.

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

UTF-16 storage contexts do not supply an ANSI code page to Asc/Chr. ASCII works;
non-ASCII ANSI conversion refuses without a configured ANSI encoding. The byte
variants and Unicode AscW/ChrW contracts are unchanged (ADR 0070).

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

Native UserForm activation follows [ADR 0077](decisions/0077-native-window-activation.md):
ordinary coordinator fields own the active application window, Show/Hide/Unload
ordering and reentrant transitions. Inert active-window inspection never loads a
form; executable window focus uses the normal input/authorization path. Shared
egui modeless routing preserves callback receipts across window changes. Automatic
host document-input barriers and full M7A.4 lifecycle acceptance remain open.

[ADR 0078](decisions/0078-document-activation-input-barrier.md) adds the bounded
pre-document raw-input barrier and opaque execution identity for native modeless
activation. Hosts authorize the ordinary Deactivate callback before document
input; Stop/Continue, cancellation and acknowledged repaint precede replay.
The remaining M7A.4 checklist and M8.5 NOT_RUN status are unchanged.

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

[ADR 0090](decisions/0090-native-caption-alignment.md) adds selected native
CheckBox/OptionButton Alignment: inverse persisted flag translation, ordinary
Long state and published constants, Right defaults, typed errors and inert
inspection. Caption text stays physically left aligned in a separate clipped
rectangle beside the real indicator widget. Input, full accessible labels/state,
Value/Null/groups/events, source/resources, global style and all budgets remain
unchanged. ToggleButton's disabled property is not enabled. Other properties,
AutoSize/scrolling and full M7A.4 remain open; fresh Office evidence is NOT_RUN.

[ADR 0091](decisions/0091-native-hide-selection.md) adds selected native
TextBox/ComboBox HideSelection flags, True defaults and ordinary Boolean state.
Changing the property preserves text, ranges, events and resources. Passive
highlights use canonical UTF-16 ranges and local clipped galley copies, including
both ComboBox presentations; focus/style queries stay outside font locks.
Inspection, global style, font caches and all VM/host budgets remain unchanged.
Other selection/property/AutoSize/scrolling and full M7A.4 work stay open; fresh
Office observations remain M8.5 NOT_RUN.

[ADR 0092](decisions/0092-native-auto-tab.md) adds native TextBox/ComboBox AutoTab,
False defaults and ordinary user-edit-driven tab traversal through the existing
validation/focus continuation. Actual edit cause survives unchanged autocomplete
text; caret movement and programmatic assignments remain passive. Reentrant
focus/window/policy changes suppress stale navigation. No VM/foreign/budget or
persistence change is implied; full M7A.4 and fresh Office acceptance remain open.

[ADR 0093](decisions/0093-native-selection-margin.md) adds native TextBox/ComboBox
SelectionMargin, True defaults and a theme-local selection strip. Whole displayed
rows select/drag through the existing mouse continuation, including wrapped text
and canonical UTF-16/CRLF ranges. Post-callback state remains authoritative;
selection-only input stays passive and empty edit fields render blank. No VM,
foreign, budget or persistence change is implied. Full M7A.4 remains open.

[ADR 0094](decisions/0094-native-textbox-scrollbars.md) adds native TextBox
ScrollBars, ordinary validated Long state/constants and actual bounded viewports.
Text/ranges remain canonical; offset/render markers are tagged UI-only state.
Track/drag waits for callbacks, current policy/focus and lifetime. Measured room,
MultiLine and WordWrap determine axes; reset, caret following and scrolled hits
are covered. Native ListBox stored Both is accepted without a new public member.
No VM/foreign/budget/persistence change; full M7A.4 remains open.

[ADR 0095](decisions/0095-native-textbox-hidden-viewports.md) separates native
TextBox scrolling axes from visible bars. None/room-hidden bars retain caret
viewports and translated pointer geometry under fixed authored bounds. Existing
canonical text/ranges, UI-only tagged offsets, authorized callback/default routing
and Reset remain intact; accessible chrome uses local caller styles. No new API,
format, foreign call or budget. AutoSize, wheel/repeat/platform, geometry/font-change
edge cases and full M7A.4 remain open; fresh Office comparisons are NOT_RUN.

[ADR 0096](decisions/0096-native-button-click-focus-and-input-identity.md) adds native
CommandButton TakeFocusOnClick, True defaults and False clicks that retain text
focus/selection and deferred validation. Existing callback/trust/Reset paths stay
intact. Queued input survives transient collection writes and shifted peer indices;
actual membership and identity are validated before delivery. IDE ticks respect
smaller active-session slices. No API beyond the Office property, new format,
foreign call or increased budget. Full M7A.4 and fresh Office comparisons stay open.
