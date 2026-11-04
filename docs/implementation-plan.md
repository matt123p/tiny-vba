# Remaining implementation plan

Updated 2026-11-04. This plan contains **remaining work only**: M7A followed by stable M8 IDs,
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
| M7A | Native Office forms and removal of .tvform | Accepted M1–M6, M6A and M7; reviewed native-form/writeback contracts |
| M8 | Expanded conformance, hardening and release | Accepted M1–M6, M6A, M7 and M7A profiles for the declared release profile |

M7 remains complete for its historical authored profile. Owner direction now
inserts **M7A before M8**, activates native Office forms/controls and requires
removal of `.tvform`; see [ADR 0038](decisions/0038-native-office-forms-and-compatibility-only.md).
Tiny-vba must mirror applicable Excel/Word behavior and native formats. Do not
add custom user-facing language features, formats or control semantics. Unsupported
Office behavior stays explicit until implemented and verified.
Fresh Office comparisons remain NOT_RUN under M8.5. Conformance/hardening can
continue alongside M7A, but release acceptance depends on M7A completion.

## M7A — Native Office forms and removal of .tvform

Status: **PLANNED / IMPLEMENTATION NOT STARTED**, added 2026-11-04 on owner direction.
Owner: tiny-vba reusable form codecs/model/runtime/editor; GridCraft worksheet
controls/Excel bindings; both forks' document, trust, event and package integration.
The [activated work breakdown](office-forms-controls-implementation-plan.md)
retains FC0–FC5 IDs and adds FC6 for removal. M7A IDs below stay stable.

- **M7A.1 — Inventory and contracts (FC0):** audit actual import/designer/runtime/
  persistence paths and Office reference behavior. Publish a per-control/property/
  event/format/host matrix, native fixture rights and provenance, supported platform
  behavior, capability versions, dependencies and acceptance ledger. Cover UserForms,
  worksheet Form Controls and applicable Office-supplied worksheet ActiveX controls
  separately. First implementation task: FC0.1 inventory, then the native worksheet
  Button slice; native UserForm codec work can proceed after its contracts are ready.
- **M7A.2 — Native UserForm codecs and attachment (FC3.1–FC3.4):** bounded, inert
  `.frm`/`.frx` and embedded designer-resource import; preserve authored source,
  component/code names, resources and control hierarchy. Attach native forms to
  shared View Object/View Code, Properties and debugger workflows in standalone,
  GridCraft and WordCraft. Native export and embedded import have separate evidence.
- **M7A.3 — Worksheet forms and controls (FC1–FC2):** implement the worksheet Button
  and remaining standard Form Controls with actual Excel geometry, properties,
  macro assignments, linked cells/list ranges, input, trust, event ordering and
  save/reopen behavior. Use real fork commands and undo/recalculation paths.
- **M7A.4 — Office form/control semantics (FC3.5/FC4):** reuse the shared runtime
  behind native forms and close Office compatibility gaps for the inventoried
  standard controls. Match instance/default-instance behavior, lifecycle, modal/
  modeless behavior, focus/tab order, properties, bindings, events, coercion and
  errors. Verify explicit multi-instance creation against Office rather than
  retaining a custom restriction as compatibility. No new widget or custom semantics.
- **M7A.5 — Native editing and persistence (FC5):** implement native `.frm`/`.frx`
  save/export and required edited embedded form/source-resource writeback in
  `.xlsm`/`.docm` workflows. Review and implement the necessary encoder/package
  path under ADR 0037, including cache consistency and signature invalidation.
  Prove native edit/save/reopen with source/resource identity, revision checks,
  atomic failure and lifecycle integrity. Retaining only original bytes is not
  an edited-save PASS; a custom format is not a substitute for native persistence.
- **M7A.6 — Remove .tvform (FC6):** remove the custom codec, public APIs, import/
  export/save routes, file associations/filters, examples and current support claims.
  Replace demonstrations and useful tests with native Office fixtures/workflows.
  Do not keep a shipped legacy importer or introduce a conversion feature. Historical
  acceptance remains traceable and is marked superseded for format direction.
- **M7A.7 — Acceptance:** prove load/compile/display/input/run/edit/save/reopen
  independently against the declared native inventory in both forks; test malformed
  resources, unsupported controls, bounds, trust loss, Reset/close, debugger waits,
  cancellation, events, undo/recalculation and signature-loss policy. Run shared
  CI/WASM/WASI/UI/native/browser and affected fork gates. Check that no shipping
  `.tvform` support remains. Preserve the M7 regressions and add native-form package
  cases; M7's unsigned, form-free fixtures alone cannot satisfy M7A.

Acceptance: supported native forms execute unchanged authored handlers and survive
native editing/export/package save/reopen with verified source/resource ownership.
The control/host matrix reports remaining gaps accurately. Product behavior and
formats follow Excel/Word; `.tvform` is removed from shipping workflows. Authored,
specification and fresh Office evidence remain distinct; required fresh comparisons
are tracked in M8.5. No arbitrary COM/OCX/DLL loading, P-code execution, protection
bypass or third-party control support is implied.

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
