# M7A native forms — implementation inventory and work ledger

Updated 2026-11-04. **IN PROGRESS**. FC0 contracts and FC3.1–FC3.4 are complete for
`native-userform-import-v1`; see the [control/property/event/format/host matrix](m7a-control-contracts.md)
and [ADR 0040](decisions/0040-native-userform-import-and-attachment.md). The first FC1.1 metadata slice remains partial under
[ADR 0038](decisions/0038-native-office-forms-and-compatibility-only.md) and
[ADR 0039](decisions/0039-m7a-native-control-metadata.md). It is not M7A sign-off.
Native import now supports flat Label/TextBox/CommandButton and selected
CheckBox/OptionButton/ToggleButton/ScrollBar/SpinButton/ListBox/ComboBox forms; see [the current runtime record](m7a4-native-runtime.md). Native edited
persistence and `.tvform` removal remain unimplemented.

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
| CheckBox | Value/three-state/group/binding, Change/Click | Native decode/runtime/input; binding and native edited save unsupported |
| OptionButton | Value/group/binding, Change/Click | Native decode/runtime/input and parent-scoped groups; binding/native edited save unsupported |
| ToggleButton | Value/binding, Change/Click | Native decode/runtime/input; binding/native edited save unsupported |
| ComboBox | Items/selection/columns/binding/style, Change/Click/drop/update events | Selected native unbound editable/drop-list packet; further list/binding/resource/lifecycle cases and edited save remain unsupported |
| ListBox | Items/selection/columns/multiselect/binding, Change/Click | Selected native unbound indexed rows/columns, single selection, Value/Text and input; 2D List/Column loading/readback and indexed/current-row Column implemented; further shape/loading cases/multiple selection/bindings/save remain unsupported |
| Frame | Caption/container/child Controls/Parent/ActiveControl/scoped grouping and tab/focus | Selected native packet under ADR 0056; further resource/event/property variants remain open |
| MultiPage | Pages/selected Value, child controls, Change | UNSUPPORTED |
| TabStrip | Tabs/selected Value, Change | UNSUPPORTED |
| ScrollBar | Value/range/increments/orientation, Change/Scroll | Selected native records, Long properties, reversed ranges and input/drag-only Scroll; further resources/properties/save remain unsupported |
| SpinButton | Value/range/increments/orientation, Change/spin events | Selected native records, Long properties and input/Change/spin; further resources/properties/save remain unsupported |
| Image | Native picture resource/geometry/display properties and applicable input events | UNSUPPORTED |

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
