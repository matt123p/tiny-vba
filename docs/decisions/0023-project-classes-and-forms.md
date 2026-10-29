# ADR 0023 — Project-local instances and form execution

Date: 2026-10-28. Status: implementation in progress.

Updated 2026-10-29: the generic class/document-state core is implemented in IR 7.
Declared events/WithEvents/RaiseEvent and the three editor error modes are implemented.
Form model/designer, static binding expansion and full acceptance remain pending;
see [the staged record](../m2-classes-and-forms.md).

Class modules are project source, separate from registered foreign object types.
Their fields use VM-owned canonical locations; methods execute as ordinary
checkpointed frames. Properties retain Get/Let/Set intent and indexed arguments.
Private access, nominal class identity, instance/session lifetime and resource
limits must be checked before effects. Lifecycle methods execute through the
same interpreter, never through a nested interpreter or a host implementation.

An explicit verified IR capability will distinguish this execution contract
from IR 1–6. Legacy consumers must refuse the new metadata and operations.
Debuggers retain inert watches and generation-checked frames. Form events must
use the existing controller and suspension model; design state must not be
silently replaced by run state. A Run command remains unavailable until real
event execution is proved.

Standard Office control names and reusable form UI belong to tiny-vba under
the existing ownership decision. Excel/Word objects, document roots/events,
trust, engine effects and package persistence remain fork-owned. Third-party
ActiveX/OLE loading and arbitrary native activation remain excluded.
