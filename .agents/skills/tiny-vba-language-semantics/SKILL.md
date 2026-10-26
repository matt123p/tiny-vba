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
