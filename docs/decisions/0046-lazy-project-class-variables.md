# ADR 0046 — lazy project-class variables

Date: 2026-11-04. Status: adopted for scalar project classes and native UserForms.
M7A.4 remains in progress; this is not full control/lifecycle acceptance.

The language/compiler/IR/VM own As New. Follow the VBA
[Dim statement](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/dim-statement),
[variable type declarations](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/d27d3dae-ac46-4113-8927-c7f60d844dbf)
and [class lifecycle contract](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/62bbe63e-379c-4dc0-8648-7d9050a2f396).
A scalar variable declared As New constructs its named class on a reference while
its value is Nothing. Declarations and Set assignment targets do not construct.
Subsequent reads retain identity; assigning Nothing permits construction on the
next reference. No external library/COM activation is added.

The parser retains the declaration modifier separately from the type. Binding and
lowering resolve constructible project classes, including native form classes;
module/static variables and per-instance fields retain their canonical storage.
Intrinsic/dependent document types, Const and WithEvents combinations refuse.
Array As New and external-class activation remain explicitly unsupported.

IR 10 `auto-instancing-v10` adds optional class-source metadata to Local and
ClassField, without new opcodes or dependency edges. Verification checks the
version/capability, exact nominal project type, scalar storage and admissible role,
constructible class, field/local agreement and persistent-slot consistency. Work
for metadata lookup is charged. IR 1–9 consumers reject executable auto metadata;
programs without it keep their previous versions.

ReadLocal and public field Get consult their verified declaration metadata. A
Nothing value is replaced through ordinary checked storage and class allocation,
and Class_Initialize runs in an ordinary receiver frame. The caller retains its
pending read; public field Get/ByRef uses an explicit return disposition. Completed
argument evaluation, writes and constructors do not replay on Stop/resume. Failed
initialization clears a still-matching variable and retires the failed object;
a later reference can retry. Normal initialization shares existing depth,
instance, location and instruction budgets. Reset runs no cleanup code.

ByRef argument preparation touches an As New declaration at its position in source
evaluation order, then passes the ordinary canonical location. Parameters acquire
no As New declaration modifier. Writes through an alias can therefore clear the
original variable, whose next declared-variable read constructs again. No second
object store or per-location construction registry is added.

Debugger inspection, watches and transient host views remain inert and never
initialize a Nothing variable. Existing safe-evaluation limits still apply.
The native form designer's launch helper also constructs non-predeclared forms
explicitly; predeclared forms retain their default-instance launch path.

Authored evidence covers unused/Set targets, lazy construction, identity/Nothing,
module/static and public/private field storage, canonical ByRef, constructor
Stop/failure/retry, malformed/legacy metadata, native explicit windows and inert
inspection. A parked modal caller additionally retains enumerated controls after
unload. These are specification-based implementation tests, not fresh measured
Excel/Word traces. Full implicit/alias edge cases, As New arrays, nominal MSForms
bindings, focus/typed events and remaining native families stay in the runtime
remaining-work record. Fresh Office comparisons remain NOT_RUN under M8.5.
