# ADR 0050 — native SpinButton and ScrollBar

Date: 2026-11-04. Status: adopted for the selected native numeric packet.
Full M7A.4 remains in progress for its unchanged inventory/property/platform scope.

The native decoder admits cached SpinButton (16) and ScrollBar (47) records,
including their distinct masks/data layouts and absence of a TextProps tail.
Range, position, increments, orientation, delay, enabled/site state and exact
resource identity are retained. Invalid positions, orientation, increments,
reserved bits and inconsistent enabled caches refuse the entire attachment.
Mouse resources, nondefault proportional-thumb packets and unknown variants
remain explicitly unsupported. No COM/OCX activation or vendor fixture is used.

Ordinary generated classes own Long Value/Min/Max/SmallChange, Orientation and
Delay; ScrollBar also owns LargeChange. MSForms nominal interfaces expose the
implemented standard event names, while keeping internal focus events private to
ordinary project-class connections. fmOrientation and its three constants use
explicit reference metadata; unavailable/conflicting references still refuse.
Numeric controls do not gain a fictitious Caption property.

Value validates the inclusive range regardless of Min/Max order and raises
Change only when the value changes. Changing an endpoint clamps the current
value when necessary. Negative increments/delay, invalid orientation and values
outside the live range raise error 380 through ordinary VBA coercion/error paths.
Stepping uses exact integral Double intermediates before narrowing/clamping to
Long, avoiding overflow for valid Long endpoints/increments. Native Long minimum
initialization uses an ordinary valid constant expression rather than a positive
literal beyond the compiler's admitted decimal width.

Native physical input uses the same session/source/host/modal validation and
ordinary focus/validation frames as other controls. SpinUp/SpinDown follow the
committed Change callback, including Stop/wait/resume. ScrollBar steps move toward
the Min/Max locations, including reversed ranges; only thumb dragging raises
Scroll. Programmatic assignment and arrow/track steps do not fabricate Scroll.
Reset and trust loss keep the existing callback/cancellation and inert design
rules; no second object store or form opcode is introduced.

The UI paints original arrows, track and thumb using injected theme tokens and
canonical runtime values. It routes small/large steps, thumb input and keyboard
arrows through explicit executable form commands. Auto orientation follows the
control dimensions; held-arrow repeat uses the injected UI clock and Delay,
with at most one action submitted per frame. Stationary drag input is not
replayed when an authored handler changes Value. Stable control IDs survive
debugger pauses for focus restoration before immediate resumed text input. UI painting never writes VBA fields
or document state directly. These are the standard control families, not a new
widget format or a parallel product feature.

Original Rust fixtures cover numeric records without font tails, normal/reversed
ranges, invalid-position refusal and unchanged resources. Runtime cases cover
MSForms typing/default Value, exact error numbers, endpoint clamping, paused
Change-before-spin delivery and drag-only Scroll. Real widget input checks arrows
and keyboard values. Browser evidence uses an independently authored native pair
and ordinary modal handler Stop/resume. Fresh Office build/locale traces remain
NOT_RUN under M8.5; boundary-event, repeat/pointer, full presentation and additional
property/binding/resource cases remain explicit comparison/implementation work.

Sources: [cached classes](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/5f649a6c-90df-4226-b8c7-2a7d98cd9647),
[SpinButton mask](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/ec5e8f6b-72f7-4682-9d8c-a7fb34c71751),
[data](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/a244aa7b-4c33-499f-aaf9-4301ff29fed6),
[ScrollBar mask](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/e30addb5-0251-4421-b45d-c2f90ab0fcc1),
[data](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/993343f1-3558-46b6-8afd-18603363d944),
[format Max defaults](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/6c8edfa4-d81a-474a-a787-163e8d8a4c96),
[Min/Max](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/max-min-properties),
[spin events](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/spindown-spinup-events),
[Change](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/change-event)
and [Scroll](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/scroll-event).
Native editing/persistence remains M7A.5. The legacy codec is not expanded.
