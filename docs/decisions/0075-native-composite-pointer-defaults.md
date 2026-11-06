# 0075 — Native composite, list, numeric and tab pointer defaults

Date: 2026-11-06. Status: implemented; validation receipts recorded in progress.
M01 and M7A.4 remain PARTIAL. Fresh Office comparisons remain M8.5 NOT_RUN.

## Ordinary callbacks and canonical state

The raw mouse barrier now projects independent text, drop-button, list-row,
arrow, track, thumb and tab-header regions from the actual renderer. Values,
selection, drop state, tab order and callback completion stay in ordinary VBA
classes. The adapter retains only identities, immutable geometry and bounded raw
input/gesture anchors. All executable defaults use the existing InstanceText,
InstanceDrop, InstanceList/InstanceListSelection, InstanceNumeric or InstanceTab
commands and their existing authorization/host-snapshot path. No VM/IR changes,
new format, second mutable control model or budget changes are introduced.

Editable ComboBox/RefEdit text uses the TextBox pointer path after mouse handlers
finish. Drop/picker buttons are separate hit regions, so arrow clicks cannot place
a caret. Mutable DblClick cancellation suppresses the second default. Eligible
release toggles current canonical drop state; a callback's focus redirect suppresses
the default. RefEdit's compact picker also registers input geometry and keyboard
focus, retaining the existing modal/capability requirement and host-owned worksheet
selection. No standalone worksheet-selection service or Word picker is invented.

Popup rows take hit priority over underlying form controls. Popup contents inherit
the callback barrier's disabled state; they cannot run widget defaults while a
handler is stopped. List/ComboBox row release uses current rendered eligibility and
the existing ordinary selection/change/click/drop methods. Shift/Ctrl modifiers
remain raw input. Existing unhandled widget input remains available when no mouse
callback/barrier needs to intercept it; text still uses canonical pointer selection.

Numeric arrows commit a user step after MouseDown finishes. Hold repetition uses
injected egui time and canonical Delay and repeats default actions, never physical
MouseDown callbacks. Release stops repetition even during Stop. Track release uses
the existing large step; thumb motion uses current Min/Max and immutable geometry.
Only the raw grab offset and last pointer sample are retained. Duplicate stationary
motion cannot replay a drag after an authored handler resets Value. Repeats never
catch up in a burst after a stopped handler. Existing ordinary numeric event order
and checked Long/range behavior remain authoritative.

TabStrip/MultiPage headers retain the tab/page object's identity. Inert SDK queries
validate current position/membership through ordinary position cells and collections;
no getters, constructors or callbacks run during inspection. Mouse arguments use
the member's current Long index. A queued release follows the same member through
reordering; removal, disabled/hidden membership, Reset and stale identities suppress
its default. This adds no external object registry or new lifetime root.

## Display modality and fixed control geometry

The existing input-owner query continues to require the current VM modal wait.
A separate inert has-modal-window query reads retained form-wait ownership so the
renderer does not temporarily turn a modal form into a modeless window during its
callback. It grants no input permission. Headless tests verify that stopped input
still refuses and Reset revokes the display metadata.

Manually positioned runtime controls render in child UIs bounded by their native
rectangles, without advancing the form's flow cursor. Popup state cannot resize
or reposition the parent form through a control's layout allocation. This preserves
native bounds and header/capture coordinates during callbacks and popup changes.

## Native ComboBox AutoWordSelect

ComboBox now exposes and initializes the same documented Boolean AutoWordSelect
property as TextBox. New dynamic ComboBox controls start True. Native MorphData
bit 27 has the inverse representation described in ADR 0074; stored defaults import
False and a cleared bit imports True. Original resources remain unchanged. Other
unsupported flags retain explicit refusals.

## Evidence and remaining acceptance

Authored decoder/model tests cover both stored bit values, dynamic defaults,
retained modal metadata without input permission and inert tab identity/mutation.
Actual egui tests cover stopped text/drop/picker/list/numeric/tab callbacks,
selection and popup close, mutable cancellation, focus redirects, compact RefEdit,
hold/release, stationary drag reset, indexed arguments and reordered/removed tabs.
Chromium imports an authored native pair and stops arrow/row callbacks before
applying drop/selection defaults; source retention and final value are checked.
Validation and consumer pins are recorded in implementation-progress.md.

This does not finish M01 or the full milestone. Exact platform/geometry/event
ordering, further word/selection/drag/wheel/scrollbar/list behavior, list mutation
and binding reentrancy, tab/page background hits, duplicate/non-ASCII accelerators,
full lifecycle/focus/keyboard/IME and remaining property/resource/host packets in
the existing checklist remain open. Native edited persistence and .tvform removal
remain M7A.5/.6; they do not replace missing M7A.4 runtime behavior.

Primary references: [MSForms DropButtonClick](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/dropbuttonclick-event),
[AutoWordSelect](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/autowordselect-property),
[DblClick](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/dblclick-event)
and the native format references in [ADR 0074](0074-native-text-pointer-selection.md).
These are reference-derived authored expectations, not measured Office traces.
