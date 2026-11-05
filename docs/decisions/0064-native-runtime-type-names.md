# ADR 0064 — Native runtime TypeName

Status: implemented; CI/WASM/WASI/UI/web and actual Chromium validation pass. M7A.4 remains open.

Generic project-object TypeName uses the object's actual verified class, preserving
identity through Object/Variant/interface aliases without invoking a default
property. Project classes and UserForms report their component name. Nothing and
nonobject values retain the runtime's existing intrinsic semantics.

Generated provider classes need their native type spelling instead of an internal
implementation name. IR 12 introduces optional bounded Class.runtime_name metadata
under runtime-class-names-v12, supplied through CompileOptions.class_runtime_names.
Owners must exist, be unique ignoring ASCII case, and fit the bounded class/name
profile. Verification charges metadata and requires the version/feature; IR 1–11
consumers refuse the new metadata. The ordinary class name still owns nominal
project identity, event binding, constructors and persistence. There is no new
user-facing syntax, runtime setter or mutable type registry.

The VM performs a bounded, identity-checked metadata query and charges work and
returned string allocation. Both intrinsic encodings use it. VM/IR contain no
Forms or Office names. The Forms provider supplies canonical Forms/RefEdit/stdole
spellings and UserForms collection names while authored form names remain intact.
No getter, macro, host callback, COM or external type activation executes during
the query. Foreign-object TypeName remains the existing Object result when no
native project class metadata is available; richer host descriptions are separate.

Evidence includes a generic class with a failing default getter, aliases, explicit
runtime name override and old-version rejection, plus native forms, controls,
runtime-added controls and scoped collections. Fresh Office comparisons remain
NOT_RUN under M8.5; the broader M7A.4 ledger is unchanged by this decision.
