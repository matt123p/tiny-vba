# Remaining implementation plan

Updated 2026-10-29. This plan contains **remaining work only**, using stable M4–M8 IDs
in implementation order after completed M1 and the [selected M2 profile](m2-acceptance.md). Completed work and the old-to-new milestone mapping
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
| M4 | Publish/pin the implemented identity/value/member prerequisite and close acceptance | Implemented M1–M3; explicit feature-branch push approval |
| M5 | Document modules, events and Range-aware UDFs | M2–M4 capabilities required by each workflow |
| M6 | Remaining GridCraft and WordCraft integration | M1; M2–M5 for corresponding form/event/Excel workflows |
| M7 | Unchanged-document and package acceptance | Applicable M1–M6 capabilities and explicit writeback design |
| M8 | Expanded conformance, hardening and release | M1–M7 acceptance for the declared release profile |

Implement shared language foundations before consumers, range references before
range-dependent value/event behavior, and runtime semantics before form events.
M1/M2 shared prerequisites are implemented for their declared profiles. WordCraft's
object-model work in M6 can proceed alongside Excel work; its document/form events
consume the shared M2 contract.
Build tests and collect lawful evidence throughout every milestone. M8 is the
final release gate, not a reason to postpone fuzzing, benchmarks or regression tests.

## M4 — Excel identity, values and member semantics: publication gate

M3.1–M3.3 and the M4.1–M4.3 code are implemented and audited in GridCraft.
The [completion/acceptance record](m3-m4-host-acceptance.md) records exact behavior,
validation and remaining profile limits. Completed implementation tasks belong in
[progress](implementation-progress.md); M5–M8 IDs remain unchanged.

Remaining: publish tiny-vba feature branch `feat/m4-getter-alias-dispatch`
(commit `22cc845e5ee74a1bcd15eec32cdb8a8f18d316f8`), pin that immutable revision
in GridCraft and remove its temporary local development override. Automatic
approval review rejected the push because external repository export was not
explicitly authorized. Approval has been requested; it is not inferred from
elapsed time. No main-branch merge is required.

Acceptance closes only after the immutable dependency resolves and the recorded
native/WASM checks pass without the temporary override. Local implementation
success does not mark this publication gate complete.

## M5 — Document modules, events and Range-aware UDFs

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

Acceptance: unchanged authored handlers and UDFs execute against real GridCraft
state. Pause/inspect/resume, exactly-once dispatch, reset/close/trust revocation
and calculation dependencies pass. No untrusted automatic open execution.

## M6 — Remaining GridCraft and WordCraft integration

Owner: application forks for Office objects, UI/storage/trust/events/undo/recalc;
tiny-vba for shared embedding APIs. M1 supplies generic types; M2 supplies shared
classes/forms. Excel workflow acceptance consumes the applicable M3–M5 behavior.

- **M6.1:** Implement the selected Word object/metadata profile, live ranges and
  character units, bridge and document roots in WordCraft. Define exact supported
  signatures, conversions, lifetime and error behavior with authored workflows.
- **M6.2:** Embed and verify the shared editor in WordCraft with real document
  projects, host theme/fonts/shortcuts, source save/return routing and catalogue
  metadata. Extend both forks' coverage to the new document/form/event workflows.
- **M6.3:** Verify real workbook/document mutations, waits, pause/edit/resume,
  event ordering, abnormal-stop/reset/close cleanup and the host's undo/recalc/
  partial-side-effect policy. Define Word document events against M2 contracts.
- **M6.4:** Add independently authored document workflows and permitted
  reference-build observations. Record each owning fork's executable evidence.

Acceptance: both forks reuse the same editor and shared interpreter repository.
Real document workflows verify effects and lifecycle without replay or retained
locks; tiny-vba-only snapshots and generic doubles cannot satisfy this gate.
Package round trips and signatures are the next milestone's separate gate.

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
