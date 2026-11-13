# Testing and evidence

Use independent authored fixtures, reviewed inherited observations and fresh
Office comparisons as distinct evidence classes. Assert exact VBA types, bounds,
errors, side effects and event order. Compilation and structural verification are
not execution results; a selected but unexecuted case remains NOT_RUN.

## Repository gates

| Command | Purpose |
|---|---|
| `cargo xtask ci` | Native repository checks, formatting, strict Clippy and tests |
| `cargo xtask layers` | Reviewed crate dependency graph |
| `cargo xtask wasm` | Library compilation for wasm32-unknown-unknown |
| `cargo xtask wasm-run` | Actual native/WASI execution, including language/import/IDE suites |
| `cargo xtask ui` | Headless IDE/UI interactions and original rendered snapshots |
| `cargo xtask ui-native` | Real native window capture; requires a display or Xvfb |
| `cargo xtask web` | Browser UI packaging; see [embedding](ui/embedding.md) for browser tests |
| `cargo xtask m8` | Local expanded conformance and bounded hardening |
| `cargo xtask m8-bench` | Asserted native/WASI benchmark scenarios |
| `cargo xtask corpora` | External source pins, notices and specimen integrity |
| `cargo xtask m7`, `cargo xtask m7a`, `cargo xtask m8-hosts` | Optional reviewed real-host workflows; require the relevant sibling checkouts |

See [development](developer-guide.md) for tool prerequisites. Run focused owning
crate tests when changing a contract; include negative, resource-bound and
pause/resume regressions where behavior warrants them. Core builds/tests must
remain independent of application siblings and reference checkouts.

## Selections and recorded results

| Evidence | Source of truth | Interpretation |
|---|---|---|
| Authored language | [M0 fixtures](../tests/fixtures/m0), [M4 fixtures](../tests/fixtures/m4) | 44 selected authored scenarios; no Office observation implied |
| Inherited language | [M4 oracle fixtures](../tests/fixtures/m4-oracle), [provenance report](m4-oracle-report.md) | 153 reviewed inherited Excel Mac results; other platforms are not inferred |
| Expanded library | [M8 oracle selection](../tests/fixtures/m8-oracle/selection.json) | 24 additional selected inherited cases; exclusions retain their reasons |
| Original packages | [M7 selection](../tests/fixtures/m7/selection.json), [execution](../tests/fixtures/m7/execution.json) | Authored unchanged-document/package workflows in the real forks |
| Native forms | [M7A selection](../tests/fixtures/m7a/selection.json), [execution](../tests/fixtures/m7a/execution.json) | Accepted common native workflows and reviewed source/input identities |
| Local hardening/hosts | [M8 selection](../tests/fixtures/m8/selection.json), [execution](../tests/fixtures/m8/execution.json), [benchmarks](../tests/fixtures/m8/benchmarks.json) | Bounded fuzzing, asserted benchmarks and original workbook workflows; finite named suites |
| Fresh Office | [M8 Office selection](../tests/fixtures/m8-office/selection.json) | Eleven prepared recipes; observations remain NOT_RUN |

Receipts describe their recorded tree, dependencies and input hashes. A passing
historical receipt does not certify a changed tree. Preserve old receipts rather
than rewriting them to describe a new run. Benchmark timings are descriptive under
the recorded machine/toolchain/build conditions, not cross-machine speed claims.

Two documentation files are themselves hash-pinned M7A evidence inputs:
[common-profile snapshot](m7a4-common-profile.md) and [deferred-scope snapshot](p9-forms-compatibility.md).
They retain their exact historical bytes and milestone wording solely for receipt
integrity. Use [forms](forms.md) and [compatibility](compatibility-matrix.md) for
current documentation. Their historical outbound links describe the original tree.

## Corpus policy and outcomes

Read [the corpus policy](../tests/corpora/README.md),
[source lock](../references/sources.lock.json) and
[inventory](../tests/corpora/inventory.json) before using external material.
A source pin is not permission to execute it or evidence of compatibility.
Do not run downloaded scripts/macros, copy interpreter implementations or silently
change expected outputs. The reviewed MIT allowlist is the bounded execution exception.

Report PASS, MISMATCH, UNSUPPORTED, BLOCKED_BY_POLICY, INVALID_FIXTURE, CRASH,
TIMEOUT and NOT_RUN accurately. Keep raw results, build/OS/locale and source hashes.
Syntax-only ingestion does not change runtime status. See [scenario format](scenario-format.md),
[fuzzing](../fuzz/README.md) and [release capture](m8-office-comparison.md).

## Release comparison

Local common-form and expanded conformance acceptance does not satisfy fresh
Office comparison. The eleven-case capture template and validator are prepared,
but actual Excel/Word observations and provenance review remain **NOT_RUN**.
`python3 tools/check_m8_release.py --validate` validates preparation only;
without a completed reviewed bundle the release gate intentionally fails.
The full capture and typed-result protocol is in [release validation](m8-office-comparison.md).

## CI environments and candidate identity

Native CI runs independently on Linux, Windows and macOS (matrix fail-fast is
disabled). Linux rendering tests require the software Vulkan/EGL packages installed
by the workflow. The dedicated UI job retains browser/native capture coverage.
CFB resource/storage identities and runtime digest paths use `/` on every OS.

The inaccessible ReflexGrid download is replaced by four exact MIT oracle input
files under `references/oracle-sources/reflex-grid/`. Both inherited selections
still verify their original hashes; unretained upstream candidates remain NOT_RUN.
Other public corpora retain strict submodule pin/notice/specimen verification.

The original M8 selection and execution bytes are preserved under
`tests/fixtures/m8/baseline-2026-11-08/`. Current candidate receipts describe only
checks actually rerun after the portability change. Release readiness remains
NOT_RUN until all required local and fresh Office gates are satisfied.
