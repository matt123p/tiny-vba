# M7A Office forms — control, property, event, format and host contracts

Updated 2026-11-05. FC0.2/FC0.3 contract inventory under
[ADR 0038](decisions/0038-native-office-forms-and-compatibility-only.md),
[ADR 0039](decisions/0039-m7a-native-control-metadata.md) and
[ADR 0040](decisions/0040-native-userform-import-and-attachment.md).
This completes the inventory/contract task, not all inventoried behavior. Unknown
defaults/errors/event order stay explicit compatibility work, never invented
implementation. [The work ledger](m7a-inventory.md) records executable evidence.

## Native UserForm property packets

In this table D = decoded, P = retained in original resources with a visible
presentation diagnostic, U = unsupported/refused in this import profile. Display,
editing, execution and saving are independent columns below. Import defaults
are file-format defaults from MS-OFORMS, not undocumented VBE toolbox defaults.

| Property / owner | Native representation and selected contract | Initial import |
|---|---|---|
| Component Name / source | PROJECT BaseClass, MODULENAME, VB_Name, VBFrame designer name agree; original MODULESTREAMNAME stays separate | D |
| Control Name / root Controls | OleSiteConcrete.Name, Unicode/compressed Unicode; case-insensitive uniqueness | D; current designer identifier limits enforced |
| Control ID | OleSiteConcrete.ID signed positive 32-bit, separate from name/caption/order | D; no renumbering |
| Left / Top | Site.Position signed HIMETRIC; format default (0,0); shared surface exposes points | D; checked conversion |
| Width / Height | Required leaf Size, signed nonnegative HIMETRIC; form DisplayedSize or explicit frame ClientWidth/Height (twips) | D; shared bounds enforced |
| Caption | Form/Label/CommandButton string; format default empty; conflicting nonempty form/frame values refuse | D; TextBox Caption not admitted |
| Text / Value | TextBox MorphData.Value string; format default empty; TextBox DisplayStyle must be 1 | D; runtime uses existing Text/Value aliases |
| Enabled | VariousPropertyBits bit 1; defaults CommandButton 0x1B, Label 0x80001B, TextBox 0x2C80081B | D; other behavior bits must match declared profile |
| Visible / TabStop | SITE_FLAG bits 1/0; format flags default 0x33; original AutoSize flag retained | D; full focus/AutoSize behavior M7A.4 |
| TabIndex | Signed 16-bit native site index, format default -1 | D; sorted assigned order then native order for ties/unassigned; parity M7A.4 |
| Form Caption / size | Native FormExtraDataBlock + VBFrame; v5 VBFrame caption supported | D |
| Form scrolling / Zoom / Enabled | Import requires unscrolled matching physical/logical size, zoom 100, enabled form | Other values U |
| Font / ForeColor / BackColor / border / effect | Known native property fields, StdFont/TextProps and original resources; no font file acquisition | P; renderer parity M7A.4 |
| Tag / HelpContextID / ControlTipText | Known frame/site strings/scalars; ordinary per-instance metadata and runtime tips | D; Help/F1 routing remains U (ADR 0055) |
| Window placement / StartUpPosition / ShowModal | VBFrame metadata retained; enabled/default-modal form required; explicit show modes use existing runtime | P for placement; disabled/default-modeless metadata U; parity M7A.4 |
| Default / Cancel / Accelerator / TakeFocusOnClick changes | Native site/control flags or fields | Default/Cancel decoded and executed; other fields U |
| Locked / multiline / password / MaxLength / scrolling changes | MorphData behavior fields or nondefault flags | Locked/MaxLength, multiline, password and selected Enter/Tab/field behavior decoded and executed; scrolling remains U |
| ControlSource / RowSource / list metadata | Native binding/site fields | U; host binding packet M7A.4 |
| Pictures / MouseIcon / custom pointers / non-cached class table | Native binary resource or reference-specific class | U; no resource execution/loading |
| Container children | Native immediate sites and separate Frame parent storage | D for selected bounded Frame packet (ADR 0056); other container variants U |

Selected textual properties are strings, control dimensions are exposed as Single
points, and Enabled/Visible/TabStop are Boolean in the existing shared runtime.
Native TabIndex is retained as signed 16-bit metadata; the runtime now exposes
an Integer rank with neighbor reordering and range validation. Form client geometry reaches the designer; additional
root Width/Height and executable MSForms reference/type bindings are runtime
capabilities under ADR 0043/0047, independently of inert import support. Import parses persisted fields, not VBA coercion.
VBA calls retain the
central coercion/error path. Geometry and state not representable by this profile
refuse before attachment; future property error-number comparisons remain M7A.4/M8.5.

## Control inventory and stage matrix

`D/P` = selected native properties decoded with presentation limitations above;
`shared` = existing editable designer/runtime subset attached to native input;
`U` = unsupported; `original` = unchanged package-byte retention only. Every
unlisted property remains outside the selected import/runtime claim.

| Native UserForm control | Property packet to complete | Events to complete | Decode | Display / edit | Execute | Native edited save |
|---|---|---|---|---|---|---|
| Label | Common caption/geometry/state; AutoSize, font, accelerator | Click; mouse/key/focus applicability audit | D/P | shared | Click() | U |
| TextBox | Common geometry/state/Text/Value; UTF-16 selection, typed validation, text limits, multiline/password; binding remains U | Change, Before/AfterUpdate, Enter/Exit, key/mouse | D/P | shared | Change, selected typed Enter/Exit/BeforeUpdate/AfterUpdate | U |
| CommandButton | Common caption/geometry/state; Default, Cancel, accelerator/focus | Click; key/mouse/focus | D/P | shared | Click() | U |
| CheckBox | Value/TripleState, caption; binding remains U | Change/Click, user cycling/Locked; selected Enter/Exit; key/mouse/binding remain U | D/P | shared | selected Value/Change/Click | U |
| OptionButton | Value/GroupName, caption; binding remains U | Change/Click/parent-scoped grouping; selected Enter/Exit; key/mouse/binding remain U | D/P | shared | selected Value/Change/Click | U |
| ToggleButton | Value/TripleState, caption; binding remains U | Change/Click/user cycling/Locked; selected Enter/Exit; key/mouse/binding remain U | D/P | shared | selected Value/Change/Click | U |
| ComboBox | Selected unbound rows/columns, 2D List/Column loading/readback, indexed/current-row Column, Value/Text/style/matching/selection; further list/binding APIs remain U | Selected Change/Click/DropButtonClick and input/focus/update | D/P | shared | selected packet | U |
| ListBox | Selected unbound indexed rows, ListCount/ListIndex/Selected, Value/Text/columns, 2D List/Column loading/readback and indexed/current-row Column; further shape/loading cases, multiple selection and bindings remain U | Selected Change/Click, input/focus/update | D/P | shared | selected packet | U |
| Frame | Caption, child Controls/Parent, ActiveControl, scoped tab/groups | Click and selected authored focus handlers | D/P (bounded parent storages) | shared | Click and selected authored focus handlers | U |
| MultiPage / Page | Pages/Controls/Value/SelectedItem, child identities/layout | Change/indexed Click, selected focus/input | D/P | shared | selected packet (ADR 0058) | U |
| TabStrip | Tabs/Value/SelectedItem, metadata, layout/selection | Change/Click and selected focus/input | D/P | shared | selected packet (ADR 0057) | U |
| ScrollBar | Selected Long Value/bounds/increments/orientation/delay; further resources/properties remain U | Change/drag-only Scroll, selected input/focus | D/P | shared | selected packet | U |
| SpinButton | Selected Long Value/bounds/increment/orientation/delay; further resources/properties remain U | Change/SpinUp/SpinDown, selected input/focus | D/P | shared | selected packet | U |
| Image | Picture identity/assignment, raster resource, PictureSizeMode/Alignment/Tiling, AutoSize and geometry | Nonfocusable Click | D/P | shared | selected packet (ADR 0059) | U |
| Excel RefEdit | Canonical text/selection/drop state; non-cached native class identity | Change/DropButtonClick, selected focus/input; host range selection | D/P | shared; picker supplied by Excel host | selected packet (ADR 0060); GridCraft picker integration PASS | U |

The last row is an Office-supplied Excel extension, not a generic MSForms control
or third-party control. Other Office references/extensions need an explicit
identity/signature/resource packet before admission. Never turn a reference name
into permission to load a library.

## Selected handler and lifetime contract

| Receiver / event | Signature in the selected shared runtime | Native attachment evidence | Remaining behavior |
|---|---|---|---|
| Label / CommandButton Click | Private Sub `<Name>_Click()` | Real native-imported Button handler, modal return, breakpoint/resume | Full native input/focus ordering M7A.4 |
| TextBox Change | Private Sub `<Name>_Change()` | Existing runtime path; native Value initializes without import-time event | Comprehensive native typing/programmatic Change packet M7A.4 |
| UserForm Initialize / Activate / Deactivate / Terminate | Private Sub `UserForm_<Event>()` | Initialize inert during import; explicit Show then once; Reset cancels through existing runtime | Shared explicit instance routing implemented by [the M7A.4 packet](m7a4-userform-instances.md); exact Office order/default-instance parity remains open |
| UserForm QueryClose | Private Sub `UserForm_QueryClose(Cancel As Integer, CloseMode As Integer)`; ByRef | Existing shared cancellation contract reused | Native source/resource lifecycle comparisons M7A.4/M8.5 |
| Native non-label Enter/Exit | Private Sub `control_Enter()` / `control_Exit(ByVal Cancel As MSForms.ReturnBoolean)` | Selected canonical focus and typed cancellation under ADR 0048 | Additional activation/container/redirection observations remain open |
| Native TextBox validation | Private Sub `control_BeforeUpdate(ByVal Cancel As MSForms.ReturnBoolean)` / `control_AfterUpdate()` | User-input update/cancellation on focus transition; ordinary frames and mutable ByVal identity | Broader binding/coercion/update cases remain open |
| Key / mouse / additional events | Microsoft Forms signatures, including ReturnBoolean/ReturnInteger where applicable | U; never approximate event argument objects | Exact typed packet and independent reference evidence required |

Unknown handlers, wrong supported signatures and generated-support/member name
collisions refuse attachment. Native export attributes are retained; generated
runtime support does not duplicate them. Native import does not initialize a
form, enqueue events, compile/run macros, or grant host trust. Existing debugger
and modal continuation budgets/Reset/close behavior remain authoritative.

## Worksheet Form Controls and ActiveX inventory

| Surface | Properties / methods / event mechanism | Decode | Display / edit / execute | Native save |
|---|---|---|---|---|
| Worksheet Button | Native ID/name, caption, point geometry, placement/state, assigned Sub | D with reconciled sources | Native widgets / existing commands / trusted bridge | Edited OOXML/VML |
| CheckBox / OptionButton | Value, linked Boolean/index, mixed check state, radio grouping, assignment | D | Native / live binding / trusted assignment | Edited OOXML/VML |
| ListBox / DropDown | Bounded list/range, one-based selection, simple/extended lists | D | Native / live range / selected ControlFormat | Edited OOXML/VML |
| ScrollBar / Spinner | Value, min/max, increments, linked cell | D within explicit numeric-default profile | Native / undo/recalc / selected ControlFormat | Edited OOXML/VML |
| Label / GroupBox | Caption, geometry, radio grouping / fmlaGroup | D | Native presentation; label text binding remains unsupported | Edited OOXML/VML |
| Office-supplied worksheet ActiveX: CommandButton, Label, TextBox, CheckBox, OptionButton, ToggleButton, ComboBox, ListBox, ScrollBar, SpinButton, Image | Separate OLEObject/control identity, persisted MSForms class, typed properties and sheet code-name event sink; no assigned-macro substitution | U | U | original package graph only |
| Additional Office-supplied ActiveX/container/RefEdit entries | Explicit class/reference/platform applicability audit before scheduling a packet | U | U | U |
| Third-party/custom ActiveX/OLE controls | Outside authorized scope | U | U | U |

Native cell checkboxes are a separate GridCraft cell feature and do not satisfy
worksheet Form Control or ActiveX rows. Worksheet macro resolution, owner sheet,
linked values/list ranges, undo/recalculation and trust are GridCraft-owned.
ActiveX event signatures are not inferred from Form Control assignments.

## Format, platform, owner and acceptance ledger

| Input / workflow | Native representation and identity | Owner / platform | Acceptance boundary |
|---|---|---|---|
| Export `.frm` + optional `.frx` | VERSION 5.00 Office designer CLSID; exact authored source; caller-named companion/offset; checked CFB f/o | Shared decoder; portable native/WASM/WASI | Selected six-family flat subset; distinct export tests and the M7A.4 runtime record |
| Embedded form | BaseClass + MODULESTREAMNAME designer storage + VBFrame + MS-OFORMS f/o | Shared decoder, fork package/trust | Independent embedded and XLSM/DOCM tests |
| Designer/code/Properties/debugger | Caller SourceId -> shared validated design + authored class/event code | Shared IDE/UI, standalone and both forks | Existing UI and runtime; inert attachment and exact handler tests |
| Native export/edited embedded save | Native source/resource encoder, cache/signature policy, revision checks, atomic commit | Shared encoder plus caller IO/package policy | U until M7A.5; no custom-format fallback |
| Worksheet forms | Spreadsheet controls/ctrlProp/VML/DrawingML identities | GridCraft Excel profile | [worksheet-form-controls-v1](m7a-worksheet-controls.md), M7A.3 |
| Worksheet ActiveX | Separate Windows Office object/resource/event model | GridCraft platform packet | U; Office for Mac does not support worksheet ActiveX |
| Rust host builds | Decode without Office/COM installation | Windows/macOS/Linux/BSD and browser/WASI toolchains | Actual tested environments recorded separately, not implied by target list |
| Office observation | Exact Office build, OS, locale, property values/error types, event order, independent lawful specimens | M8.5 | NOT_RUN; public-reference and authored evidence stay distinct |

Capability versions are internal API/acceptance labels, never new exchange formats:
`native-userform-import-v1` (shared) and `worksheet-form-controls-v1` (GridCraft; includes the earlier read-only metadata API).
IR 9–11 metadata extends ordinary class/event/debugger protocols for enumeration,
scalar As New and nominal native interfaces; older IR capability refusal remains. Dependencies:
reviewed cfb/encoding_rs, shared source/design limits, caller IO and fork trust/
package retention. Only a downward IDE -> project edge is added; no host-shaped
objects or resource decoder enter the VM. Future packets must extend native
decoding and this matrix together without changing Office behavior.

## Fixture rights, provenance and sources

All new fixtures are deterministic original Rust builders under repository
MIT OR Apache-2.0. No Microsoft-produced workbook/form, proprietary binary,
artwork/font, downloaded macro or interpreter implementation is included or run.
Core builder: `crates/vba-project/tests/support/native_forms.rs`; the same
authored builder is copied into each fork's self-contained test support. Fixtures
have no network/COM/ambient-IO requirements. Their only handler effects are
Debug.Print and hiding the form. Malformed variants are authored mutations.
Tests separately assert format decode, source/resource identity, designer display,
execution after trust, debugger continuation and original-byte saving. They do
not establish native edited persistence or fresh Office equivalence.

References: [MS-OFORMS](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/9c79701a-8c3e-4429-a139-b60ac3a1d50a),
[MS-OVBA VBFrame](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/76ecef26-5e1c-4729-8074-c76584e25b6b),
[Microsoft Forms objects](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/objects-microsoft-forms),
[properties](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/properties-microsoft-forms),
[events](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/events-microsoft-forms),
and [Excel worksheet controls](https://support.microsoft.com/en-gb/excel/overview-of-forms-form-controls-and-activex-controls-on-a-worksheet).
Exact broader property defaults/signatures/error/event-order observations remain
packet dependencies, not invented compatibility results from this inventory.

M7A.3 is implemented for [the bounded worksheet profile](m7a-worksheet-controls.md)
under [ADR 0041](decisions/0041-worksheet-form-controls.md). This does not add
MSForms/UserForm properties or event semantics, binary project writeback or ActiveX.

M7A.4 first packet (shared): [explicit UserForm instances](m7a4-userform-instances.md)
now have independent ordinary class state, windows and event keys. This supersedes
the M2 predeclared-only restriction. It does not expand the native control decoder,
claim full lifecycle parity or change existing fork dependency pins.

Current M7A.4 additions and exact remaining scope are recorded in
[the native runtime record](m7a4-native-runtime.md) under ADR 0043. Older
three-control/Long-tab-index/predeclared-only descriptions are historical for the
import checkpoint; the runtime record supersedes those restrictions. ADR 0045–0048
also add bounded Controls/UserForms iteration, scalar As New, MSForms interfaces,
VBA.UserForms access and selected native focus/TextBox typed validation. Full native
family/property/binding/platform scope remains required under M7A.4.

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
