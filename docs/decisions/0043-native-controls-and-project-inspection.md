# ADR 0043 — native control properties and collection inspection

Date: 2026-11-04. Status: adopted for the second M7A.4 implementation increment.
M7A.4 remains in progress; this is not its full acceptance decision.

Extend the existing flat native form profile with CheckBox, OptionButton and
ToggleButton. Decode cached native class IDs, matching MorphData DisplayStyle,
Value/Caption/GroupName and Locked from MS-OFORMS. Retain original resources and
refuse unimplemented classes, containers, bindings and property masks. Native
edited encoding remains M7A.5. Do not extend the legacy .tvform codec to admit
these controls; its encoder refuses them. No asset or external macro is adopted.

Generated ordinary classes own control values and existing WithEvents delivery.
Boolean Value retains Null; assigning Null raises Change without Click. Option
selection clears peers in its root container/group; no Click is emitted for a
peer becoming False. Input honors Locked. TripleState changes user cycling only.
The UI emits a click, not a second authoritative value write. These are documented
semantic rules and authored tests, not fresh Office event-order observations.

TabIndex uses Integer, rejects negative values, clamps to the last position and
moves intervening indices. Generated internal events keep this ordering within
one form instance. Default/Cancel are exclusive within that instance; Enter/Escape
use eligible buttons in the input-owning form. TextBox MaxLength limits user input,
not programmatic Text/Value writes. Property names such as Name and Default are
contextual in property declarations/results; ordinary reserved identifiers stay
refused. Load/Unload accept Object-valued expressions through normal dispatch.

Controls and UserForms reuse ordinary classes and canonical VM objects. Controls
holds the existing child objects in a bounded array and supports Count plus Item
by zero-based index/name. UserForms Count/Item query loaded native instances;
Add constructs the named project class through ordinary New. Neither counting nor
inspection constructs a default form. Enumeration and broader type bindings are
still gaps, not invented array-valued VBA enumerator behavior.

For UserForms, vba-foreign adds an optional transient ProjectObjectView host hook.
vba-vm supplies bounded class enumeration and public canonical field reads during
one already-authorized host call. The view rejects private fields, getters,
stale/foreign handles and oversized requests/results. It cannot mutate storage,
execute code or outlive dispatch. Queries consume existing instance-work and
reply-payload allowances. The foreign call's argument validation, permissions,
writeback and suspension path is unchanged. Existing hosts default to invoke;
there is no new bytecode, dependency edge, ambient capability or Office name in
the VM. No parallel mutable instance registry is created in the IDE/host.

The form wrapper forwards context-aware calls and intrinsic host services through
the same existing wait-token routing. In particular adding a form cannot replace
an embedding's MsgBox provider. Reset cancels tokens without running cleanup.

Scope, authored evidence, current gates and remaining M7A.4 work are maintained in
[the runtime record](../m7a4-native-runtime.md). Fresh Excel/Word observations remain
NOT_RUN under M8.5; public specifications and authored tests remain distinct.
