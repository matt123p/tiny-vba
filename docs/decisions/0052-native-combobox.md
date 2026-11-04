# ADR 0052 — native ComboBox text, selection and drop state

Date: 2026-11-04. Status: adopted for the selected unbound native packet.
Full M7A.4 remains in progress for its unchanged control/binding/platform scope.

The decoder admits cached ComboBox class 25, MorphData DisplayStyle 3 (editable)
and 7 (drop-list), selected columns, ListRows, MatchEntry, MatchRequired,
MaxLength and field-selection metadata. Noneditable style ignores the optional
Editable flag as specified by MS-OFORMS. Invalid styles/matching/rows, unknown
masks, bindings, custom column/picture/mouse resources and nonempty drop-list
Value without list data refuse atomically. Original bytes remain independent of
ordinary runtime state; there is no control activation, COM/OCX or new format.

The IDE composes the same ordinary row/column operations as ListBox, without a
second data store. ComboBox owns Variant Value, text, zero-based ListIndex,
selection, matching and drop visibility in class fields. It provides the selected
List/row/column APIs plus Style, MatchEntry, MatchRequired, MatchFound, ListRows,
MaxLength, EnterFieldBehavior, SelStart/SelLength/SelText and DropDown. The
MSForms nominal provider adds ComboBox with Change/Click/DropButtonClick and
standard fmStyle/fmMatchEntry enums under its existing conflict/refusal rules.
ComboBox does not acquire ListBox.Selected, Caption, multiline or password APIs.

Editable controls accept free text and exact programmatic row matching; drop-list
controls require an existing row. Null clears. Selecting a row derives Value
from BoundColumn and Text from TextColumn. Typing uses first-letter cycling,
extended prefix completion or no matching; completion selects the generated
suffix in UTF-16. Deletion does not immediately restore that suffix. An ambiguous
prefix can commit a row without a definite Click; becoming unambiguous delivers
Click through an ordinary continuation. Matching calls the existing VBA LCase
intrinsic, preserving its injected-locale/unsupported policy: no new hand-written
Unicode/locale comparer is installed. MatchEntry=None preserves arbitrary UTF-16
through the shared text adapter. Programmatic matching remains the selected
exact-match profile; broader locale/coercion parity is still open.

Value, text, selection and match state commit before Change/Click, so authored
Stop/resume cannot replay input. Locked blocks physical edits while leaving
programmatic writes available. MatchRequired prevents focus transfer while text
has no matching row, before following control activation. Ordinary validation and
mutable typed cancellation remain authoritative. DropButtonClick follows an
actual visibility transition; open/close state commits before a handler pauses.
Navigation preserves an open list; definitive selection closes it. Enter/Escape
belong to an open list; contextual F4 leaves the editor Properties shortcut intact
outside an input-owning ComboBox. Closing the popup does not itself steal focus.

All physical text/row/navigation/drop commands execute code and retain existing
source/session/host/trust/modal checks. UI reads canonical fields, reuses the
UTF-16 text adapter and bounded list renderer, and emits explicit commands. Popup
state is VM-owned; toolkit cursor/scroll state is presentation only. Theme/fonts
stay injected and arrow artwork is original code. Native persistence remains
M7A.5; the legacy codec is not expanded. Row/cell/column and VM limits from ADR
0051 remain, with the popup bounded to the available display profile.

Evidence is independently authored Rust resources, native/WASI runtime cases,
actual egui input and Chromium modal Stop/resume. These encode expectations from
public documentation, not fresh measured Office traces. Exact errors, ambiguous/
duplicate matching, first-letter ordering, Escape rollback, hide/reload/reentrant
drop lifecycle, remaining selection/property/coercion and locale cases still need
M8.5 comparison. Whole-array/Column APIs, widths, bindings, further list resources,
containers/pages/images/RefEdit and applicable ActiveX/platform work remain open.

Sources: [cached classes](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/5f649a6c-90df-4226-b8c7-2a7d98cd9647),
[MorphData](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/19014e19-67fa-4060-8c81-d1463809f117),
[Style](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/style-property),
[MatchEntry](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/matchentry-property),
[MatchRequired](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/matchrequired-property),
[DropButtonClick](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/dropbuttonclick-event),
[TextColumn](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/textcolumn-property).
