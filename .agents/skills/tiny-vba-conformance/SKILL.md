---
name: tiny-vba-conformance
description: Create or review tiny-vba test scenarios, external corpus use, oracle evidence, compatibility reports and release claims. Use for fixture adoption, semantic evidence or measuring implemented behaviour.
---

# Conformance and evidence

Read [methodology](../../../docs/conformance-methodology.md),
[compatibility matrix](../../../docs/compatibility-matrix.md) and
[current progress](../../../docs/implementation-progress.md). Before external corpus work, read
[corpus policy](../../../tests/corpora/README.md), the
[source lock](../../../references/sources.lock.json) and
[inventory](../../../tests/corpora/inventory.json).

M0 validates [schema 1](../../../docs/scenario-format.md) self-authored fixtures only.
`vba-cli conformance validate` reports VALID_FIXTURE and NOT_RUN; it does not parse/execute VBA.
External/oracle metadata needs a reviewed schema extension. Keep the 20 fixture identities unique
and matching filenames; update owning integration tests when changing that suite.

## Select evidence deliberately

Keep syntax/binding, language runtime, generic foreign calls, source import, debugger and UI evidence
separate. Office model, real workbook mutation, package round trips and host trust need fork tests.
A fixture pin/hash proves integrity, not permission to execute or behavioural compatibility.

Prefer independently authored deterministic cases. For external material establish provenance/licence,
required capabilities, host profile and justified expected outcomes before adoption. Never edit upstream
submodules or run downloaded scripts/macros merely to obtain a result. Do not reuse other interpreters'
implementation code. Observe Office only as a permitted black box; never inspect proprietary binaries.

Record source/version/hash, scenario identity, exact Office build/OS/locale and expected types, values,
error numbers, output and side effects. Preserve array bounds and distinguish formulas from results.
Keep Mac and Windows profiles separate. Document inference; never invent an oracle observation or
silently rewrite a golden to match the engine.

## Report what actually ran

Use PASS, MISMATCH, UNSUPPORTED, BLOCKED_BY_POLICY, INVALID_FIXTURE, CRASH, TIMEOUT and NOT_RUN.
Do not turn unavailable or unsupported execution into a pass. Report suite/version and denominators;
compilation, candidate file counts and CLI help tests establish no executable VBA semantics.

Put Rust integration tests under the owning crate's tests/; root tests/ holds shared data. Cover
negative/error/budget paths and minimized crash regressions where relevant. Execution changes need
native/WASM runtime evidence when available; UI changes need rendered/interaction evidence as well.
Run the appropriate checks and repository CI/WASM gates; corpus integrity is a separate check.
Update compatibility/progress from actual outcomes, including unavailable checks and the next smallest
task. Never claim full Office compatibility from a narrow generic-host or visual demonstration.

M1's [reviewed grammar allowlist](../../../docs/m1-corpus-report.md) permits syntax-only ingestion
of five directly authored tree-sitter-vba examples, not execution or third-party subtree adoption.
All 20 M0 bodies parse, but execution statuses and the corpus inventory remain NOT_RUN.

M2 VERIFIED/diagnostic disassembly is structural compilation evidence only; see
[compiler tests and boundaries](../../../docs/compiler.md). Do not mark any execution scenario PASS
from compilation. All M0/external execution statuses remain NOT_RUN until a real VM runs assertions.
