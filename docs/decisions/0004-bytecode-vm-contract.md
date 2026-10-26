# 0004 — Verified stack IR and explicit VM continuation contract

Date: 2026-10-26. Status: adopted design on user request, before M1/M2 implementation.

## Decision

Adopt [bytecode/VM specification, IR 1](../bytecode-vm-specification.md) as the detailed implementation
contract. It refines section 6 of the preserved supplied specification without editing that input.
This is a design deliverable: vba-ir, vba-vm and debugger remain scaffolds and VBA cases remain NOT_RUN.

Use immutable in-memory Rust instructions, procedure-local checked u32 PCs, a V/Location operand
stack per explicit frame, and blocks that begin at source checkpoints with empty stacks. Blocks
terminate with explicit Jump/Branch/Return. The verifier checks every procedure and is the sole
constructor of VerifiedProgram. Dynamic location/type/host checks remain in runtime dispatch.
There is no persisted bytecode encoding or Microsoft P-code execution in this decision.

Define exact instruction operands/stack effects, slot/signature/result layouts, deterministic
source maps/disassembly, calling conventions, budget ceilings and negative acceptance cases before
implementation. Version changes must be explicit; unavailable features fail instead of acting as no-ops.
The initial capability subset does not promise complete VBA operator or argument semantics.

Direct VBA ByRef aliases are canonical locations with immediate writes. Foreign invocation uses the
restricted M0 owned snapshot protocol until its broader ABI is designed; M3 needs a typed-writeback
adapter and rejects foreign alias cases the prototype cannot model. The VM stores owned, single-use
call continuations, so wait/resume and debugger controls cannot redispatch earlier calls. Preserve
successful host effects on failure; do not introduce macro rollback.

Checkpoint visits have identities independent of lines and loop iteration PCs. Explicit one-visit
continuation permits avoid stopping forever at the same breakpoint. Quota yields, host waits,
normal editable pauses, explicit Stop and fault inspection are distinct states. Stop/disconnect
must cancel pending host work; a missing host cannot be reported as successful external cleanup.
Inspection handles expire on continuation/restart. No native recursive evaluator, UI types or host
borrows cross pause/yield boundaries.

## Layering consequence

Plan a vba-compiler upper-layer crate for orchestration/lowering. The existing semantics crate cannot
lower into IR without an unreviewed DAG change, and reusable compilation must not be implemented
inside the CLI or VM. Introduce the real compiler package and reviewed edges at M2; no package or
architecture.json node is added for a documentation-only proposal. vba-ir stays dependent only on
diagnostics/values. VM translates IR-owned foreign descriptors to vba-foreign.

## Deferred extensions

IR 1 excludes arrays/records, global/static storage, optional/named direct VBA binding, builtins,
error-handler/Resume instructions, classes and callbacks. Later IR versions must specify their
instruction/exception/source-map contracts and tests. M4 still owns deep semantics, and M5 still
owns freezing the cross-fork protocol. No compatibility or oracle observation is established here.

M1 lexer/parser work is next, preserving the statement identities/spans required by this contract;
M2 and M3 implement its verifier/lowering and execution/debugger acceptance gates respectively.
