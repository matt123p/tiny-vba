# ADR 0044 — UserForm window, default reference and code lifetime

Date: 2026-11-04. Status: adopted for selected M7A.4 lifecycle behavior.
This supersedes the earlier M2/ADR 0042 unload field-clearing and immediate
Terminate forwarding. It does not sign off the entire M7A.4 milestone.

[MS-VBAL Unload](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/c8116615-fea7-4aa2-84af-ae4b56a903fd)
distinguishes the displayed component from referenced code state and describes
reload on control access. [Word's control-value guidance](https://learn.microsoft.com/en-us/office/vba/word/concepts/objects-properties-methods/using-control-values-while-code-is-running-word)
describes initial control values after unload. [Terminate](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/terminate-event-visual-basic-for-applications)
depends on removal of the last reference. Keep specification evidence distinct
from fresh Office observations (still NOT_RUN under M8.5).

Generate read-only named control properties over canonical internal WithEvents
fields. On access after unload, recreate the window/control state and run the
supported Initialize path. Generated event forwarding invokes unchanged authored
handlers, preserving their source checkpoints. Inert UI/inspection reads backing
fields directly and never cause this reload. Controls and form display properties
use the same reload path. No authored source or resource bytes are rewritten.

Successful Unload clears displayed controls/collection state and releases the
form's default reference. It preserves authored code fields if another reference
or active/parked frame still owns that instance. Subsequent default access creates
a fresh class object; an explicit alias retains the previous code object and can
reload its own controls. QueryClose cancellation retains identity and values.
Unload does not synthesize Deactivate, as specified by the VBA event reference.
Normal Terminate forwards from actual class finalization, including when the
window was unloaded earlier. Reset/abnormal cleanup never execute user cleanup.
Default replacement/alias expectations are implementation reasoning plus authored
regressions, not a fresh Excel/Word object-identity trace; broader alias cases stay
in the remaining-work record until independently compared.

The VM provides a generic release_project_object(name, expected_object) API that
only removes a matching predeclared root. It validates the current object/class,
leaves ordinary references/receivers intact and refuses document code-name roots.
It does not execute cleanup or introduce an Office type/opcode/dependency.

The IDE uses an existing host-wait boundary to perform this release between VM
slices. Pending release records contain only the bounded component name/handle,
not another object store. They share existing tokens/cancellation machinery and
are capped at eight. The IDE reports these internal continuations as Running,
resolves the root before resuming, and preserves the per-tick work quota. There
is no user prompt or automatic execution permission. Modal callbacks, Stop,
Reset and host-owned waits retain their existing ownership/replay contracts.

Evidence: the [native lifecycle regressions](../../crates/vba-ide/tests/form_instances.rs)
assert preserved code fields, reset control values, independent replacement,
delayed exactly-once termination, cancellation and paused Reset. The
[generic VM regression](../../crates/vba-vm/tests/classes.rs) asserts root release,
retained aliases and fresh default construction without any Office definitions.
