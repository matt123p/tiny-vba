# Remaining implementation plan

Updated 2026-11-04. This plan contains **remaining work only**: stable M8 IDs,
after completed M1–M6, M6A and M7 for their declared profiles.
[M7 acceptance](m7-acceptance.md) records the authored unchanged-document/package
profile and [ADR 0037](decisions/0037-m7-package-preservation-and-writeback.md)
records the reviewed binary-writeback/signature contract. Completed task definitions
are retained in [progress](implementation-progress.md#completed-m7-plan-scope).
Historical reports and ADRs keep their original milestone numbers; use this
document's numbers for new work. Task IDs within an active milestone stay stable.

Tiny-vba owns reusable language/runtime, debugger, editor and form contracts.
GridCraft and WordCraft own Office objects, document state, storage, trust,
events, undo/recalculation and application integration. Keep the repositories
separate and native host dispatch in Rust.

## Sequence and dependencies

| Milestone | Remaining work | Prerequisites |
|---|---|---|
| M8 | Expanded conformance, hardening and release | Accepted M1–M6, M6A and M7 profiles for the declared release profile |

M7 completes the reviewed authored source/package profile; edited binary project
writeback is explicitly unsupported under its design contract. Native worksheet
Form Controls, binary UserForm import and broader controls stay in the
[separate deferred plan](office-forms-controls-implementation-plan.md).
Fresh Office comparisons remain NOT_RUN under M8.5; they were not prerequisites
for M7. Continue lawful specimen curation alongside hardening. The next smallest
code task is M8.2's VM host-call payload reservation/accounting review.

## M8 — Expanded conformance, hardening and release

Owner: tiny-vba language/debugger/import evidence; forks own Office behavior and
document results. Evidence work starts alongside M1 and feeds this final gate.

- **M8.1:** Curate further licensed library/corpus cases and independently
  established oracle expectations with exact versions, OS/locale, types,
  errors and side effects. Keep unreviewed/unexecuted material NOT_RUN.
- **M8.2:** Add importer/parser/VM fuzz targets and minimized regressions,
  bounded-work/timeout tests and repeatable native/WASM performance benchmarks.
  **Review VM host-call payload accounting:** foreign-call paths currently
  reserve 64 KiB for reply/snapshot space by charging it to the session payload
  budget. Determine whether that reservation represents live, reusable memory or
  cumulative work; if it is temporary, track peak in-flight payload separately
  from cumulative allocation/work counters and release the reservation when the
  call completes or fails. Add tests proving repeated host calls can reuse the
  reservation while oversized arguments/replies and aggregate work still fail
  within documented limits. Cover pause/resume and failure before host effects.
  Do not raise payload defaults solely to compensate for reusable per-call space.
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
