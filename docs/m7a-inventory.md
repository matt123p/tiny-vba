# M7A native forms — implementation inventory and work ledger

Updated 2026-11-04. **IN PROGRESS**. This is the FC0.1 code audit, initial FC0.2
control matrix and first FC1.1 metadata slice under
[ADR 0038](decisions/0038-native-office-forms-and-compatibility-only.md) and
[ADR 0039](decisions/0039-m7a-native-control-metadata.md). It is not M7A sign-off.
Native UserForm import and `.tvform` removal remain unimplemented.

## Audited implementation paths — FC0.1

| Surface / owner | Actual path and current boundary | Required native work |
|---|---|---|
| Shared project decode / tiny-vba | `crates/vba-project/src/lib.rs::decode` refuses `/PROJECTlk`; `text.rs` refuses PROJECT `BaseClass` designers | Bounded MS-OFORMS resource/component results; explicit reference/owner/source identities; retain inert/protected/corrupt/limit refusals |
| Shared design/runtime / tiny-vba | `vba-ide/src/forms.rs` has Label/TextBox/CommandButton only, 128 controls and 2 MiB model limit. `form_project.rs` generates ordinary class/event support separately from authored source | Map native formats to validated internal data; audit instance/lifecycle/property/event differences before declaring compatibility |
| Shared exchange / tiny-vba | `Ide::import_form` / `export_form` consume the custom archive. Standalone native/browser IO filters accept `.bas/.cls/.tvform`; `.frm/.frx` is not a native import workflow | Caller-supplied native source/companion resources, atomic attachment, native export and edited-source/resource encoding; remove custom routes after replacement |
| Worksheet package / GridCraft | `xlsx::passthrough::KeptReader::sheet_elements/vml` preserves `controls`, `ctrlProp` parts and VML. New `read_worksheet_controls` decodes a read-only Button snapshot | Finish DrawingML/VML reconciliation before native rendering/activation; preserve source graph separately from future writable state |
| DrawingML shape identity / GridCraft | `xlsx::drawing::read_shape` reads nonempty `macro` into `on_action` but assigns model `id: 0`; source `cNvPr` identity is not retained as a native-control join key | Retain/reconcile native source identity; reject conflicting assignment/representation data; do not join by model sequence number or guessed caption |
| Native cell checkbox / GridCraft | `engine::cmd::extra` and `extra::cells` use cell values/number-format state and normal commands | Do not mistake this implemented feature for a VML CheckBox or worksheet ActiveX implementation |
| Workbook attachment / GridCraft | `vba::Bridge::import_project`, shared IDE and `ui-egui::vba` source routes; imported document names must match live workbook/sheets | Attach native form/resource components through the same project/trust/debugger lifecycle; native macro ownership and bounded activation |
| Word attachment / WordCraft | `WordVbaProject::new` decodes standard/class/ThisDocument source inertly. `docx` retains the original project and direct related parts; shared editor routes source requests | Native form attachment and source/resource save acknowledgments; retain document identity and event/trust policy |
| Native persistence / all | M7 preserves original bytes. Neither shared editor nor forks encode edited embedded UserForm resources | Explicit native encoder/save path under ADR 0037; cache/signature invalidation, identity/revision checks and atomic file commit |

The audit is based on inspected code, not planned APIs. No crate or dependency
edge is added to the shared graph. Office package objects stay in their forks.

## Control/property/event inventory — FC0.2 seed matrix

These are compatibility packets to audit, not claims of native implementation or
an exhaustive property/signature contract. Each packet must later record exact
Office signatures, defaults, errors, ordering and fixture coverage. Shared legacy
runtime support does not establish native decode or persistence support.

| Native UserForm control | Property/event families to audit | Current implementation |
|---|---|---|
| CommandButton | Caption/geometry, Enabled/Visible, default/cancel/focus, Click | Three-control shared legacy subset only; native resource decode/save UNSUPPORTED |
| Label | Caption/geometry, formatting, visibility, Click | Three-control shared legacy subset only; native resource decode/save UNSUPPORTED |
| TextBox | Text/Value, geometry, selection, validation, Change/focus/key events | Shared legacy Text/Change subset only; native resource decode/save UNSUPPORTED |
| CheckBox | Value/three-state/group/binding, Change/Click | UNSUPPORTED |
| OptionButton | Value/group/binding, Change/Click | UNSUPPORTED |
| ToggleButton | Value/binding, Change/Click | UNSUPPORTED |
| ComboBox | Items/selection/columns/binding/style, Change/Click/drop/update events | UNSUPPORTED |
| ListBox | Items/selection/columns/multiselect/binding, Change/Click | UNSUPPORTED |
| Frame | Caption/container/child Controls/grouping/tab behavior | UNSUPPORTED |
| MultiPage | Pages/selected Value, child controls, Change | UNSUPPORTED |
| TabStrip | Tabs/selected Value, Change | UNSUPPORTED |
| ScrollBar | Value/range/increments/orientation, Change/Scroll | UNSUPPORTED |
| SpinButton | Value/range/increments/orientation, Change/spin events | UNSUPPORTED |
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
| FC0.1 path inventory | DONE for the initial audit | Inspected implementation paths above |
| FC0.2 compatibility matrix | STARTED | Seed control/property/event/host matrix; detailed signatures/defaults and native resource packets remain |
| FC0.3 contracts | STARTED | ADR 0039 defines first metadata slice; native UserForm resource/encoding/persistence contracts still needed |
| FC1.1 native worksheet Button read | PARTIAL | 8 native + 8 JS/WASM tests; source DrawingML identity/reconciliation remains |
| FC1.2–FC1.4 geometry/render/input/launch | NOT_STARTED | Complete native ID/source reconciliation and geometry/default policy first |
| FC1.5 untouched save | Initial preservation regression PASS | Edited caption/anchor/assignment persistence not implemented |
| FC2 / FC4 broader controls | NOT_STARTED | Implement native packets against the matrix; preserve explicit gaps |
| FC3 native UserForm codecs/attachment | NOT_STARTED | Review MS-OFORMS + `.frm`/`.frx` contracts and caller-supplied resources |
| FC5 edited native persistence | NOT_STARTED | Required encoder, native resource and signature/cache contract |
| FC6 remove .tvform | NOT_STARTED | Requires native replacement workflows; do not extend the legacy format |

Next smallest worksheet task: preserve source DrawingML `cNvPr` identity, reconcile
it with worksheet/VML/property metadata and test conflicting macro representations;
then implement bounded native Button geometry/rendering and trusted activation.
The next shared task is a bounded native UserForm component/resource contract from
[MS-OFORMS](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/9c79701a-8c3e-4429-a139-b60ac3a1d50a).
No native resource layout, default or event order may be inferred from `.tvform`.
