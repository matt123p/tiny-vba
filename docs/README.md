# Documentation index

M1–M6, M6A and M7 are complete for their declared profiles; the
remaining plan starts at M7A, then M8. Fresh Excel/Word comparison is
deferred to M8.5. The plan retains M1–M8 numbering with inserted M6A and M7A. Earlier implementation reports,
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

[M7A native Office forms/control breakdown](office-forms-controls-implementation-plan.md)
activates native UserForms, worksheet controls and native persistence before M8,
and requires removal of `.tvform`. [ADR 0038](decisions/0038-native-office-forms-and-compatibility-only.md)
records the owner-directed compatibility-only policy. M7A is in progress: the [inventory/ledger](m7a-inventory.md) and
[ADR 0039](decisions/0039-m7a-native-control-metadata.md) record the audited paths
and partial native worksheet Button metadata reader. M7A.1/M7A.2 are complete for
the selected native UserForm import/attachment profile under
[ADR 0040](decisions/0040-native-userform-import-and-attachment.md) and the
[control/format/host contract matrix](m7a-control-contracts.md). Native UserForm edited saving
and `.tvform` removal are still open; current custom-format code remains legacy.
[Native MultiPage/Page](decisions/0058-native-multipage.md) extends the current
M7A.4 runtime; the full milestone remains in progress.

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

M7A.3 (2026-11-04): GridCraft implements the nine standard native worksheet Form
Controls for [worksheet-form-controls-v1](m7a-worksheet-controls.md).
Model/engine/XLSX own state, commands and packaging; the optional adapter/shell own
VBA bindings, trusted launches and input. No shared crate/dependency edge changed.
Native UserForm semantics/persistence and .tvform removal remain M7A.4–M7A.7.

M7A.4 has started under [ADR 0042](decisions/0042-userform-instance-routing.md).
The [first instance packet](m7a4-userform-instances.md) removes the
predeclared-only restriction in shared tiny-vba. FormInstance keys separate native
runtime objects from designer/source IDs; UI input/window scopes use those keys.
Read-only VM class queries never instantiate objects; active and parked receivers
remain lifetime roots. Keep Reset/source/resources inert. Broader lifecycle,
control/property packets, fork integration and fresh Office evidence remain open.

M7A.4 now also includes the [native runtime increment](m7a4-native-runtime.md)
under ADR 0043: six flat control families, selected property/input semantics,
Controls/UserForms lookup and bounded transient project inspection. Canonical
VM state remains authoritative; queries never load forms or invoke getters.
Keep intrinsic providers/wait routing intact and do not extend .tvform. The
milestone remains in progress with its remaining requirements recorded there.

[ADR 0045](decisions/0045-class-and-native-collection-enumerators.md) adds IR 9
class enumerators and native Controls/UserForms For Each, including retained
collection ownership and active/parked snapshot lifetimes. The broader M7A.4
remaining scope stays in the [runtime record](m7a4-native-runtime.md).

[ADR 0046](decisions/0046-lazy-project-class-variables.md) adds scalar project-class
and native-form As New under IR 10 `auto-instancing-v10`, including module/static/
instance storage, canonical ByRef preparation and constructor Stop/failure/retry.
Reads construct only when Nothing; declaration/Set targets and inspection remain
inert. Arrays and external activation refuse explicitly. Native designer launch
constructs non-predeclared forms explicitly. Broader M7A.4 requirements remain open.

[ADR 0047](decisions/0047-native-forms-nominal-types-and-library-access.md) adds
IR 11 nominal class interfaces with bounded event exposure. The native Forms
provider binds MSForms form/control/collection/return types and qualified
VBA.UserForms through explicit source-function aliases; scope shadows and missing
references remain authoritative. Ordinary canonical values/frames are retained.
This is not full native lifecycle/event/control parity or edited persistence.

[ADR 0048](decisions/0048-native-container-focus-and-validation.md) adds selected
flat native ActiveControl/SetFocus and container Enter/Exit/BeforeUpdate/AfterUpdate
with typed mutable cancellation, ordinary frames and authorized UI focus/input.
Programmatic writes do not schedule update events. Tab/Shift+Tab and pointer focus
follow canonical VBA state through cancellation and debugger waits. The broader
[M7A.4 scope](m7a4-native-runtime.md) remains in progress.

[ADR 0049](decisions/0049-native-textbox-selection-and-input.md) implements native
TextBox UTF-16 selection, multiline/wrap, Enter/Tab/field behavior and custom
password masking through ordinary classes and authorized atomic input. Native
metadata decoding remains inert; original resources and canonical text are
preserved. The broader [M7A.4 scope](m7a4-native-runtime.md) remains in progress.

[ADR 0050](decisions/0050-native-spinbutton-and-scrollbar.md) adds selected native
SpinButton/ScrollBar decoding, Long properties, nominal interfaces and ordinary
Change/spin/drag-only Scroll delivery. Shared arrows/track/thumb/keyboard input
uses authorized callbacks and canonical state, including reversed ranges and
pause/resume. Full [M7A.4](m7a4-native-runtime.md) remains in progress.

[Native list array loading (ADR 0053)](decisions/0053-native-list-array-loading.md) records the selected bulk-loading/Column packet and remaining array gaps.

[Native array readback (ADR 0054)](decisions/0054-native-list-array-readback.md) and [native metadata/control tips (ADR 0055)](decisions/0055-native-form-metadata-and-control-tips.md) extend the current M7A.4 packet.

[Native Frame containers (ADR 0056)](decisions/0056-native-frame-containers.md) add bounded nested resources, scoped collections and shared native/browser input.

[ADR 0057](decisions/0057-native-tabstrip.md) adds selected native TabStrip
records and separate ordinary Tab/Tabs identity, indexed/named lookup,
enumeration/default selection and authorized widget input. Canonical state stays
in ordinary classes; inert queries/rendering do not instantiate or execute VBA.
Further MultiPage variants, dynamic page/tab mutation/index/name semantics and the remaining M7A.4
ledger stay open. Native edited persistence and .tvform removal remain separate.

[ADR 0059](decisions/0059-native-image-pictures.md) adds selected native Image,
bounded inert BMP/GIF/JPEG/ICO decoding, canonical stdole picture identity and
assignment, display properties and nonfocusable Click. Immutable raster assets
and capped derivative texture caches preserve source/resources and existing VM
budgets. Further picture metadata/platform/format and event/presentation cases,
RefEdit, bindings, dynamic collections and remaining M7A.4 semantics stay open.
Native edited saving/.tvform removal remain M7A.5/.6; Office traces remain NOT_RUN
under M8.5.

[ADR 0060](decisions/0060-native-refedit.md) adds bounded inert RefEdit class-table
resolution, ordinary typed RefEdit text/drop state and selected focus/validation/
Change/DropButtonClick behavior. The shared renderer accepts host range-selection
availability and collapses/restores modal RefEdit; GridCraft owns worksheet
selection and authorization. Original source/resources stay separate. Broader
RefEdit/platform/property/event/binding semantics and the unchanged M7A.4 ledger
remain open. Native edited saving/removal stay M7A.5/.6; fresh Office traces stay
NOT_RUN under M8.5.

[ADR 0061](decisions/0061-native-tab-page-mutation.md) adds native Tabs/Pages
Add/Remove/Clear, writable Integer Index and runtime member Name. Scalar position
cells preserve member/selection identity without owning cycles; inert queries and
page input/rendering follow canonical ordering. Generic VM reference-scan metadata
keeps primitive slots out of object lifetime scans under unchanged budgets.
Dynamic Controls, bindings/multiple selection and the broader M7A.4 ledger remain
open; native edited saving/removal remain M7A.5/.6. Fresh Office traces are NOT_RUN.

[ADR 0062](decisions/0062-native-dynamic-controls.md) implements selected native
Controls.Add/Remove/Clear, runtime names, scoped collections/notifications and
WithEvents focus/validation. An inert runtime projection drives widgets and
input from ordinary canonical objects; imported source/design/resources stay
unchanged. Weak context requests avoid ownership cycles. Shared typed helpers,
indexed verifier metadata and generic dispatch optimizations preserve all existing
VM limits. Full M7A.4, native edited persistence/removal and M8.5 comparisons remain
separate requirements; CI/WASM/WASI validation of ADR 0062 passed.

[ADR 0063](decisions/0063-native-listbox-multiple-selection.md) adds native ListBox
multiple selection, independent focus/selection, row mutation and authorized
pointer/keyboard input. [ADR 0064](decisions/0064-native-runtime-type-names.md) adds
IR 12 inert runtime class names and native TypeName through canonical aliases.
State stays in ordinary classes, rendering stays inert, and budgets are unchanged.
Bindings and the remaining M7A.4 ledger remain open; M7A.5/.6 and M8.5 are separate.

[ADR 0065](decisions/0065-native-cell-and-row-bindings.md) defines native
ControlSource/RowSource through ordinary classes and the existing explicit host
range import. Import and rendering remain inert; GridCraft owns cell commands,
trust and revision-driven callbacks. The owner approved 4,096 static permission
entries on 2026-11-05; per-site authorization and executed-call/work/memory limits
remain in force. Further binding timing/types and the broader M7A.4 ledger remain open.


[ADR 0066](decisions/0066-foreign-runtime-type-names.md) extends existing read-only
host type resolution with bounded native TypeName metadata. Identity/lifetime,
no-getter/no-activation rules, lookup/allocation budgets and generic Object fallback
are retained. TypeName/VarType/IsArray named varname binding matches their documented
signatures. Shared and pinned fork validation passed; the remaining M7A.4 ledger stays open.


[ADR 0067](decisions/0067-vartype-default-value-inspection.md) adds IR 13
inspection-defaults-v13: VarType resolves one compatible public default value
through normal project frames or authorized host calls, preserving errors and
Stop/wait continuations. Public default functions and New value contexts use
ordinary binding. Inert watches, host ownership and existing budgets are retained.
Shared CI, WASM and actual WASI validation pass; the full M7A.4 ledger stays open.


[ADR 0068](decisions/0068-lazy-project-class-array-elements.md) defines lazy project-class/native-form array elements, capability-gated
IR 14 TouchElement, ordinary constructor frames and inert inspection.
