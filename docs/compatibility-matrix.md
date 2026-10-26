# Compatibility matrix

Updated 2026-10-26. Stage: scaffold. **No VBA feature is executable or verified.**

| Dimension | Status | Evidence / next gate |
|---|---|---|
| Workspace and crate boundaries | Implemented setup | cargo xtask layers |
| CLI help/version and explicit refusal | Implemented setup | apps/vba-cli/tests/cli.rs |
| External source pins and candidate hashes | Implemented setup | strict corpus integrity check; not behavioral validation |
| Parser and binder | Not implemented | M1/M2 |
| VBA values/coercion/builtins | Not implemented | M3/M4 |
| Bytecode execution / generic foreign host | Not implemented | M3 |
| Breakpoints, stepping and resume | Not implemented | M3 |
| MS-OVBA source decoding | Not implemented | M6 |
| Scenario parser / oracle comparison | Not implemented | M0/M9 |
| Fuzzing | Not implemented | APIs required before targets |
| Excel/Word APIs and OOXML round-trip | External ownership | Gridcraft/Wordcraft forks |
| COM, DLLs, UserForms, P-code and binary source writing | Deferred / outside MVP | Separate design required |

Scaffold compilation/tests establish development infrastructure only. They do not establish VBA semantics,
workbook execution, Office compatibility, WASM runtime behavior or corpus pass rates.
