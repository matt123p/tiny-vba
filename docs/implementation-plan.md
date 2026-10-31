# Remaining implementation plan

Updated 2026-10-31. This plan contains **remaining work only**, using stable M6–M8 IDs
in implementation order after completed M1, the [selected M2 profile](m2-acceptance.md),
and the [M3/M4 host profiles](m3-m4-host-acceptance.md). Completed work and the old-to-new milestone mapping
belong in [implementation progress](implementation-progress.md#plan-renumbering--2026-10-28).
Historical reports and ADRs retain their original milestone numbers; use this
document's numbers for new work.
Task IDs within an active milestone stay stable; completed tasks move to progress.

tiny-vba owns reusable language/runtime, debugger, editor and form contracts.
GridCraft and WordCraft own Office objects, document state, storage, trust,
events, undo/recalculation and application integration. Keep the repositories
separate and native host dispatch in Rust.

## Sequence and dependencies

| Milestone | Remaining work | Prerequisites |
|---|---|---|
| M6 | Remaining GridCraft and WordCraft integration; module-context and name-resolution conformance | M1; M2–M5 for corresponding form/event/Excel workflows |
| M7 | Unchanged-document and package acceptance | Applicable M1–M6 capabilities and explicit writeback design |
| M8 | Expanded conformance, hardening and release | M1–M7 acceptance for the declared release profile |

Implement shared language foundations before consumers, range references before
range-dependent value/event behavior, and runtime semantics before form events.
M1–M5 are complete for their declared profiles. WordCraft's
object-model work in M6 can proceed alongside Excel work; its document/form events
consume the shared M2 contract.
Build tests and collect lawful evidence throughout every milestone. M8 is the
final release gate, not a reason to postpone fuzzing, benchmarks or regression tests.

## M5 completion record — Document modules, events and Range-aware UDFs

Owner: tiny-vba shared class/document/callback semantics; GridCraft document
bindings, event ordering, standard Office controls and calculation context.
Depends on applicable M2–M4 capabilities; reuse shared contracts.

- **M5.1:** Execute workbook/worksheet modules with code-name roots, correct
  signatures and code-owner context while keeping import inert.
- **M5.2:** Deliver Workbook_Open/Worksheet_Change and selected document/standard
  control events with defined ordering, EnableEvents, reentrancy/recursion
  budgets, cancellation and partial-side-effect policy. Connect M2 forms.
- **M5.3:** Add Range/reference-aware UDF arguments and Caller/Volatile/context
  behavior, persistent state and recalculation rules. Enforce the declared UDF
  mutation policy without silently substituting value arrays for references.

Accepted for the declared profile. Unchanged authored handlers and UDFs execute
against real GridCraft state. Pause/inspect/resume, exactly-once dispatch,
reset/close/trust revocation and calculation dependencies pass. Workbook_Open
never executes on import. The [M5 acceptance map](m5-acceptance.md) records all
ten passing scenarios and the exact native, WASM and authored-test evidence.

## M6 — Remaining GridCraft and WordCraft integration

Owner: application forks for Office objects, UI/storage/trust/events/undo/recalc;
tiny-vba for shared embedding APIs. M1 supplies generic types; M2 supplies shared
classes/forms. Excel workflow acceptance consumes the applicable M3–M5 behavior.

Completed for the declared profile (2026-10-31), with the [M6.5 per-layer
check](m6-context-resolution-check.md) as the scope-driving record:

- **M6.5:** the check records implemented, refused and missing behavior per owned layer
  against [module types and execution contexts](vba_module_types_and_execution_contexts.md) and
  [Excel context and object resolution](vba_excel_context_and_object_resolution.md), and the
  [compatibility matrix](compatibility-matrix.md) now carries the result.
- **M6.6:** staged identifier resolution over locals/parameters/module/project declarations,
  host-declared implicit `Me` members for document modules, then host globals; per-module-kind
  binding; local/member shadowing with project-before-global precedence; innermost-only
  leading-dot `With` receivers that never change `Me` or active state; standard-module-only
  bare cross-module callees; no implicit current-workbook frame parameter.
- **M6.7:** the forks dispatch declared host globals per invocation against live application
  state with distinct `ThisWorkbook`/`ActiveWorkbook`/`ActiveSheet`/`Me` identities and stable
  code names; GridCraft proves the specifications' unshadowed binding and live-consultation
  cases in real workbook workflows, and WordCraft supplies the document roots under its M6.1
  profile. `Application.Run`, add-in context and chart sheets stay refused/NOT_RUN in the check.
- **M6.8:** IDE member completion types receivers from host-declared import result types and
  call shapes (`ThisWorkbook.`, `Range("A1").`, `Worksheets("Data").`, `Cells(1, 1).`) plus
  `Me`/code-name document types; each fork declares the member/result types it supports in its
  catalogue. Code aids stay read-only and metadata-driven; they never invoke host members.

Remaining before M6 closes (M6.2/M6.3/M6.4 partials in WordCraft, plus evidence): the
shared-renderer localization bridge so WordCraft's translation audit passes with the editor
open (M6.2), an ordinary-run `ThisDocument` binding proof (M6.3), permitted Microsoft Office
reference-build observations while no Office session is available — recorded NOT_RUN (M6.4) —
and each fork's full CI gate on the updated immutable pin. These are tracked in the check and
the forks' M6 documents, not silently dropped.

Acceptance: both forks reuse the same editor and shared interpreter repository.
Real document workflows verify effects and lifecycle without replay or retained
locks; tiny-vba-only snapshots and generic doubles cannot satisfy this gate. The
M6.5 record drove M6.6/M6.7 scope, and acceptance includes the specifications'
unshadowed module-kind binding and live host-global consultation cases in real
fork workflows. Declared host globals and typed/call receivers complete their
members in the shared editor without host invocation. Package round trips and
signatures are the next milestone's separate gate.

## M7 — Unchanged-document and package acceptance

Owner: fork acceptance/package layers, coordinated with tiny-vba source-import
and evidence contracts. Depends on each specimen's M1–M6 capabilities; form
specimens require M2 and the corresponding host event/persistence integration.

- **M7.1:** Curate licensed, reviewed Excel workbooks and Word documents with
  declared language/object/reference/standard-control requirements and independently
  established expected results. Keep candidate/unrun material separate.
- **M7.2:** Report load, compile, run and save separately for unchanged source.
  Verify source identity, typed values/errors, effects/events, code names,
  package relationships/payloads, trust and lifecycle for .xlsm/.docm workflows.
- **M7.3:** Design binary VBA project writeback and signature preservation/
  invalidation as a separate reviewed contract before implementation. Distinguish
  original-byte preservation from edited-source writeback. No protection/P-code
  bypass or unrestricted binary add-ins.
- **M7.4:** Execute the two specifications' module-context and name-resolution
  matrices ([module types](vba_module_types_and_execution_contexts.md),
  [object resolution](vba_excel_context_and_object_resolution.md)) as
  unchanged-document evidence: per-module-kind binding with a different workbook
  active, `With` versus bare identifiers, shadowing, code names after tab rename,
  late binding, and chart-sheet/add-in cases where licensed specimens permit.
  Record PASS/MISMATCH/UNSUPPORTED/NOT_RUN per case; encode binding precedence
  only from verified observations, never from documentation alone.

Acceptance: per-document profile/version/evidence records distinguish PASS,
MISMATCH, UNSUPPORTED and NOT_RUN. Verify forms/standard controls, source
attachment, no untrusted auto-run and signature validity/loss decisions.
Run both repositories' required CI/WASM gates and real fork workflows.
Publish incompatibilities; subset success does not imply universal compatibility.

## M8 — Expanded conformance, hardening and release

Owner: tiny-vba language/debugger/import evidence; forks own Office behavior and
document results. Evidence work starts alongside M1 and feeds this final gate.

- **M8.1:** Curate further licensed library/corpus cases and independently
  established oracle expectations with exact versions, OS/locale, types,
  errors and side effects. Keep unreviewed/unexecuted material NOT_RUN.
- **M8.2:** Add importer/parser/VM fuzz targets and minimized regressions,
  bounded-work/timeout tests and repeatable native/WASM performance benchmarks.
- **M8.3:** Complete native/WASM CI and affected UI/native/browser gates.
  Publish the supported feature/profile/version matrix and remaining gaps.
- **M8.4:** Expand feature-specific asserted benchmarks and assemble an integrated
  suite of 20 independent reviewed workbooks; publish provenance and coverage.
  Report language and Office-host categories separately; no global percentage
  substitutes for a feature's acceptance gate.

Acceptance: the declared release profile meets its applicable M1–M7 gates,
retains prior regressions and reports PASS, MISMATCH, UNSUPPORTED,
BLOCKED_BY_POLICY, INVALID_FIXTURE, CRASH, TIMEOUT and NOT_RUN accurately.
No fresh Office result may be inferred from public documentation or inherited
oracle records.

## Ongoing implementation rules

Use independent Rust code, public documentation, lawful observations and licensed
fixtures/assets. Keep downloaded evidence ignored and provenance/notices tracked.
Never execute unreviewed macros or import another interpreter's implementation.
Keep the existing reviewed crate graph and headless native/WASM boundaries.
For each change, run the repository's required CI/WASM gates and affected UI/host
checks; update [progress](implementation-progress.md), [compatibility](compatibility-matrix.md)
and the owning contract/ADR. The [UI scope index](ui/post-m8-roadmap.md) points
to the remaining UI/host requirements without duplicating this plan.
