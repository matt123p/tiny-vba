# ADR 0096: Native button click focus and input identity

Date: 2026-11-08. Status: adopted; acceptance in progress. M7A.4 remains open.

CommandButtonPropMask bit 9 encodes the nondefault TakeFocusOnClick value without
a DataBlock payload. The reader admits it as False; absence means True. Original
resources remain separate. The IDE seeds ordinary Boolean state before authored
initialization and gives runtime-created CommandButtons the same True default.
Other native control interfaces do not gain this property. Normal Boolean
coercion, Null/type errors and inert field inspection use existing mechanisms.

Mouse-down focus and trusted physical-click input consult the policy. False
preserves the current control, selection and pending dirty validation while still
raising ordinary button callbacks. Tab, Space after Tab, explicit SetFocus and
an authored handler's focus request retain their existing paths. Changes during
a click govern later presses. False buttons use the raw mouse barrier even when
there are no authored mouse handlers, so egui cannot transfer text focus early.
Stop postpones release/Click, Continue delivers once, and Reset discards old input.

Collection mutations exposed two input-identity problems. A temporary lookup
failure between ordinary collection writes must not discard a queued release.
The UI keeps bounded receipts while callbacks cannot accept input, then checks
control identity and Tab/Page membership before delivery. Hiding or releasing the
owning form and generation changes still invalidate input. No queued event gains
execution permission. After callbacks the generated runtime checks the captured
control's object membership, rather than requiring its original collection index
to remain unchanged. The usual unchanged-index path stays constant time; a shifted
index uses a bounded scan of the canonical collection. Removing a peer preserves
the target; removing the actual target still cancels its default processing.

IDE ticks use min(256, session.limits().slice). This fixes rejected launches and
continuations when a caller chooses a smaller positive slice. The active session's
limits, not future launch configuration, determine its quota. Instruction, memory,
stack, callback and host-permission limits are unchanged; no budget is raised.

Eight model cases and ten real input cases cover import/defaults/types/coercions,
selection and deferred validation, true/false focus, Tab/Space, explicit focus,
runtime policy changes, nested runtime buttons, Stop/Continue/Reset and peer
removal during a paused press. An embedding regression covers slices 1/64/128,
Stop and future configuration changes. The retained tab reorder/removal test also
runs at a smaller slice, with its original identity/value expectations. Full
shared, WASI/browser/theme and pinned host results are recorded after running.

This does not complete live collection enumeration, remaining focus/lifecycle,
keyboard/accelerator/platform or property/control requirements. The full M7A.4
checklist stays authoritative. Native edited persistence and legacy removal stay
M7A.5/.6; fresh Office observations remain M8.5 NOT_RUN.

References: [MSForms TakeFocusOnClick](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/takefocusonclick-property),
[MS-OFORMS TakeFocusOnClick](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/f53f9f1d-38fe-4e4e-ae5c-1c3f73b124b3),
[CommandButtonPropMask](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/5c5ad1f1-d511-4f2e-b9c2-35cc31be8061),
[MSForms mouse capture](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/mousedown-mouseup-events).
