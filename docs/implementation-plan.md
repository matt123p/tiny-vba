# Remaining implementation plan

Updated 2026-11-02. This plan contains **remaining work only**: stable M7–M8 IDs,
after completed M1–M6 for their declared profiles and accepted M6A.
[M6 sign-off](m6-acceptance.md) records the owner-approved move of fresh Office
comparisons to M8.5; they no longer block M7. The subsequently requested
[M6A acceptance](m6a-acceptance.md) records the completed intrinsic milestone
and its declared limits ([ADR 0033](decisions/0033-intrinsics-before-package-acceptance.md)).
Completed work and the old-to-new milestone mapping
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
| M7 | Unchanged-document and package acceptance | Accepted M1–M6 and M6A profiles; explicit writeback design |
| M8 | Expanded conformance, hardening and release | M1–M6, M6A and M7 acceptance for the declared release profile |

Implement shared language foundations before consumers, range references before
range-dependent value/event behavior, and runtime semantics before form events.
M1–M6 are signed off for their declared profiles. M6A was accepted on
2026-11-02 ([acceptance](m6a-acceptance.md)); **M7 is next**, beginning with
M7.1 specimen curation and capability classification. Completed task details
are retained in [progress](implementation-progress.md#completed-m5m6-plan-scope)
and the owning acceptance records.
Build tests and collect lawful evidence throughout every milestone. M8 is the
final release gate, not a reason to postpone fuzzing, benchmarks or regression tests.

## M7 — Unchanged-document and package acceptance

Owner: fork acceptance/package layers, coordinated with tiny-vba source-import
and evidence contracts. Starts after M6A acceptance and depends on each
specimen's M1–M6 and intrinsic capabilities; form
specimens require M2 and the corresponding host event/persistence integration.
Native Excel worksheet Form Controls, binary UserForm import and expanded control
support are deferred outside this delivery profile; see the
[Office forms and controls plan](office-forms-controls-implementation-plan.md).
Classify specimens needing those capabilities as UNSUPPORTED/NOT_RUN rather than
expanding M7 into their implementation. Existing `.tvform` acceptance stays scoped
to its supported shared/host workflow.

- **M7.1:** Curate licensed, reviewed Excel workbooks and Word documents with
  declared language/object/reference/standard-control requirements and independently
  established expected results. Keep candidate/unrun material separate.
- **M7.2:** Report load, compile, run and save separately for unchanged source.
  Verify source identity, typed values/errors, effects/events, code names,
  package relationships/payloads, trust and lifecycle for .xlsm/.docm workflows.
  Include M6.9–M6.12 attribute retention and code-view/raw-source mapping:
  unchanged headers/member metadata survive load and plain-source export,
  and code edits preserve attached hidden records. Report these separately
  from original-package preservation and edited binary writeback.
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
  Exercise the adopted implementation contracts and record
  PASS/MISMATCH/UNSUPPORTED/NOT_RUN per case, with each expectation’s basis
  (authored contract, public specification or inherited observation). Fresh
  comparisons against Excel/Word reference builds belong to M8.5; do not label
  these M7 checks as fresh Office observations or change precedence from an
  unverified assumption.

Acceptance: per-document profile/version/evidence records distinguish PASS,
MISMATCH, UNSUPPORTED and NOT_RUN. Verify declared supported forms/controls, source
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
- **M8.5 — Fresh Excel/Word comparison (deferred from M6.4):** collect permitted
  black-box observations against named Microsoft Excel and Word builds. Compare
  the shared engine and real fork workflows for module-context/name resolution,
  host events, attributes/code views and applicable unchanged-document/package
  cases from M7. Record source identity, Office build/OS/locale, exact types,
  values/errors, side effects, event order and raw results. Keep authored,
  inherited and fresh evidence distinct. Until Office is available these checks
  remain NOT_RUN; they block this release-validation gate, not M6 sign-off or M7.

Acceptance: the declared release profile meets its applicable M1–M6, M6A and M7 gates,
retains prior regressions and reports PASS, MISMATCH, UNSUPPORTED,
BLOCKED_BY_POLICY, INVALID_FIXTURE, CRASH, TIMEOUT and NOT_RUN accurately.
M8.5 must be satisfied for the declared release comparison profile; unavailable
Office comparisons remain an explicit release gate. No fresh Office result may
be inferred from public documentation or inherited oracle records.

## Ongoing implementation rules

Use independent Rust code, public documentation, lawful observations and licensed
fixtures/assets. Keep downloaded evidence ignored and provenance/notices tracked.
Never execute unreviewed macros or import another interpreter's implementation.
Keep the existing reviewed crate graph and headless native/WASM boundaries.
For each change, run the repository's required CI/WASM gates and affected UI/host
checks; update [progress](implementation-progress.md), [compatibility](compatibility-matrix.md)
and the owning contract/ADR. The [UI scope index](ui/post-m8-roadmap.md) points
to the remaining UI/host requirements without duplicating this plan.
