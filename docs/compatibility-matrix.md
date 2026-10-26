# Compatibility matrix

Updated 2026-10-26. Stage: M0–M2 subset complete. **No VBA feature is executable or runtime-verified.**

| Dimension | Status | Evidence / next gate |
|---|---|---|
| Workspace and crate boundaries | Implemented setup | cargo xtask layers |
| CLI help/version, parse/check/compile/disasm and explicit execution refusal | Implemented M2 | apps/vba-cli/tests/cli.rs |
| External source pins and candidate hashes | Implemented setup | strict corpus integrity check; not behavioral validation |
| Source IDs/spans/statements and diagnostic attachment | Implemented M0 | vba-diagnostics/tests/source.rs and vba-syntax/tests/frontend.rs; same-line/control statement mapping |
| Typed scalar states/UTF-16 strings and bounded locations | Implemented M0 foundations | vba-values/tests/locations.rs; no coercion/runtime |
| Generic foreign ownership/error/wait prototype | Implemented M0 | vba-testkit/tests/round_trip.rs; no VM or Office bridge |
| Scenario validation / 20 independent fixtures | Implemented M0 | vba-conformance/tests/scenarios.rs and CLI; all VBA cases NOT_RUN |
| Bounded source-preserving lexer/parser | Implemented M1 subset | vba-syntax/tests/frontend.rs; [grammar matrix](frontend.md) |
| External grammar evaluation | Partial: five reviewed files | [report](m1-corpus-report.md); two unsupported, three intentionally broken rejected |
| Scalar module/procedure binding and Option Explicit | Implemented M2 subset | vba-semantics/tests/binding.rs; [exact scope/type/call boundaries](compiler.md) |
| Required positional ByVal / exact local ByRef binding | Implemented M2 subset | vba-compiler/tests/compiler.rs; no alias execution yet |
| Checkpoint IR lowering / deterministic source maps | Implemented M2 subset | Answer golden, nested calls, same-line statements, If/For and unreachable-code tests |
| IR 1 verifier / disassembly / capabilities / budgets | Implemented M2 structural contract | vba-ir/tests/verification.rs; all instruction shapes, malformed tables/CFG/stacks and limits |
| VBA values/coercion/builtins | Not implemented | M3/M4 |
| Bytecode/VM implementation specification | M2 IR implemented; VM not implemented | IR 1, ADR 0004/0006 and M2 tests; no execution results |
| Bytecode execution / VM host wiring | Not implemented | M3; standalone protocol prototype only |
| Breakpoints, stepping and resume | Not implemented | M3 |
| MS-OVBA source decoding | Not implemented | M6 |
| Oracle comparison / executable scenario runner | Not implemented | M3/M9 |
| Fuzzing | Not implemented | APIs required before targets |
| Excel/Word APIs and OOXML round-trip | External ownership | Gridcraft/Wordcraft forks |
| COM, DLLs, UserForms, P-code and binary source writing | Deferred / outside MVP | Separate design required |

Frontend/binder/IR tests establish syntax and structural compilation results only. They do not establish VBA semantics,
workbook execution, Office compatibility, WASM runtime behavior or corpus pass rates.
