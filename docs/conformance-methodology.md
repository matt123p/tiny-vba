# Conformance methodology

Status: planned harness, 2026-10-26. Candidate source inventory is implemented; scenario execution is not.

Separate syntax/binding, pure language runtime, generic foreign calls, project decoding and debugger tests.
Office model, real document engine, file round-trip and host trust checks belong to the forks.
Use deterministic time/random/locale/prompts; verify types as well as values, exact errors and side effects.

A future versioned scenario records ID, source/module/entrypoint, provenance/licence/source hash,
requirements, capabilities, setup, target Office product/build/OS/locale and independently justified expected
values/types/errors/output. Arrays include element kind, dimensions and lower bounds.
A specimen's availability does not establish expected behavior. Initial corpus entries remain NOT_RUN,
capabilities unclassified, profile unverified and expected behavior not curated.

Use explicit PASS/MISMATCH/UNSUPPORTED/BLOCKED_BY_POLICY/INVALID_FIXTURE/CRASH/TIMEOUT/NOT_RUN results.
Do not count absent execution as a pass. Never silently update oracle goldens. Keep ReflexGrid Mac data
separate from the intended Windows profile until evidence supports portability.
Keep malformed/deep/huge/budget tests and minimized regressions alongside implemented APIs.
