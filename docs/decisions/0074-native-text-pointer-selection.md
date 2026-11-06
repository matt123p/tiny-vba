# 0074 — Native TextBox pointer selection

Date: 2026-11-06. Status: implemented; validation recorded in progress. M01 and
M7A.4 remain PARTIAL. Fresh Office comparisons remain M8.5 NOT_RUN.

## Ownership and execution

Native TextBox presses, drags and double-clicks use ADR 0073's bounded raw pointer
queue even when there are no authored mouse handlers. Ordinary VBA mouse/focus
callbacks finish before default caret or selection processing. Double-click
cancellation suppresses its default selection; a redundant move retains the
selected word. Capture permits dragging/releasing outside the original control.
TextBox has no Click event. Non-primary buttons do not place the caret.
Single-line hit testing uses horizontal position even when capture leaves the
row vertically. The execution barrier no longer inserts an extra message row
that re-centers modal forms while input is pending; existing disabled controls
and debugger status retain execution feedback. Simple-control presses arriving
behind a pending keyboard callback are retained for that same form. The standalone
app requests a repaint when dispatch/ticking changes execution mode, so a final
Completed/Break status is rendered even when execution no longer requests ticks.

The renderer publishes immutable font, wrap-width, galley-origin and widget-ID
geometry. After callbacks, the adapter re-reads canonical UTF-16 text and text
properties and lays out the current display. It converts the pointer cursor back
to UTF-16 offsets, preserving CRLF and isolated surrogates. Only the raw gesture
anchor is retained; no second mutable value/selection model is introduced.
Existing InstanceText executes the canonical selection update through ordinary
VBA frames and the existing executable-command authorization classifier. A
selection-only change does not replace text, mark it dirty or raise Change.

The raw barrier waits for both mouse callbacks and selection updates before
processing the next queued input. A completed callback's canonical ActiveControl
is restored to widget focus before another input can be inferred from old widget
focus. Defaults re-check the target identity, visibility and active control;
focus redirects are respected. Hover-only modeless mouse callbacks preserve
source-editor focus. Stop/Continue retains pending input once; Reset,
source changes and stale identity discard it. No execution, memory, permission or
metadata limit changes, generic VM/IR changes, resource writeback or .tvform
extensions are needed.

## AutoWordSelect

TextBox/RefEdit generated text classes expose the Boolean AutoWordSelect field;
new dynamic text controls start True. The native TextBehavior metadata records
its stored value. MS-OFORMS VariousPropertiesBitfield bit 27 encodes character
selection when set, so the public property is its inverse. The binary format's
default 0x2C80081B therefore imports as False; this is deliberately distinct from
the documented public property default True for new controls. Clearing that bit
imports as True. Other unsupported bits remain refusals. Original resources are
retained byte for byte.

The TextBox adapter extends selections by characters or whole words using this
canonical field and the injected egui font/cursor helpers. Double-click selects
words only after Cancel returns False; shift-click extends the existing anchor.
Exact platform/locale word boundaries, selection margins, auto-scroll, text drag
and drop, further multiwindow/reentrant geometry behavior, and editable ComboBox/
RefEdit pointer defaults still need acceptance. This implementation does not
claim those cases or full M01/M7A.4 parity.

## Evidence

Authored decoder cases distinguish the inverse bit and preserve original bytes.
Shared native/WASI model cases check persisted and dynamic defaults. Actual egui
cases cover stopped text replacement before caret placement, no-handler outside
drag without Change, double-click allow/cancel, focus redirects, UTF-16/CRLF,
shift-click and Reset/source retention. Chromium imports an authored native pair,
stops TextBox MouseDown, continues a held drag outside the control, checks the
canonical selection in MouseUp and checks unchanged source. Gate receipts and
consumer revisions are recorded in implementation-progress.md.

## Primary references

- [MSForms DblClick](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/dblclick-event)
- [MSForms AutoWordSelect](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/autowordselect-property)
- [MS-OFORMS VariousPropertyBits defaults](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/7a72ac4a-39d9-4e2b-829e-19e3e9a1f60d)
- [MS-OFORMS bit layout](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/c38cc3f9-969b-45da-ab41-b361c044431e)

These are reference-derived authored expectations, not measured Office traces.
