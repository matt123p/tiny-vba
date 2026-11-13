# ADR 0048 — native container focus and validation

Date: 2026-11-04. Status: adopted for the flat native control profile.
M7A.4 remains in progress for its unchanged broader scope.

The IDE provider owns native ActiveControl/SetFocus and container Enter/Exit,
TextBox BeforeUpdate/AfterUpdate delivery. Canonical control references and focus/dirty
state live in ordinary generated classes. There is no form opcode, second object
registry, new format or native dependency. Focus methods, typed arguments and
continuations use verified ordinary VBA frames and existing VM budgets.

Native form source accepts private no-argument Enter/AfterUpdate handlers and
private Exit/BeforeUpdate handlers with exactly one ByVal MSForms.ReturnBoolean
parameter. Import/signature checks remain inert and atomic. Generated handler
commands use those signatures. These container events do not become leaf
MSForms WithEvents events; ADR 0047's event exposure remains unchanged.

ActiveControl is read-only. Initial Show selects the first visible enabled
non-label TabStop control by current TabIndex; Hide preserves its selection and
Unload releases it. SetFocus can select a non-tab-stop control. Unavailable
focus targets fail with error 2110 and preserve the prior selection. Physical
native input is one ordinary form callback: focus validation completes before
its text/click action. Default/Cancel keyboard activation uses an ordinary button
callback without moving the current control focus. Callback authorization, host identity, modal ownership,
stale source and debugger boundaries apply equally to focus requests.

User input marks TextBox controls dirty; programmatic setters do not schedule
BeforeUpdate/AfterUpdate. A transition runs BeforeUpdate, AfterUpdate, Exit and
Enter in order when applicable. Each cancellation object is an ordinary native
return holder with shared ByVal identity and a default Value property. A cancelled
BeforeUpdate retains focus and suppresses AfterUpdate/Exit/Enter/input; a cancelled
Exit retains focus and suppresses Enter/input. Repeated requests to the active
control are inert. Source-driven redirection or Hide/Unload during a callback
invalidates an older transition through a bounded ordinary serial counter and
loaded/visible checks. Errors, Stop, nested waits and Reset retain normal frame
and cancellation behavior; no user cleanup is executed on Reset.

The UI emits explicitly classified executable focus commands. Tab/Shift+Tab and
pointer focus use the same authorized callback path. Rendering only caches the
last painted active object to synchronize widget focus after ordinary code runs;
VBA state is authoritative through cancellation and debugger waits. It never
executes getters during inspection. Editor and modeless window activation edge
cases remain part of the broader lifecycle requirement.

Authored evidence covers programmatic focus, canonical identity, disabled targets,
non-tab-stop selection, programmatic writes without update callbacks, typed
signature refusal/generation, cancelled/accepted validation with Stop/resume,
modal caller continuation, Exit cancellation and Reset. Actual egui input covers
Tab/cancelled Tab/Shift+Tab. A separate independently authored native browser
fixture checks real text input, typed validation, cancellation, subsequent typing
in the retained TextBox, ordered delivery and exact source preservation.
No Office-produced asset or downloaded macro is adopted or executed. Fresh Office
build/locale observations remain NOT_RUN under M8.5.

Source contracts: [ActiveControl](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/activecontrol-property),
[SetFocus](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/setfocus-method),
[Enter/Exit](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/enter-exit-events),
[BeforeUpdate](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/beforeupdate-event),
[AfterUpdate](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/afterupdate-event)
and [ByVal return objects](https://learn.microsoft.com/en-us/office/vba/language/concepts/forms/byval-references-in-microsoft-forms).
A [firsthand published Word UserForm report](https://learn.microsoft.com/en-us/answers/questions/4772989/cant-move-focus-to-the-control)
also identifies error 2110 for invisible/disabled/non-focusable targets. That is
reported evidence, not a fresh observation or an adopted macro/fixture.
These public contracts are not fresh measured event traces. Recursive redirection,
container/activation boundaries, boolean-family update timing, full update/binding/coercion and keyboard/mouse
ordering, selection/multiline/password, further native families/resources and
platform packets remain in the [runtime work record](../forms.md).
