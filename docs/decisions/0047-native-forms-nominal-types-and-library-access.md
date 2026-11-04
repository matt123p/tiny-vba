# ADR 0047 — native Forms nominal types and library access

Date: 2026-11-04. Status: adopted for selected native Forms type bindings.
M7A.4 remains in progress; focus, validation/keyboard/mouse input and remaining
control families are not signed off by this increment.

The compiler/IR/VM own generic, explicitly registered class interfaces. The
embedding supplies class-name -> nominal interface descriptors from its validated
reference snapshot. No type library, DLL or COM object is discovered/activated.
Source cannot add interface claims; this is not implementation of Implements.
The reviewed crate graph and foreign host ownership stay unchanged.

IR 11 `class-interfaces-v11` adds bounded Class.interfaces, each with nominal
identity, event-source capability and an optional exposed-event allowlist.
There are at most 32 claims per class and 128 exposed names per claim; compiler
registration preflights aggregate descriptor storage (one MiB or the lower
configured metadata budget). Verification checks the version/capability, unique
non-project identities, event-source/list consistency and actual declared event
names. Older consumers refuse this metadata. Programs without it retain their
previous IR versions. No opcode is added.

Native instances retain immutable interface descriptors derived from their
verified class. Canonical type checks accept the exact project identity or an
explicit interface. Wrong interfaces remain VBA error 13; stale handles remain
91. Typed variables, parameters/results, arrays, fields, ByRef and debugger edits
retain the existing nominal rules. Descriptor storage and queries consume
existing payload/type-work budgets. Restoration derives claims from the receiving
program and revalidates values; there is no second mutable object registry.

WithEvents validates signatures against native publishers claiming the requested
event interface. Non-source interfaces refuse connection with catchable error
459 before the location/connection changes, following the documented
[trappable error](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/trappable-errors). Both verification and delivery respect
exposed names: implementation bridge events are not exposed through a standard
Forms interface, and lookalike private helper names are not wired accidentally.
Existing project-class events and foreign host event validation remain intact.

The IDE's native provider owns Microsoft Forms names. When a project has attached
forms, compilation derives an MSForms reference snapshot and interface registrations
for UserForm, Control, Controls, six admitted leaf controls, ReturnBoolean and
ReturnInteger. Existing reference availability/version is retained; missing or
conflicting descriptors refuse without import execution or mutation. The mutable
return classes use ordinary default Value properties and shared object identity,
as specified by [Forms ByVal references](https://learn.microsoft.com/en-us/office/vba/language/concepts/forms/byval-references-in-microsoft-forms).
They prepare typed event arguments; this increment does not deliver focus/key
callbacks. The selected native Control interface is a reference type, not a
connectable leaf event source; leaf interfaces expose only the implemented
standard events. Container focus events remain separate from leaf event exposure.

The compiler also accepts bounded explicit library function aliases targeting
public standard-module Functions. Binding retains ordinary source calls,
argument/default rules and checkpoints, with no new IR operation. Project
variables/functions/imports and namespace names shadow aliases. Invalid/private,
ambiguous or duplicate registrations and assignments to library functions refuse.
The default API has no aliases and preserves its existing source behavior.

The form provider registers UserForms and VBA.UserForms to the same ordinary
support routine, whose internal name avoids a duplicate project global. Qualified
Count/Item/index/enumeration therefore survives a project Function named UserForms.
No component/default is loaded by collection queries. Both ordinary Run and idle
callback compilation use the same type/alias metadata. Existing isolated Immediate
restrictions remain; no new evaluation or reference-manager workflow is claimed.

Authored evidence covers interface identity/storage/arrays/results, wrong/stale
handles, restoration, signature checks, event exposure and error 459; library
aliases, argument/default handling, scope shadows and malformed registration;
and native typed form/control/collection access, leaf WithEvents, mutable ByVal
return objects, qualified UserForms, missing/conflicting references and exact
resource preservation. Internal bridge factory tests are not Office macro fixtures.
Public documentation and implementation expectations are distinct from fresh
Office observations, still NOT_RUN under M8.5.

The runtime remaining-work record continues to require full focus/lifecycle,
validation/key/mouse events, native property/default/coercion contracts (including
dynamic TypeName), empty-project/dynamic collection cases, As New arrays, further
native families/bindings/resources and applicable platform packets. No .tvform
extension, new public form format or full M7A.4 acceptance is introduced.
