# Compatibility matrix

Updated 2026-10-26. Stage: M4 first slice complete. **The documented m3-core-v1 subset plus the M4 real-division/Double widths has execution evidence; the 20 self-authored fixtures are asserted 17 PASS / 3 UNSUPPORTED.**

| Dimension | Status | Evidence / next gate |
|---|---|---|
| Workspace and crate boundaries | Implemented setup | cargo xtask layers |
| CLI parse/check/compile/disasm/run, help/version and explicit unsupported errors | Implemented M3 | apps/vba-cli/tests/cli.rs |
| External source pins and candidate hashes | Implemented setup | strict corpus integrity check; not behavioral validation |
| Source IDs/spans/statements and diagnostic attachment | Implemented M0 | vba-diagnostics/tests/source.rs and vba-syntax/tests/frontend.rs; same-line/control statement mapping |
| Typed scalar states/UTF-16 strings and bounded locations | Implemented M0 foundations | vba-values/tests/locations.rs; shared M3 runtime rules live in vba-runtime |
| Generic foreign ownership/error/wait prototype | Implemented M0 | vba-testkit/tests/round_trip.rs and M3 VM host tests; no Office bridge |
| Scenario validation / 20 independent fixtures | Implemented M0 | vba-conformance/tests/scenarios.rs and CLI; all VBA cases NOT_RUN |
| Typed scenario assertion runner (M0 fixtures) | Implemented M4 first slice | `conformance run`: 17 PASS / 3 UNSUPPORTED asserted in CLI tests; [ADR 0009](decisions/0009-m4-runner-and-numeric-widths.md); external corpora stay NOT_RUN |
| Bounded source-preserving lexer/parser | Implemented M1 subset | vba-syntax/tests/frontend.rs; [grammar matrix](frontend.md) |
| External grammar evaluation | Partial: five reviewed files | [report](m1-corpus-report.md); two unsupported, three intentionally broken rejected |
| Scalar module/procedure binding and Option Explicit | Implemented M2 subset | vba-semantics/tests/binding.rs; [exact scope/type/call boundaries](compiler.md) |
| Required positional ByVal / exact local ByRef binding | Implemented M2 subset | vba-compiler/tests/compiler.rs and vba-vm/tests/execution.rs; canonical alias writes/edits proven |
| Checkpoint IR lowering / deterministic source maps | Implemented M2 subset | Answer golden, nested calls, same-line statements, If/For and unreachable-code tests |
| IR 1 verifier / disassembly / capabilities / budgets | Implemented M2 structural contract | vba-ir/tests/verification.rs; all instruction shapes, malformed tables/CFG/stacks and limits |
| Scalar runtime matrix | Implemented M3 subset, extended M4 | [Exact supported combinations](runtime.md), vba-runtime/tests/scalars.rs; `/` and Double widths added at M4 with error numbers 6/11 |
| Broader coercion/arrays/builtins/On Error handlers | Not implemented | Rest of M4; error handlers need a new handler-capable IR contract |
| Bytecode/VM implementation specification | IR and VM subset implemented | IR 1, ADR 0004/0006/0007 and M2/M3 tests |
| Bytecode execution / generic host continuations | Implemented M3 subset | vba-vm/tests/execution.rs and host.rs; permissions, typed patches, wait/cancel, no replay |
| Breakpoints, stepping, local inspect/edit/resume | Implemented M3 headless API | vba-vm execution tests with vba-debug controller; no graphical UI/DAP |
| Native / WASM execution | Implemented M3 smoke | cargo xtask wasm-run; identical typed results, pause/edit/host counts and cancellation |
| Replacement-session isolation | Implemented M3 correction | recycled-client-ID regression in VM tests and native/WASI smoke; [ADR 0008](decisions/0008-vm-instance-isolation.md) |
| MS-OVBA source decoding | Not implemented | M6 |
| Oracle comparison / executable scenario runner | M4 runner implemented for self-authored fixtures; oracle comparison not implemented | `conformance run` (17 PASS / 3 UNSUPPORTED); M9 owns oracle goldens |
| Fuzzing | Not implemented | APIs required before targets |
| Excel/Word APIs and OOXML round-trip | External ownership | Gridcraft/Wordcraft forks |
| COM, DLLs, UserForms, P-code and binary source writing | Deferred / outside MVP | Separate design required |

M3 execution evidence is restricted to its explicit tests/examples and runtime matrix. It does not establish
workbook/Office compatibility, browser UI behavior or corpus pass rates; M0/external manifests remain NOT_RUN.
