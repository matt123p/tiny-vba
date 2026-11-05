# ADR 0063 — Native ListBox multiple selection

Status: implemented; CI/WASM/WASI/UI/web and actual Chromium validation pass. M7A.4 remains open.

The Forms provider in `vba-ide` implements native `MultiSelect` values Single (0),
Multi (1) and Extended (2), including reference constants, native MorphData decoding,
ordinary per-instance selection storage and authorized physical input. Unknown
persisted modes and unsupported external list storages refuse atomically. Original
source/design/resources remain separate from runtime state.

Selection lives in a Boolean array in the ordinary canonical class field. In
multiple-selection modes, ListIndex is the focus row and Value remains Null;
Selected reads/writes selection independently. Insert/remove preserve selection
of surviving rows; Clear and whole-list replacement discard selection. Invalid
Selected writes are inert, following the Forms reference. Other list/property
range errors continue to use the existing coercion/error path. Nonempty Value
assignment in multiple mode refuses with error 380 instead of implicitly selecting.
Exact mode-transition/event/error details are authored inference pending M8.5.

Pointer input toggles Multi rows. Extended input replaces selection, uses Ctrl to
toggle and Shift to select an anchored range; Ctrl navigation moves focus without
selection. Multi arrow navigation moves focus and Space toggles its row. Selection
is committed before Change and ordinary Stop/resume; multi-select input does not
synthesize Click while Value remains Null. Focus cancellation, Locked, modal input
ownership and per-site authorization remain in force. Rendering reads fields
inertly and distinguishes focus from selection; no mutable host selection store
or getter execution is introduced. Editor completion consumes Ctrl+Space only
while its source editor owns focus.

The 1,024-row and ten-column profile, native resource and VM budgets are unchanged.
Typed local array views avoid repeated late-bound array invocations; they do not
raise permission limits or bypass host authorization. RowSource/ControlSource and
other remaining M7A.4 requirements remain separate work; native edited persistence
and legacy format removal remain M7A.5/.6.

Primary contracts:

- [MultiSelect](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/multiselect-property-microsoft-forms)
- [Selected](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/selected-property)
- [ListIndex](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/listindex-property)

Fixtures and implementation are independently authored Rust, MIT OR Apache-2.0.
Fresh Office comparisons remain NOT_RUN under M8.5.
