# ADR 0069 — UserForms without form components

Date: 2026-11-06. Status: adopted; shared validation passed.

The shared IDE form provider owns built-in UserForms access. A project does not
need a form component to inspect that collection. Reuse ordinary UserForms and
enumerator classes, the existing Query/Enumerator host imports and transient
project views. The empty result has Count=0, empty iteration and existing Item
bounds errors. Add of an unavailable form raises documented error 424. No form
is loaded, activated, decoded or manufactured by collection inspection.

A conservative bounded lexer pass selects possible UserForms use, excluding
comments and strings; normal semantic binding still resolves bare/project
shadows and qualified VBA.UserForms. With no form components, generate only the
collection, enumerator and standard wrapper modules. Do not add MSForms/RefEdit/
picture types or the control library to such projects. Projects without a code
reference retain their existing compiler/runtime path and internal namespace.
Existing reserved support-name checks apply when support is actually selected.
Invalid source is still rejected by the ordinary source/parser/compiler pipeline.

Compile-time selection is separate from execution: active runtime detection uses
verified generated class/source identity in the current program. Tick never
relexes changing source or switches services underneath a paused program. Edits
still require Reset; local inspection and safe watches remain inert. The service
root is included consistently for normal launch and idle document callbacks.
Callback draining uses the same wrapped host as the first slice, preserving the
existing wait/cancel/identity protocol. Ordinary document hosts retain their
nonzero identity requirement.

The existing empty-component gate enumerates only a validated UserForms owner;
normal populated control/tab/page enumeration keeps its established lookup order
and budgets. State stays in the existing VM objects and provider context. There
is no new registry, IR, intrinsic, format, ambient authority or budget increase.
The owner's existing 4,096 static permission allowance is unchanged; actual host
reads still use the normal per-site permissions and runtime limits.

Authored cases cover aliases/type inspection, bounds/Add errors, empty iteration,
project shadowing, comments/strings, Reset/source changes, Stop/resume and repeated
idle document callbacks. Host acceptance and the broader M7A.4 checklist remain
separate; fresh Office build/OS/locale comparisons remain M8.5 NOT_RUN.

Primary references:

- [VBA UserForm/UserForms](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/userform-object)
- [Missing form Add error 424](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/form-not-found-error-424)

Shared validation PASS: full CI, WASM compilation and actual WASI execution,
including all six empty-collection and 99 native form cases. Pinned host validation
remains pending. No new UI renderer or VM/foreign-protocol code was needed.
