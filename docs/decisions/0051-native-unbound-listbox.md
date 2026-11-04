# ADR 0051 — native unbound ListBox

Date: 2026-11-04. Status: adopted for the selected single-selection packet.
M7A.4 remains in progress; this does not reduce its remaining scope.

The project decoder admits cached ListBox class 24 and MorphData DisplayStyle 2,
with selected ColumnCount/BoundColumn/TextColumn fields, default list style and
single selection. Empty native lists start empty; authored initialization supplies
rows. Unknown masks, external list/binding resources, nonempty persisted Value
without matching list data, multiple selection, headings, picture/mouse resources
and nondefault column-width records still refuse atomically. Exact source and
resource bytes remain separate from runtime/design changes. No COM/OCX or vendor
fixture is used.

The IDE generates an ordinary MSForms.ListBox-compatible project class. Fields
own rows, selection, Value and Text; methods/properties use ordinary coercion,
errors, WithEvents and debugger continuations. AddItem inserts a row (optional
index, default append); indexed List(row, optional column) supplies string cells
in ten unbound columns. RemoveItem/Clear preserve or clear selection and current
values. ListCount, ListIndex, Selected, default Value, Text, ColumnCount,
BoundColumn and TextColumn implement the selected profile. BoundColumn=0 yields
the row index; TextColumn=0 yields its textual form; -1 selects the first visible
column in the admitted equal-width profile. Matching is exact in this packet;
locale/ambiguous-match behavior is still comparison work. Whole-array List/Column
assignment, ColumnWidths, TopIndex, MultiSelect and RowSource/ControlSource are
not advertised by this packet.

Rows are capped at 1024, cells at 16384 UTF-16 units, columns at ten; existing VM
allocation/instruction/continuation limits remain authoritative. Resource failures
use error 7. Invalid selection/property indexes use 380 and invalid indexed list
access/removal uses 381 through ordinary frames. Value/selection state is committed
before Change; definitive non-Null changes then raise Click, including Stop/resume
without replay. Programmatic writes do not mark input dirty. New physical row
selection respects Locked, focus validation and ordinary typed BeforeUpdate/
AfterUpdate on focus transfer. Duplicate-text/index and structural-mutation event
edge cases still require fresh Office observation under M8.5 (NOT_RUN).

A typed executable instance-list command uses the existing host/source/session/
visibility/modal checks. UI rows read canonical fields, render bounded visible
rows and equal-width columns (minimum calculated width 72 points), and route
pointer/arrows/Home/End/PageUp/PageDown through that command. Runtime focus is not
re-requested when already correct: re-requesting cleared egui's per-widget arrow
filter and caused subsequent keys to leave the list. The list owns arrow keys;
Tab/validation remain container-owned. Widgets use injected theme/fonts, original
outlines and accessible ListBox/selected-option roles. Cell painting uses a bounded
512-unit display prefix for long text while preserving canonical UTF-16 data. No new format/dependency,
VM form opcode or second object store is introduced.

Independent Rust native-resource fixtures, headless runtime, actual egui input
and Chromium modal Stop/resume tests provide authored evidence. These are not
fresh measured Office traces. ComboBox editable/drop-list behavior, further list
properties/bindings, containers/pages/images/platform and remaining lifecycle
contracts stay open. Native edited persistence remains M7A.5; .tvform is not
expanded.

Sources: [cached classes](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/5f649a6c-90df-4226-b8c7-2a7d98cd9647),
[MorphData masks](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/19014e19-67fa-4060-8c81-d1463809f117),
[data](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/8a95d726-067d-4cd0-8fd9-3a4b6cc6188d),
[List](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/list-property-microsoft-forms),
[AddItem](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/additem-method),
[ListIndex](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/listindex-property),
[BoundColumn](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/boundcolumn-property),
[TextColumn](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/textcolumn-property),
[ColumnCount](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/columncount-property),
[ColumnWidths](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/columnwidths-property),
[Click](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/click-event).
