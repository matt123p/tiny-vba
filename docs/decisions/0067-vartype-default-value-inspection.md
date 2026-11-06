# ADR 0067 — VarType and default value inspection

Date: 2026-11-06. Status: adopted; shared validation passed.

The Office VBA function reference specifies that VarType inspects an object's
default property. MS-VBAL's VarType table describes an object-reference data value
as type 9. For the Office compatibility path, resolve a readable zero-argument
default value first, then apply the existing central subtype classifier. Nothing,
scalars/arrays, and objects without a compatible public default getter retain
ordinary classification. A getter result is classified once: an Object result is
9, rather than recursively coercing its own default member.

IR 13 adds the capability-gated InspectGet implicit operation under
inspection-defaults-v13. Binding inserts it for VarType operands that can be
objects, retaining source-order evaluation and named argument mapping. It is not
ordinary ValueGet: it permits the opaque-object result and consumes one getter.
Verifier checks the IR/capability, empty member/arguments and ordinary result mode.
Older consumers reject the new operation. Programs without it retain prior IRs.
All existing work, host-call, location, permission and memory limits remain.

Project objects use existing verified class/member metadata. Public zero-argument
compatible Property Get/default functions execute through ordinary receiver
frames; optional arguments use normal binding. Getter errors are not mistaken for
absent members. Stop, stepping, unwinding and constructor continuations retain
normal behavior. Public default functions, previously refused by source binding,
are admitted under the same IR 13 capability and verified Function signature.
Object construction in a value context uses normal default-value evaluation.

The existing host-neutral object resolver gains optional has_default_value
metadata. Overrides validate full handle identity/lifetime, including a negative
answer, without getters/VBA/IO/activation. Opaque/unadapted hosts retain type 9.
An actual foreign read uses the normal authorized DefaultGet site, quotas,
validated replies and wait/continuation/cancel path. No member name or Office
object enters the VM and no alternate object/value registry is introduced.
GridCraft and WordCraft own their existing default-member selectors and reads.
GridCraft advertises Range.Value and Application.Name; WordCraft advertises
Range.Text and Selection.Text. No Workbook/Worksheet/Document default is guessed.
Host regressions preserve contents/undo across aliases and Stop; Word's canonical
final paragraph mark and Excel Date/Currency/array tags remain intact.

Safe watches remain restricted to inert locals/arrays/arithmetic and reject calls
and properties. TypeName/IsObject/IsArray keep their reference inspection behavior;
this change adds no unsafe debugger-evaluation route.

Evidence is independently authored, specification/reference-based, not a fresh
Office observation. No connected Office document session was available. M8.5
build/OS/locale comparisons remain NOT_RUN, including additional default/arity
edge cases. Full M7A.4 event/lifecycle/array/platform requirements remain open.

Primary references:

- [Office VarType](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/vartype-function)
- [MS-VBAL VarType table](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/2deec703-c2ef-4bf1-a246-1d51c60895da)
- [MS-VBAL data-value evaluation](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/ff101375-b301-4cba-80b9-e1ab59f7a8be)
- [Word Range default Text](https://learn.microsoft.com/en-us/office/vba/word/concepts/customizing-word/assigning-ranges)
- [Word Selection default Text](https://learn.microsoft.com/en-us/office/vba/api/word.selection)

Validation: shared full CI, WASM compilation and actual WASI execution pass.
The final generic class suite has 41 cases, the nominal suite 18, and the native
form-instance suite 97. Host local integration passes GridCraft's 12 native and
12 Node/WASM identity cases and WordCraft's 28 native workflows and two selected
Node/WASM workflows. Both strict optional VBA/UI Clippy checks, form UI checks
and optional web builds pass. Pinned consumer full CI remains pending.
