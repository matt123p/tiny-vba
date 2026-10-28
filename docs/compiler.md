Current compiler additions are in [the M4 language contract](m4-language.md) and
[ADR 0012](decisions/0012-m4-oracle-acceptance.md): ir3-structural is the default target;
IR 1/2 remain supported. Array parameter/layout metadata, ParamArray packing and named reference
capture are checked before execution. The historical M2/M4-first-slice restrictions below are
superseded only by those explicitly documented additions.

New M1.1 additionally supports qualified declared-type syntax and explicit
`CompileOptions.references` / `bind_with_references` snapshots. Registered host
enum types bind to Long; typed reference constants resolve in expressions,
constant declarations, defaults and bounds. Source symbols retain precedence.
Missing/undefined types and references fail explicitly. Nominal host object types
remain unsupported; they are not aliases for Object. See
[the exact profile](m1-host-references.md) and
[ADR 0020](decisions/0020-explicit-host-reference-snapshots.md).

# M2 binding and verified bytecode

Updated for [M4 / IR 2](m4-language.md), adopted by [ADR 0011](decisions/0011-m4-language-extension.md).
The additions there supersede the historical M2 subset/refusals below. Default compilation now targets
ir3-structural, preserving IR 1 instruction shapes and emitting IR 2 for extension features.

Implemented 2026-10-26. `vba-semantics` binds the M1 AST; `vba-compiler` lowers it;
`vba-ir` owns IR 1, structural verification, opaque immutable `VerifiedProgram` and disassembly.
Compilation alone executes no VM or VBA runtime operation. M3 now provides the separate [runtime profile](runtime.md). The target `ir1-structural`
advertises the shapes of IR 1 operations for verification, not an executable compatibility profile.

```sh
cargo run -p vba-cli -- check examples/pure-vba/Arithmetic.bas
cargo run -p vba-cli -- compile examples/pure-vba/M2Control.bas --entry Answer
cargo run -p vba-cli -- disasm examples/pure-vba/Arithmetic.bas
cargo test -p vba-semantics -p vba-ir -p vba-compiler
```

`check` and `compile` both perform the full parse/bind/lower/verify pipeline and print
`VERIFIED ... (structural target; use run for execution)`. `disasm` prints the same verified program's diagnostic
text. These commands accept UTF-8 standard `.bas` modules and default to entry `Answer`;
`--entry <name>` selects another entry. Errors return exit 2, with source coordinates when
available. There is no bytecode file format, binary loader or persisted compile output.

Arithmetic's reviewed [golden](../tests/semantics/m2/answer.disasm) has Integer constants 6/7,
Long result slot 0, max_stack 3, and the specified instructions:

```text
0000 Checkpoint(SiteId(0))
0001 RefLocal(LocalSlot(0))
0002 Const(ConstantId(0))
0003 Const(ConstantId(1))
0004 Binary(Multiply)
0005 WriteRef(Value)
0006 Jump(Pc(7))
0007 Checkpoint(SiteId(1))
0008 Return
```

M3 now executes this example as Long 42. Neither VERIFIED nor the presence
of an operation in `Capabilities::ir1_structural()` claims runtime support.

## Binding boundary

| Construct | Implemented M2 behavior |
|---|---|
| Modules and procedures | Ordered standard modules, unique SourceIds, case-insensitive procedure names, forward/recursive lookup, local-module precedence and public cross-module calls; private procedures stay module-local |
| Scopes | Procedure-wide parameters, Function result, scalar Dim locals and literal Consts; local symbols shadow module constants; implicit Variant locals without Option Explicit |
| Option Explicit | Reject every unresolved variable with its exact identifier span; procedure calls must resolve regardless of this option |
| Declarations | All 12 ScalarKind descriptors, required ByVal/ByRef parameters, Function result types; omitted type means Variant; no coercion implementation is implied |
| Calls | Exact positional present arity, Sub/value distinction, ByVal expressions, exact-declared-kind local ByRef; Function results used as statements are dropped explicitly |
| Expressions | Decimal nonnegative Integer/Long literals (including % and & suffixes within range), strings, Boolean, Empty/Null/Nothing, local/constant names, parentheses, unary minus/Not and the IR 1 binary operator tags |
| Assignment | Local/result value assignment and explicit Set mode; Set targets must be Object/Variant; dynamic assignment/conversion rules remain runtime work |
| Control | If/ElseIf/Else including inline forms, Exit in a matching context, Stop, numeric For with Integer/Long counter and positive integral literal step (default 1); matching Next name |
| Constants | Literal untyped module/procedure Const only; exact tags retained; cross-module Const lookup, typed Const and computed constant expressions fail explicitly |

Rejected M2 forms include persistent module variables, classes/properties, default-member
binding, arrays, optional/default/named/omitted direct VBA arguments, parenthesized direct arguments,
nonlocal ByRef, identifier type suffix binding, unknown types, unary plus, Is/Like, nondecimal/wider/
floating/Currency literals, Option Compare Text, computed/nonpositive/nonintegral For steps, and
all syntax outside M1. All procedures are bound, even if the selected entry never calls them.
A ByRef entry is rejected; a ByVal entry records its required launch kinds but receives no values
until M3. Dynamic operand combinations are deliberately not inferred from structural verification.

The Rust library `compile_modules` accepts ordered `ParseResult`s with distinct source identities.
Only a syntax-valid result can enter binding. The CLI currently takes one source file. Source
snapshots, revision, spelling, declaration spans and statement identities are retained; no paths
are reopened by the compiler. Metadata identifiers have the IR 256-byte ceiling.

## Lowering and source mapping

Parameters occupy the first slots, followed by the hidden Function result, declared locals in
source order, implicit locals in first bound-use order, then hidden compiler scratch slots.
Procedure IDs follow module/declaration order. CFG blocks are built with symbolic targets and
flattened by a deterministic iterative traversal (true edge first), so PCs need not follow source
line order. Site IDs follow emitted PCs. Constants are interned by exact tag/payload and reordered
by first emitted use; equal Integer/Long payloads remain distinct. Unreachable blocks, their calls
and constants are removed; the complete nonexecutable statement table remains.

Every emitted block starts with Checkpoint and ends with Jump/Branch/Return. Calls inside an
expression share the enclosing checkpoint, preserve their operand prefix, and keep argument
emission order. Colon-separated statements have different StatementIds/sites. Dim/Const statements
have hidden source checkpoints; initial local values are a runtime storage obligation, not repeated
initialization on reaching Dim. Stop immediately precedes a Jump. End If and procedure endings
have explicit sites. Synthetic procedure exits use the End Sub/Function statement identity.

For captures start/limit/step in three hidden scratch slots, then initializes the counter. A hidden
first test avoids a duplicate immediate source stop; subsequent visible tests share the For
StatementId but have distinct sites. Next has a checkpoint before counter advance. Bounds are not
re-evaluated on the backedge. This is the restricted lowering contract; runtime loop conversion and
overflow cases must be tested before M3 execution support is declared.

`resume_next` records the next retained source-order statement site (None at procedure exit),
not an arbitrary next instruction. IR 1 has no Resume execution; complex branch/loop Resume
semantics require the later versioned handler contract. Compiler output always passes the same
whole-program verifier used for untrusted hand-built IR.

## Verifier and budgets

Verification checks every procedure, source/revision identity, constant payload, signature/slot
layout, launch signature, instruction operand, declared feature/op/conversion capability, checkpoint
ownership, statement-contained span, unique site PC, call ownership and exact ByRef reference kind.
Every block has empty entry/exit stacks (Branch consumes its sole Value). V and Location never
substitute for one another. All branch targets are checkpoints. Reachability uses an iterative
worklist; a reachable self-loop is valid, and unreachable blocks/falloff are rejected. Maximum stack
must exactly match the computed maximum. Foreign descriptors verify Call/Get/Let/Set, Value/
Reference/Omitted modes, names and ordering, without implementing source-level host binding or dispatch.

`verify(Program, &Capabilities, &VerifyLimits)` is the sole VerifiedProgram constructor. Its accessor
returns only `&Program`; mutable/cloned unverified tables must be verified again. `measure_stack`
checks other invariants and reports maxima for the compiler; it never returns a verified program.
Unknown IR versions/features/profiles/capabilities return VBA0022, malformed IR VBA0020, verifier
budget exhaustion VBA0021. Binder errors use VBA0110/0111; lowering errors VBA0120/0121;
unsupported language forms retain VBA0001. Errors do not return a partially executable program.

Defaults follow the [IR 1 limits](bytecode-vm-specification.md): 256 sources/16 MiB source text,
4,096 procedures, 65,536 instructions per procedure/1,048,576 total, 65,536 constants/16 MiB
constant payload, 4,096 slots and stack entries, and 256 call arguments. Each procedure's source,
checkpoint and call tables is independently capped at the total instruction limit. Verification
charges 256 logical bytes per table cell plus source/name/constant buffers, with 64 MiB payload
and 8,388,608 scanned entries as separate ceilings. Limits can be lowered by callers.

Binding preflights the AST iteratively before cloning it, bounds depth at 64, charges 256 bytes
per node, 64 per retained token, source/string buffers and constant substitutions, with 64 MiB
logical payload and 1,048,576 visits. Constant expansion is charged before cloning. Compiler
emission charges 256 bytes per instruction/metadata cell and three copies of interned constant
payload (pool/key/canonicalization), with a separate 64 MiB ceiling; it checks instruction/slot/
constant counts before growth. Disassembly has a separate 64 MiB output ceiling and fallible
String reservations. Vectors use fallible reservations at input-derived growth points; bounded
standard-library tree/string/AST clones may still rely on Rust's allocator. These are conservative
logical accounting limits per stage, not an RSS guarantee. No runtime budget is consumed by M2.

## Evidence and remaining gates

`vba-semantics/tests/binding.rs`, `vba-ir/tests/verification.rs`, `vba-compiler/tests/compiler.rs`
and CLI integration tests cover positive/negative binding, exact errors, slot/constant/site order,
nested/same-line calls, If/For/ByRef lowering, the full Answer golden, version/feature rejection,
all IR 1 stack instruction shapes, malformed indexes/blocks/metadata/calls, all verifier budget
dimensions, exact UTF-16/float-bit disassembly and bounded constant amplification. Tests are
independent of corpus checkouts and Office. All 20 M0 scenario execution statuses stay NOT_RUN.
Native CI, library WASM compilation and the M3 native/WASI execution gate are required.

## M3 source extension

The binder/compiler now support generic member Call/Get/Let/Set on an explicitly declared Object
local or parameter. Calls and member applications use Call; plain reads use Get; member assignments
retain Let/Set. Source foreign arguments are ByVal or omitted with names preserved; foreign ByRef
snapshot descriptors are verified/tested separately. Nested/default/indexed property ambiguity and
implicit Office globals remain unsupported. These sites need explicit embedding permissions at
runtime. See [M3 contract](runtime.md); runtime operations never enter the compiler layer.
