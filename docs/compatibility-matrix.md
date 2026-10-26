# Compatibility matrix

Updated 2026-10-26. Stage: M0 complete. **No VBA feature is executable or verified.**

| Dimension | Status | Evidence / next gate |
|---|---|---|
| Workspace and crate boundaries | Implemented setup | cargo xtask layers |
| CLI help/version and explicit refusal | Implemented setup | apps/vba-cli/tests/cli.rs |
| External source pins and candidate hashes | Implemented setup | strict corpus integrity check; not behavioral validation |
| Source IDs/spans/statements and diagnostic attachment | Implemented M0 | vba-diagnostics/tests/source.rs; no parser mapping yet |
| Typed scalar states/UTF-16 strings and bounded locations | Implemented M0 foundations | vba-values/tests/locations.rs; no coercion/runtime |
| Generic foreign ownership/error/wait prototype | Implemented M0 | vba-testkit/tests/round_trip.rs; no VM or Office bridge |
| Scenario validation / 20 independent fixtures | Implemented M0 | vba-conformance/tests/scenarios.rs and CLI; all VBA cases NOT_RUN |
| Parser and binder | Not implemented | M1/M2 |
| VBA values/coercion/builtins | Not implemented | M3/M4 |
| Bytecode execution / VM host wiring | Not implemented | M3; standalone protocol prototype only |
| Breakpoints, stepping and resume | Not implemented | M3 |
| MS-OVBA source decoding | Not implemented | M6 |
| Oracle comparison / executable scenario runner | Not implemented | M3/M9 |
| Fuzzing | Not implemented | APIs required before targets |
| Excel/Word APIs and OOXML round-trip | External ownership | Gridcraft/Wordcraft forks |
| COM, DLLs, UserForms, P-code and binary source writing | Deferred / outside MVP | Separate design required |

Scaffold compilation/tests establish development infrastructure only. They do not establish VBA semantics,
workbook execution, Office compatibility, WASM runtime behavior or corpus pass rates.
