# M7A.4 first packet — explicit UserForm instances

Updated 2026-11-04. **Implemented in shared tiny-vba; M7A.4 remains IN PROGRESS.**
[ADR 0042](decisions/0042-userform-instance-routing.md) replaces the earlier custom
restriction to predeclared form instances. This packet admits explicit Set/New
construction of the existing flat native Label/TextBox/CommandButton form profile.
It does not certify all native MSForms semantics or all variants of New/As New.

An imported component supplies one source/designer and any number of independent
runtime instances within the existing VM budgets (1,024 native class instances,
4,096 field locations, eight nested suspended callbacks, and the caller's lower
instruction/payload limits). Every form's controls use its own canonical class
fields and WithEvents connections. Explicit and predeclared instances can coexist.
An inspection or UI redraw never constructs a default instance.

Rust callers enumerate Ide::form_instances and use FormInstance keys with
form_instance_field/event/visible/is_modal/can_deliver. Existing SourceId-only
methods address the default instance for compatibility. Keys combine source and
session-stamped ObjectRef identity; forged/stale/reset keys refuse. UI windows,
focus order, text edits, clicks and close actions use those same keys. Runtime
Caption supplies the window title without becoming its identity.

Shown modeless instances survive the procedure that created them. Hide releases
the window's reference; another authored reference can retain the hidden form.
Normal collection forwards the supported Terminate handler once. The existing
Unload support invokes QueryClose and respects cancellation. Modal callbacks use
the actual top waiting object, including nested instances of the same component.
Debug pause/continue retains fields and resumes the parked caller without replay;
Reset releases runtime state without user cleanup or design/resource mutation.

The VM changes are generic: bounded read-only class inspection and correct lifetime
roots for active and suspended method receivers/locals. Form design, resource
interpretation, source generation and UI orchestration stay in the existing upper
layers. No new crate/dependency edge, IR capability, VBA feature, exchange format
or parallel mutable object registry is introduced.

Evidence uses the existing original Rust native .frm/.frx fixture builder under
MIT OR Apache-2.0; no Office-produced artifact or downloaded macro is executed:

- [Native instance tests](../crates/vba-ide/tests/form_instances.rs): separate fields/handlers, default coexistence, foreign keys, modeless ownership, nested modal ownership, breakpoint/resume, termination and inert Reset.
- [Form execution regressions](../crates/vba-ide/tests/form_execution.rs): the earlier refusal case is replaced with actual explicit construction; default form behavior, aliases, events and host-wait token regressions remain.
- [VM regression](../crates/vba-vm/tests/classes.rs): clearing the last variable inside a method cannot terminate its receiver before the method returns.
- [Actual egui input](../crates/vba-ui-egui/tests/form_instances.rs): two windows/control sets from one native source, each button invokes its own handler and hides its own instance.
- [Render builder](../crates/vba-ui-egui/examples/native_instances_snapshots.rs): light/dark/narrow/scaled/injected-host theme views of actual independent runtime instances.

Validation results are recorded in [implementation progress](implementation-progress.md).
The shared compiler/runtime/IDE/UI gates and authored evidence are distinct from
fresh Office observation. Microsoft documents [explicit instances](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/userform-object),
[modal/modeless Show](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/show-method)
and [normal/abnormal Terminate](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/terminate-event-visual-basic-for-applications).
Fresh Excel/Word build/OS/locale comparisons remain **NOT_RUN** under M8.5.

Next M7A.4 work: exact default-instance Load/Unload/alias identity and lifetime,
UserForms collection/type bindings, activation/focus/tab/default/cancel/validation
and event ordering, typed property semantics, then broader native control packets
and the separately inventoried Office-supplied ActiveX/platform profile. Decoder
coverage remains the M7A.2 three-control profile. Native saving stays M7A.5;
.tvform removal stays M7A.6. Forks remain pinned to their last accepted shared
revision until their separate integration gates are run; this packet alone does
not advertise new instance behavior in already-pinned GridCraft/WordCraft builds.
