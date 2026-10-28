# M1 host references — implementation log

Started 2026-10-28 on `feat/m1-host-types`, from `1765945`.
This follows the **new** remaining plan numbering, not historical M1 frontend work.

## Stage state

Continuation 2026-10-28: user explicitly requested M1.1 completion and a boundary
audit. Nominal identity/IR 5 metadata and VM storage checks are complete
on the same feature branch. Type queries use a read-only host-supplied resolver
over the host's existing object store, not an engine-side registry. No Office
type names or object implementations enter the engine. Positive/negative/lifecycle
tests and the fork-owned typed-source proof below all pass.

| Stage | State / next proof |
|---|---|
| M1.1a explicit reference snapshots, enum types and constants | Complete for the documented profile; all gates below pass |
| M1.1b nominal host object types | Core complete: explicit resolver, IR 5 metadata and typed storage/lifetime checks; native/WASI gates pass |
| M1.1c unchanged typed Office source | Complete for registered types: GridCraft native/JavaScript-WASM Worksheet/Excel.Range source and real cell effects pass |
| M1.2 default members / M1.3 collections | Pending; no capability claim from reference metadata |

## Nominal core checkpoint

[ADR 0021](decisions/0021-nominal-host-object-types.md) extends the first increment
below. `ReferenceTypeKind::Object` now binds with a canonical `NominalType` and
verified IR 5 metadata. `Session::new_with_type_resolver` and
`Ide::configure_type_resolver` accept a read-only host query backed by the host's
existing object store. tiny-vba has no Office definitions or parallel object registry.

Checks cover scalar/array assignment, ByVal/ByRef, compiler temporaries, returns,
entry/callback arguments, foreign results/writebacks, debugger edits, isolated
Immediate snapshots, persistent restore and handle lifetime. Object-array identity
survives Variant copies/resize/Erase. Wrong types give error 13, stale handles error
91, missing providers explicit unsupported errors. Generic Object/Variant remain.

Core validation: **304 workspace tests PASS**, strict CI/Clippy/format/layer checks
PASS, headless WASM compile PASS, **118 actual WASI tests PASS**. Thirteen new VM
nominal tests and three IDE workflows prove the boundaries, including atomic waited
writeback, callbacks, state restore and query-budget exhaustion. The structural
golden's profile header changes to ir5-structural; its scalar IR stays unchanged.
Qualified declared types also resolve catalogue completion/Quick Info without
evaluation; duplicate class names require an explicit library/qualified owner.

Boundary audit found no Excel/GridCraft object definitions in shipping interpreter
libraries. The only Excel-related matches in non-test Rust code are conformance
oracle metadata, not object definitions. The 19-crate DAG remains headless/acyclic
with no application dependency. Excel descriptors/resolver and unchanged-source
proof live solely in GridCraft for M1.1c.

## M1.1 final cross-repository acceptance

M1.1 is complete for explicit registered host types. GridCraft commit
[`139016d715665a0e8f7a5ec2502f45bc8ab7dfb7`](https://github.com/matt123p/gridcraft/commit/139016d715665a0e8f7a5ec2502f45bc8ab7dfb7)
contains the Excel definitions, shared existing object store, read-only resolver
and [fork acceptance report](https://github.com/matt123p/gridcraft/blob/139016d715665a0e8f7a5ec2502f45bc8ab7dfb7/docs/vba-m1-types.md).
It pins tiny-vba's tested implementation
[`98a88c37ac125cf4de65d42c0e9d9120e731e49d`](https://github.com/matt123p/tiny-vba/commit/98a88c37ac125cf4de65d42c0e9d9120e731e49d)
in both manifests and all lockfile source entries. No sibling path override,
embedded interpreter checkout or reverse dependency is introduced.

All final fork gates pass against that exact pin: **33 adapter tests natively
and in actual Node/wasm-bindgen execution**, **2,561 full-CI test/doctest results**
and all ten CI steps, **1,150 optional window tests**, strict all-target window
Clippy, and the browser VBA feature build. Original source is retained byte for
byte while typed declarations/calls/results execute real cell writes. Invalid
class assignments and paused edits preserve values and prior committed effects.
Qualified Worksheet/Range completion is verified with the explicit Excel namespace.

Neither main branch was merged. Both feature branches are `feat/m1-host-types`.
The supplied Excel reference is a four-class adapter profile, not an installed
typelib, COM loader or complete Excel API. Existing unregistered types/default
members/collections/events/forms/package gaps remain explicit. No new Office
oracle observation, downloaded macro execution or vendor implementation/assets
were used. Only public behavior references and independently authored fixtures.

## First reference increment contract (historical checkpoint)

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

Next: M1.2 default-member dispatch, followed by M1.3 enumeration. These remain
separate work; nominal type metadata does not silently implement either.
