# Compatibility matrix

Updated 2026-11-05. Stage: M5 document/event/UDF profile accepted against GridCraft; M6 signed off for its tested integration profile (see the [M6.5 check](m6-context-resolution-check.md)); M6A VBA intrinsic coverage accepted for the [declared profile](m6a-acceptance.md), with GridCraft and WordCraft pinned to the published shared revision; **M7 authored unchanged-document/package acceptance is complete for [m7-unchanged-v1](m7-acceptance.md)**; **M7A is in progress: native Button metadata and selected native UserForm import/attachment are implemented; broader semantics, native edited saving and .tvform removal remain open**; M8 release gates remain. **The documented m5-host-v1 consumer, including M3 control, M4 scalar widths and IR 4 host roots, has execution evidence; the 44 authored and 153 reviewed fixtures remain asserted PASS.**

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
