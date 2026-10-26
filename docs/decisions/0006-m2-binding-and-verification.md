# 0006 — M2 binding, lowering and IR 1 structural target

Date: 2026-10-26. Status: adopted and implemented for M2.

Implement ADR 0004's layer split: vba-semantics owns ordered scopes/signatures/argument checks,
vba-compiler owns orchestration and lowering, vba-ir owns in-memory IR 1, verification and stable
text disassembly. Put ScalarKind in vba-values, shared by binding/IR and the future runtime.
Add semantics -> values, compiler -> diagnostics/syntax/semantics/IR/values and CLI -> compiler/IR
to the reviewed DAG. No IR -> semantics or VM -> compiler dependency is introduced.

The public IR uses distinct checked u32 index types and typed Rust instruction/descriptor enums;
unknown opcode tags are unrepresentable in-process. Unknown versions/features and consumer
capabilities fail verification. There is no serialized loader. Only the verifier constructs the
immutable VerifiedProgram; trusted compiler output is not exempt. A separate maximum-stack
measurement pass validates other structure before setting maxima, followed by full verification.

Expose `ir1-structural` as a verification target accepting all IR 1 instruction shapes. It is
not a runtime support profile; no VM functionality can be enabled by passing this target to a
future session. M3 must advertise/recheck its actual supported operation matrix at session creation.

Bind procedure-local scalars, literal constants, positional calls, required parameters, exact local
ByRef, If and restricted positive integral-step For. Reject persistent storage and deeper binding
forms explicitly. Keep source handling headless and corpus-independent. Use checkpoint blocks,
iterative CFG traversal/reachability and bounded work/payload accounting. Deduplicate constants by
exact tags/payload and assign final IDs in first emitted-use order. Retain source statement tables
when unreachable executable blocks are removed. Source-order resume_next is metadata only; no
Resume runtime behavior is enabled. The [compiler contract](../compiler.md) records the exact subset,
resource charges, API, standalone examples and tests.

CLI `check`, `compile` and `disasm` compile and verify UTF-8 .bas input; check/compile print a
verification summary and disasm prints diagnostic text. They do not create a binary file or
execute a procedure. Runtime/coercion/host/stepping, canonical alias storage and returning Long 42
remain M3/M4 obligations. M0/external execution results remain NOT_RUN.
