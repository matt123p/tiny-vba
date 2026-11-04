# Documentation index

M1–M6, M6A and M7 are complete for their declared profiles; the
remaining plan starts at M8. Fresh Excel/Word comparison is
deferred to M8.5. The plan retains M1–M8 numbering with inserted M6A. Earlier implementation reports,
specifications and ADRs retain historical IDs; see the
[mapping](implementation-progress.md#plan-renumbering--2026-10-28).

[M7 authored package acceptance](m7-acceptance.md) records separate load/compile/run/save,
source identity and module-context evidence. [ADR 0037](decisions/0037-m7-package-preservation-and-writeback.md)
separates original-project preservation from future edited-source writeback and signatures.
The optional `cargo xtask m7` gate verifies reviewed hashes and runs real fork workflows.

Read [architecture](architecture.md), [implementation plan](implementation-plan.md),
[scope decision](decisions/0001-repository-scope.md) and [progress](implementation-progress.md) first.

[VBA intrinsic functions — M6A](vba-intrinsic-functions.md) records the audited
inventory, worksheet counterpart matrix and per-function semantics. [M6A acceptance](m6a-acceptance.md)
records the implemented profile and validation; [ADR 0033](decisions/0033-intrinsics-before-package-acceptance.md)
records insertion before M7.

[M6A implementation framework](m6a-implementation-framework.md) records the
code extension points and packet design used to implement the milestone.
[ADR 0034](decisions/0034-intrinsic-implementation-framework.md) records the design;
the [M6A work ledger](m6a-work-ledger.md) records completed packet results.

[Deferred Office forms and controls implementation plan](office-forms-controls-implementation-plan.md)
records worksheet Form Controls (including the missing Food.xlsm button), native
UserForm import, broader shared controls and persistence. It is outside the current
M7–M8 delivery profile and has not started.

[VBA attributes and Excel code-view investigation](vba-attributes-and-editor-projection.md)
documents hidden module/member metadata, recorded-macro attributes, the blank
`ThisWorkbook` code view, related declarations and the now-implemented M6.9–M6.12 fixes.
[UI ownership decision](decisions/0002-editor-debugger-ui.md),
[editor/debugger UI contract](editor-debugger-ui.md), [developer guide](developer-guide.md),
[debugger](debugger.md), [conformance](conformance-methodology.md)
and [compatibility matrix](compatibility-matrix.md) describe development contracts and current limits.

[M0 contract decision](decisions/0003-m0-source-scenarios-foreign.md) and
[scenario format](scenario-format.md) document the implemented foundations and fixture validator.

[Bytecode and VM specification](bytecode-vm-specification.md) defines the adopted IR 1 instruction,
verification and execution contract; [ADR 0004](decisions/0004-bytecode-vm-contract.md) records the choices.
M2 IR/binding/lowering is implemented; the documented VM execution subset is implemented at M3.
[M2 compiler contract](compiler.md) and [ADR 0006](decisions/0006-m2-binding-and-verification.md) record the current API, budgets and evidence.

[M3 runtime/debugger contract](runtime.md) and [ADR 0007](decisions/0007-m3-execution-and-debugger.md)
record supported execution, generic host continuations and identical native/WASM acceptance.

[Agent design skills](agent-skills.md) lists the shared Codex/OpenCode skills and their task routing.

## Supplied design inputs (preserved)

- [Interpreter implementation specification](vba-interpreter-rust-implementation-spec.md), originally in tiny-vba/docs/.
- [Combined project overview](design/VBA_Combined_Project_Overview.md), copied unchanged from workspace docs/.
- [Office reference and generation plan](design/office_api_reference_and_generation.md), copied unchanged from workspace docs/.

All three available source documents were read during setup on 2026-10-26.
They contain historical repository names and proposals. Current scope is the tiny-vba engine, reusable editor/debugger UI and standalone IDE, plus
compatibility layers and document integration in the Gridcraft and Wordcraft forks. The current decision records
resolve those differences; original documents are not silently rewritten.

The inputs link `office_automation_architecture.md`, but it was absent from both supplied
docs folders. Its detailed Office design is an input gap for fork integration, not a
prerequisite for standalone interpreter development. Other relative links in original
snapshots may also refer to historical locations; use this index to locate available files.

M1 implementation: [frontend subset and API](frontend.md),
[adopted frontend contract](decisions/0005-m1-frontend.md), and
[reviewed external grammar results](m1-corpus-report.md).

[M2/M3 acceptance audit](m2-m3-acceptance-audit.md) records the gate review and
[VM instance isolation](decisions/0008-vm-instance-isolation.md) correction for reused caller labels.

[M4 profile and verification hardening](decisions/0010-m4-profile-and-verification-hardening.md)
records the current consumer profile, truthful fixture outcomes and corrected checkout selection.

[M4 language profile and IR 2](m4-language.md) specifies current semantic support, feature gates,
error handlers, arrays and persistent state. [ADR 0011](decisions/0011-m4-language-extension.md)
records the extension and shared default/constant conversion dependency.

[M4 oracle acceptance report](m4-oracle-report.md) and [ADR 0012](decisions/0012-m4-oracle-acceptance.md)
record the completed external gate, schema 2 provenance and IR 3 extension.

[M5 native host contract](decisions/0013-m5-host-contract.md) records the versioned
foreign/value/debug APIs, hidden IR 4 host-root imports and the reusable bridge driver;
[M7 bounded callbacks and lifecycle](decisions/0015-m7-callbacks-and-lifecycle.md) records the
generic callback queue, static reset and cleanup ownership; [runtime](runtime.md) and
[the compatibility matrix](compatibility-matrix.md) carry the implemented surface.

[M6 read-only project import](project-import.md) and [ADR 0014](decisions/0014-m6-project-decoding.md)
record the bounded decoder, reviewed external dependencies, source identities, protection refusals
and controlled native/WASI acceptance.

## M8 UI implementation

- [User-supplied public-source UI specification](ui/craft_vba_editor_debugger_ui_spec.md)
- [Implementation plan](ui/m8-implementation-plan.md), [ADR 0016](decisions/0016-m8-embeddable-editor.md)
- [M10 implementation and acceptance map](ui/m10-implementation.md)
- [ADR 0018: classic tools and debugger policy](decisions/0018-m10-classic-tools-and-debugger-policy.md)
- [Remaining UI/host scope: standard Office forms and integration](ui/post-m8-roadmap.md)
- [ADR 0019: remaining plan scope and numbering](decisions/0019-remaining-plan-scope.md)
- [New M1 host-reference implementation log](m1-host-references.md)
- [M1.2/M1.3 default members, collections and acceptance](m1-defaults-and-collections.md)
- [M2 class/form staged implementation](m2-classes-and-forms.md)
- [ADR 0023: project classes and form execution](decisions/0023-project-classes-and-forms.md)
- [ADR 0022: generic defaults and enumeration](decisions/0022-default-members-and-enumeration.md)
- [ADR 0020: explicit host reference snapshots](decisions/0020-explicit-host-reference-snapshots.md)
- [ADR 0021: nominal host object identity and validation](decisions/0021-nominal-host-object-types.md)
- [Run, embed and reproduce visual evidence](ui/embedding.md)
- [Acceptance and broader-spec feature status](ui/m8-acceptance.md)
- [M8 completion audit: corrections, regression coverage and verification](ui/m8-completion-audit.md)
- [Public source manifest](provenance/vbe_sources.yaml), [dependency/font provenance](provenance/ui-dependencies.json)

Downloaded reference evidence is ignored under `.cache/evidence/`; original rendered PNGs are
ignored under `reports/ui/`. Neither is required for the ordinary offline test suite.

- [M2 selected class/form acceptance and exchange format](m2-acceptance.md)
- [ADR 0024: standard form execution and exchange](decisions/0024-standard-form-execution-and-exchange.md)

[M3/M4 fork acceptance and publication status](m3-m4-host-acceptance.md) records
the current host milestones separately from historical language milestone IDs.
[M5 document-event and Range-aware UDF acceptance map](m5-acceptance.md) records the
independently authored GridCraft scenarios accepted for the declared profile.

## Module-context and name-resolution specifications (2026-10-31)

- [VBA module types and execution contexts](vba_module_types_and_execution_contexts.md):
  module kinds, `Me`/instance identity, entry-point context preservation and a
  differential compatibility matrix.
- [Excel context, name resolution and object discovery](vba_excel_context_and_object_resolution.md):
  staged identifier resolution, host globals versus `Me`/`With`, active versus
  code context and compatibility tests.

The completed M6.5–M6.8 work checked these specifications, implemented shared
binding/fork context behavior and added shared host-object member completion;
see the [M6.5 check](m6-context-resolution-check.md). The completed M7.4
[unchanged-document matrix](m7-acceptance.md#both-supplied-matrices--m74) records
per-case authored results and explicit unsupported profiles. Fresh Office
comparison remains M8.5 in the [remaining plan](implementation-plan.md).

[M6 host-event subscriptions](decisions/0029-host-event-subscriptions.md) defines
bounded host-neutral delivery to nominal WithEvents fields. Word Application event
names and document effects remain fork-owned; Office reference observations are
separate from authored acceptance.

- [M6 acceptance audit](m6-acceptance.md): shared/fork evidence and unavailable Office observations.

- [ADR 0031: scalar Variant For controls](decisions/0031-variant-for-counters.md).

- [ADR 0032: M6 sign-off and deferred Excel/Word comparison](decisions/0032-m6-sign-off-and-reference-comparison-deferral.md).
