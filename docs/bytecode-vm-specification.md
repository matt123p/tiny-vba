# Bytecode and explicit-state VM specification

Version: IR 1. Date: 2026-10-26. Status: adopted implementation contract under
[ADR 0004](decisions/0004-bytecode-vm-contract.md). M2 structural IR/verifier/lowering and M3 execution subset implemented. See [compiler](compiler.md) and [runtime](runtime.md).

This document defines the M2/M3 instruction representation, verifier, calling convention and execution
lifecycle. It supplements section 6 of the preserved [implementation specification](vba-interpreter-rust-implementation-spec.md).
Current code implements [M0 foundations](decisions/0003-m0-source-scenarios-foreign.md), the M1 frontend, [M2 compilation](compiler.md) and [M3 runtime subset](runtime.md).
Structural verification alone adds no execution result; the M3 runtime contract records actual execution evidence.

## 1. Scope, versions and owners

IR 1 is a checked, immutable, in-process Rust instruction representation. Instruction PCs are indexes
into a procedure's instruction vector, not byte offsets. Instruction names and operand types below
are normative; Rust spelling may follow idiomatic enums/newtypes. There is no serialized bytecode
file format, numeric opcode assignment, binary loader, Microsoft P-code compatibility or binary
project writer in this version. A future disk format needs its own version, decoder budgets and ADR.

A program carries `ir_version = 1`, a compatibility profile and an explicit required-feature set.
Unknown versions, features, instructions or operand kinds fail verification before execution.
Changing an instruction's operands, meaning or stack contract, or adding an instruction, increments
the IR version. Enabling another tested scalar combination under an existing instruction requires
an explicit feature/profile capability update; it cannot silently change an existing profile's result.
Do not encode unimplemented syntax as a no-op or a fabricated constant.

| Owner | Contract |
|---|---|
| vba-diagnostics | Existing SourceId, StatementId, checked Span and diagnostic attachment |
| vba-values | Existing Value/VbaString/ObjectRef/Location; future declared-slot typing and alias storage |
| vba-syntax | Source-preserving tokens/AST; distinct same-line statement identities |
| vba-semantics | Bound symbols, procedure signatures, argument mapping and supported-language checks |
| vba-compiler | Compilation orchestration and lowering from bound program to unverified IR |
| vba-ir | Program/Instruction/metadata, bounded verifier, opaque VerifiedProgram and disassembler |
| vba-runtime | Shared operator, conversion, assignment and VBA runtime-error rules |
| vba-foreign | Existing owned native generic-host protocol and pending-call controller |
| vba-vm | Frames, dispatch, storage, budgets, pending continuations and inspection events |
| vba-debug | Breakpoint binding, stepping policy, paused inspection/edit commands |

The compiler is an implemented upper-layer crate depending on diagnostics, syntax, semantics, IR
and values. Its M2 package and reviewed DAG edges are recorded in ADR 0006. IR never depends on
semantics; the VM never depends on compiler, debug, DAP or UI. M2's `ir1-structural` target verifies
all IR 1 shapes without claiming any runtime implementation; M3 checks the actual m3-core-v1 capabilities at launch.


### Delivery capabilities

| Gate | Required subset / explicit exclusions |
|---|---|
| M2 | Program/signature/source metadata, every IR 1 instruction's structural verification, deterministic disassembly and malformed-IR rejection; execution may be unavailable |
| M3 first execution | Scalar constants, local/result slots, checked Integer/Long arithmetic needed by Answer, assignment conversion, Jump/Branch, Return, bounded run slices and checkpoints |
| M3 completion | Small procedure/If/For/call subset, direct ByVal and exact-type local ByRef calls, generic host invocation/wait/cancel, Stop, debugger controls, native and WASM execution proofs |
| M4 | Broader numeric/coercion rules, arrays/records, optional/named VBA binding, full alias/temporary semantics, builtins, persistent state and VBA error handlers; extend IR/version where required |

Feature names for IR 1 are `scalar-core`, `integer-long-ops`, `direct-calls`, `byref-locals`,
`foreign-snapshot-v1` and `debug-stop`. A producer declares every feature it uses, and the verifier
checks instruction/descriptor requirements against that declaration and the consumer's supported set.
`integer-long-ops` enables only the documented/tested operator and conversion matrix shipped by that
consumer. Structurally valid dynamic operations outside that matrix fail as Unsupported, never PASS
or a guessed VBA error. Record the exact matrix with the M3 implementation before enabling this feature.
Every program requires scalar-core. Unary/Binary and numeric Convert require integer-long-ops;
Call requires direct-calls; a ByRef signature or Reference direct-call descriptor additionally requires
byref-locals. Invoke requires foreign-snapshot-v1, and Stop requires debug-stop. RefLocal/ReadRef/
WriteRef for ordinary local assignment are scalar-core operations, not evidence of call-binding support.
Other supported conversion kinds are advertised individually in the consumer capability set. M2 may
verify a producer's requested execution-capability target without providing a VM; M3 session creation
must check those requirements against its actual runtime. No verification result implies execution.

## 2. Program and procedure layout

All IR indexes are distinct checked `u32` newtypes, even on 64-bit hosts: SourceIndex, ProcedureId,
ConstantId, LocalSlot, CallSiteId, ForeignSiteId and SiteId. A PC is a procedure-local `u32`.
Converting indexes or adding sizes must be checked. Host tokens, SourceId and runtime Location remain
their existing types; none is an executable PC or a host object pointer.

An unverified Program contains:

- IR version, required features, profile identifier, immutable project revision identifier and entry ProcedureId.
- Ordered source table: existing SourceId, module name and retained SourceFile. A source identity is unique
  in the program and refers to one immutable text revision. Do not reopen source paths while running.
- Constant pool of validated scalar Value entries. No Missing or live Object references in constants;
  omitted arguments belong in descriptors, and hosts supply live object handles at launch. Empty, Null,
  Nothing and CVErr remain different values. Strings retain exact UTF-16 units.
- Procedure table, including declaration identities, local layouts, signatures, instruction vectors,
  checkpoint tables and direct/foreign call-site tables.
- A launch signature identifying any host-provided entry arguments. IR 1 entry procedures otherwise
  follow the same binding rules as ordinary direct calls; argument mismatch is a launch error.
  IR 1 launch entries accept only ByVal parameters (or no parameters), because launch supplies Values,
  not external writable locations. ByRef procedures remain callable from other VBA procedures but
  selecting one with ByRef entry parameters as the launch entry is rejected.

A Procedure contains a source/module identity, name, kind (`Sub` or `Function`), parameter descriptors,
local descriptors, optional function-result LocalSlot, entry PC, declared maximum operand-stack depth,
code and metadata tables. A statement table maps each syntax StatementId to its checked source span;
statements removed as unreachable remain in this table but have no executable checkpoint binding.
Property procedures/classes/persistent module storage need later features.
Parameters occupy the first N slots in declaration order. A Function's result occupies slot N; ordinary
locals and compiler temporaries follow in source/lowering order. A Sub has no result slot. Parameters
have name, declared scalar kind (`Variant` or a supported exact Value kind), ByVal/ByRef mode and
required-argument status. Optional/default/ParamArray descriptors are rejected in IR 1.

Each local descriptor records declared kind, binding role (parameter/result/user local/compiler
scratch), source identity when applicable, and debugger visibility. Scratch slots are never named as
user locals. Initial values are Empty for Variant, numeric zero of the declared numeric kind, False,
empty UTF-16 String, or Nothing for Object. Unsupported kinds fail verification; nothing is initialized
by flattening to f64. Scratch slots obey the same initialization rules. M3 supports only a published
subset of these kinds; the existence of an M0 Value variant is not execution support.

Immutable source and metadata tables plus project revision form the identity of compiled code.
Programmatically supplied entry values are validated and bound before Ready; they are not constants.
The initial generic host target can therefore be an Object-valued entry parameter without an Office
global registry or an object literal embedded in bytecode.

ScalarKind is the shared declared-type descriptor planned in vba-values: Variant, Boolean, Byte,
Integer, Long, LongLong, Single, Double, Currency, Date, String and Object. Empty/Null/Missing/Nothing/
CVErr are Value states, not extra declared types. Variant slots retain the actual Value tag without
wrapping it in another Value variant. Object declarations carry only the generic external-reference
kind in this version; class-specific typing needs a later contract. Convert's Object destination is
rejected in IR 1; object-reference assignment uses WriteRef's explicit mode instead.

For deterministic lowering, procedures follow ordered module/declaration order; parameters and user
locals follow declaration order, compiler slots follow creation order, and sites follow emitted-PC
order. Constants are deduplicated in first-emitted-use order by exact tag/payload (including float bits
and UTF-16 units). Never let hash-table iteration assign executable IDs. Name lookup remains the
binder's responsibility; display spelling is preserved independently of case-insensitive resolution.

### Checkpoint/source metadata

Every SiteId identifies one checkpoint record in its procedure:

```text
{ statement: StatementId, span: Span, role, visible: bool,
  enter_pc: PC, resume_next: Option<SiteId> }
```

StatementId belongs to the source's syntax statement; SiteId distinguishes lowered visits such as
loop initialization and recheck. `role` is `Source`, `LoopInit`, `LoopTest`, `LoopAdvance` or
`SyntheticExit`. Two source statements on one line have different StatementIds. Multiple lowered
sites for one source statement retain its StatementId and have distinct SiteIds. `visible` controls
source stepping, not instruction budgeting or cancellation. Synthetic sites retain a valid source
span and meaningful parent statement identity; only synthetic sites may use an empty span.

`resume_next` identifies the normal source successor used by a future error-handler implementation,
not the next arbitrary instruction. It is None at procedure exit. M2 validates that it names a
checkpoint in the same procedure. IR 1 never executes VBA Resume. Complex branch/loop Resume rules
need additional verified metadata in a future IR version, not inference from `pc + 1`.

## 3. Operand stack and instruction set

Each frame has its own operand stack; there is no shared native recursive evaluator stack. Stack
entries are `V` (an owned scalar Value) or `R` (an opaque writable Location). A Location is not a
Value and cannot be serialized, returned as a VBA value, passed ByVal or stored into another value.
`R` entries retain their declared slot-kind obligation. RefLocal on a ByRef parameter returns the
caller's canonical location, not a location containing a copied argument.

Notation shows the stack bottom on the left; `P` is a preserved prefix. Constants, locals, sites and
PC operands belong to the current procedure/program unless a ProcedureId is explicit. Each basic
block starts with exactly one Checkpoint, has straight-line instructions (calls can suspend), and
ends in exactly one Jump, Branch or Return. There is no implicit fallthrough between blocks.

| Instruction and operands | Required stack → result | Defined action |
|---|---|---|
| `Checkpoint site: SiteId` | `[] → []` | Enter this dynamic statement visit; expose source safe point before block effects |
| `Const constant: ConstantId` | `P → P,V` | Copy validated scalar constant with its exact tag |
| `ReadLocal slot: LocalSlot` | `P → P,V` | Read through the local's owned slot or ByRef alias |
| `RefLocal slot: LocalSlot` | `P → P,R` | Push its validated canonical writable location |
| `ReadRef` | `P,R → P,V` | Read checked location without retaining a Rust borrow |
| `WriteRef kind: AssignmentKind` | `P,R,V → P` | Apply the specified assignment mode through runtime, then commit one checked location write |
| `DupValue` | `P,V → P,V,V` | Copy one scalar value; does not duplicate a location |
| `DropValue` | `P,V → P` | Discard a value |
| `Convert kind: ScalarKind` | `P,V → P,V` | Explicit conversion using the shared runtime rules |
| `Unary op: UnaryOp` | `P,V → P,V` | Apply supported unary runtime operation |
| `Binary op: BinaryOp` | `P,Vleft,Vright → P,V` | Apply operation with preserved operand order |
| `Jump target: PC` | `[] → []` | Finish statement block and transfer to a Checkpoint |
| `Branch if_true: PC, if_false: PC` | `[V] → []` | Convert condition through the shared runtime rule; finish block and select a Checkpoint |
| `Call procedure: ProcedureId, site: CallSiteId` | `P,args(site) → P[,V]` | Bind arguments and push explicit callee frame; Function produces V, Sub produces none |
| `Invoke site: ForeignSiteId` | `P,Vtarget,args(site) → P,V` | Owned generic host call; always produces its return Value on success, including Let/Set |
| `Stop` | `[] → []` | Advance past Stop, then expose a resumable explicit-stop pause |
| `Return` | `[] → terminal` | Read the Function result slot or finish a Sub; unwind one explicit frame |

AssignmentKind is Value (VBA Let/default value assignment) or ObjectReference (VBA Set).
ObjectReference accepts only Nothing or Object reference values into a compatible Object/Variant
location. Value assignment must not silently substitute Set or invoke an unimplemented default member;
object-to-value conversion outside the supported matrix is Unsupported. This mode is retained even
when the initial supported scalar cases happen to have the same storage effect. Parameter binding is
separate and follows its signature's declared-kind/copy rules.

`UnaryOp` tags are Negate and Not. `BinaryOp` tags are Add, Subtract, Multiply, Divide, IntegerDivide,
Modulo, Power, Concat, Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual, And, Or, Xor, Eqv and Imp.
These tags describe operations, not blanket support for every Value combination. No short-circuit
substitution for VBA And/Or is permitted. Unknown tags and unsupported op tags are rejected before
execution; consumer capabilities include explicit supported instruction/op/conversion-kind sets in
addition to feature names, and these sets are checked during verification; unsupported dynamic operand combinations use the explicit Unsupported failure path.
Conversion, Null/Empty behavior, integer promotion/overflow, Boolean numeric representation and
runtime error numbers live only in vba-runtime, with profile-tagged tests before support is claimed.

IR 1 does not include general global lookup, fields, array indexing, object construction, enumeration,
With, handler installation, Raise, Resume or builtin dispatch instructions. Syntax requiring these
fails compilation explicitly until an appropriate feature/IR extension exists. Later extensions must
specify operands, stack effects, exceptional edges, budgets and debugger source mapping as above.

A Branch condition is not a Rust truthiness test. IR 1 may initially accept only actual Boolean values;
other cases are Unsupported until the runtime's conditional conversion matrix is implemented. Jump
and Branch may target the same checkpoint, including self-loops; every visit consumes work budget.
Calls/Invoke require their declared feature. Stop requires `debug-stop` and the next instruction must
be a Jump, so continuing Stop reaches a new source checkpoint without executing Stop again.

## 4. Calls, aliases and host dispatch

### Direct calls

A CallSite contains its source checkpoint, callee ProcedureId, and arguments in source evaluation
order. Each argument descriptor records mode (`Value`, `Reference` or `Omitted`), optional name and
resolved parameter index. Stack consumption is the descriptor's ordered suffix: one V for Value,
one R for Reference, and no entry for Omitted. A descriptor's callee must match the Call operand.
The resolved mapping must be one-to-one and complete for required parameters. Value descriptors
match ByVal parameters and Reference descriptors match ByRef parameters in the initial subset;
forced-value temporary binding for ByRef is not inferred by the VM. IR 1 accepts only
positional, present arguments in declaration order; named/omitted mappings are reserved and rejected
for direct VBA calls until the M4 binding contract is implemented.

The compiler evaluates arguments in source order and never sorts emitted effects to match parameter
order. This is the engine's deterministic lowering contract; compatibility for effectful expressions
needs profile-tagged observations before broad VBA ordering claims. In the initial exact-type subset,
ByVal binds a copied, assignment-checked value into new callee storage; ByRef binds a canonical caller
Location of the same declared kind. Distinct parameters referring to the same location see immediate
writes through that location, including writes made before a later callee runtime error. There is no
copy-in/copy-out implementation of direct VBA ByRef calls. ByVal of an Object copies the reference
identity, never the host object's contents.

Parenthesized arguments, conversions through temporary ByRef values, property l-values and arrays
need M4 rules and must be rejected before executing their call in the initial subset. Caller-owned
locations outlive callee aliases; returning the callee invalidates its owned parameter/local/result/
scratch locations, not caller aliases. The result Value is copied before invalidation. A Function
name assignment writes its hidden result slot; Return does not pop a result expression from the stack.
A Sub returns no operand to its caller. Invoking a Sub as a value fails binding/verification.

Before dispatch, capture the call's fault PC/checkpoint, validated arguments and continuation PC.
The caller's PC advances to the continuation; its operand prefix remains in that caller frame.
Push the callee only after frame/storage budget and binding checks succeed. A successful return
pushes exactly one result for a Function and none for a Sub. A callee failure is delivered to the
saved call site, never by redispatching the Call instruction. Capacity checks must reserve the
caller's result space before callee side effects occur. No native Rust recursion implements VBA calls.

### Foreign calls

A ForeignSite contains source checkpoint, IR-owned kind (`Call`, `Get`, `Let`, `Set`), member name and
argument descriptors in evaluation order. Names and omitted arguments are retained, not prebound to
an Office signature. The descriptor layout is the same Value/Reference/Omitted suffix convention;
Vtarget below that suffix must resolve to a valid external ObjectRef. IR owns its small descriptor
enums; vba-vm translates to vba-foreign, without adding an IR-to-foreign dependency.

At dispatch, validate value/location lifetimes, host identity, per-site capability permission and
budgets. Then move arguments into M0 Invocation and call the host natively. Capabilities are embedding
policy checks supplied to VM invocation; bytecode cannot grant itself permissions. Let and Set stay
different kinds. Invoke pushes one result V; statement-only callers emit DropValue. Invalid targets
and host failures use documented runtime mapping where supported; unsupported mapping stays explicit.

`foreign-snapshot-v1` is the restricted M0 generic embedding protocol, not a frozen Office or full VBA
ByRef ABI. The M3 adapter must reject aliased ByRef argument locations and unsupported declared-type
writeback conversions before host dispatch. Variant-like locations and non-aliased, exact-tag scalar
writebacks may be exposed for the generic proof. Validate every proposed writeback, including declared
kind and allocation capacity, before any VM patch is committed. This requires an M3 extension/adapter
around the current untyped M0 Locations store; it is not already implemented. The M3 integration must
add a precommit typed-validation hook to the foreign controller (or an equivalent checked store API).
The controller is the sole writeback committer: validation occurs before its current patch loop,
and the VM consumes its outcome without applying writes again. Calling the untyped M0 API first and
checking types afterward is invalid. Hook failures remain host-protocol diagnostics, not fabricated
ForeignError runtime numbers. Malformed host replies
are terminal host-protocol failures; preserve successful external effects and report the failure.

M0 errors carry no VM writebacks. Calls needing partial ByRef writes on error or live foreign aliasing
remain Unsupported until the M4 foreign contract is extended and tested. Do not quietly apply the
snapshot protocol to those language constructs or claim compatibility from generic-host tests.

### Exactly-once host continuation

Before calling Host::invoke, remove argument operands into an owned continuation, save fault PC,
source checkpoint, caller stack base, expected result shape and continuation PC, and mark it dispatched.
A synchronous success installs validated writes/result once and continues at that PC. A synchronous
error follows the saved failure path. A Wait stores the continuation plus the vba-foreign pending
Session; no mutable host, document or location borrow survives returning WaitingForHost.

Only explicit `resume_host` on the matching session/host may poll the pending call. `run_slice` while
waiting reports Waiting without polling or charging a second invocation. Repeated waits keep the
same VM continuation and use the host controller's replacement token. Completion consumes the
continuation once, observes the controller's already-committed validated patches, pushes one result
and continues after Invoke. Duplicate
resume/completion and wrong-session/host actions return command errors without changing valid state.
The fault checkpoint remains active until a later Checkpoint is reached. A requested debugger pause
while waiting is serviced at the next visible checkpoint after completion, not by replaying the call.

Stop, disconnect, host close, timeout and explicit cancellation must call the existing Session::cancel
when a token is pending, discard the VM continuation, and invalidate inspection handles. Stale ByRef
locations cancel before polling, as in M0. Closure of a host requires a cancellation opportunity before
destruction; if it disappears first, record HostDisconnected and abandon VM-owned obligations without
claiming that external resources were released. Terminal VM paths cannot retain live frames or VM-owned
references; keep bounded immutable failure snapshots instead. Automatic Drop cleanup cannot contact a
host and is not a replacement for this lifecycle API. External side effects are never rolled back by VM.
Existing M0 ObjectRefs are non-owning opaque
host tokens; dropping VM-owned Value memory does not claim to destroy external objects. Hosts own
their object table lifetime. Retain/release semantics for stronger ownership need a later protocol
extension and tests before they can be exposed as VBA object lifecycle support.

## 5. Verification and disassembly

Only vba-ir can construct VerifiedProgram, through a bounded `verify(unverified, capabilities, limits)`
operation. It owns immutable checked tables; callers cannot mutate its instructions through a public
reference. VM construction accepts only VerifiedProgram and rechecks that its profile/features/op requirements
match the actual runtime implementation and launch policy. A caller-supplied capability set cannot
make missing VM functionality executable. Verification returns source-aware structured
errors, never a partially executable program. Trusted compiler output goes through the same verifier.

The verifier performs these checks before allocating session state:

1. Version/features/profile and all table/string/aggregate limits; unique source/procedure identities;
   checked sources/spans; exact constant tags and payload validation. Reject live-object/Missing constants.
2. Procedure/signature/slot layouts, entry existence, result-slot consistency, parameter mapping,
   supported declarations and nonempty code. Entry is a Checkpoint. Every operand index is in bounds.
3. Divide code into blocks: each begins at its Checkpoint's unique enter_pc; the next Checkpoint follows
   a terminator. No instructions occur after a terminator before the next Checkpoint. Every block has a
   terminator, including the last. Stop appears only immediately before Jump.
4. Every branch target is a Checkpoint in the same procedure. No branch into an expression, caller
   frame, operand sequence or past end. All blocks are reachable from that procedure's entry; lowering
   removes unreachable code while retaining nonexecutable source metadata. Infinite reachable cycles
   are legal; verification must not require termination or a reachable Return.
5. Simulate V/R stack kinds independently from an empty stack at each block entry, using a bounded
   iterative scan. Reject underflow, wrong kinds, nonempty terminator stacks (Branch consumes its sole
   V), excessive computed depth and a declared maximum different from the computed maximum. This is
   stack-shape verification, not proof of VBA dynamic value types or program termination.
6. Check local references and writable roles, direct-call signature/operand consistency and each call's
   argument suffix/result shape. A ByRef local parameter resolves canonically at runtime; a location
   may survive a nested call only while its owning frame remains live. Ref values never escape as V.
7. Validate metadata ownership: each instruction belongs to its enclosing checkpoint block; call-site
   records name that checkpoint, spans use its procedure source and exact immutable source revision,
   StatementId sources agree, SiteIds/enter PCs are unique, and resume_next references remain in-procedure.
   Checkpoint StatementIds must exist in the statement table and use a span contained in that statement's span; synthetic exits use the declaration's
   end-statement identity/span.
   All visible sites have nonempty spans. Multiple sites may intentionally share a StatementId/span.
8. Reject opcodes/descriptors not supported by the declared feature set. Verify all procedures even if
   not reachable from the selected launch entry. Bound verification work separately from execution.

Every instruction belongs to one block, so stack simulation does not need unbounded fixed-point
iteration. Checkpoint-to-checkpoint edges all have empty operand stacks. A caller can still retain a
prefix while its callee runs, within a different explicit frame. Runtime checks remain mandatory for
location generations, typed assignment, host replies and dynamic dispatch; verification cannot prove them.

Disassembly is diagnostic UTF-8 text, not a loader format. For identical ordered sources, revision,
profile and lowering options, output must be byte-for-byte stable. Print IR version, feature set and
profile, constants with exact tags, procedures/signatures/slot layouts and numbered PCs with all
operands. Print checkpoint StatementId/SiteId, role, visible flag, byte span and resume_next. Strings
use escaped UTF-16 code units when needed; float payloads include exact bit patterns, Currency shows
scaled i64 and Date shows tagged serial value. Never use host Debug pointer addresses or locale formats.
Disassemble malformed programs only through a separately bounded diagnostic path, without execution.

## 6. Dispatch, states and failure behavior

A VM session owns its immutable VerifiedProgram, fresh session identity, explicit frame vector,
location storage, execution counters, host binding identity, optional pending host continuation,
checkpoint visit serial, control requests and bounded diagnostic/output records. A Frame owns its
ProcedureId, frame generation, PC, operand stack, slot bindings, current checkpoint and optional
saved caller continuation. The frame vector is the VBA call stack; Rust calls only perform bounded
individual dispatch operations. Frame IDs are never recycled within a live inspection generation.

### State transitions

| Current state | Accepted operation / event | Result |
|---|---|---|
| Ready | run_slice with positive allowance | Run from entry checkpoint; return one of the outcomes below |
| Running (internal to run_slice) | quota exhausted | Yielded, preserving exact PCs/stacks; externally Runnable |
| Running | checkpoint predicate/pause request or Stop | Paused with a new inspection generation |
| Running | foreign Wait | WaitingForHost, with exactly one pending continuation |
| Running | outermost Return | Completed with Function Value or Sub unit result |
| Running | runtime/unsupported/protocol/resource failure | Failed, after pending-host cleanup where applicable |
| Runnable | run_slice | Continue from exact next instruction; never restart procedure |
| Paused | continue/step command | Invalidate handles; permit the paused checkpoint visit if applicable; become Runnable |
| Paused | run_slice without continue/step | Return existing Paused snapshot; no execution |
| WaitingForHost | run_slice | Return Waiting; no host poll |
| WaitingForHost | resume_host with matching identity | Complete, fail or remain waiting as defined above |
| Ready/Runnable/Paused/WaitingForHost | cancel/stop/disconnect | Cleanup, then Cancelled or Failed(HostDisconnected) |
| Completed/Failed/Cancelled | run_slice | Return same bounded terminal outcome, without re-execution |

Running is not an externally reentrant session state: nested mutable dispatch/callback is rejected.
Callback/event queues are specified and implemented for M7 (ADR 0015, `schedule_callback`/
`run_next_callback`); a synchronous host still cannot recursively drive this session. A
`run_slice(0)` is a command error and changes nothing. For a valid runnable slice, check cancellation/
disconnect first, then hard limits, then remaining slice quota, then the checkpoint predicate, then
dispatch. An expired hard limit is not hidden behind a soft yield or breakpoint. Validate command
shape/state first; terminal outcome queries remain idempotent. Invalid commands are structured API
errors, not VBA runtime errors and not state transitions. Host resume polling is separately charged to host
poll budgets; it does not re-charge an already-dispatched Invoke instruction.

`run_slice` reports Yielded, Paused(reason), Waiting, Completed(result), Failed(failure), or Cancelled.
A result distinguishes a Sub's unit return from a Function's Value::Empty. Yielded is an execution
quota boundary, not a debugger pause: stacks may be nonempty, and local editing is unavailable.
The scheduler may change thread/task execution between slices only if the embedding API permits it;
this specification requires no Send/Sync promise or native worker thread.

### Instruction commitment and failures

Each instruction has a saved fault PC and either commits once or reports a failure. For scalar
operations, validate operands/conversions/budget before mutating a destination, preserve exact
operand order and never retry a faulting operation on continue. Allocation reservations for result
slots/stacks are made before side-effecting calls. Failure snapshots retain the fault checkpoint,
source span, runtime number when known, and bounded stack trace. Already performed host writes and
direct ByRef writes remain observable; VM failure is not a macro transaction.

IR 1 has no On Error handler instructions. A catchable VBA runtime error is recorded as a structured
VbaRuntimeError and is unhandled in this version. Unsupported, HostProtocol, ResourceLimit,
HostDisconnected and InternalInvariant are separate non-VBA failure categories; they must not be
swallowed or mapped to a guessed runtime number. Cancelled is distinct from timeout/resource failure.
Unexpected invalid internal state terminates with a diagnostic rather than panic or unchecked access.

Reserve diagnostic codes VBA0020 (invalid IR), VBA0021 (verification resource limit), VBA0022
(incompatible IR version/features/profile), VBA0030 (VM invariant failure), VBA0031 (execution resource
limit), VBA0032 (host disconnected), VBA0033 (host protocol failure) and VBA0034 (invalid VM command).
VBA0001 remains Unsupported. These are planned engine diagnostic codes, not VBA runtime error numbers
or already implemented constants. A runtime failure carries its independently mapped VBA number plus
source and stack metadata; infrastructure codes cannot be caught as invented VBA errors.

Error-break policy belongs to vba-debug. The VM can expose a fault-safe snapshot with a saved pending
failure before terminal cleanup; continuing delivers that failure once rather than retrying the
instruction. Such a fault pause permits inspection but no local edit or instruction retry in IR 1.
A future handler-capable IR must define enabled/active handlers, fault-versus-return PCs, Resume targets,
Err/Erl storage, propagation across frames and partially completed statements before enabling On Error.

## 7. Checkpoints, stepping and inspection

Checkpoint is the only normal source breakpoint/local-edit safe point. It runs before that block's
effects with an empty top-frame operand stack. On first arrival, increment a session-wide checked
visit serial and record `(session, frame, SiteId, visit)`. A breakpoint pause leaves the checkpoint
not yet passed; continuing grants a permit for that exact visit, suppressing a repeated stop there.
A loop arriving at the same SiteId again gets a new visit, so its breakpoint can fire again. Quota
yields at that PC do not allocate another visit or suppress a future breakpoint accidentally.

Check hard limits/cancellation before offering a checkpoint. Offering its source pause precedes
committing/charging that instruction: retain an unpassed visit when paused, then charge and advance
past Checkpoint exactly once when its permit is consumed. Returning an existing Paused outcome does
not dispatch another instruction. Clear the pending visit after passage, so a later branch back
creates a fresh visit. Continuing a post-Stop pause does not grant a permit for the next checkpoint;
continuing a fault pause delivers its pending failure rather than granting a checkpoint permit.

The VM exposes immutable checkpoint/frame snapshots and accepts a bounded controller-supplied
break/continue predicate; it imports no vba-debug type. The predicate cannot evaluate VBA, call hosts
or mutate the VM. Controller errors return a structured command error and retain the same PC and
unpassed visit without granting execution; a retry cannot skip that checkpoint. Synthetic hidden checkpoints still enforce
budgets and cancellation but are skipped by normal source stepping.

vba-debug uses dynamic visits and frame identities, not just line numbers or stack depths:

- Step Into stops at the next visible checkpoint visit, including the callee's first visible site.
- Step Over stops at the next visible visit in the starting frame, or the surviving caller after it
  returns; descendant frames do not satisfy the step target. Explicit breakpoints/errors/Stop still stop.
- Step Out stops at the next visible checkpoint in a surviving ancestor after the starting frame
  returns. If the entry frame returns, report Completed instead of inventing another checkpoint.
- From an ordinary checkpoint pause, each step first permits that paused visit once. From Stop it
  starts after Stop without suppressing the next checkpoint. Same-line statements and repeated loop
  sites remain separate visits. Host waits do not count as a new source statement.

After a callee returns, a caller may still have expression instructions to execute before its next
checkpoint. Step Out/Over finishes that portion before a normal editable stop; it never reconstructs
or repeats the original call. A user pause request is checked at dispatch boundaries and serviced at
the next visible checkpoint. An infinite single-statement loop must contain a checkpoint on its
backedge; hard budgets/cancellation are checked even between visible checkpoints.

Stop pauses after its PC has advanced, with an empty stack; continuing executes its mandatory Jump.
Inspection handles include session, frame and pause generations. Continue, step, host resume, terminal
transition and code restart invalidate old handles. Retained immutable snapshots may still be displayed
as historical data, but never resolve for mutation through those handles.

M3 local edits are limited to the active frame at an ordinary checkpoint/Stop pause. Read and validate
the candidate through shared assignment rules before committing; rejected edits leave storage unchanged.
Editing a canonical ByRef local updates its aliased caller location. Other-frame edits, fault-pause edits,
waiting-state edits, hidden scratch/result replacement and arbitrary watch execution are Unsupported
until separately specified. Caller expression operands already captured before a callee pause are not
re-evaluated after local edits. Read-only inspection may show them as execution state.

Changing source while paused creates a different revision: mark it stale and require stop, recompile
and restart to execute the new source. Keep execution highlights/breakpoints tied to the old revision
until that restart; no live code replacement is implied. Watches initially read stored values only;
no Property Get, default member, function or host invocation is allowed by a pure watch.

## 8. Budgets and portability

All limits are checked before input-derived allocation, arithmetic, copying or host dispatch. These
are conservative initial engine defaults, not claims about VBA language maxima. Callers may lower
them; raising them needs explicit configuration and checked arithmetic. The M0 limits remain enforced.

| Resource | IR 1 default ceiling |
|---|---:|
| Sources / aggregate source text | 256 / 16 MiB, with existing 1 MiB per-source cap |
| Procedures | 4,096 |
| Instructions per procedure / total | 65,536 / 1,048,576 |
| Constants / aggregate constant payload | 65,536 / 16 MiB |
| Source checkpoint records / call-site records | Each bounded by total instruction count |
| Slots per procedure | 4,096 |
| Operand entries per frame / live total | 4,096 / 65,536 |
| Explicit call depth | 128 |
| Location allocations in the M0 session store | 4,096 cumulative; invalidation does not reuse slots |
| Individual string | Existing 32,768 UTF-16 unit cap |
| Call arguments / member or argument-name UTF-8 bytes | Existing 256 / 256 |
| Total dispatched instructions / per run_slice | 1,000,000 / 4,096 |
| Foreign invocations / host polls per session | 1,024 / 4,096 |
| Wait replies per pending call | Existing 1,024 |
| Logical session-owned mutable payload | 64 MiB, including live stacks, slots, snapshots and pending arguments |
| Retained diagnostics/output | 1,024 records and 1 MiB aggregate payload |

Metadata names/profile identifiers additionally have a 256-byte cap; source filenames keep the
existing 4,096-byte cap. Count aggregate metadata payload under the verifier's own 64 MiB logical
budget and charge each scanned instruction/table entry; abort after 8,388,608 such work units.
Logical payload accounting counts value/tag/slot cells with documented fixed charges plus owned
string buffers; it is not an operating-system RSS promise. Shared immutable program payload is
charged at verification, while per-session copies/snapshots are charged to the session. Implementations
must publish the charge sizes and reserve before growing buffers. Allocator failure is returned where
recoverable; use fallible capacity reservation for input-derived buffers.

Each dispatched instruction costs at least one unit; calls charge additional bounded argument-copy/
conversion work and runtime operations charge proportional work for strings/large values. Checking
cancellation, consuming a control permit, or returning Waiting cannot reset a hard budget. Quota yield
preserves hard counters. Cancel is checked before each dispatch and host poll. Timeouts use an injected
clock/policy; no ambient clock, randomness, IO, shell or COM belongs in bytecode execution. A blocked
synchronous host cannot be forcibly interrupted by this cooperative contract.

Native and wasm32 execute the same logical PCs, tagged values and checked integer operations. Host
pointer width never affects language widths; NaN/infinity handling follows existing Value validation,
and locale/date/random services are explicit dependencies. WASM compilation alone is insufficient:
M3 must execute the same selected programs natively and in a WASM runner with types/results and
pause/resume/side-effect counts checked. No browser-dependent host call belongs in headless core.

## 9. Lowering examples and acceptance obligations

The Answer example now has an M2 golden disassembly test; examples here remain normative pseudocode, not measured VBA execution.
Feature declarations and full metadata headers are abbreviated here; a real disassembler prints them.

### Answer returning Long 42

For the existing self-authored Arithmetic.bas, Function result slot 0 is Long and constant entries
0/1 preserve the frontend's Integer literal tags for 6/7. The shared runtime supplies multiplication
and assignment widening for this supported case.

```text
procedure 0 Answer() As Long; slots=[0:result Long]; max_stack=3
0000 Checkpoint site=0           ; visible assignment, its exact source span/StatementId
0001 RefLocal slot=0
0002 Const constant=0            ; Integer(6)
0003 Const constant=1            ; Integer(7)
0004 Binary op=Multiply
0005 WriteRef kind=Value
0006 Jump target=7
0007 Checkpoint site=1           ; hidden SyntheticExit, End Function source span
0008 Return
```

Return yields Long(42) from slot 0, not Integer(42), a JSON number, or f64(42). Declared maximum is
three stack entries: R,V,V immediately before Multiply. M2 verifies/disassembles; M3 executes it.

### Direct alias and loop lowering

`Increment x` emits RefLocal(x), then Call with one Reference descriptor. Callee ReadLocal/WriteRef
resolve the same canonical location. A nested failure leaves writes already committed to x visible.
If a callee is a Sub, its caller consumes no invented Empty return; a Function result can be dropped
explicitly when used as a statement.

A For loop uses compiler scratch locals for evaluated start/end/step values, so bounds are not
re-evaluated on every iteration. Source-header initialization and later recheck have different
SiteIds with the same StatementId. Initialization is visible before evaluating bounds; the first
internal test may be hidden to avoid a second immediate header stop. Next has its own source
checkpoint before increment, and subsequent rechecks return to a visible header site. Jump/Branch
edges always land at checkpoints. Counter kind, inclusive test, sign, conversion and overflow behavior
must follow the supported runtime/profile; do not infer all For semantics from a positive-step example.
Unimplemented loop forms fail compilation. A loop backedge cannot bypass instruction/cancel budgets.

### Host call trace

For one Invoke at PC k, a legal run is:

```text
Checkpoint visit A -> evaluate target/args -> Invoke k dispatched once
  -> Wait(token 5) -> run_slice returns Waiting without polling
  -> resume_host(5) -> Wait(token 6)
  -> resume_host(6) -> Complete(writebacks, result)
  -> commit once -> continue at k+1 -> later Checkpoint visit B
```

Pausing at B, editing a local and continuing never invokes k again. Cancellation during either Wait
cancels the current token and prevents its late completion from writing into a terminal session.

### Required acceptance cases

| Owner / gate | Required evidence |
|---|---|
| IR / M2 | Answer golden disassembly; deterministic constant/slot/site order; same-line/nested-call maps; version/feature rejection |
| IR / M2 | Bad constant/local/callee/site/target indexes, entry into expression, falloff, missing terminator, unreachable block, V/R underflow/mismatch, wrong max_stack and oversized metadata rejected |
| IR / M2 | Valid self-loop verifies within bounded work; invalid program cannot construct VerifiedProgram |
| Compiler/runtime/VM / M3 | Answer returns exact Long 42; small If/For, nested Function/Sub and exact-type ByRef; snapshots retain scalar tags |
| VM / M3 | Infinite loop exhausts/cancels; recursion/depth/storage/stack/string/call limits fail structurally; quota yield does not reset counters |
| VM/debug / M3 | Pause before effect, same-line separate stops, loop repeated visit, nested step into/over/out, local edit and continue; caller partial expressions never replay |
| VM/host / M3 | Invoke counter stays one across repeated waits, quota yields, pause and resume; named/omitted distinctions and all four invoke kinds preserved |
| VM/host / M3 | Wrong host/session, stale location/reference, duplicate resume, invalid/typed writeback, wait budget and stop/disconnect cleanup; no retained mutable host borrow |
| VM/debug / M3 | Continue across Stop once; fault pause never retries; stale pause handles and source revisions rejected; unsafe watches stay Unsupported |
| Native/WASM / M3 | Execute selected identical programs with result tags, cancellation and host-write counts; report runner/version and failures separately |

Array/UDT l-values, optional/named direct calls, error handlers, callbacks, foreign aliasing and partial
error writebacks need additional explicit contracts and evidence at their milestones. Passing this
subset must not mark all 20 M0 fixtures or external corpus candidates PASS. Update compatibility and
progress only from actual compilation/execution results.
