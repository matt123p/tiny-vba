# 0009 — M4 typed scenario assertion runner and real-division widths

Date: 2026-10-26. Status: adopted and implemented for the first M4 slice.

Amended by [ADR 0010](0010-m4-profile-and-verification-hardening.md): the current consumer is
m4-scalars-v1, failures exit nonzero, destination narrowing is range-checked, and validation roots
are resolved at runtime. Evidence below records the original slice; current authoritative checks
are recorded in progress under the hardening entry.

## Decision

Start M4 with the two tasks the implementation plan sequences first: a typed scenario assertion
runner and a separately tested scalar conversion/error matrix.

The runner lives in `vba-conformance` (typed `Expected::judge_value`/`judge_error`, deterministic
`describe_value`) and `vba-cli` (`conformance run <scenario>` orchestration: parse manifest,
compile the embedded body with the VM's real capability set, execute host-free, map the outcome to
a Status, print `RESULT <id>: <STATUS> (<detail>)`). MISMATCH exits 1 so a regression cannot hide
behind a success exit; PASS/UNSUPPORTED and the other truth classes exit 0; malformed fixtures and
usage exit 2. `conformance validate` keeps reporting VALID_FIXTURE/NOT_RUN and never executes.
External corpora stay NOT_RUN: this runner executes only the 20 self-authored M0 fixtures.

The numeric extension enables real division and Double widths under the existing IR 1 `Binary`
and `Convert` instructions through an explicit `m3-core-v1` capability update, per the IR 1 rule
that a tested scalar combination can be enabled by capability/profile without an IR version bump:
`/` accepts Integer/Long/Double operands, always produces Double, reports error 11 for a zero
divisor and error 6 for a non-finite result (VBA raises overflow there); Add/Subtract/Multiply
promote to Double only when an operand is Double; `Dim x As Double` and assignment of
Integer/Long/Double into Double slots are exact widenings; narrowing Double into Integer/Long
rounds half to even and reports overflow 6 out of range. Decimal/exponent literals (`1.5`, `1e2`)
bind as exact finite Double constants; radix and suffixed literal forms stay unsupported. The
integer operator matrix, including its overflow-6 and division-11 behavior, is unchanged.

## Consequences

The 20-fixture inventory moves from 20 NOT_RUN to 17 PASS plus 3 explicit UNSUPPORTED (cverr:
builtins; named: named/omitted direct arguments; optional-missing: optional parameters/IsMissing).
Those remain unsupported by design, not silently failed. The prior M3 CLI negative case
`Answer=1/2` is now correct behavior (Double 0.5 narrowed to Long 0), so the unsupported-operator
case uses `^`, and the overflow case uses Integer arithmetic that still reaches error 6.

No dependency graph change: vba-cli already depended on conformance, compiler, ir and vm;
vba-conformance already depended on values. The scenario schema is unchanged; the runner reads
the existing `expected` field. On Error/Resume/Err/Erl still requires the handler-capable IR
contract before implementation (plan step 2), and arrays, optional/named direct-call binding,
builtins, Currency/Date/Byte/LongLong/Single arithmetic remain later M4 work.

## Evidence

`cargo xtask ci` (81 integration tests, fmt, strict Clippy, layers), `cargo test --workspace
--release --locked`, `cargo xtask wasm` and `cargo xtask wasm-run` (native and WASI smoke) pass.
New coverage: runtime division/width matrix, VM end-to-end Double execution and narrowing, CLI
17-PASS/3-UNSUPPORTED boundary, truthful MISMATCH exit code, and conformance judgment units.
