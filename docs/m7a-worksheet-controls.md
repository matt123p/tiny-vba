# M7A.3 — Worksheet Form Controls

Updated 2026-11-04. GridCraft-owned `worksheet-form-controls-v1`, under
[ADR 0041](decisions/0041-worksheet-form-controls.md). FC1–FC2 implement a bounded
native worksheet profile. This does not complete M7A.4–M7A.7 or worksheet ActiveX.

| Family | Native kind | State and input |
|---|---|---|
| Button | Button | Caption, pointer/keyboard, assigned macro |
| CheckBox | CheckBox | Excel 1 / -4146 / 2; Boolean / #N/A linked cell, mixed display |
| OptionButton | Radio | 1 / -4146, exclusive group, one-based linked group index |
| DropDown | Drop | One-based selection, live range or item list, bounded popup |
| ListBox | List | Single/simple/extended selection, bounded virtualized rows |
| ScrollBar | Scroll | Value, Min/Max, SmallChange/LargeChange, horizontal/vertical |
| Spinner | Spin | Bounded value and increments, increase/decrease buttons |
| Label | Label | Caption and presentation; no activation |
| GroupBox | GBox | Caption, geometric radio grouping, native group link |

Common properties are name/caption, anchor/placement, enabled/visible/print state
and macro assignment. ControlFormat exposes the selected Value, Enabled,
PrintObject, Min, Max, SmallChange, LargeChange, LinkedCell, ListFillRange,
ListIndex, DropDownLines, MultiSelect, ListCount, List, AddItem, RemoveItem,
RemoveAllItems, Parent and Application members. Shape adds AddFormControl through
Shapes, FormControlType and ControlFormat and reuses native text, geometry,
OnAction, placement, deletion and lifetime handling. The fork catalogue retains
exact signatures and marks unimplemented members separately.

Geometry uses native IDs, not captions, for joining. Existing DrawingML identities
are preserved and new IDs avoid collisions. Source names/captions/macros and
worksheet/VML/DrawingML geometry must agree (bounded cell-fraction rounding tolerance).
Existing source parts are retained separately from live Shape state. VBA geometry
is Single points (3/4 of a model pixel). Placement follows cell edits, resizing,
hiding, undo and reopening; egui handles scale/focus and the grid handles zoom/RTL.
Print uses live values and excludes hidden/nonprinting controls.

LinkedCell resolves one local cell, a sheet-qualified address or a local/workbook
name with at most eight name indirections. ListFillRange resolves at most 1,024
rows in one column and overrides cached items. Values are queried from the current
workbook, including edits made during a debugger break; there is no timer-driven
copy. Structural edits, sheet/name rename and sheet deletion rewrite references.
Invalid references/value types refuse mutation and disable input with a diagnostic.
Control actions write value plus linked cell in one undo/recalc transaction. Radio
groups use the smallest enclosing GroupBox, otherwise native firstButton groups;
fmlaGroup overrides an option's own link. Multiselect lists have no single linked
value/ListIndex. List returns a one-based Variant array, distinct from MSForms lists.

Assigned macros use the existing trust and bounded bridge. Local [0]! syntax,
current-workbook qualification and a verified optional module name are supported;
foreign workbook targets, private/parameterized/missing/ambiguous targets refuse
before control mutation. Opening a file grants no consent. Clicking or using the
keyboard commits the value, recalculates, then starts exactly one assigned Sub.
Application.Caller returns the control name; implicit Range resolves against its
captured owner sheet. Debugger continuation retains that scope. Another click
while running/waiting/paused/stale refuses, and Reset never replays it. Programmatic
property/cell writes do not invoke the assignment. Linked control writes do not
synthesize Worksheet_Change; ordinary VBA cell writes retain existing events.

Native save creates or patches worksheet controls/controlPr anchors, ctrlProp
properties, VML ClientData/captions, DrawingML identities and relationships.
Untouched parts remain unchanged; unknown extension XML survives known-field edits.
Creation, caption/geometry/assignment/state changes and deletion survive reopening.
Source-only VBA is still stored by the fork's existing mechanism; this work does
not rewrite a binary VBA project, validate signatures or encode UserForms.

The declared limits are 1,024 controls/items/selections, 16 KiB per caption/item,
256-byte names, 1,024-byte bindings, 1 MiB item text per control, 4 MiB aggregate
control text/XML, XML depth 32 and 8,192 parsed nodes. Numeric values/increments are
0–30,000 with ordered bounds. New controls emit explicit Max/Page; imported numeric
controls missing both native representations of those unspecified defaults stay
inert. Protected-sheet edits/input, external bindings, editable combo styles,
dialog-sheet controls, unsupported classes and unlisted property behaviors refuse
or remain retained-only. Original extended styling/resources are retained; no
pixel-identical Office theme, arbitrary resource execution or ActiveX claim is made.

Evidence is original Rust fixtures under MIT OR Apache-2.0; no Office-produced
binary, proprietary asset or downloaded macro is committed or executed:

- [XLSX graph/metadata tests](../../gridcraft/crates/xlsx/src/tests/form_controls.rs): namespaces, bounds, identities, conflicts, unchanged parts and unknown XML.
- [Engine integration](../../gridcraft/crates/engine/tests/form_controls.rs): all nine families, create/edit/delete/reopen, bindings, radio/list state, undo/recalc, placement and invalid operations.
- [VBA integration](../../gridcraft/crates/vba/tests/form_controls.rs): real typed VBA, units, identity, lists, errors, native output and grouped undo.
- [UI integration](../../gridcraft/crates/ui-egui/tests/worksheet_controls.rs): actual widgets, accessibility, pointer/keyboard, trust, qualified targets, breakpoint/resume/Reset and no replay.
- [Print regression](../../gridcraft/crates/engine/src/cmd/print/tests.rs) and [render fixture](../../gridcraft/crates/ui-egui/examples/worksheet_controls_snapshot.rs).

Validation results are recorded in [implementation progress](implementation-progress.md#m7a3--native-worksheet-form-controls-2026-11-04).
Fresh Office build/OS/locale/error/event comparisons remain **NOT_RUN** (M8.5).

Public format basis: [CT_FormControlPr](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/8577637a-66c3-485b-ab6e-f8c85689a7c4),
[CT_ListItem](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/de80878b-2bb4-45b0-b830-5c435058f2a4),
[ST_SelType](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/9a7dc803-b078-474a-8328-7c7c636cf6b7),
[Excel AddFormControl](https://learn.microsoft.com/en-us/office/vba/api/excel.shapes.addformcontrol),
[ControlFormat](https://learn.microsoft.com/en-us/office/vba/api/excel.controlformat),
and [worksheet control overview](https://support.microsoft.com/en-gb/excel/overview-of-forms-form-controls-and-activex-controls-on-a-worksheet).
These references and independent authored tests are not fresh Office observations.
