# 0010 — Versioned M4 scalars and trustworthy verification

Date: 2026-10-26. Status: adopted and implemented. Amends ADR 0009.

## Scalar contract

Publish `m4-scalars-v1` as the current execution target. ADR 0009 initially enabled Double
behavior under `m3-core-v1`; that name must not silently acquire a different operation matrix.
The new consumer rejects programs targeting the old M3 profile with VBA0022. Recompile source
against current VM capabilities to opt in. IR 1 instructions and operand shapes are unchanged;
this is a consumer-profile revision, not a new bytecode representation.

Double narrowing rounds half to even, then checks the actual Integer/Long destination range
before casting. Previously the Long path checked only i64 representability and then cast to i32,
allowing 2147483648.0 to wrap instead of producing error 6. Add negative Double unary operations
and mixed Integer/Long/Double comparisons; preserve floating signed-zero payloads in arithmetic.
Overflow remains 6, division by zero 11; unimplemented combinations stay Unsupported. These are
reasoned tests, not Microsoft oracle observations. The [local official type-conversion reference](../../references/VBA-Docs/Language/Concepts/Getting-Started/type-conversion-functions.md)
records destination ranges and ties-to-even rounding; boundary tests make this profile's exact
round-then-range behavior explicit.

## Scenario assertion contract

Only explicit unsupported diagnostics produce UNSUPPORTED. Syntax/binding/launch errors become
INVALID_FIXTURE; compile/runtime resource exhaustion becomes TIMEOUT; invalid IR, invariant and
unrecognized engine failures become CRASH. Known unimplemented intrinsic names are reported
explicitly as unsupported by the binder, preserving the reviewed CVErr case without treating
arbitrary unknown procedures as unsupported capabilities. Missing runtime error numbers cannot
be replaced by an invented zero.

PASS/UNSUPPORTED remain exit 0, MISMATCH/CRASH/TIMEOUT exit 1, INVALID_FIXTURE exit 2. Unknown
engine diagnostics fail closed. Exact typed outcome judgment compares float/Date bits and UTF-16
units; reports escape UTF-16 units so lone surrogates and line breaks cannot disappear or create
ambiguous result lines. Scenario schema 1 is unchanged. The 20 self-authored cases remain asserted
17 PASS / 3 UNSUPPORTED; external corpus execution remains NOT_RUN.

## Validation contract

A cached xtask executable must resolve the checkout from its invocation directory. Its previous
compiled-in manifest path could select an older checkout when target artifacts were shared, so a
successful command did not prove that the requested workspace was tested. Resolve the runtime
workspace, print it in the log and fail if no repository root is found. The cross-checkout regression
launches the cached binary from a different nested checkout and verifies that checkout's checker ran.
WASM execution also honors CARGO_TARGET_DIR instead of loading a potentially stale default artifact.

Previously recorded validation claims are historical; the authoritative checks after this fix are
89 tests in the current checkout in debug/release, strict Clippy/format/layers/corpus integrity,
wasm32-unknown-unknown library checks and native/WASI smoke in an isolated target directory. The
smoke adds Double(3.5), mixed-comparison Boolean(true) and overflow-6 assertions to M3's existing
pause/edit/host/cancel/isolation proof. No new remote platform, MSRV or Office run is claimed.

M4 remains in progress. Arrays, optional/named direct binding, intrinsics, persistent state and the
handler-capable IR contract/implementation still have their separate acceptance obligations.
