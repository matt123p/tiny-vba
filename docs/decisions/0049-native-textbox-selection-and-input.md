# ADR 0049 — native TextBox selection and text input

Date: 2026-11-04. Status: adopted. M7A.4 remains in progress for the full native inventory.

The shared native provider owns TextBox SelStart/SelLength/SelText, MultiLine,
WordWrap, EnterKeyBehavior, TabKeyBehavior, EnterFieldBehavior and PasswordChar.
Properties and UTF-16 text/selection live in ordinary generated VBA classes. No
VM opcode, IR version, mutable proxy store, new dependency or exchange format is
introduced. Existing native import bounds and VM work/storage limits apply.

SelStart and SelLength use Long and validate nonnegative positions; out-of-range
positive values clamp to available text. Setting SelStart clears selection.
SelText reads/replaces the selected UTF-16 substring; replacement collapses the
selection at its end. Programmatic replacement uses the existing Change event
and does not mark an update dirty. Text and selection commit before Change, so
pausing a handler preserves one canonical result. Positions remain valid without
focus. Native input transports text and selection atomically through the same
host/session/modal/source validation and authorization as other form input.
Locked text remains selectable but user input cannot modify its value.

MSForms.fmEnterFieldBehavior and its two standard constants are explicit provider
reference metadata. Existing library availability/version and conflicting names
remain authoritative; no COM/type-library discovery is performed. SelectAll and
RecallSelection operate on focus transitions, and authored Enter code can adjust
the selection before delivery completes. There is no implicit evaluation in the
renderer or debugger inspection.

The native MorphData decoder admits selected documented VariousPropertyBits and
PasswordChar (including the two-byte field alignment). Unknown flags and
inapplicable non-TextBox fields still refuse atomically. Original source/resources
remain byte-for-byte independent of runtime changes. Import alone executes nothing.
PasswordChar uses the first UTF-16 unit and an empty/zero character clears masking.
The UI displays the requested character without replacing the VBA value and keeps
password input classified as password editing by the widget toolkit.

The UI translates between Unicode character positions and native UTF-16 offsets,
including surrogate pairs, isolated UTF-16 units and CRLF. A caret-only update preserves existing line
endings exactly; edited multiline input uses CRLF. Multiline Enter/Tab behavior
uses the native properties; Ctrl+Enter and Ctrl+Tab create a new line for multiline
controls and have no effect for single-line controls, following the published
Forms contract. Otherwise Tab/Enter use focus/default-button routing. WordWrap
only affects multiline display. Rendering never changes installed host fonts or
global font policy. A missing display glyph receives a one-character fallback,
keeping editing positions and canonical Unicode data intact rather than panicking. Edits preserve unchanged raw
UTF-16 units around the changed span; caret-only input transports the original
VbaString rather than a lossy rendering copy.

Authored tests cover native flags/resource identity and refusal; UTF-16 selection,
clamping/error 380 and replacement without focus; ordinary Change Stop/resume;
recall and metadata initialization; actual Unicode/Enter/Tab/selection input;
and persisted custom password rendering with unchanged canonical text. Independent
Rust fixture/browser paths cover native text and password input. No Office-
produced resource, downloaded macro, proprietary asset or other interpreter is
adopted or executed.

Sources: [selection start](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/selstart-property),
[length](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/sellength-property),
[text](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/seltext-property),
[multiline](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/multiline-property),
[Enter](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/enterkeybehavior-property),
[Tab](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/tabkeybehavior-property),
[password](https://learn.microsoft.com/en-us/office/vba/language/concepts/forms/passwordchar-property),
[VariousPropertiesBitfield](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/c38cc3f9-969b-45da-ab41-b361c044431e),
and [MorphDataPropMask](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/19014e19-67fa-4060-8c81-d1463809f117).
Public/specification and authored expectations are distinct from fresh measured
Office traces, still NOT_RUN under M8.5. The replacement-caret convention and
additional coercion, selection/focus edge cases remain subject to that comparison.
Typed keyboard/mouse events, other native control/container/list/resource families,
bindings and applicable worksheet ActiveX/platform packets remain M7A.4 work.
Native edited persistence remains M7A.5; the legacy codec is unchanged.
