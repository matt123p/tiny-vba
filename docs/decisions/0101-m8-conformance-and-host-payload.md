# ADR 0101 — M8 conformance and reusable host-call payload space

Date: 2026-11-08. Status: adopted implementation contract; fresh Office release
comparison remains NOT_RUN.

## Context and ownership

M7A is accepted for ADR 0100's common profile. Its original 61-artifact,
130-case integration gate was rerun successfully before these changes. M8 owns
expanded conformance, robustness, measurement and release evidence. The owner
confirmed that no Excel/Word installation is available and directed work to
continue without them. This authorizes local work; it does not manufacture
fresh observations or satisfy the existing M8.5 release gate.

The VM owns session accounting, foreign owns precommit reply validation, values
owns intrinsic result descriptors and runtime owns scalar results. Workbook
state, trust, persistence, recalculation and undo remain in GridCraft. No shipping
dependency edge, default budget, host permission or executable capability is added.

## Temporary host space and cumulative work

The former 64 KiB charge on every dispatch and poll represented temporary space,
not cumulative allocation work: the foreign session retains tokens and canonical
locations while waiting, not an unused reply buffer. Before each synchronous
dispatch/poll the VM now checks cumulative allocation plus 64 KiB scratch and
256 bytes per argument against the unchanged payload limit. Result/writeback
validation also includes that live scratch headroom. Returning, failing or waiting
ends that synchronous reservation; subsequent calls reuse it. `Counters` exposes
the largest scratch reservation as `host_payload_peak`.

Foreign ByRef snapshots charge their actual canonical value copies before dispatch;
argument packet metadata also remains cumulative allocation work. Repeated large
snapshots cannot evade the payload budget by reusing scratch. Actual copied
arguments, results, patches, strings and execution still consume
cumulative budgets. Call/poll/instruction limits remain independent. Oversized
arguments fail before host dispatch; oversized replies fail before VM patches
commit. Host effects already completed are not rolled back. Host errors charge
their descriptions. Intrinsic host operations use the same checked result path;
their old immediate-reply accounting omission is corrected. Wait/resume does not
replay an invocation. This is conservative logical accounting, not an RSS bound.

## Library corrections and independently fixed expectations

The additional inherited cases exposed two bugs. `VarType` now returns Long,
including arrays and Decimal. Its formal declaration returns `VbVarType`;
enum declarations use Long, and nine independent published observations agree.
The prose reference's word Integer does not override the formal enum declaration.
Runtime results and the values descriptor change together. `QBColor` now uses the
formal 16-entry numeric table; the earlier table incorrectly mixed bright and
dark colors. Old authored expectations are corrected from the independent
specification, without changing any inherited expectation.

Primary contracts consulted 2026-11-08:

- [MS-VBAL VarType declaration](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/2deec703-c2ef-4bf1-a246-1d51c60895da).
- [MS-VBAL enum declared types](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/aaee9b48-168f-4cfe-95d1-14c5cc427b69).
- [MS-VBAL QBColor numeric table](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/4b7087c6-20fc-4d90-8d83-730a0c6a4aad).

Only public contracts and permitted fixture data were consulted; no interpreter
implementation was imported. Fresh Office evidence remains separate.

## Evidence and reproducibility

The M8 selection adds 24 directly authored MIT ReflexGrid expression data entries
and their inherited Mac observations under the same data-only exception as ADR
0012. There are no helpers, external objects or environment calls in this finite
selection. The upstream source, raw result, report, adaptation and MIT notice are
pinned independently. No upstream Python or macro script runs. M4's original
153-case denominator is unchanged. Nine date-literal candidates and one
numeric-service candidate remain explicitly excluded/UNSUPPORTED; no literal
substitution or service simulation is used to turn them into passes.

Three Rust fuzz entrypoints use bounded stdin and deterministic mutation campaigns.
They run the actual parser, inert importer/decompressor and compiler-verified VM;
the VM has no host or ambient IO. These are mutation campaigns, not a claim of
coverage-guided or long-duration fuzzing. Eight benchmarks assert exact result
types/values on every sample and stable instruction counts, with two warmups,
ten measured samples and native/WASI timing distributions. Timings are reported,
not portable performance thresholds or Office comparisons.

GridCraft owns twenty original independent native XLSM workflows built in Rust.
They assert inert load/compile, denied execution before consent, exact cell values
and output types, retained inputs, native save/reopen, original project bytes,
undo and Reset. These are authored workbooks, not third-party or Office-produced
workbooks; their coverage does not imply real-world corpus coverage.

`cargo xtask m8`, `m8-wasm`, `m8-bench` and optional `m8-hosts` reproduce the
new checks. Normal native CI and WASI gates run the local campaigns/comparisons;
ordinary CI still requires no sibling checkout. The optional host gate applies
temporary local candidate dependencies and restores original lockfile bytes in
`finally`; shipping host pins are unchanged. Native and Node WASM host results
must identify that candidate tree separately from pinned-runtime receipts.

The original M7A selection/execution records remain immutable. When the shared
tree changes, its checker requires the reviewed M8 tree and warns that historical
receipts cannot be reused for the candidate. M8 records fresh execution and its
own input hashes. Integrity validation remains VALID_FIXTURE / NOT_RUN, never an
execution result. The release gate remains open until permitted fresh comparisons
satisfy M8.5. P9 stays after M8.
