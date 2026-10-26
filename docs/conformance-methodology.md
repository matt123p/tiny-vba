# Conformance methodology

Status: M0 fixture validation implemented, 2026-10-26. Candidate source inventory and
[schema 1](scenario-format.md) validation are implemented; scenario execution is not.
Twenty independently authored cases carry typed, reasoned expectations and remain NOT_RUN.
CLI validation reports VALID_FIXTURE, never a semantic PASS. External corpus entries are unchanged.

Separate syntax/binding, pure language runtime, generic foreign calls, project decoding and debugger tests.
Office model, real document engine, file round-trip and host trust checks belong to the forks.
Use deterministic time/random/locale/prompts; verify types as well as values, exact errors and side effects.

A future external/oracle schema extends the M0 envelope to record source hashes,
requirements, capabilities, setup, target Office product/build/OS/locale and independently justified expected
values/types/errors/output. Arrays include element kind, dimensions and lower bounds.
A specimen's availability does not establish expected behavior. Initial corpus entries remain NOT_RUN,
capabilities unclassified, profile unverified and expected behavior not curated.

Use explicit PASS/MISMATCH/UNSUPPORTED/BLOCKED_BY_POLICY/INVALID_FIXTURE/CRASH/TIMEOUT/NOT_RUN results.
Do not count absent execution as a pass. Never silently update oracle goldens. Keep ReflexGrid Mac data
separate from the intended Windows profile until evidence supports portability.
Keep malformed/deep/huge/budget tests and minimized regressions alongside implemented APIs.
