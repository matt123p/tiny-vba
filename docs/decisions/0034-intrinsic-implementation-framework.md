# ADR 0034: Intrinsic implementation framework

Adopted 2026-11-01 for implementation under M6A, on owner direction to prepare
a concrete framework for Luna. **Implemented and verified under M6A.** The
[acceptance record](../m6a-acceptance.md) gives the resulting profile and evidence.

Use [the M6A framework](../m6a-implementation-framework.md) as the task entry point.
Keep a single typed descriptor catalogue in vba-values and one intrinsic argument
binder. Preserve source-order evaluation with an explicit declaration-order call
plan, structural omissions, per-parameter value modes and module comparison.
Runtime family modules reuse central coercion; IDE code aids use the catalogue.

Preserve legacy Intrinsic instructions and v2/v3 capability contracts. The
**IR 8 extended intrinsic call**, gated by **intrinsics-m6a-v1**, carries new
argument/context/variant metadata. Verify the complete mapping before execution.
Bytecode/runtime specifications and version refusal tests were updated with the
implementation. Do not imply this capability exists in older pinned revisions.

Pure semantics remain in runtime; VM owns execution state and effectful host
continuations. Use explicit context/services with deterministic doubles, existing
host completion/cancellation machinery, bounded work and read-only inspection.
No new crate/DAG edge, ambient IO, unrestricted COM/DLL activation, worksheet
dispatch or alternate value/coercion model is authorized by this design.
Decimal/locale/code-page/calendar/target-width support needs real shared
prerequisites; approximation does not satisfy a required function row.

The work landed in small dependent packets with per-function contracts and
discriminating typed tests, retaining legacy evidence. M6A.8 completed pinned
fork acceptance. This ADR records the implementation framework; future changes
still require the current repository gates. Fresh Office observations remain M8.5.
