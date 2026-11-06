# 0073 — Native typed mouse callbacks and capture

Date: 2026-11-06. Status: adopted for the implemented cases below. M7A.4 and
completion row M01 remain PARTIAL. Fresh Office comparisons remain M8.5 NOT_RUN.

## Contract and ownership

The IDE generates ordinary MouseDown/MouseMove/MouseUp and DblClick events for
native controls, except Page, whose events belong to MultiPage. TabStrip and
MultiPage callbacks carry a Long index; other controls and UserForm do not.
Button/Shift accept Integer and the corresponding MSForms enum aliases; X/Y
are Single. DblClick receives a mutable ByVal MSForms.ReturnBoolean. The Forms
provider adds the published fmButton constants without changing the generic VM.
Source attachment and handler generation validate signatures before publication.

The authorized native mouse command enters an ordinary form callback, retains
its receiver and publishes canonical epoch/done/allow fields only on completion.
Focus validation can suppress a press before its handler. Detached, hidden,
disabled, replaced or ineligible receivers suppress subsequent default processing.
Stop/Continue preserves frames and event arguments; Reset/source changes invalidate
receipts without running cleanup. No host permissions or execution/memory limits
increase, external activation, second mutable control store, IR version, native
resource format or `.tvform` extension is introduced.

The shared UI retains only bounded raw pointer data, identities, geometry and a
focus bookmark. Capture routes subsequent buttons/moves/releases to the original
object until all buttons are released, including release while a handler is
stopped. Raw click timing uses the injected egui input policy; callback execution
delay does not change double-click classification. Consecutive compatible motion
samples can coalesce. At most 64 queued raw events and 16,384 geometry records are
admitted; overflow reports a budget diagnostic. Receiver identity is revalidated
before dispatch, and Reset clears capture, queues and bookmarks.

For the selected UserForm background, Label, CommandButton, CheckBox,
OptionButton, ToggleButton, Image and Frame paths, default action waits for the
mouse receipt. A double-click uses DblClick on its second press and retains the
following MouseUp; a true Cancel suppresses the second default activation.
Simple control Click actions use their existing ordinary value/event methods;
UserForm.Click runs its authored handler without closing the form. A temporary
focus bookmark and keyboard dispatch arbitration retain text arriving during a
mouse callback without moving it into the source editor.

Event declarations now retain typed arguments and receiver metadata without
copying unused instance-field aliases into their signature-only local tables.
Executable methods retain all field aliases and nominal checks. A generic compiler
regression with 256 typed fields and 20 events proves the distinction; the previous
representation exceeded the unchanged 4,096 nominal-local metadata cap in two
existing GridCraft MultiPage cases. No permission, metadata or runtime budget is
raised, and all public mouse event declarations/forwarding remain available.

## Evidence and remaining work

Authored runtime cases cover masks/Single arguments, enum widths, indexed events,
mutable cancellation, root/nonfocusable events, focus validation, dynamic
WithEvents removal, Stop/Continue, Reset, invalid inputs and unchanged resources.
Actual egui cases cover captured/outside release, stopped release, cancelled
second toggle, source-preserving Reset and text during a mouse callback. Chromium
imports native `.frm`/`.frx`, stops MouseDown, resumes through MouseUp and one Click,
and verifies exact source retention. Full validation receipts live in progress.

This does not finish M01. Text selection, list/numeric/drop/picker defaults and
indexed Tab/Page hit routing need further raw-pointer integration; unsupported
pointer defaults report explicitly rather than editing the control first. Further
capture, geometry-change, multiwindow, button-specific double-click defaults,
focus/event ordering, mixed input and platform timing cases remain open. Down/Up
coordinates currently follow the parent form/Frame/Page basis in the reference;
Move follows its control-local basis. Their differing wording and exact Office
behavior need explicit comparison, not an invented measured trace. Native edited
persistence/removal remain M7A.5/.6 and are not runtime acceptance substitutes.

## Primary references

- [MSForms MouseDown/MouseUp](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/mousedown-mouseup-events)
- [MSForms MouseMove](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/mousemove-event)
- [MSForms DblClick](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/dblclick-event)
- [MSForms Click](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/click-event)

Access/Outlook control events are not used as an Excel/Word MSForms oracle.
