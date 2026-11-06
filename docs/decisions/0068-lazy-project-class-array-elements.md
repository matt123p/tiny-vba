# ADR 0068 — lazy project-class array elements

Date: 2026-11-06. Status: adopted; shared validation passed.

The language/compiler/IR/VM own this extension of ADR 0046. MS-VBAL's variable
and ReDim contracts explicitly retain automatic instantiation for each dependent
array variable. Fixed and dynamic arrays of project classes, including native
UserForms, now admit As New. Bounds and nominal element identity use existing
arrays; construction stays lazy per referenced element. Reading the whole array,
declaring, inspecting, erasing, resizing or assigning a Set target does not eagerly
construct elements. Assigning Nothing rearms only that element. External class
activation and object-module Public arrays retain their existing refusals.

IR 14 adds auto-array-elements-v14 and TouchElement(declaration-slot). Array
local/field auto_new metadata requires that capability, exact project nominal
identity, Object elements and ordinary declared storage. Scalar metadata retains
IR 10. Programs using neither array metadata nor TouchElement keep prior version
selection. Old consumers, absent capabilities and malformed declaration/stack
shapes refuse before execution.

TouchElement preserves its checked element reference on the caller's stack. It
validates that the reference belongs to the declared canonical array variable,
then uses existing allocation, typed writes and Class_Initialize frames. Stop,
quota yields and failures retain the pending instruction and do not reevaluate
indices or earlier arguments. Failed initialization clears a still-matching cell
and retires the failed instance; a subsequent reference retries normally. Direct
and project-method ByRef arguments preserve canonical element locations and source
order. Existing argument capture is reused; parameters acquire no As New modifier.
Further alias/reentrant structural-change comparisons remain explicit M8.5 work.

The existing Locations store provides bounded flat element resolution and
validated element-owner inspection; stale generation/foreign/nested/out-of-range
references refuse. VM element kind/nominal registration is shared with ordinary
RefElement. There is no second object store or mutable construction registry.
Private fields, module/static arrays and restored project state retain their
existing owners and identity remapping.

For Each retains its existing bounded enumeration profile. A direct declared array
cursor carries its verified auto-new source; each dependent element is touched
at its iteration read using the same constructor continuation. Whole-array and
Variant copies acquire no declaration modifier. Existing array-lock, reference
roots, execution/allocation/location/instance/callback and permission budgets stay
unchanged, including the owner's existing 4,096 static permission allowance.

Debugger local reads, safe watches and native design/render inspection never
execute TouchElement or constructors. Native form instances use ordinary class
array storage; clearing one element creates a fresh independent form on its next
reference while other windows and retained aliases keep their identities.

Evidence is independently authored and reference-based. Full M7A.4 remains open;
fresh Office build/OS/locale comparisons remain NOT_RUN under M8.5.

Primary references:

- [MS-VBAL variable declarations](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/cf31e152-6b15-4ddd-a415-3257a30b1ac8)
- [MS-VBAL ReDim](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/22b5d372-0a54-4617-9462-4934b5edc88c)
- [VBA Dim](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/dim-statement)

Shared validation PASS: full CI, WASM compilation and actual WASI execution,
including 18 generic auto-instancing cases, 98 native form-instance cases and
flat/stale/foreign location assertions. Both host-local array callback/trust/
resource regressions pass. Pinned fork full CI and Node/WASM remain pending.
