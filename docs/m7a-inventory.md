# M7A native forms — implementation inventory and work ledger

Updated 2026-11-07. **IN PROGRESS**. FC0 and FC3.1–FC3.4 are complete for
`native-userform-import-v1`; see the [control/property/event/format/host matrix](m7a-control-contracts.md).
M7A.3's worksheet profile is implemented under
[ADR 0041](decisions/0041-worksheet-form-controls.md). Shared runtime increments
now include selected native Frames under [ADR 0056](decisions/0056-native-frame-containers.md).
Remaining semantics, native edited persistence, format removal and acceptance
retain their full M7A scope. The audited paths below preserve the initial inventory.

## Audited implementation paths — FC0.1

| Surface / owner | Actual path and current boundary | Required native work |
|---|---|---|
| Shared project decode / tiny-vba | `crates/vba-project/src/lib.rs::decode` retains `/PROJECTlk` refusal; `BaseClass` now yields a distinct UserForm with bounded designer resources | Implemented selected f/o/VBFrame resource/component result; broader controls/resources remain explicit gaps |
| Shared design/runtime / tiny-vba | `vba-ide/src/forms.rs` has eleven selected native families including bounded Frame parents, 128 controls and 2 MiB model limit. `form_project.rs` generates ordinary class/event support separately from authored source | Native input now maps to validated design; instance/lifecycle/property/event gaps remain M7A.4 |
| Shared exchange / tiny-vba | `Ide::import_form` / `export_form` consume the custom archive. Native `.frm/.frx` imports now attach through a caller-supplied bundle; native/browser IO routes accept the pair | Atomic native import implemented; native export and edited-resource encoding remain M7A.5; remove legacy routes in M7A.6 |
| Worksheet package / GridCraft | `xlsx::passthrough::KeptReader::sheet_elements/vml` preserves `controls`, `ctrlProp` parts and VML. New `read_worksheet_controls` decodes a read-only Button snapshot | Finish DrawingML/VML reconciliation before native rendering/activation; preserve source graph separately from future writable state |
| DrawingML shape identity / GridCraft | `xlsx::drawing::read_shape` reads nonempty `macro` into `on_action` but assigns model `id: 0`; source `cNvPr` identity is not retained as a native-control join key | Retain/reconcile native source identity; reject conflicting assignment/representation data; do not join by model sequence number or guessed caption |
| Native cell checkbox / GridCraft | `engine::cmd::extra` and `extra::cells` use cell values/number-format state and normal commands | Do not mistake this implemented feature for a VML CheckBox or worksheet ActiveX implementation |
| Workbook attachment / GridCraft | `vba::Bridge::import_project`, shared IDE and `ui-egui::vba` source routes; imported document names must match live workbook/sheets | Selected native package form attachment implemented; preserve trust/lifecycle ownership and expand semantics in M7A.4 |
| Word attachment / WordCraft | `WordVbaProject::new` now also attaches selected native UserForms inertly. `docx` retains the original project and direct related parts; shared editor routes source requests | Selected native form attachment implemented; edited source/resource save remains M7A.5 |
| Native persistence / all | M7 preserves original bytes. Neither shared editor nor forks encode edited embedded UserForm resources | Explicit native encoder/save path under ADR 0037; cache/signature invalidation, identity/revision checks and atomic file commit |

The audit is based on inspected code, not planned APIs. No crate or dependency
edge is added to the shared graph. Office package objects stay in their forks.

## Initial inventory and current control disposition

The completed FC0.2 [contract matrix](m7a-control-contracts.md) supersedes the
initial seed families below with per-property/event/format/host dispositions.
Additional runtime signatures/defaults/errors/ordering remain per-packet work
in M7A.4; a declared unsupported packet is not implementation evidence.

| Native UserForm control | Property/event families to audit | Current implementation |
|---|---|---|
| CommandButton | Caption/geometry, Enabled/Visible, default/cancel/focus, Click | Selected native decode + shared runtime; edited native save UNSUPPORTED |
| Label | Caption/geometry, formatting, visibility, Click | Selected native decode + shared runtime; edited native save UNSUPPORTED |
| TextBox | Text/Value, geometry, selection, validation, Change/focus/key events | Selected native decode + shared Text/Change runtime; edited native save UNSUPPORTED |
| CheckBox | Value/three-state/group/binding, Change/Click | Native decode/runtime/input; selected cell binding (ADR 0065); native edited save unsupported |
| OptionButton | Value/group/binding, Change/Click | Native decode/runtime/input and parent-scoped groups; selected cell binding (ADR 0065); native edited save unsupported |
| ToggleButton | Value/binding, Change/Click | Native decode/runtime/input; binding/native edited save unsupported |
| ComboBox | Items/selection/columns/binding/style, Change/Click/drop/update events | Selected native editable/drop-list packet and cell/row binding (ADR 0065); further list/binding/resource/lifecycle cases and edited save remain unsupported |
| ListBox | Items/selection/columns/multiselect/binding, Change/Click | Selected native unbound indexed rows/columns, single selection, Value/Text and input; 2D List/Column loading/readback and indexed/current-row Column implemented; multiple selection (ADR 0063) and selected cell/row binding (ADR 0065) implemented; further shape/loading/binding timing and edited save remain unsupported |
| Frame | Caption/container/child Controls/Parent/ActiveControl/scoped grouping and tab/focus | Selected native packet under ADR 0056; further resource/event/property variants remain open |
| MultiPage | Pages/selected Value, child controls, Change | Selected native f/o/x joins, Page/Pages identity, lookup/enumeration, selection-gated child input/focus/rendering (ADR 0058); selected Add/Remove/Clear/Index/runtime Name (ADR 0061); broader event/resource contracts open |
| TabStrip | Tabs/selected Value, Change | Selected native decode, ordinary Tab/Tabs, lookup/enumeration/selection/input; selected Add/Remove/Clear/Index/runtime Name (ADR 0061); further APIs/events/presentation remain open |
| ScrollBar | Value/range/increments/orientation, Change/Scroll | Selected native records, Long properties, reversed ranges and input/drag-only Scroll; further resources/properties/save remain unsupported |
| SpinButton | Value/range/increments/orientation, Change/spin events | Selected native records, Long properties and input/Change/spin; further resources/properties/save remain unsupported |
| Image | Native picture resource/geometry/display properties and applicable input events | Selected bounded native raster, picture identity/assignment, layout and Click (ADR 0059); further formats/platform/metadata/events/presentation remain open |
| Excel RefEdit | Native class-table identity, text/reference, drop and host selection | Selected inert decode and ordinary text/drop runtime (ADR 0060); GridCraft picker/authored integration PASS; broader host/platform/property/event semantics remain open |

Inventory basis: Microsoft's [Forms object list](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/objects-microsoft-forms),
[properties index](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/properties-microsoft-forms)
and [events index](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/events-microsoft-forms).
Pinned documentation counterparts were read in `references/VBA-Docs`; no sample
macro or implementation was adopted. Exact property/event acceptance remains
NOT_RUN until its packet is implemented and independently asserted. Excel-only
RefEdit and other Office-supplied host extensions need separate reference/resource
inventory; they are not silently treated as generic MSForms controls.

| Worksheet Form Control | Metadata / properties to audit | Decode | Display/input/run | Edited save |
|---|---|---|---|---|
| Button | Native shape ID/name, caption, anchor/style, assignment; enabled/visible/print/placement next | First UTF-8 controls/ctrlProp/VML snapshot implemented | NOT_RUN / not implemented by this slice | UNSUPPORTED; original graph retention only |
| CheckBox / OptionButton | Typed state, grouping, linked cell, assignment/order | Reported outside Button slice when a valid graph/type is present | UNSUPPORTED | UNSUPPORTED |
| ListBox / ComboBox (DropDown) | Item/list range, selection, linked cell, assignment | Outside Button slice | UNSUPPORTED | UNSUPPORTED |
| ScrollBar / Spinner | Value/min/max/increments, linked cell, assignment | Outside Button slice | UNSUPPORTED | UNSUPPORTED |
| Label / GroupBox | Native presentation/container/grouping | Outside Button slice | UNSUPPORTED | UNSUPPORTED |
| Worksheet ActiveX | Separate persisted component/type/reference profile, event sinks, properties, host/platform policy | UNSUPPORTED by the Form Control decoder | UNSUPPORTED | UNSUPPORTED |

Worksheet families follow Microsoft's [worksheet controls overview](https://support.microsoft.com/en-gb/excel/overview-of-forms-form-controls-and-activex-controls-on-a-worksheet).
Its [button guidance](https://support.microsoft.com/en-gb/excel/assign-a-macro-to-a-form-or-a-control-button)
distinguishes Form Control macro assignment from ActiveX event code and excludes
ActiveX on Office for Mac. The future platform profile must reflect that distinction;
no COM/OCX activation is authorized in any build. Neither these references nor this
inventory is a fresh Office observation.

## First native metadata contract and evidence

Capability label: `worksheet-button-metadata-v1` (a Rust reader contract, not a
file format or executable Office reference library). Entry:
[GridCraft read_worksheet_controls](../../gridcraft/crates/xlsx/src/form_controls.rs).
It operates on the already preserved sheet graph and never modifies document
state, compiles VBA, follows external targets, loads a control or runs an assignment.
No second writable control registry/model is created.

It matches worksheet `shapeId`, a `ctrlProp` relationship/property part and the VML
`id`/Office `spid`; preserves the name separately from caption; decodes caption text
in source order and native macro strings from VML/worksheet properties. Conflicting
nonempty assignments refuse. XML namespaces, expanded attribute uniqueness and
known x14 markup-choice/fallback selection are checked. Other native Form Control
types have explicit unsupported records; malformed/ambiguous graphs fail atomically.

Limits: 4 MiB aggregate XML input and 4 MiB stored XML strings; 8,192 XML nodes,
32 nesting levels, 1,024 control/shape records, 16 KiB fields; graph index at most
4,096 parts, 10,000 relationships and 4 MiB metadata. DTDs and unknown entities
are refused, entity text is coalesced/bounded, and invalid XML 1.0 characters fail.
Only the preserved UTF-8 controls/ctrlProp/VML representation is admitted. VML-only
legacy controls, other encodings and complete DrawingML reconciliation are still
gaps. A missing snapshot after an upstream preservation warning is not a decode PASS.

Caption formatting, VML anchor/offset units, style conversion and native enabled/
visible defaults are intentionally not interpreted by this initial snapshot. They
must be resolved before UI use. Source DrawingML IDs and nonempty DrawingML macro
conflicts still need reconciliation before activation. This is **partial FC1.1**;
it does not satisfy FC1.2–FC1.5 or native UserForm import.

[Original fixtures/tests](../../gridcraft/crates/xlsx/src/tests/form_controls.rs)
build ZIP/XML data in Rust under the fork's MIT OR Apache-2.0 licence. They contain
an authored textual `[0]!AuthoredButton` assignment, never user workbook source.
No existing workbook or downloaded macro was executed. Tests assert:

- Native name/shape ID, Office `spid`, mixed caption/entity order and macro text.
- Matching/empty/conflicting assignment sources and unchanged source graph on error.
- Known markup-choice or fallback exactly once; namespace aliases and spoof refusal.
- Missing/external/wrong/duplicate relationships, parts and native identities.
- Explicit unsupported CheckBox metadata and empty native-control absence.
- XML nesting/node/shape/text/input limits, fragmented entities and malformed markup.
- Worksheet rename, package save/reopen and unchanged decoded native metadata.

Eight focused native tests and the same eight actual Node 22 JavaScript/WASM tests
PASS. Strict XLSX Clippy and GridCraft's full 10-step CI pass; shared CI, WASM
and actual WASI gates pass. Commands are recorded in
[progress](implementation-progress.md). This is authored format/contract evidence; fresh Office
build/OS/locale comparisons remain NOT_RUN under M8.5.

## Work ledger and next task

| Task | State | Evidence / dependency |
|---|---|---|
| FC0.1 path inventory | DONE | Updated code paths above, both fork attachments and shell routes |
| FC0.2 compatibility matrix | DONE | [Per-property/event/format/host contracts](m7a-control-contracts.md), selected defaults and explicit future packets |
| FC0.3 contracts | DONE | ADR 0039 + ADR 0040; identities, limits, unsupported handling, atomicity, source/resources, persistence boundary |
| FC1.1 native worksheet Button read | DONE for worksheet-form-controls-v1 | Native IDs, DrawingML/VML/property joins and conflict refusals |
| FC1.2–FC1.4 geometry/render/input/launch | DONE for worksheet-form-controls-v1 | Native widgets, placement, pointer/keyboard, consent, owner scope and debugger continuation |
| FC1.5 native worksheet save | DONE for worksheet-form-controls-v1 | Untouched bytes plus edited caption/anchor/assignment/state/create/delete/reopen |
| FC2 worksheet Form Controls | DONE for worksheet-form-controls-v1 | Nine native families, bounded bindings/list state, selected Shape/ControlFormat APIs; [contract/evidence](m7a-worksheet-controls.md) |
| FC4 broader UserForm controls | IN_PROGRESS | [Selected flat control runtime](m7a4-native-runtime.md) includes boolean, text, numeric and unbound single-selection ListBox packets; further list/container/image/binding/platform packets remain |
| FC3.1–FC3.4 native UserForm codecs/attachment | DONE for native-userform-import-v1 | Export + embedded parity, shared designer/runtime/debugger, standalone IO, XLSM/DOCM trust and byte retention |
| FC3.5 native lifecycle parity | IN_PROGRESS | [Shared explicit instance packet](m7a4-userform-instances.md) implemented; default/collection/lifecycle/focus/event and fork gates remain |
| FC5 edited native persistence | NOT_STARTED | Required encoder, native resource and signature/cache contract |
| FC6 remove .tvform | NOT_STARTED | Requires native replacement workflows; do not extend the legacy format |

M7A.3 completes the selected worksheet profile under ADR 0041. The next shared
task is M7A.4 / FC3.5 / FC4 native lifecycle/property/input parity and
broader native control packets against the completed contract matrix.
No native resource layout, default or event order may be inferred from `.tvform`.

M7A.3 update: the initial metadata-only description above is historical. The
[current worksheet profile](m7a-worksheet-controls.md) supplies native rendering,
input, bindings, host access and edited OOXML/VML persistence; ActiveX remains
retained-only. No shared VM/UI dependency enters GridCraft's model/XLSX layers.

ADR 0053 adds shared ListBox/ComboBox two-dimensional List/Column loading and
indexed/current-row Column access. See [the decision](decisions/0053-native-list-array-loading.md)
for the selected string-cell profile, atomic validation and remaining array gaps.

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

[ADR 0070](decisions/0070-native-keyboard-events.md) adds selected native
KeyDown/KeyPress/KeyUp with mutable ByVal ReturnInteger, Shift flags, ordinary
static/dynamic WithEvents callbacks and canonical completion receipts. A bounded
raw-input UI barrier waits before default edits and preserves Stop/Continue/Reset.
UTF-16 storage is kept separate from ANSI Asc/Chr conversion; ASCII is stable,
while non-ASCII ANSI conversion needs a configured code page. Shared full CI,
WASM/WASI, eight keyboard runtime cases, 24 native form UI cases and Chromium
acceptance pass. Keyboard/platform/shortcut/root-window focus/IME and the full
M7A.4 checklist remain partial; mouse and accelerators remain open.

Both hosts pin corrected shared revision `9fde04ce9ed8eab15b7f0609160617c774c2fc1b`.
Full shared/host gates and 21 GridCraft/16 WordCraft native and Node-WASM
package cases pass; ADR 0070 records the integration commits and exact evidence.

[ADR 0071](decisions/0071-native-keyboard-focus-and-default-order.md) adds
selected root keyboard focus, canonical Tab/SetFocus/Space order and debugger
shortcut isolation. Native/browser raw ordering is normalized only for a matching
adjacent single ASCII text/key pair within one input batch. Full shared gates,
nine runtime keyboard cases, eleven new UI cases, 24 existing form UI cases and
Chromium pass. Both hosts pin shared revision
`38578095ba1fc1ae3695f92a6c2e89cdb29d42f6`; full host gates and 22 GridCraft/
17 WordCraft native and Node-WASM package cases pass. Broader M7A.4 stays open.


[ADR 0072](decisions/0072-native-form-accelerators.md) adds native persisted
accelerators and canonical Accelerator properties, selected eligible Alt actions
for Label/buttons/booleans/Tab/Page, validation/callback continuations and inert
caption underlines. Duplicate routing explicitly refuses; non-ASCII/platform and
further ordering cases remain open. Checklist K03 is PARTIAL; M7A.4 remains in
progress, edited persistence/removal remain M7A.5/.6 and Office observations remain
M8.5 NOT_RUN.


[ADR 0073](decisions/0073-native-mouse-callbacks.md) adds typed native mouse events,
indexed Tab/MultiPage arguments, mutable DblClick cancellation and canonical
completion receipts. Selected shared pointer capture/default paths preserve
Stop/Continue, Reset, identity and keyboard focus; remaining text/list/numeric/
indexed-tab/platform defaults and full event ordering keep M01 PARTIAL. M7A.4
remains incomplete and fresh Office observations remain M8.5 NOT_RUN.

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

ADR 0077 adds selected native Show/Hide/modeless Activate/Deactivate ordering,
unload retirement without Deactivate, reentrant lifecycle guards and cross-window
keyboard continuations. Inert inspection and executable window-focus commands
retain existing trust. Automatic document-canvas input barriers, broader binding/
collection/platform ordering and full M7A.4 acceptance remain open.

[ADR 0078](decisions/0078-document-activation-input-barrier.md) adds the bounded
pre-document raw-input barrier and opaque execution identity for native modeless
activation. Hosts authorize the ordinary Deactivate callback before document
input; Stop/Continue, cancellation and acknowledged repaint precede replay.
The remaining M7A.4 checklist and M8.5 NOT_RUN status are unchanged.

[ADR 0079](decisions/0079-form-input-before-document-activation.md) orders selected
native key/text/paste/IME prefixes before document deactivation. The raw hook
transfers earlier input to the existing bounded keyboard barrier and waits for
receipts, defaults and ordinary callbacks. Spanning IME composition retains only
owner/lifetime metadata; its raw continuation stays with the form through Stop.
Reset/context invalidation discards unapplied input; a modal opened by the prefix
cancels the later document gesture while preserving the parked keyboard receipt.
No VM/foreign behavior or budget changes. Full M7A.4 and platform/Office comparison
requirements remain open.

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
