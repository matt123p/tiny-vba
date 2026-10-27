# Compatibility matrix

Updated 2026-10-27. Stage: M4 external acceptance complete; M5 native host contract and M6 source decoding implemented. **The documented m5-host-v1 consumer, including M3 control, M4 scalar widths and IR 4 host roots, has execution evidence; the 44 authored and 153 reviewed fixtures remain asserted 197 PASS / 0 UNSUPPORTED.**

| Dimension | Status | Evidence / next gate |
|---|---|---|
| Workspace and crate boundaries | Implemented setup | cargo xtask layers |
| CLI parse/check/compile/disasm/run, help/version and explicit unsupported errors | Implemented M3 | apps/vba-cli/tests/cli.rs |
| External source pins and candidate hashes | Implemented setup | strict corpus integrity check; not behavioral validation |
| Source IDs/spans/statements and diagnostic attachment | Implemented M0 | vba-diagnostics/tests/source.rs and vba-syntax/tests/frontend.rs; same-line/control statement mapping |
| Typed scalar states/UTF-16 strings and bounded locations | Implemented M0 foundations | vba-values/tests/locations.rs; shared M3 runtime rules live in vba-runtime |
| Generic foreign ownership/error/wait prototype | Implemented M0 | vba-testkit/tests/round_trip.rs and M3 VM host tests; no Office bridge |
| Scenario validation / 20 independent fixtures | Implemented M0 | vba-conformance/tests/scenarios.rs and CLI; validation alone reports NOT_RUN; assertion results are separate |
| Typed scenario assertion runner (M0 fixtures) | Implemented M4 first slice | `conformance run`: 20 PASS / 0 UNSUPPORTED asserted in CLI tests; [ADR 0009](decisions/0009-m4-runner-and-numeric-widths.md); external corpora stay NOT_RUN |
| Bounded source-preserving lexer/parser | Implemented M1 subset | vba-syntax/tests/frontend.rs; [grammar matrix](frontend.md) |
| External grammar evaluation | Partial: five reviewed files | [report](m1-corpus-report.md); two unsupported, three intentionally broken rejected |
| Scalar module/procedure binding and Option Explicit | Implemented M2 subset | vba-semantics/tests/binding.rs; [exact scope/type/call boundaries](compiler.md) |
| Required positional ByVal / exact local ByRef binding | Implemented M2 subset | vba-compiler/tests/compiler.rs and vba-vm/tests/execution.rs; canonical alias writes/edits proven |
| Checkpoint IR lowering / deterministic source maps | Implemented M2 subset | Answer golden, nested calls, same-line statements, If/For and unreachable-code tests |
| IR 1 verifier / disassembly / capabilities / budgets | Implemented M2 structural contract | vba-ir/tests/verification.rs; all instruction shapes, malformed tables/CFG/stacks and limits |
| Scalar runtime matrix | Implemented M3 subset, extended M4 | [Exact supported combinations](runtime.md), vba-runtime/tests/scalars.rs; `/` and Double widths added at M4 with error numbers 6/11 |
| Gate-root isolation / M4 scalar bounds and report fidelity | Implemented M4 hardening | cross-checkout xtask regression; numeric boundary, failure status and UTF-16/float-bit tests; ADR 0010 |
| Coercion/arrays/builtins/On Error/state | Implemented M4 reviewed profile | [M4 exact scope and IR2](m4-language.md), 24 additional PASS fixtures, numeric/handler/array/state regressions; no oracle claim |
| Bytecode/VM implementation specification | IR and VM subset implemented | IR 1, ADR 0004/0006/0007 and M2/M3 tests |
| Bytecode execution / generic host continuations | Implemented M3 subset | vba-vm/tests/execution.rs and host.rs; permissions, typed patches, wait/cancel, no replay |
| Versioned native host protocol and source imports (M5) | Implemented M5 contract | `vba_foreign::check_protocol` refuses foreign protocol versions before dispatch (crates/vba-foreign/tests/protocol.rs); `Import`/`validate_imports` bound names to hidden IR 4 host-root slots (crates/vba-compiler/tests/compiler.rs); missing/mismatched/stale roots refuse launch or surface host error 91 without a second write (crates/vba-vm/tests/host.rs, tests/embedding.rs); [ADR 0013](decisions/0013-m5-host-contract.md) |
| Reusable bridge driver for forks (M5) | Implemented generic proof | vba-testkit `run_to_boundary`/`statement`; vba-vm/tests/embedding.rs proves break/edit/resume over an imported root with exactly one dispatch. Real workbook FillCells, undo/recalc and permissions stay fork-owned |
| Breakpoints, stepping, local inspect/edit/resume | Implemented M3 headless API | vba-vm execution tests with vba-debug controller; no graphical UI/DAP |
| Native / WASM execution | Implemented M3 smoke | cargo xtask wasm-run; identical typed results, pause/edit/host counts and cancellation |
| Replacement-session isolation | Implemented M3 correction | recycled-client-ID regression in VM tests and native/WASI smoke; [ADR 0008](decisions/0008-vm-instance-isolation.md) |
| MS-OVBA source decoding | Implemented M6 bounded profile | [Import contract](project-import.md), [ADR 0014](decisions/0014-m6-project-decoding.md), 14 native/WASI tests; controlled authored projects only; package/trust/preservation gates stay fork-owned |
| Oracle comparison / executable scenario runner | M4 runner implemented for self-authored fixtures; oracle comparison not implemented | `conformance run` (20 PASS / 0 UNSUPPORTED); M9 owns oracle goldens |
| Fuzzing | Not implemented | APIs required before targets |
| Excel/Word APIs and OOXML round-trip | External ownership | Gridcraft/Wordcraft forks |
| COM, DLLs, UserForms, P-code and binary source writing | Deferred / outside MVP | Separate design required |

M3 execution evidence is restricted to its explicit tests/examples and runtime matrix. It does not establish
workbook/Office compatibility, browser UI behavior or external corpus pass rates; self-authored
M0 assertions are 20 PASS / 0 UNSUPPORTED and external corpus execution remains NOT_RUN.

## Current M4 external acceptance

[The acceptance report](m4-oracle-report.md) supersedes the earlier authored-only status: 44
authored scenarios and 153 reviewed ReflexGrid cases PASS. The latter carry upstream-reported
Microsoft Excel Mac evidence with exact source hashes and raw results, not a new Office run.
Whole-array/ParamArray/Option Base/named-cell argument tests and IR 3 version/metadata refusal
checks pass. Unselected external cases retain NOT_RUN; future oracle acquisition stays in M9.
