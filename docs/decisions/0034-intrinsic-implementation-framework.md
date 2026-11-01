# ADR 0034: Intrinsic implementation framework

Adopted 2026-11-01 for implementation under M6A, on owner direction to prepare
a concrete framework for Luna. This is an implementation design, not a claim
that its APIs, IR or runtime capabilities already exist.

Use [the M6A framework](../m6a-implementation-framework.md) as the task entry point.
Keep a single typed descriptor catalogue in vba-values and one intrinsic argument
binder. Preserve source-order evaluation with an explicit declaration-order call
plan, structural omissions, per-parameter value modes and module comparison.
Runtime family modules reuse central coercion; IDE code aids use the catalogue.

Preserve legacy Intrinsic instructions and v2/v3 capability contracts. Implement
an **IR 8 extended intrinsic call** gated by **intrinsics-m6a-v1** for the new
argument/context/variant metadata. Verify the complete mapping before execution.
Update the bytecode/runtime specifications and version refusal tests with the
implementation, not by pretending IR 8 is already supported in current docs.

Pure semantics remain in runtime; VM owns execution state and effectful host
continuations. Use explicit context/services with deterministic doubles, existing
host completion/cancellation machinery, bounded work and read-only inspection.
No new crate/DAG edge, ambient IO, unrestricted COM/DLL activation, worksheet
dispatch or alternate value/coercion model is authorized by this design.
Decimal/locale/code-page/calendar/target-width support needs real shared
prerequisites; approximation does not satisfy a required function row.

Implement small dependent packets with per-function contracts and discriminating
typed tests, retaining legacy evidence. M6A.8 still requires actual pinned fork
acceptance. Planning/handoff preparation adds no runtime code and does not publish
or update dependency pins. M6A acceptance remains required before M7; fresh Office
observations remain M8.5.
