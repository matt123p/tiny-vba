# GridCraft integration work log

## Spreadsheet entry points continuation — complete for the selected profile

User scope: editor launch, toolbar/menu commands, assigned worksheet buttons and
worksheet-callable VBA Functions. tiny-vba main was pulled and merged locally at
`f46cf23`, retaining upstream editor assistance and both progress records.

Stages: (1) generic native calculation provider and existing hidden-name source
storage; (2) fork-owned bounded pure UDF driver, entry discovery and persistence;
(3) Developer/toolbar/menu/keyboard and worksheet shape assignment UI;
(4) authored native/WASM/control/UI/package tests, full gates and visual inspection.
All four stages are complete for the selected GridCraft profile; final acceptance
below supersedes intermediate checkpoint counts and designs.

Public behavior sources consulted 2026-10-27 (facts paraphrased; no source code,
text, artwork, screenshots, binaries or downloaded fixtures copied):

- https://support.microsoft.com/en-us/excel/run-a-macro-in-excel — Developer Macros,
  procedure picker, edit/step/run and toolbar/custom ribbon invocation.
- https://support.microsoft.com/en-us/excel/assign-a-macro-to-a-form-or-a-control-button
  — worksheet placement, Assign Macro, right-click reassignment, click invocation.
- https://support.microsoft.com/en-us/excel/create-custom-functions-in-excel
  — Function return assignment, formula arguments, User Defined discovery and
  restriction against workbook mutation from worksheet functions.

These public behavior references are not redistributed or treated as licensed
source code. All code, examples and test workbooks are independently authored.
No private material or Office implementation files are used. Excel feature names
use GridCraft's own UI components and original code-drawn icons.

Checkpoint: merged tiny-vba `cargo xtask ci`, `wasm` and `wasm-run` pass
(`target/merged-integration-*.log`). GridCraft's six authored worksheet bridge
tests pass; optional UI tests are in progress. UDF calls use independently generated
VBA wrappers so ordinary ByRef parameters and omitted typed defaults flow through
the existing compiler's call rules, preserving the shared ByVal launch contract.
The generic calc hook has no VBA dependency; GridCraft's fork owns its provider.

Second checkpoint: 23 native and JavaScript-WASM adapter tests and 17 optional
editor/worksheet interaction tests pass. The first complete GridCraft gate and
full optional UI suite passed. Final review replaced a proposed numeric-ID button
assignment table with inert `Shape.on_action`, set by native `shape.setMacro` and
stored as DrawingML's `macro` attribute. Shape IDs can be reused after deletion;
ownership on the shape avoids accidentally assigning a replacement object and
preserves assignments when package writers renumber drawings or sheets rename.
The regression exercises deletion, reused IDs and undo. Final full gates are being
rerun after this correction; only the final acceptance record is authoritative.

Additional public behavior/schema references, paraphrased without source reuse:

- https://support.microsoft.com/en-us/accessibility/excel/keyboard-shortcuts-in-excel
  — Alt+F11 editor and Alt+F8 procedure picker.
- https://support.microsoft.com/en-us/office/add-ins/show-the-developer-tab
  — initially hidden, persisted Developer preference.
- https://learn.microsoft.com/en-us/office/vba/api/excel.shape.onaction
  — shape-owned assigned macro name.
- https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.drawing.spreadsheet.shape.macro
  — the public DrawingML `macro` attribute.

### Final acceptance (2026-10-27)

GridCraft commit `b496325b8e4d2fd471ae747f4036d504aee5c9df` on
`feat/tiny-vba-integration` contains the completed spreadsheet-facing scope.
It pins shared tiny-vba revision `ea38c3f346b4010d6017b3a6d27e645ebad7ef79`,
which includes the pulled editor improvements and imported-root fix. Subsequent
tiny-vba commits only update this interruption/acceptance record; the dependency
stays pinned to its tested product code. No WordCraft or application dependency
is introduced into tiny-vba.

Implemented: VB toolbar and Alt+F11 editor toggle; Alt+F8/View/Developer macro picker;
initially hidden, persisted Developer tab; original ribbon buttons and Insert menu;
public no-argument Sub run/edit/assign/toolbar controls; sheet buttons/right-click
reassignment/Design Mode; inert shape-owned OnAction metadata with DrawingML/JSON
round trips and deletion/ID-reuse/undo safety; source/toolbar persistence through
existing hidden-name storage; public Functions in formulas and User Defined discovery.
UDFs use bounded fresh VMs, compiler-owned default/ByRef semantics, normal dependencies
and rectangular spills, with no mutable workbook or asynchronous host access.
Consent, stale source/document guards and imported project decoding remain explicit.

Final local evidence in GridCraft's ignored target directory:

- `worksheet-final-ci.log`: all ten gates PASS, **2,553 tests**, including the
  newly authored shape metadata and ink preservation cases; fmt, strict Clippy,
  assets, layers, browser compilation, parity and keytips all pass.
- `worksheet-final-ui-all.log`: **1,143 tests PASS**, including **18** VBA
  editor/control/keyboard/worksheet cases; `worksheet-final-ui-tests.log` reruns
  the 18 cases. Includes real pointer clicks, Design Mode, right-click reassign,
  toolbar/picker use, permission revocation, dirty source, atomic failed saves,
  stale-document refusal, and avoiding demo execution for absent/corrupt source.
- **25 native and JavaScript-WASM adapter cases PASS**: 2 project, 13 workbook,
  10 worksheet. Final worksheet-only logs `worksheet-final-native-udf.log` and
  `worksheet-final-wasm-udf.log` include implicit ByRef input isolation and
  repeated shared strings exceeding 32-bit logical byte totals. Saturating counts
  and preallocation checks refuse oversized payloads before UTF-16 array creation.
- `worksheet-final-clippy.log`: optional all-target strict Clippy PASS.
  `worksheet-final-desktop.log` / `worksheet-final-web.log`: real desktop/browser
  applications compile with the feature enabled. WASM execution uses Node and
  wasm-bindgen-test-runner 0.2.129, not unsupported GridCraft WASI.
- Nine original PNGs in `target/worksheet-ui` render the real paused debugger,
  Developer/worksheet/function result, and macro picker in light/dark/narrow/scaled
  modes. Visually inspected; no vendor/reference art or screenshots are committed.

The complete optional UI suite and its pinned craft-fonts input are now included
in GridCraft's optional CI job. These are local results; no hosted workflow run or
fresh Office oracle observation is claimed. Merged tiny-vba passes its own CI,
browser WASM compilation and actual WASI execution (`target/merged-integration-*`).
All examples, fixtures and code are independently authored from public behavior
references; no downloaded implementation, private material or vendor assets enter
either repository.

Launch from GridCraft: `cargo +1.99.1 run -p gridcraft --features vba`.
Click **VB** or press **Alt+F11**. View → **Developer tab** reveals the ribbon;
**Macros** / **Alt+F8** opens assignments. Save Source to Workbook applies code;
save the workbook normally to retain it. File-derived code requires session
permission. Exact control parameters and a working authored Function example are
in GridCraft `docs/vba.md`.

Remaining compatibility limits are deliberate: the optional feature is disabled
in default builds; no Office binary project writeback, ActiveX/UserForm or actual
Form Control serialization, cross-workbook/add-in UDFs, Range/Caller/Volatile host
objects inside UDFs, persistent UDF state, events, classes or complete Excel object
coverage. Reference arguments are bounded value arrays, not Excel Range objects.
Source and permission replacement invalidate compiled functions; apply desired
source again after source undo. Sheet topology changes require reopening the editor.
Next independent stage: stable worksheet host identities and event/caller policy,
or WordCraft's own adapter acceptance; neither is required for this requested slice.

Publication update (2026-10-28): the user approved publishing tiny-vba `main`;
the tested merge and acceptance record (`0b709ae`) are now on remote `main`.
The earlier default-branch approval block is resolved for tiny-vba.

The user clarified that GridCraft must remain on its feature branch. GridCraft's
`main` push to `b496325b` completed before that clarification interrupted the
command; its prior remote main was `d8c5f588`. The local checkout is back on
`feat/tiny-vba-integration`, retaining all work. With explicit rollback approval,
GridCraft's local and remote `main` were restored to `d8c5f588` using an exact
`--force-with-lease` expectation of `b496325b`. The integration was first published
and is preserved on remote `feat/tiny-vba-integration` at `b496325b`. Both remote
refs were verified after the rollback; tiny-vba remains merged on its own `main`.

Started 2026-10-27. This is the interruption/resumption record for the user-requested
integration. Interpreter and shared IDE remain in tiny-vba; Excel objects and the
adapter belong in GridCraft. WordCraft can consume the same tiny-vba revision.

## Continuation stage — native typed writes complete

User requested continuation after the committed baseline. Continue on the same
feature branches; starting GridCraft revision is `421a6a4c` and both trees were clean.

Committed in GridCraft as `8168bda4c7aa7422d96e01fcf88950233f0b1736`.
The native `CellContent`/`Session::set_range_contents` API and equivalent
`range.setContents` command share the original engine command/edit lifecycle.
VBA now supports Text-format (`@`) literal/formula/CVErr writes, preserving formats;
serde_json is a test-only direct dependency of the adapter. The shared interpreter
and pinned dependency revision are unchanged.

Validation: full GridCraft CI passes all ten gates and 2,540 tests; five new engine
tests pass; all 15 adapter tests pass natively and in JavaScript-hosted WASM; all
eight editor/control cases, strict optional-feature Clippy and browser compilation
pass. tiny-vba CI, WASM and WASI gates pass. Logs are in the owning repositories'
ignored `target/typed-contents-*` files. Native/wire replay, atomic refusal,
protection, undo/recalc, invalid coordinates/types, Unicode/formula/payload limits
and Text-format storage are covered. GridCraft `docs/native-cell-contents.md`
specifies the API. Remaining object lifecycle/events/form work is unchanged.

Plan recorded before implementation:

1. Add a generic engine-owned `range.setContents` command and native
   `Session::set_range_contents` API accepting bounded row-major scalar values or
   explicit formulas. Preserve destination formats, literal strings, Empty and
   cell errors; explicit formulas must work in Text-formatted cells.
2. Reuse the existing command execution lifecycle for native calls: shared
   enabled/panic recovery/history/groups/recording/journal behavior. Serialize only
   the recorded command envelope; the native mutation path must not decode JSON.
   Keep the existing `range.setValues` typed-input behavior unchanged.
3. Route the VBA adapter's conversion through that API and remove the `@` refusal.
   Keep VBA tags until the worksheet-storage conversion boundary. No new tiny-vba
   dependency or shared-protocol revision is needed.
4. Prove native/JSON-command equivalence, recording replay, exact literal/error
   contents, format preservation, formula recalculation, atomic error/protection
   refusal, budgets, undo/redo and VBA break/edit/resume, natively and JavaScript/WASM.
5. Run affected and complete gates, update scope/evidence and commit the stage.

Remaining broader object-model/events/WordCraft stages are unchanged.

## Final baseline checkpoint — 2026-10-27

The requested GridCraft baseline is implemented and committed. This section is
authoritative; the chronological notes below retain intermediate results.

| Repository | Branch / revision | State |
|---|---|---|
| tiny-vba | `feat/gridcraft-integration`; reusable code pin `4d80a9584b3daeb75d589c629eff272ecd5881fe` | Shared compiler fix, corpus-download packaging and process log pushed with explicit user approval; final documentation follows on the same branch |
| GridCraft | `feat/tiny-vba-integration`; `421a6a4c6a793e0b90d3bcbf65595fc9ef21ffb5` | Integration committed locally; no GridCraft branch push or PR was requested/performed |

The fork contains a toolkit-free L6 `gridcraft-vba` adapter with bounded native
Application/Workbook/Worksheet/Range objects, case-insensitive and named dispatch,
typed scalar/rectangular-array conversion and exact numbered refusal paths.
Workbook writes stay in existing engine commands with protection, recalculation
and one undo group through debugging, completion/error/reset/close. Prior effects
survive failure/stop. Document identity, consent and topology are rechecked before
ticks and host calls. The shared editor mounts through an optional `vba` feature,
with host themes, source IO, control-command discovery and dirty-source handling.
VBA project extraction is inert; original package bytes are preserved.

All normal dependencies use the published Git pin. Temporary local Cargo patches
have been removed from GridCraft `.cargo/config.toml`; no sibling checkout is a
build input. WordCraft remains untouched and can consume the same shared revision.

### Verification performed

| Check | Result |
|---|---|
| tiny-vba `cargo xtask ci` | PASS: formatting, strict all-target Clippy, workspace tests including exact imported-root dispatch, DAG/provenance/bundled evidence checks |
| tiny-vba `cargo xtask wasm` | PASS: every library and standalone web shell compile |
| tiny-vba `cargo xtask wasm-run` | PASS: native/WASI smoke, importer, callback, IDE and VM embedding regressions |
| GridCraft `cargo +1.99.1 xtask ci` | PASS: all 10 gates, 2,534 tests, including ordinary workspace tests, doctests/assets/layers/wasm/parity/keytips |
| GridCraft `cargo +1.99.1 test -p gridcraft-vba --locked` | PASS: 14 real workbook/project tests |
| Same 14 adapter tests on `wasm32-unknown-unknown` with wasm-bindgen-test-runner 0.2.129 | PASS in Node JavaScript-hosted WASM, including formula clocks, recalc/undo, handlers, pause/edit/resume and package round-trip |
| GridCraft entire `gridcraft-ui-egui --features vba --lib --tests` suite | PASS: 1,132 tests at checkpoint (included seven new integration cases); final added idle-close regression and all eight integration cases subsequently PASS |
| Optional-feature all-target Clippy with `-D warnings` | PASS after final lifecycle change |
| `gridcraft-web/vba` plus adapter for wasm32-unknown-unknown | PASS: actual browser application compiles |
| `cargo +1.99.1 build -p gridcraft --features vba --locked` | PASS: Windows executable links |
| Snapshot harness | PASS: actual post-FillCells breakpoint with locals rendered and visually inspected in light/dark/narrow/scaled host themes; ignored `gridcraft/target/vba-ui/*.png` |
| Git diff/format checks | PASS; mixed test-file line endings normalized before final local commit |

Final GridCraft gates used the existing pinned craft-fonts input in ignored
`target/craft-fonts`: set `CRAFT_FONTS_DIR` to its absolute path and
`CRAFT_FONTS_REQUIRED=1`. Set `RUST_TEST_THREADS=1` for the complete suite here:
an existing parallel global-font-binding test raced in one repeat (another full
parallel run passed). Without craft-fonts, two existing multilingual tests lack
required glyphs. Neither limitation was hidden by weakening tests. Final full log:
GridCraft `target/integration-ci-final.log`; optional UI log:
`target/integration-ui-all.log`.

The extra GridCraft WASI experiment is **not supported/passing**: its browser
chrono/js-sys clock imports cannot execute in WASI. The correct JavaScript/WASM
target above passes all adapter tests. tiny-vba itself retains its passing WASI
gate. No fresh Office observation, complete Excel compatibility or interactive
browser UI session is claimed. External corpus checkouts are absent here; bundled
197 language expectations pass, seven pins are checked, 0/646 upstream candidate
file hashes were available for local rechecking.

### Resume, run and extend

From GridCraft: `cargo +1.99.1 run -p gridcraft --features vba`, then View → Macros →
VBA Editor. Programmatic `vba.open`, `vba.run`, status/step/reset/close/trust commands
use the existing authenticated control route; exact parameters and the implemented
profile are in GridCraft `docs/vba.md`. New optional CI covers native/UI/Clippy/
browser compilation/JavaScript-WASM execution. Screenshots, fonts, temporary
runners/overrides and test logs stay in ignored build storage.

The scoped baseline is complete. Next smallest extension: a native typed engine
value-write command to handle Text-format (`@`) cells faithfully; current
string/formula/CVErr writes there fail explicitly rather than silently corrupting
values. Other remaining scope: stable worksheet lifetime IDs for topology edits,
broader/default-member/collection objects, events/EnableEvents/UDFs, classes/forms,
signatures and binary project writeback, and WordCraft's own document acceptance.
Value currently follows Value2, without format-based Currency/Date inference.
These are explicit profile limits, not implemented compatibility claims.

Branches: tiny-vba `feat/gridcraft-integration`; GridCraft `feat/tiny-vba-integration`.
Both working trees were clean before branch creation. Starting tiny-vba revision:
`f4629598b7ce475f5fef4b478c5b6d93338877b6`.

## Stages and acceptance

1. Inspect M5/M7/M8 contracts and actual GridCraft seams; record design before code.
2. Implement a fork-owned, toolkit-free `gridcraft-vba` adapter above engine. Native
   `vba-foreign` protocol; bounded object handles tied to existing document/sheet
   identities; explicit unsupported-member/type errors. No Office code in tiny-vba.
3. Drive the existing reusable IDE through the adapter. All workbook writes use
   GridCraft commands, so protection, recalculation, recording and undo remain owned
   by the engine. Group successful writes into one undo entry, also on failure/stop;
   do not roll back successful macro effects.
4. Mount shared editor behind an opt-in GridCraft feature using host theme tokens.
   Explicit launch/trust, bounded frame ticks, cleanup on stop/close/document loss.
   Preserve original VBA package bytes; source import never executes code.
5. Test authored workbook macros, types/arrays/errors/negative paths, trust,
   identity/lifecycle, break/edit/resume, undo/recalc, package round trips and UI.
   Run both repositories' CI/WASM gates and inspect rendered integration UI.
6. Record exact results, limits, commands and the next task here. No unrun check is
   a pass, and this bounded profile is not full Excel object-model compatibility.

## Design decisions

- Use a separate optional adapter crate above `gridcraft-engine`; the engine does
  not depend on VBA. Small shell hooks consume adapter/shared UI APIs.
- Reuse M5 imports/HostBinding and M8 Ide, not a second compiler/editor/VM driver.
- Root dependency revisions must be pinned; tiny-vba stays a separate repository.
- Shared craftrules and GridCraft's ignored plan/ files are absent from this
  checkout. Follow tracked AGENTS.md and docs/architecture.md; no assumptions about
  unavailable local planning files.

## Earlier checkpoints (chronological work notes)

- Shared compiler fix `16f2a492f68bd361bc2c4e06fa9df1c3a1d47d5e` is committed
  and pushed to the tiny-vba feature branch with explicit user approval.
- A fresh Cargo Git fetch also fetched unrelated reference/corpus submodules.
  Set their `.gitmodules` update strategy to `none`; corpus CI/docs explicitly
  opt in with `git submodule update --init --checkout --depth 1`. This keeps
  host dependency builds independent of corpus downloads, as required by scope.
  Cargo behavior is documented at https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#git-submodules.
- GridCraft's complete Windows `cargo +1.99.1 xtask ci` passes all ten steps,
  2,534 tests at that checkpoint, with the existing pinned craft-fonts input
  `b076417b5880bd4154bd22f00695b1d28c6bea83` in ignored `target/craft-fonts`.
  Without it, two existing multilingual UI tests fail on missing fallback glyphs.
- All seven optional editor/control/source save-discard/accessibility tests pass;
  strict all-target optional-feature Clippy passes. Original paused-state light,
  dark and narrow/scaled PNGs in GridCraft `target/vba-ui` were rendered and inspected.
- GridCraft's actual `gridcraft-web/vba` browser application and toolkit-free
  adapter compile for wasm32-unknown-unknown. An additional WASI adapter run is
  not a pass: GridCraft's formula clock uses browser JS imports (chrono/js-sys),
  unavailable in WASI. JavaScript-hosted WASM acceptance is being added instead.

- Read both AGENTS.md files and tiny-vba development, host, architecture and editor
  skills, M5/M8 contracts, and actual GridCraft command/undo/trust APIs.
- Matching egui fork revisions confirmed in both workspace roots.
- Windows has Rust 1.99.1 installed; GridCraft's unqualified cargo uses 1.92, so
  use `cargo +1.99.1` there.
- Baseline `cargo +1.99.1 test -p gridcraft-engine --lib --offline
  --no-default-features` first failed because the sandbox denied linker temporary
  files. Retried with approved compiler access; result pending.
- Stages 1–3 implemented in the fork; stage 4 shell hooks implemented. Acceptance
  and review continue. Ten authored workbook tests pass against the local compiler
  fix: FillCells, recalc/undo/redo, typed values/literal formula-like text/CVErr,
  arbitrary-bound rectangular arrays, atomic conversion failure, named arguments,
  VBA handlers, exact errors, trust/revocation, break/edit/resume without replay,
  stop/error partial effects, document loss/topology and XLSM byte preservation.
- Those tests exposed root-object member calls incorrectly taking the fixed-alias
  binder shortcut. ADR 0017 and a generic exact-dispatch VM regression fix this
  here. The fork currently uses ignored `target/vba-local.toml` overrides to test
  that change; replace its Git pin with the tested tiny-vba commit before delivery.
- `cargo xtask ci` passes on Windows with the bundled Python runtime on PATH:
  fmt, strict Clippy, workspace tests including the new regression, layer graph,
  bundled evidence/provenance checks. External corpus checkouts are absent:
  seven pins checked, 0/646 upstream candidate hashes checked (not a corpus pass).
- `cargo xtask wasm` and `cargo xtask wasm-run` pass on Windows after installing
  wasm32-unknown-unknown and wasm32-wasip1: native/WASI smoke, 14 import tests,
  17 callback tests, 25 headless IDE tests. Embedded GridCraft UI tests pending.
- Windows Python discovery fix: xtask uses `python` on Windows, `python3` elsewhere.
  Available Python directory for this machine:
  `C:/Users/mattp/.cache/codex-runtimes/codex-primary-runtime/dependencies/python`.
