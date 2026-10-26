# Conformance methodology

Status: M4 external acceptance implemented, 2026-10-26. Schema 1 preserves authored, unmeasured
expectations. Schema 2 records reviewed external source hashes, upstream build/OS/date/locale
availability and raw oracle results, which must agree with typed expectations. The selected
inventory is 44 authored PASS and 153 inherited Microsoft Excel/ReflexGrid PASS; see
[the report](m4-oracle-report.md). These are local native/WASI comparisons to upstream-reported
observations, not a fresh Office run. Unselected external cases remain NOT_RUN.

Separate syntax/binding, pure language runtime, generic foreign calls, project decoding and debugger tests.
Office model, real document engine, file round-trip and host trust checks belong to the forks.
Use deterministic time/random/locale/prompts; verify types as well as values, exact errors and side effects.

Schema 2 extends the M0 envelope to record source hashes,
requirements, capabilities, setup, target Office product/build/OS/locale and independently justified expected
values/types/errors/output. Arrays include element kind, dimensions and lower bounds.
A specimen's availability does not establish expected behavior. Initial corpus entries remain NOT_RUN,
capabilities unclassified, profile unverified and expected behavior not curated.

Use explicit PASS/MISMATCH/UNSUPPORTED/BLOCKED_BY_POLICY/INVALID_FIXTURE/CRASH/TIMEOUT/NOT_RUN results.
Do not count absent execution as a pass. Never silently update oracle goldens. Keep ReflexGrid Mac data
separate from the intended Windows profile until evidence supports portability.
Keep malformed/deep/huge/budget tests and minimized regressions alongside implemented APIs.

M1 adds a separate syntax layer: all 20 self-authored scenario bodies parse under its documented
subset, without changing NOT_RUN runtime statuses. Five reviewed external grammar files were read
by an opt-in Rust allowlist: two unsupported modules and three intentionally malformed inputs. See
[the review, provenance and exact results](m1-corpus-report.md). No upstream code was executed.

M2 [compiler and IR evidence](compiler.md) adds verified compilation, deterministic disassembly
and malformed-program rejection. VERIFIED is a structural result; all M0/external execution
statuses remain NOT_RUN. The ir1-structural capability set supplies no VM or Office runtime.

M3 adds actual native/WASI execution evidence for the explicit runtime examples and tests in
[runtime](runtime.md). Do not transfer those results to fixture manifests or external specimens.

M4 adds the typed scenario assertion runner
([ADR 0009](decisions/0009-m4-runner-and-numeric-widths.md)): `conformance run` executes each
self-authored M0 fixture and compares its typed expectation. The asserted inventory is 17 PASS and
3 UNSUPPORTED (cverr, named, optional-missing); this measures the implemented subset, not Office
behavior. External/oracle specimens stay NOT_RUN. The same slice enables real division and Double
widths with exact error numbers 6/11 across widths.

The [M4 hardening contract](decisions/0010-m4-profile-and-verification-hardening.md) requires fail-closed
engine classification, nonzero failure exits, bit-exact floating expectations and lossless UTF-16
report text. Verification logs must identify the actual invoking checkout; cached build paths cannot
establish that the current source was tested. Historical m4-scalars-v1 programs opt into the expanded
numeric matrix explicitly; legacy M3 execution targets must be recompiled.

[ADR 0012](decisions/0012-m4-oracle-acceptance.md) adopts the exact external selection and
source-preserving Err-state probe. Never broaden its PASS denominator or reuse Mac observations
as Windows observations. The evidence checker validates raw data, hashes, adaptations and the
native/WASI bundle; schema parsing alone is not evidence verification. Existing first-slice
17/3 figures above are historical; current acceptance is the 44+153 inventory reported at the top.
