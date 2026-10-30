# M3/M4 runtime and debugger

Implemented 2026-10-26, extended by new M1.1. The current consumer is `m5-host-v1`, using verified IR 1/2/3/4/5/6/7.
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
(cap 1,024 entries) and named host roots (cap 64). Roots are `ObjectRef` values whose `host` must
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
