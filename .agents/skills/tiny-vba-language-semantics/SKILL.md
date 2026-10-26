---
name: tiny-vba-language-semantics
description: Implement or review VBA syntax, binding, values, coercion, bytecode and runtime behaviour in tiny-vba. Use for language correctness and execution changes rather than Office object-model implementation.
---

# VBA language semantics

Read [architecture](../../../docs/architecture.md) and the relevant sections of the
[interpreter specification](../../../docs/vba-interpreter-rust-implementation-spec.md).
Use MS-VBAL and documented, profile-tagged observations for semantics; distinguish a proposed
expectation from a measured Microsoft oracle result.

M0 foundations are in [ADR 0003](../../../docs/decisions/0003-m0-source-scenarios-foreign.md):
checked UTF-8 source spans, caller-assigned source/statement identities, UTF-16 value strings,
scalar states and bounded owner-scoped locations. These are not parser/coercion/VM implementations.
Reuse these types rather than introducing a second value/source model.

The adopted [IR 1 specification](../../../docs/bytecode-vm-specification.md) defines exact stack effects,
canonical direct ByRef, empty-stack checkpoint blocks, verifier and version/feature refusal rules.
Use shared runtime semantics for operators/conversions; IR verification checks shape, not full VBA types.
Later instructions need an explicit IR-version/contract update. The specification itself is not support.

## Preserve the language model

- Preserve source spelling while resolving identifiers case-insensitively. Retain module/source
  identity, spans and individual statement IDs, including multiple statements on one source line.
- Keep Empty, Null, Missing, Nothing and CVErr distinct. Preserve numeric widths, Currency/Date,
  array element types, dimensions and lower bounds; do not reduce values to f64 or JSON.
- Centralize coercion, operators, argument binding, ByRef locations and error state. Use the same
  rules across execution and debugger evaluation. A language change must not create a parallel
  conversion implementation inside UI, DAP or a host bridge.
- Verify bytecode before execution. Use explicit frames, instruction position and per-frame error
  state so execution can pause at statement checkpoints and resume without replay.
- Inject time, randomness, locale and prompts. Bound input allocations, parsing/nesting and VM work;
  return structured diagnostics for unsupported constructs and malformed input.
- Implement generic foreign invocation through vba-foreign; Excel/Word names and defaults stay
  in the application's Office layer. Never add a workbook store to complete a language example.

Implement the smallest semantic slice with cases checking types, values, exact error numbers,
side effects and unsupported behaviour. Include same-line source mapping and pause/resume cases
when lowering or control flow changes. Put integration tests under the owning crate's tests/.
Use the conformance skill for external fixtures or oracle claims, and debugger skill for stepping
or inspection changes. Preserve native/WASM execution paths and report the exact supported subset.

## Implemented M1 frontend

Read [frontend](../../../docs/frontend.md) and [ADR 0005](../../../docs/decisions/0005-m1-frontend.md).
Reuse vba-syntax's owned source/tokens/AST and distinct StatementSites. A recovered tree with diagnostics
is never valid input to binding. Apply nodes remain unresolved; retain parentheses, call style,
Let/Set and omitted/named/optional syntax. M1 parsing does not make those IR 1 capabilities.
M2 implements the documented subset of binding/Option Explicit/types/call legality. No execution results change from parsing.

## Implemented M2 contract

Read [compiler](../../../docs/compiler.md) and [ADR 0006](../../../docs/decisions/0006-m2-binding-and-verification.md).
Reuse vba-values ScalarKind, vba-semantics bound symbols and vba-compiler lowering. The compiler always
passes vba-ir verification; only vba-ir constructs VerifiedProgram. Keep IR independent of binding and
VM independent of compilation/debug/UI. M2 ir1-structural capabilities describe verification shapes,
not runtime support. Actual execution must check the M3 runtime matrix. Preserve exact constant tags,
ordered slots/sites, complete nonexecutable statement metadata and explicit unsupported diagnostics.

## Implemented M3 contract

Read [runtime](../../../docs/runtime.md) and [ADR 0007](../../../docs/decisions/0007-m3-execution-and-debugger.md).
The m3-core-v1 runtime is intentionally narrower than structural IR. Preserve explicit frame/alias
storage, one-visit checkpoint permits, saved fault PCs, generation checks and nonreplayed continuations.
Use vba-runtime for operations/edits. VM host dispatch must use checked precommit return/writeback
validation and explicit per-site permissions; foreign aliases remain unsupported. Run cargo xtask ci,
wasm and wasm-run (native plus actual WASI execution). Runtime example results do not mark unrun M0
or external scenario assertions PASS. Graphical UI, DAP and VBA error-handler semantics remain later work.

[ADR 0008](../../../docs/decisions/0008-vm-instance-isolation.md) distinguishes the caller's routing
label from the non-reused SessionInstance. Key handles, locations, breakpoints and stepping by the
engine instance; caller-label/frame/generation equality alone cannot identify an execution. Include
the replacement-session regression in native/WASI evidence when changing these identities.

[ADR 0010](../../../docs/decisions/0010-m4-profile-and-verification-hardening.md) publishes
m4-scalars-v1; do not silently expand m3-core-v1. Keep failure classifications and nonzero
CRASH/TIMEOUT/MISMATCH exits intact, compare exact payloads, and check narrowing bounds before casts.
Confirm xtask's logged workspace matches the intended checkout; shared binaries may originate elsewhere.
