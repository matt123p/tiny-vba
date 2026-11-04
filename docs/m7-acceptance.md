# M7 — Unchanged-document and package acceptance

Updated 2026-11-04. Profile: `m7-unchanged-v1`, authored-contract evidence.
M7.1–M7.4 are complete for this declared profile; verification results are below.
This profile accepts original source in generated `.xlsm` and `.docm` packages;
it does not claim universal Office compatibility, fresh Office observations,
edited binary writeback or cryptographic signature verification.

## Selection and independent expectations — M7.1

The [reviewed selection](../tests/fixtures/m7/selection.json) contains two original,
deterministically generated specimens. Both forks license their authored source
and test builders MIT OR Apache-2.0. Each record declares exact source/builder/test
SHA-256 hashes, required language/object/reference/control capabilities and fixed
expected typed outcomes. Only these original, bounded, reviewed host workflows
are selected. No downloaded macros, external implementation code or Office-created
binaries are committed or executed. Existing external candidates remain separate
in the unchanged corpus inventory. Historical Food.xlsm inspection is not admission
of that workbook to this execution set.

| Specimen | Owner | Requirements | Independent expected results |
|---|---|---|---|
| `m7-xlsm-context-v1` | GridCraft | Standard/class/workbook/worksheet modules; Excel reference snapshot; Range/Cells/Rows/Columns; explicit events; `With`, Object dispatch, error state, CCur/VarType | Active B A1=Double(11), A3=Double(33); owner A/Data A2=Double(22), A5=Double(6), A6=Double(11), A9=Double(301), A10=Double(1004); callback/identity expectations in selection |
| `m7-docm-context-v1` | WordCraft | Standard/class/ThisDocument modules; Word reference snapshot; main-story Range; explicit document and subscribed Application events | Main body String(`package11`); Document_Open body String(`owner-bound`); Application BeforeSave body String(`before-save`), Cancel Boolean(True); division error Long(11) |

Both use explicit fork-owned reference metadata, never ambient type-library loading.
There are no native worksheet controls or binary UserForms in either specimen.
Existing selected `.tvform` import/designer/runtime/export acceptance stays under
[M2](m2-acceptance.md) and its real fork workflow tests; it is not binary form
package support. Native worksheet Form Controls, binary UserForm import and broader
controls remain [deferred](office-forms-controls-implementation-plan.md).

## Separate workflow stages — M7.2

The actual execution assertions live in the owning forks:

- [GridCraft package workflows](../../gridcraft/crates/vba/tests/package_acceptance.rs).
- [GridCraft payload/relationship regressions](../../gridcraft/crates/xlsx/src/tests/passthrough_tests.rs)
  (`m7_vba_*`).
- [WordCraft package workflows](../../wordcraft/crates/vba/tests/package_acceptance.rs).

| Stage | XLSM assertions | DOCM assertions | Evidence basis |
|---|---|---|---|
| Load | Inert ZIP/project decode; exact extracted raw source; workbook/sheet code names; Auto_Open never runs; file provenance requires consent | Inert decode/attachment of standard/class/ThisDocument modules; exact source; no document mutation or undo entry | Authored package/host contract |
| Compile | Explicit Compile succeeds without running source or granting permission | Explicit Compile succeeds without running source or granting permission | Authored compiler/host contract |
| Run | Explicit permission; exact numeric effects/error tags; class Me, Object dispatch, module contexts; pause/activation/resume without replay; subscribed save cancellation/reset | Authorization checked before VBA entry; exact String/error results; owner-bound explicit Document_Open; actual WithEvents save cancellation/reset | Authored host/runtime contract |
| Save/reopen | Original project bytes preserved; code names/tab rename and cell effects survive; related payload graph, relationship IDs and content types survive; reopen requires fresh consent | Original project, direct VBA signature/data payloads and relationships survive; body effects survive; reopen requires fresh authorization | Authored package/host contract |
| Plain-source edit/export | Raw module/member attributes retained through projected code edits; Save Source request contains edited raw text; stale acknowledgment refuses; document save retains original project | Same assertions through WordVbaProject source/save interface | ADR 0030 / authored revision contract |
| Negative/policy | Missing/rewritten graph warnings; macro-free export removes VBA graph; replaced captured project refuses stale related-part reuse; mixed-owner Range returns 1004 before target mutation | Corrupt/unknown-owner projects refuse attachment without effects; untrusted invocation refuses before source executes; package-wide signature roots removed on rewrite | Authored refusal policy; not Office oracle evidence |

Tests assert actual values and effects before a stage can pass. The selection's
`initial_execution_status` stays NOT_RUN: hash validation alone never proves
execution. The acceptance run and this report provide the separate results.
Source identity is exact after the importer's documented code-page conversion;
original binary identity is a separate byte comparison. ZIP/document XML bytes
are regenerated; whole-package byte identity is not claimed. Related graphs
that exceed a writer's supported profile have warnings/refusals, not a pass.

## Binary writeback and signature policy — M7.3

[ADR 0037](decisions/0037-m7-package-preservation-and-writeback.md) is the reviewed
contract. M7.3 requires the design before implementation; an edited-source CFB
encoder is not required or represented as implemented. Saving a document retains
the original project; exporting source edits does not change that project's bytes
or acknowledge an edited binary project save. A future encoder needs a separate
bounded source/cache/encoding design, source round-trip validation, revision and
identity checks, removal of every original VBA signature, and atomic commit.

VBA signature payload sentinels test opaque transport beside the original project.
They do not establish publisher trust or validity of a real signature. Package-wide
OPC signatures are invalidated by rewrite: GridCraft excludes their roots and
WordCraft explicitly drops/logs them rather than passing old signatures through
its generic preservation path. Neither fork verifies/re-signs a cryptographic
signature. Actual signed-Office-document validity remains NOT_RUN and outside
this unsigned authored preservation profile. No signature grants execution consent.

## Both supplied matrices — M7.4

Rows `T01–T15` follow [module contexts §13](vba_module_types_and_execution_contexts.md#13-differential-compatibility-test-matrix);
rows `R01–R18` follow [object resolution §10](vba_excel_context_and_object_resolution.md#10-compatibility-tests).
All PASS expectations below are **authored adopted contracts**, not fresh Excel
observations. Unsupported rows are outside the declared profile, not passes.
The source selection is unchanged across each packaged execution; host activation,
worksheet captions and document values vary as the scenario prescribes.

| Matrix rows | Scenario | Status / basis and evidence |
|---|---|---|
| T01 / R01 | Standard bare Range with B active | PASS — Main writes B/A1; owner remains untouched |
| T02–T03 / R02 | Worksheet bare Range and explicit Me with B active | PASS — packaged Worksheet_Change writes owner B1/B2 |
| T04 / R03 | Workbook-module bare Range with B active | PASS — packaged Workbook_Open writes B/C1 |
| T05 / R04 | Explicit project-workbook worksheet | PASS — Main/Workbook_Open write A/Data independent of active B |
| T06 | UserForm Me in native binary form package | UNSUPPORTED — binary UserForm import is deferred; separate `.tvform` M2 acceptance is retained |
| T07 | Class Me | PASS — packaged Counter.Read returns its own Number=301 |
| T08 | Worksheet event calls standard helper | PASS — Helper writes B/A7 while event writes owner A |
| T09 / R10 | Code name after tab rename | PASS — SheetCode writes original owner after rename; saved caption and code name are checked independently |
| T10 / R05 | With leading-dot Range | PASS — Main and event explicit receiver writes stay on A/selected receiver |
| T11 / R06 | With bare Range | PASS — standard Main follows active B; worksheet event follows its module owner |
| T12 / R13 | Add-in ThisWorkbook | UNSUPPORTED — no selected add-in model/specimen |
| T13 | WithEvents publisher/sink lifetime | PASS — packaged Excel and Word sinks receive/cancel BeforeSave while connected; Reset removes subscription |
| T14 / R12 | Chart sheet active, Range error | UNSUPPORTED — executable chart-sheet context not in this host profile; preservation is not execution |
| T15 / R09 | Local Range shadows global | PASS — Shadow uses local Object receiver and writes A/Data/A8 |
| R07 | Range endpoints all leading-dot Cells | PASS — Main fills A/Data/A4:B4=44 |
| R08 | Mixed implicit Cells from different active worksheet | PASS — Mixed returns adopted error 1004 before mutation; exact Excel equivalence remains M8.5 |
| R11 | Late-bound Object.Range | PASS — Main's Object receiver resolves Range on its captured owner |
| R14 | Active workbook changes between calls | PASS — Live pauses after Application.Range writes B/D1; resume after activating A writes A/D2 exactly once |
| R15 | Shape/chart Selection treated as Range | UNSUPPORTED — no nonrange Selection in selected executable profile |
| R16 | Worksheet Cells/Rows/Columns implicit members | PASS — event writes owner B2/B6/B7 with B active |
| R17 | Explicit Application.Range | PASS — Live writes active B/D1 under adopted host contract; fresh Excel comparison NOT_RUN |
| R18 | Reference-library/project-module collision matrix | UNSUPPORTED — ambiguous external-library collision profiles are not admitted by the selected explicit reference snapshot; no new precedence rule is inferred |

`Application.Run`/arbitrary re-entry, add-ins and chart-sheet execution remain outside
this profile. Fresh Excel/Word comparisons for these matrices are still **NOT_RUN**
and belong to M8.5, including exact Office build/OS/locale/raw observations.

## Reproduce and verification

With the repositories as sibling checkouts, `cargo xtask m7` first verifies every
selected artifact hash, then runs both real fork package suites and GridCraft's
VBA graph tests. It fails on any mismatch or failed test. Ordinary tiny-vba core
CI remains independent of application siblings/corpus checkouts. The hash checker
alone reports VALID_FIXTURE / NOT_RUN; it never launches or asserts execution.

The [execution record](../tests/fixtures/m7/execution.json) retains per-document
stage results, environment, exact profiles and source pins. Native environment:
Linux x86_64, Rust/Cargo 1.99.1; string comparison context is explicitly invariant,
Gregorian, UTF-16. Fixture source is ASCII/CP1252 with LF endings; inherited M6
CRLF/Unicode regressions remain in the full suites. No Office build/OS/locale
observation is represented by these authored runs.

| Verification | Result |
|---|---|
| `cargo xtask m7` | PASS — 12 reviewed artifact hashes checked before/after; 5 Excel package, 6 Word package and 2 graph tests |
| tiny-vba `cargo xtask ci`, `cargo xtask wasm`, `cargo xtask wasm-run` | PASS — core/IDE gates plus actual WASI execution |
| GridCraft `CRAFT_FONTS_DIR=/tmp/gridcraft-acceptance-fonts cargo xtask ci` | PASS — all 10 stages, including WASM; ordinary build first failed two existing glyph-dependent tests with default fonts |
| WordCraft `cargo xtask ci` | PASS — all 8 stages, including WASM |
| GridCraft `cargo test -p gridcraft-ui-egui --features vba-window --locked vba` | PASS — 9 filtered VBA integration tests |
| WordCraft `cargo test -p wordcraft-ui-egui --features vba --locked vba` | PASS — 12 rendered/model/source/lifecycle tests, including untrusted Run refusal before VBA entry |
| WordCraft strict Clippy for vba/docx/ui-egui with the vba feature, all targets | PASS |
| WordCraft `cargo check -p wordcraft-ui-egui --features vba --target wasm32-unknown-unknown --locked` | PASS — actual browser shell and decoder/adapter |
| Both forks: `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner cargo test -p <fork>-vba --test package_acceptance --target wasm32-unknown-unknown --locked` | PASS — actual Node 22.23.4 execution, 5 Excel and 6 Word tests; Node 18.19.1 aborted inside the test runtime, so the existing Node 22 installation was used |
| New local documentation links and all three `git diff --check` runs | PASS |

The native/JavaScript package tests use the same unchanged fixture source and
independent assertions. There were no renderer/layout changes; this revision's
Word shell change is command/trust routing. No fresh manual native-window or
browser-GUI walkthrough is claimed. Cryptographic verification/re-signing and fresh
Excel/Word runs remain NOT_RUN. The full release gate is M8, not this profile.
