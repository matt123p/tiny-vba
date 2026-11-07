# ADR 0088: Native Boolean-control caption wrapping

Date: 2026-11-07. Status: implementation; shared/browser acceptance passed, host acceptance in progress. M7A.4 remains open.

CheckBox, OptionButton and ToggleButton admit the native VariousPropertyBits
WordWrap bit and ordinary Boolean WordWrap state. File and dynamic defaults are
True; explicit native False seeds canonical state before authored initialization.
Other applicable flags keep their existing validation, and ComboBox/ListBox still
require the persisted bit to be set without gaining an invented public member.
No VM, bytecode, foreign operation, mutable property store or budget is added.

The IDE provider owns ordinary state, standard Boolean coercion and inert queries.
Assignments do not change Value, Null/TripleState, option groups, focus, captions
or events. Source/resource metadata remains immutable and Reset preserves it.
Imports remain inert and malformed/unsupported flags still refuse atomically.

The view keeps CheckBox/RadioButton widgets and their checked/mixed accessibility
and input semantics. A local pre-shaped caption uses the real widget's style/id,
icon gap and authored width, with complete text/glyph data and a bounded local
allocation rectangle. The font cache and global styles remain untouched.
ToggleButton uses the existing fixed caption painter with its selected button
classes and full accessible label/state. This avoids the negative-centre offset
of a pre-shaped galley passed through an atom painter. Explicit newlines survive
WordWrap=False; clipping remains native. Legacy rendering keeps ordinary widgets.
Other TextAlign/Alignment, vertical overflow, AutoSize, RTL/platform and remaining
property requirements are still open.

Six new model cases cover flags/defaults, typed errors, value/event independence,
Null/groups, inert inspection, authored form names, Stop/Continue and Reset.
Four GPU/input cases cover real row changes, fixed accessible bounds/labels,
newlines, physical ToggleButton centring and an actual checkbox callback that
changes WordWrap and stops/resumes exactly once. Shared/browser and affected host
receipts are recorded in implementation progress. These are reference-based
authored tests; fresh Office observations remain M8.5 NOT_RUN.

Native edited saving remains M7A.5; .tvform removal remains M7A.6. Neither the
broader M7A.4 checklist nor the compatibility-only scope is reduced.

References:

- [WordWrap](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/wordwrap-property)
- [Native VariousPropertiesBitfield](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/c38cc3f9-969b-45da-ab41-b361c044431e)
- [File defaults](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/7a72ac4a-39d9-4e2b-829e-19e3e9a1f60d)
