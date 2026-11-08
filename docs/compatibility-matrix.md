# Compatibility matrix

Updated 2026-11-07. Stage: M5 document/event/UDF profile accepted against GridCraft; M6 signed off for its tested integration profile (see the [M6.5 check](m6-context-resolution-check.md)); M6A VBA intrinsic coverage accepted for the [declared profile](m6a-acceptance.md), with GridCraft and WordCraft pinned to the published shared revision; **M7 authored unchanged-document/package acceptance is complete for [m7-unchanged-v1](m7-acceptance.md)**; **M7A is in progress: native Button metadata and selected native UserForm import/attachment are implemented; broader semantics, native edited saving and .tvform removal remain open**; M8 release gates remain. **The documented m5-host-v1 consumer, including M3 control, M4 scalar widths and IR 4 host roots, has execution evidence; the 44 authored and 153 reviewed fixtures remain asserted PASS.**

| Dimension | Status | Evidence / next gate |
| Imported object member dispatch | Implemented binder correction, including getter aliases | ADR 0017/0025; vba-vm/tests/embedding.rs preserves actual Get/Call/Let/Set, named arguments and returned object targets |
|---|---|---|
| Workspace and crate boundaries | Implemented setup | cargo xtask layers |
| CLI parse/check/compile/disasm/run, help/version and explicit unsupported errors | Implemented M3 | apps/vba-cli/tests/cli.rs |
| External source pins and candidate hashes | Implemented setup | strict corpus integrity check; not behavioral validation |
| Source IDs/spans/statements and diagnostic attachment | Implemented M0 | vba-diagnostics/tests/source.rs and vba-syntax/tests/frontend.rs; same-line/control statement mapping |
| Typed scalar states/UTF-16 strings and bounded locations | Implemented M0 foundations | vba-values/tests/locations.rs; shared M3 runtime rules live in vba-runtime |
| Generic foreign ownership/error/wait prototype | Implemented M0 | vba-testkit/tests/round_trip.rs and M3 VM host tests; no Office bridge |
| Scenario validation / 20 independent fixtures | Implemented M0 | vba-conformance/tests/scenarios.rs and CLI; validation alone reports NOT_RUN; assertion results are separate |
| Typed scenario assertion runner (M0 fixtures) | Implemented M4 first slice | `conformance run`: 20 PASS / 0 UNSUPPORTED asserted in CLI tests; [ADR 0009](decisions/0009-m4-runner-and-numeric-widths.md); external corpora stay NOT_RUN |
| Bounded source-preserving lexer/parser | Implemented M1 subset | vba-syntax/tests/frontend.rs; [grammar matrix](frontend.md) |
| External grammar evaluation | Partial: five reviewed files | [report](m1-corpus-report.md); two unsupported, three intentionally broken rejected |
| Scalar module/procedure binding and Option Explicit | Implemented M2 subset | vba-semantics/tests/binding.rs; [exact scope/type/call boundaries](compiler.md) |
| Required positional ByVal / exact local ByRef binding | Implemented M2 subset | vba-compiler/tests/compiler.rs and vba-vm/tests/execution.rs; canonical alias writes/edits proven |
| Checkpoint IR lowering / deterministic source maps | Implemented M2 subset | Answer golden, nested calls, same-line statements, If/For and unreachable-code tests |
| IR 1 verifier / disassembly / capabilities / budgets | Implemented M2 structural contract | vba-ir/tests/verification.rs; all instruction shapes, malformed tables/CFG/stacks and limits |
| Scalar runtime matrix | Implemented M3 subset, extended M4 | [Exact supported combinations](runtime.md), vba-runtime/tests/scalars.rs; `/` and Double widths added at M4 with error numbers 6/11 |
| Gate-root isolation / M4 scalar bounds and report fidelity | Implemented M4 hardening | cross-checkout xtask regression; numeric boundary, failure status and UTF-16/float-bit tests; ADR 0010 |
| Coercion/arrays/builtins/On Error/state | Implemented M4 reviewed profile | [M4 exact scope and IR2](m4-language.md), 24 additional PASS fixtures, numeric/handler/array/state regressions; no oracle claim |
| VBA intrinsic library (M6A) | Accepted for declared profile; policy/platform refusals and provider limits remain explicit | [Inventory/counterpart matrix](vba-intrinsic-functions.md), [acceptance](m6a-acceptance.md), [packet evidence](m6a-work-ledger.md); shared CI/WASM/WASI and pinned fork gates pass; fresh Office observations remain M8.5 |
| Bytecode/VM implementation specification | IR and VM subset implemented | IR 1, ADR 0004/0006/0007 and M2/M3 tests |
| Bytecode execution / generic host continuations | Implemented M3 subset | vba-vm/tests/execution.rs and host.rs; permissions, typed patches, wait/cancel, no replay |
| Versioned native host protocol and source imports (M5) | Implemented M5 contract | `vba_foreign::check_protocol` refuses foreign protocol versions before dispatch (crates/vba-foreign/tests/protocol.rs); `Import`/`validate_imports` bound names to hidden IR 4 host-root slots (crates/vba-compiler/tests/compiler.rs); missing/mismatched/stale roots refuse launch or surface host error 91 without a second write (crates/vba-vm/tests/host.rs, tests/embedding.rs); [ADR 0013](decisions/0013-m5-host-contract.md) |
| Reusable bridge driver for forks (M5) | Implemented generic proof | vba-testkit `run_to_boundary`/`statement`; vba-vm/tests/embedding.rs proves break/edit/resume over an imported root with exactly one dispatch. Real workbook FillCells, undo/recalc and permissions stay fork-owned |
| Document-module code names and host-authorized callbacks (M5.1) | Accepted for the declared profile | IDE retains validated code names separately from source names and imports inertly; VM lazily creates predeclared document instances, queues initialization before the requested callback, and forwards unresolved owner members through the host permission path. Native/WASI regressions and GridCraft workbook tests cover real owner binding, event signatures/order, trust, lifecycle and unchanged-workbook behavior; [M5 evidence](m5-acceptance.md) |
| GridCraft document events (M5.2) | Accepted for the declared profile | GridCraft binds callback `Me` to workbook/sheet owners, exposes explicit trusted Workbook_Open from Design mode and queues Worksheet_Change for reviewed native/VBA edits. Queue limits, EnableEvents, callback-batch cancellation, grouped partial effects, pause/resume, trust/reset/close, direct control and drag-cut are covered by authored GridCraft tests; [GridCraft contract](../../gridcraft/docs/vba.md), [M5 evidence](m5-acceptance.md) |
| Range-aware worksheet UDFs (M5.3) | Accepted for the declared profile | GridCraft preserves direct Range identity and Caller context, exposes read-only Range properties and Volatile, preserves typed Value/Value2 distinctions, persists Static state, tracks dependencies and refuses mutation/asynchronous calls. Authored recalculation and negative-path tests pass against GridCraft workbook state; [M5 evidence](m5-acceptance.md) |
| Module kinds, staged resolution, shadowing, `With` and implicit `Me` members (M6.5/M6.6) | Implemented for the declared profile | [M6.5 per-layer check](m6-context-resolution-check.md) against both supplied specifications; staged bare-name resolution with project-before-global shadowing, host-declared document implicit members, bounded `With` blocks with innermost-only leading dots and no `Me`/active-state mutation. Native and WASI regressions in vba-semantics, vba-syntax and vba-vm (`with_resolution.rs`) |
| Fork host globals, live active context and document roots (M6.7) | Implemented for the declared profile | GridCraft host globals dispatch per invocation against live state with distinct `ThisWorkbook`/`ActiveWorkbook`/`ActiveSheet`/`Me` identities and stable code names; WordCraft document roots under its M6.1 profile. `Application.Run`, add-ins and chart sheets stay refused/NOT_RUN, recorded in the check |
| IDE receiver typing for host objects (M6.8) | Implemented for the declared profile | Host-declared import result types, call-shaped receivers, `Me`/code-name document types and catalogue member walks drive read-only completion/Quick Info; code aids never invoke members. `crates/vba-ide/tests/host_completion.rs` |
| Bounded generic callbacks, static reset, lifecycle cleanup (M7) | Implemented M7 contract | `schedule_callback`/`run_next_callback`/`reset_state`/`rebind_roots` with native/WASI drain, budget, non-reentrancy, permanent closure and entry-restart tests in crates/vba-vm/tests/callbacks.rs; [ADR 0015](decisions/0015-m7-callbacks-and-lifecycle.md); fork-owned event order stays in the forks |
| Breakpoints, stepping, local inspect/edit/resume | Headless API plus M8/M10 native/web UI and optional DAP | VM/controller/model and real UI/process tests; [M10 map](ui/m10-implementation.md) |
| Classic editing, metadata code aids and advanced debugging | Implemented M10 profile | Shared source Undo, scoped literal search, split/procedure editing, browser transfer, read-only watches, local arrays, restricted relocation and opt-in isolated calls; [limits/evidence](ui/m10-implementation.md) |
| Classic editor/debugger panels | Implemented M8 repository gate | [Surface matrix and explicit broader-spec limitations](ui/m8-acceptance.md); independent host themes, source/inspection lifetimes and bounded execution |
| Native / WASM execution | Implemented M3 smoke | cargo xtask wasm-run; identical typed results, pause/edit/host counts and cancellation |
| Replacement-session isolation | Implemented M3 correction | recycled-client-ID regression in VM tests and native/WASI smoke; [ADR 0008](decisions/0008-vm-instance-isolation.md) |
| MS-OVBA source decoding | Implemented M6 bounded profile | [Import contract](project-import.md), [ADR 0014](decisions/0014-m6-project-decoding.md), 14 native/WASI tests; controlled authored projects only; package/trust/preservation gates stay fork-owned |
| Oracle comparison / executable scenario runner | Historical M4 runner and reviewed oracle cases implemented; further acquisition remains planned | See current external acceptance below; new M8 owns expanded conformance |
| Fuzzing | Not implemented | APIs required before targets |
| Excel/Word APIs and OOXML round-trip | External ownership | Gridcraft/Wordcraft forks |
| GridCraft Excel.v1 baseline | Fork-owned implementation | Native and JavaScript/WASM workbook tests, shared editor/control/source IO tests, inspected native snapshots; [scope/evidence](gridcraft-integration.md). Broader events, WordCraft, signatures/forms remain planned |
| Native UserForm import/attachment | M7A.1/M7A.2 native-userform-import-v1 | [Contracts](m7a-control-contracts.md): flat Label/TextBox/CommandButton export + embedded import, exact source/resources, shared designer/debugger/runtime, inert XLSM/DOCM attachment. Native edited saving and broader controls remain unsupported |
| Native worksheet Button metadata | Partial FC1.1 format decoder; no interactive control support | GridCraft native controls/VML/ctrlProp snapshot, 8 native and 8 actual JS/WASM tests, bounds/identity/namespace/conflict/save cases; [inventory](m7a-inventory.md). Full DrawingML reconciliation, rendering, execution and edited native save remain pending |
| Classes and legacy shared form profile | M2 selected profile implemented; native replacement/removal planned in M7A | [Class/designer/runtime/exchange scope and evidence](m2-acceptance.md); Label/TextBox/CommandButton, real modal/modeless events and legacy .tvform persistence. [M7A](implementation-plan.md) now requires native forms/persistence and removal of the custom format under ADR 0038; native import/attachment now follows the separate selected profile below |
| Default members and bounded collections | New M1.2/M1.3 complete for the declared profile | IR 6 value/reference contexts, Get/Let/Set, dynamic arrays/ByRef, For Each, canonical paused edits and native/WASI plus real fork native/JS-WASM workflows; [evidence](m1-defaults-and-collections.md). Excel definitions remain in GridCraft |
| Explicit host references, enum/nominal types and constants | New M1.1 complete for registered types | Qualified types, ordered constants, IR 5 nominal storage/lifetime/ByRef checks and native/WASI tests plus fork-owned native/JavaScript-WASM typed-source proof; [profile](m1-host-references.md). Office definitions stay in forks |
| Third-party ActiveX/OLE controls not supplied with Office | Out of scope for now | No loading, execution, designer or acceptance work scheduled |
| Arbitrary COM/DLL execution, P-code and binary add-ins | Outside current scope | Standard Office control support does not imply unrestricted binary compatibility |
| Binary VBA source writing/signature handling | M7 reviewed contract; required native form/source encoder planned in M7A; still unsupported today | [ADR 0037](decisions/0037-m7-package-preservation-and-writeback.md); original binary and supported opaque VBA signature payloads preserved separately from source exports; package-wide signatures invalidated; real signature verification NOT_RUN |
| Unchanged XLSM/DOCM source and package workflows | M7 authored profile complete | Two reviewed generated specimens; native and actual JS/WASM load/compile/trust/run/save/reopen; hidden attributes/code names/context/events; [acceptance](m7-acceptance.md). No universal Office, binary-form or edited-writeback claim |

M3 execution evidence is restricted to its explicit tests/examples and runtime matrix. It does not establish
workbook/Office compatibility, physical-device fidelity or unselected external corpus pass rates; self-authored
M0 assertions are 20 PASS / 0 UNSUPPORTED and external corpus execution remains NOT_RUN.

## Current M4 external acceptance

[The acceptance report](m4-oracle-report.md) supersedes the earlier authored-only status: 44
authored scenarios and 153 reviewed ReflexGrid cases PASS. The latter carry upstream-reported
Microsoft Excel Mac evidence with exact source hashes and raw results, not a new Office run.
Whole-array/ParamArray/Option Base/named-cell argument tests and IR 3 version/metadata refusal
checks pass. Unselected external cases retain NOT_RUN; future oracle acquisition belongs to new M8.

## Remaining scope

The [remaining plan](implementation-plan.md) uses new M1–M8 numbers; implemented
milestones elsewhere in this matrix retain their historical IDs. See the
[mapping](implementation-progress.md#plan-renumbering--2026-10-28) to distinguish them.
The [UI scope index](ui/post-m8-roadmap.md) assigns shared classes/standard Office
forms to M2 and remaining host/package acceptance to M6/M7. Extensions/translation
and AI assistance are removed from the plan. Planned work is not a compatibility
pass; unsupported statuses require actual execution evidence before changing.

Current host M3/M4 are complete for their declared GridCraft profiles, using
the published immutable shared dependency without development path overrides. See the
[M3/M4 acceptance record](m3-m4-host-acceptance.md) for exact profiles/evidence.

## M6 metadata and event audit (2026-11-01)

| Surface | Implemented profile / refusal |
|---|---|
| Module/member attributes | Owned literal metadata, procedure-local invocation records, signed IDs, bounded class header; no executable checkpoints |
| Metadata semantics | Property Get default supported; enumerators, default fields/functions and unsupported activation requests refuse |
| Code view | Lossless projected editing/debugging/search, raw inspector, code-name Properties and preserved source filenames |
| Document rename | Host validation/acknowledgement API; current forks refuse code-name changes |
| Host WithEvents | Capability-gated live foreign subscriptions with signature/identity validation and bounded ordered delivery |
| Word Application events | Trusted BeforeClose/BeforeSave/WindowSelectionChange in WordCraft; authored workflow evidence |
| Microsoft Office reference observations | NOT_RUN; deferred by owner direction to M8.5; M6 signed off for its tested profile |

See [M6 acceptance](m6-acceptance.md). Binary project writeback, third-party
controls and Excel Macro Options remain outside this implemented profile.

2026-11-01 For-counter correction: explicit/implicit scalar Variant controls now
compile and execute for the existing positive integral literal-Step profile.
Nested loops, Next without a name, Exit For, zero iterations, numeric-string
bounds, catchable invalid bounds and checkpoint resume have authored regressions.
Food.xlsm was inspected read-only to identify its implicit counters; no workbook
macro was executed, and this does not claim its clipboard/sort workflows run.

M7A.4 collection enumeration: Controls/UserForms For Each is implemented under
[ADR 0045](decisions/0045-class-and-native-collection-enumerators.md), including
nested/empty loops, canonical identity and object lifetime through unload/Stop.
IR 9 validates public zero-argument Object-returning class enumerators and rejects
legacy targets. Retained native enumerators own their collections. Qualified
access is implemented by ADR 0047 below. Direct cursor interfaces and structural-
mutation parity remain gaps; no full M7A.4 claim.

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

[ADR 0051](decisions/0051-native-unbound-listbox.md) adds selected native unbound
ListBox decoding, ten-column indexed rows, selection/default Value and ordinary
Change/Click frames. Authorized shared list input uses canonical state and
preserves focus/validation and Stop/resume. Whole-array APIs, further list/binding
semantics and ComboBox remain open with the unchanged broader M7A.4 scope.

[ADR 0052](decisions/0052-native-combobox.md) adds the selected native editable/
drop-list ComboBox packet, shared ordinary list rows, text/selection/matching and
VM-owned drop state. Authorized input preserves focus/validation and debugger
continuations; contextual F4 leaves the editor shortcut intact elsewhere. Locale
limits, further properties/bindings/resources and broader M7A.4 remain open.

[ADR 0053](decisions/0053-native-list-array-loading.md) adds shared native ListBox/
ComboBox two-dimensional List loading, transposed Column loading and indexed/
current-row Column access through ordinary class procedures. Source bounds/types
remain independent; complete validation precedes canonical row/selection changes
and ordinary callbacks. Whole-array reads, additional list/binding semantics and
the broader M7A.4 scope remain open.

[ADR 0054](decisions/0054-native-list-array-readback.md) adds detached zero-based
List/Column array snapshots to the existing unbound ten-column native packet,
preserving selection, Err and ordinary debugger lifetime. [ADR 0055](decisions/0055-native-form-metadata-and-control-tips.md)
promotes native Tag/ControlTipText/HelpContextID into ordinary instance metadata
and renders canonical control tips without executing code or changing focus.
Original resources stay independent; actual Help-file/F1 routing and the broader
M7A control/persistence/removal requirements remain open.

Native Frame containers follow [ADR 0056](decisions/0056-native-frame-containers.md):
bounded native parent storages, immediate child Controls/Parent identity, scoped
TabIndex/option groups and nested ActiveControl/focus cancellation. Canonical state
remains ordinary classes; effective ancestor flags, live relative geometry,
clipping and accessible groups use the shared renderer. Original source/resources
stay inert and unchanged. Further container/page/image/binding/platform semantics,
native edited persistence and .tvform removal remain in M7A; fresh Office traces
remain NOT_RUN under M8.5.

[ADR 0057](decisions/0057-native-tabstrip.md) adds selected native TabStrip
records and separate ordinary Tab/Tabs identity, indexed/named lookup,
enumeration/default selection and authorized widget input. Canonical state stays
in ordinary classes; inert queries/rendering do not instantiate or execute VBA.
Dynamic page/tab mutation/index/name semantics and the remaining M7A.4
ledger stay open. Native edited persistence and .tvform removal remain separate.

[ADR 0058](decisions/0058-native-multipage.md) adds selected native MultiPage
f/o/x reconciliation, distinct Page/Pages identity, immediate child collections,
selection-gated input/focus and actual page rendering. Internal tabs stay private;
canonical state remains ordinary classes and source/resources stay inert. Indexed
Click and repeated-selection behavior also correct the prior TabStrip contract.
Dynamic page/tab mutation, further persistence/property/event/lifecycle cases and
the full remaining M7A.4 ledger remain open. Native edited saving/removal stay
M7A.5/.6; fresh Office traces remain NOT_RUN under M8.5.

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

VarType's named argument follows the documented varname spelling. ADR 0067 adds
its compatible zero-argument default-value evaluation; additional default/arity
comparisons remain M8.5 NOT_RUN. TypeName preserves object identity and never
adopts VarType's getter execution.


[ADR 0067](decisions/0067-vartype-default-value-inspection.md) adds IR 13
inspection-defaults-v13: VarType resolves one compatible public default value
through normal project frames or authorized host calls, preserving errors and
Stop/wait continuations. Public default functions and New value contexts use
ordinary binding. Inert watches, host ownership and existing budgets are retained.
Shared CI, WASM and actual WASI validation pass; the full M7A.4 ledger stays open.


[ADR 0068](decisions/0068-lazy-project-class-array-elements.md) adds IR 14 lazy project-class/native-form array elements.
Fixed/dynamic arrays, Preserve/Erase, private/static storage, canonical ByRef,
source-order capture and constructor/For Each Stop continuations reuse existing
locations, classes and budgets. Whole-array reads and safe inspection stay inert;
external activation and object-module Public arrays remain refused. Shared CI,
WASM and actual WASI pass; the broader M7A.4 ledger and M8.5 comparisons remain open.


[ADR 0069](decisions/0069-userforms-without-form-components.md) enables the
existing UserForms collection in projects without form components. A bounded
code-token hint selects only the minimal ordinary collection/enumerator support;
semantic shadowing, compiled identity, inert inspection, host ownership and
existing budgets remain authoritative. Shared/host acceptance is tracked in
progress; [the completion checklist](m7a4-completion-checklist.md) retains the
full remaining M7A.4 scope. No new IR, format or control library is introduced.

[ADR 0074](decisions/0074-native-text-pointer-selection.md) adds selected native
TextBox pointer caret/drag/word selection after mouse callbacks, cancellation and
focus redirects, with ordinary authorized selection commits. Canonical UTF-16,
CRLF, Stop/Reset and source/resource identity are preserved. Native AutoWordSelect
bit 27 is the inverse of the Boolean property; imported defaults and new dynamic
controls are distinguished. ComboBox/RefEdit pointer defaults, further platform/
word/selection cases and the full M7A.4 checklist remain open.

[ADR 0075](decisions/0075-native-composite-pointer-defaults.md) adds selected
ComboBox/RefEdit text and drop/picker, list/popup row, numeric step/repeat/drag and
indexed tab/page-header pointer defaults after ordinary mouse callbacks. Inert
modal-display metadata is separate from input ownership; popup input inherits the
execution barrier and tab identity is revalidated after mutation. ComboBox native
AutoWordSelect uses its inverse file bit. Canonical state and existing executable
command authorization remain authoritative. Full M01/M7A.4 acceptance stays open.

[ADR 0076](decisions/0076-native-modal-input-continuations.md) parks bounded raw
mouse input by modal caller, so child callbacks can run while their parent waits.
Physical release stays with its original capture, keyboard barriers use the
current callback owner, and Reset/unload discard stale defaults. The shared raw
queue stays at 64, modal scope depth stays bounded by eight existing waits, and
canonical state/receipts remain ordinary VBA. Selected nested lifecycle/input
cases are covered; L02 and the full M7A.4 checklist remain partial.

[Native colours (ADR 0080)](decisions/0080-native-form-colours.md) retain signed
OLE_COLOR metadata and ordinary BackColor/ForeColor fields, with RGB/system-theme
rendering and an injected UI palette resolver. Invalid assignments preserve prior
state; import/source/resources and inert inspection remain separate. Runtime Page
lookup searches the existing MultiPage collections. Font/border/transparency,
ambient/platform appearance and the full M7A.4 checklist remain open.


ADR 0081 adds selected native font metadata, ordinary typed Font objects, copied
container defaults and inert UI snapshots. Font sizes retain Currency precision;
local typography and optional caller-owned face selection preserve global host
styles/fonts. The static nominal metadata ceiling is 8,192, with the existing
64 MiB metadata/work bounds and VM/4,096-permission limits unchanged. The separate
ADR 0073 event regression still stays below 4,096. Replacement/ambient/platform
font behavior and the unchanged full M7A.4 checklist remain open; fresh Office
comparisons remain M8.5 NOT_RUN. See [ADR 0081](decisions/0081-native-form-fonts.md).

Native Font assignment follows [ADR 0082](decisions/0082-native-font-assignment.md):
typed ordinary setters preserve explicit reference identity, old aliases and root
lazy reload; new controls still copy independent defaults. No VM, permission or
budget change is needed. Nothing/external/other-assignment and ambient/platform
font cases and the full remaining M7A.4 checklist stay open.

[ADR 0083](decisions/0083-page-appearance-ownership.md) corrects Page appearance
ownership: no Page Font/BackColor/ForeColor APIs; MultiPage owns defaults/painting.
[ADR 0084](decisions/0084-native-control-appearance.md) retains native border/effect/
background data and adds applicable typed properties, local effects/transparency
and fixed-bound clipping. [ADR 0085](decisions/0085-native-font-attribute-aliases.md)
adds published Font aliases forwarding to canonical objects. Full member, platform
and M7A.4 acceptance remains open; no VM/permission/budget increase is made.

[ADR 0086](decisions/0086-native-text-alignment.md) adds selected native TextAlign
on Label/TextBox/ListBox/ComboBox. Native ParagraphAlign swaps centre/right values
relative to the public enum; ordinary Long state is independent of Font/text/
selection. Inert inspection preserves authored form fields, local layout matches
pointer/caret geometry, and ComboBox popup alignment stays independent. Source/
resources, global styles/fonts and all budgets are unchanged. Broader member,
wrap/AutoSize, Boolean/RefEdit and platform cases and full M7A.4 remain open.

[ADR 0087](decisions/0087-native-caption-word-wrap.md) adds selected native
Label/CommandButton WordWrap flags, typed Boolean state, distinct file/runtime
defaults and fixed-bound captions. Dynamic TextBox defaults to True; multiline
unwrapped text retains fixed horizontal interaction bounds with complete UTF-16
cursor rows. Canonical state, original source/resources, global font/style policy
and all VM/permission budgets remain unchanged. Broader Boolean-control wrap,
AutoSize/scrolling and full M7A.4 acceptance remain open.

[ADR 0088](decisions/0088-native-boolean-caption-wrapping.md) extends selected
native WordWrap flags/defaults and ordinary Boolean state to CheckBox, OptionButton
and ToggleButton. Rendering keeps widget input/accessibility, local bounded
captions and physical ToggleButton centring. Value/Null/groups/events, original
source/resources, global font/style policy and all VM/host budgets are unchanged.
Other properties, Alignment/TextAlign, AutoSize/scrolling and full M7A.4 stay open.

[ADR 0089](decisions/0089-restricted-wasi-callback-forwarding.md) keeps a plain JS
forwarding boundary around native WASI callbacks to avoid a reproduced Node 22
runner crash. Guest bytes, stack, budgets, args/env/preopens and error/exit behavior
remain unchanged; five independent runner guests accompany full native/WASI
acceptance. This does not extend Office APIs or complete M7A.4.

[ADR 0090](decisions/0090-native-caption-alignment.md) adds selected native
CheckBox/OptionButton Alignment: inverse persisted flag translation, ordinary
Long state and published constants, Right defaults, typed errors and inert
inspection. Caption text stays physically left aligned in a separate clipped
rectangle beside the real indicator widget. Input, full accessible labels/state,
Value/Null/groups/events, source/resources, global style and all budgets remain
unchanged. ToggleButton's disabled property is not enabled. Other properties,
AutoSize/scrolling and full M7A.4 remain open; fresh Office evidence is NOT_RUN.

[ADR 0091](decisions/0091-native-hide-selection.md) adds selected native
TextBox/ComboBox HideSelection flags, True defaults and ordinary Boolean state.
Changing the property preserves text, ranges, events and resources. Passive
highlights use canonical UTF-16 ranges and local clipped galley copies, including
both ComboBox presentations; focus/style queries stay outside font locks.
Inspection, global style, font caches and all VM/host budgets remain unchanged.
Other selection/property/AutoSize/scrolling and full M7A.4 work stay open; fresh
Office observations remain M8.5 NOT_RUN.

[ADR 0092](decisions/0092-native-auto-tab.md) adds selected native TextBox/ComboBox
AutoTab: bit-30 import, False defaults and user-edit-driven canonical tab traversal
through existing validation/focus frames. Actual mutation cause survives unchanged
autocomplete text; programmatic assignments and caret-only input stay passive.
Full property/platform/input and M7A.4 acceptance remains open; M8.5 is NOT_RUN.

[ADR 0093](decisions/0093-native-selection-margin.md) adds native TextBox/ComboBox
SelectionMargin, True defaults and a theme-local selection strip. Whole displayed
rows select/drag through the existing mouse continuation, including wrapped text
and canonical UTF-16/CRLF ranges. Post-callback state remains authoritative;
selection-only input stays passive and empty edit fields render blank. No VM,
foreign, budget or persistence change is implied. Full M7A.4 remains open.
