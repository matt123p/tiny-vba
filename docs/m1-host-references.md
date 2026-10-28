# M1 host references — implementation log

Started 2026-10-28 on `feat/m1-host-types`, from `1765945`.
This follows the **new** remaining plan numbering, not historical M1 frontend work.

## Stage state

| Stage | State / next proof |
|---|---|
| M1.1a explicit reference snapshots, enum types and constants | Complete for the documented profile; all gates below pass |
| M1.1b nominal host object types | Pending: specify type identity/assignability and verify every storage path, including ByRef, arrays, host results and debugger edits |
| M1.1c unchanged typed Office source | Pending: generic object-type execution proofs first, then fork-owned `As Worksheet` / `As Excel.Range` tests |
| M1.2 default members / M1.3 collections | Pending; no capability claim from reference metadata |

## Implemented contract

[ADR 0020](decisions/0020-explicit-host-reference-snapshots.md) records ownership,
bounds and refusals. Hosts construct `vba_foreign::ReferenceSet::new` with ordered
`ReferenceLibrary` values containing `ReferenceType` and `ReferenceConstant` rows.
Pass the validated snapshot as `CompileOptions.references`, or configure it in the
IDE with `configure_reference_bindings` before a session exists. There is no loader.

Registered enum types support unqualified `As Mode` and qualified `As HostLib.Mode`
declarations, parameters, arrays and Function results. They retain VBA Long values
and shared coercion rules. Constants resolve as `Flag`, `HostLib.Flag`, `Modes.Flag`
or `HostLib.Modes.Flag`; reference order breaks unqualified ties. Module/local
symbols and imported roots retain precedence. Defaults, bounds and constant
expressions use the same reference resolution and typed IR constants.

The IDE derives discoverable constants from executable bindings and retains the
same snapshot during isolated Immediate compilation. Replacing it during a pause,
wait or completed session requires Reset. Display-only metadata does not silently
enable constants or types. No UI geometry or host dispatch protocol changes.

Unknown declared types now give an explicit source-attached VBA0132 diagnostic;
unavailable reference snapshots give VBA0131, including a missing library with no
symbol rows. Object descriptors remain explicitly unsupported. Accepting them as
Object aliases would hide missing nominal assignment checks, so it is not a
temporary compatibility shortcut.

## Requirement-to-test map

| Requirement | Independently authored evidence |
|---|---|
| Snapshot ownership, ordered/case-insensitive lookup, library/module qualification | `crates/vba-foreign/tests/references.rs` |
| Malformed/duplicate/reserved names, library/symbol/payload bounds, nonfinite values, missing references and nominal refusal | Same protocol suite |
| Qualified type spelling and exact Unicode/CRLF spans | `crates/vba-syntax/tests/frontend.rs` |
| Enum declarations, parameters/ByRef, results and arrays actually execute | `crates/vba-vm/tests/references.rs`, using NoHost; no dispatch can succeed |
| Constants in expressions, source constants, Optional defaults and bounds | Same VM suite |
| Source shadowing, assignment/call refusal and precise missing/undefined type errors | Same VM suite |
| IDE discovery, reset/reconfiguration guards and isolated compilation snapshot | `crates/vba-ide/tests/references.rs` |

Validation commands: `cargo xtask ci`, `cargo xtask wasm`, `cargo xtask wasm-run`.
The WASI gate includes the new VM `references` target and the IDE suite. Logs are
ignored `target/m1-*.log`. No Office/Excel or downloaded macro is executed.

Final local results: **288 workspace tests PASS** (11 added), strict all-target/
all-feature Clippy and format/layer/provenance checks PASS; headless WASM compile
PASS; **102 tests executed under WASI PASS**, including the five new VM reference
tests and two IDE reference tests. Existing 30 UI interaction tests remain green
in workspace CI. This increment changes no visual layout, external dependencies,
Office fork or host invocation ABI. `git diff --check` passes.

Public behavior references consulted, with original code/tests:

- [Microsoft: using constants and library/module qualification](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/using-constants)
- [Microsoft: Enum statement and library reference priority](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/enum-statement)
- [Microsoft: undefined types and required references](https://learn.microsoft.com/en-us/office/vba/language/how-to/user-defined-type-not-defined)

Next: M1.1b's nominal host object contract. Keep generic Object/Variant late binding
unchanged, and do not enable nominal declarations until assignment, lifetime and
host-result checks are enforced. Do not start default-member fallback or enumerate
host objects implicitly while implementing this type boundary.
