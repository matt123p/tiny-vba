# Remaining implementation plan

Updated 2026-11-08. This plan contains **remaining work only**: stable M8 IDs and deferred P9,
after completed M1–M6, M6A, M7 and common-profile M7A.
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
| M8 | Expanded conformance, hardening and release | Accepted M1–M6, M6A, M7 and M7A profiles for the declared release profile |
| P9 | Extended/uncommon Office forms compatibility | After M8; finite follow-up packets, not a prerequisite of common-profile M7A/M8 |

M7 remains complete for its historical authored profile. Owner direction now
inserts **M7A before M8**, activates native Office forms/controls and requires
removal of `.tvform`; see [ADR 0038](decisions/0038-native-office-forms-and-compatibility-only.md).
Tiny-vba must mirror applicable Excel/Word behavior and native formats. Do not
add custom user-facing language features, formats or control semantics. Unsupported
Office behavior stays explicit until implemented and verified.
Fresh Office comparisons remain NOT_RUN under M8.5. Common-profile M7A acceptance is complete; release acceptance now depends on
M8 conformance/hardening and its declared evidence requirements.

M7A.1–M7A.7 are **COMPLETE** for the owner-approved common profile under
[ADR 0100](decisions/0100-common-native-forms-acceptance.md). See the
[acceptance matrix and limits](m7a-acceptance.md), reviewed selection/execution
records and [completion evidence](implementation-progress.md) for native forms,
worksheet controls, explicit native file/edited package copies and removal of
the custom format. Ordinary document Save retains original VBA bytes. Remaining
uncommon/exhaustive compatibility belongs to P9 after M8; fresh Office comparison
remains M8.5 NOT_RUN. Next is M8; task IDs below stay stable.

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
  cases from M7 and native forms/controls from M7A. Record source identity, Office
  build/OS/locale, exact types,
  values/errors, side effects, event order and raw results. Keep authored,
  inherited and fresh evidence distinct. Until Office is available these checks
  remain NOT_RUN; they block this release-validation gate, not M6 sign-off or M7.

Acceptance: the declared release profile meets its applicable M1–M6, M6A, M7 and M7A gates,
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

## P9 — Extended Office forms compatibility

Status: **PLANNED**, deferred by owner direction on 2026-11-08. P9 is the owner's
requested label; existing M1–M8 task IDs remain stable. It follows M8 and does not
block native common-profile persistence/removal/integration or the declared M8
release gate. [The P9 backlog](p9-forms-compatibility.md) preserves every migrated
outstanding checklist ID and partial evidence.

- **P9.1:** Unusual aliases/default reloads and mutation/enumeration parity.
- **P9.2:** Exhaustive reentrant lifecycle, activation, focus and validation cases.
- **P9.3:** Platform/IME/DBCS, repeat/modifier/accelerator and remaining mouse cases.
- **P9.4:** Advanced/uncommon properties, AutoSize, scrolling/placement, fonts,
  appearance, selection and remaining member/default/resource audit.
- **P9.5:** Additional binding value types/shapes, update timing and reentrancy.
- **P9.6:** Further container/tab/page/list/combo/numeric contracts.
- **P9.7:** Further picture and RefEdit/platform contracts.
- **P9.8:** Office-supplied worksheet ActiveX in its owning fork.

Each packet needs a finite acceptance contract and appropriate native/WASM,
debugger/trust/lifetime, UI and host evidence. Unsupported behavior stays explicit;
no complete MSForms parity or P9 implementation is claimed. Third-party controls
and arbitrary COM/OCX/DLL loading remain excluded. M8.5 comparisons for the current
release profile stay required; later P9 extensions carry their own evidence.
