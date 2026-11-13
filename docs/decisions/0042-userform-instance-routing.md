# ADR 0042 — UserForm instance identity and window ownership

Date: 2026-11-04. Status: adopted for the first M7A.4 / FC3.5 implementation packet.
The milestone remains in progress. This supersedes ADR 0024's restriction against
explicit New form instances; the remaining M2 lifecycle limitations stay explicit.

Microsoft's [UserForm reference](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/userform-object)
permits multiple instances using New. The shared runtime already owns ordinary
project classes and native instance fields, so a second form object store or an
Office-shaped VM type is unnecessary. Remove the IDE's lexical New refusal and
reuse ordinary construction, WithEvents delivery, coercion and checkpoint frames.

The VM exposes bounded, inert class-instance inspection. It never constructs a
default instance or executes a property. Class receivers on active and parked
invocation frames are lifetime roots, just like their locals/arguments. This fixes
a generic class lifetime hole exposed by explicit form callbacks; no Office name,
UI dependency, new bytecode, capability or ambient service enters the VM.

The IDE uses FormInstance { source, object }: the designer/source component is
separate from a runtime ObjectRef, whose generation identifies the VM session.
Instance queries/events validate the compiled class and owner before use. Existing
SourceId-only APIs continue to address the predeclared instance. The renderer
creates distinct window/focus/widget scopes per runtime instance and emits
instance-addressed commands; the same component can now have multiple windows.
Window titles use runtime Caption and remain independent of their identity.

Generated form support privately retains a visible instance while its window is
shown and releases that reference on Hide/Unload. Hidden instances with another
VBA reference remain available. A normally collected loaded instance forwards
Class_Terminate to its authored UserForm_Terminate; normal Unload's existing
explicit handler path prevents duplicate delivery. Reset/close never execute
cleanup code. This is implementation reasoning based on documented object/window
lifetime, not a fresh measured Office event trace.

Modal waits remain keyed to the actual object. Only the top waiting modal instance
owns input; other forms cannot reenter it. Parked caller frames, ByRef locations,
error state and work budgets retain the existing suspension contract. Failed,
paused/stale, foreign and reset-instance input refuses without replay.

Native source/resources and design remain unchanged by runtime values. No native
encoder or custom-format extension is added. Full default-instance replacement,
loaded UserForms collection/type binding, activation/focus/event ordering, broader
controls, host pin updates and fresh Office comparisons remain later M7A.4 work.
The exact implementation and evidence are in [the packet record](../forms.md).
