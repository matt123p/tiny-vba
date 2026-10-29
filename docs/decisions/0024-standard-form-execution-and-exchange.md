# ADR 0024 — Standard form execution and exchange

Date: 2026-10-29. Status: adopted and implemented; [acceptance](../m2-acceptance.md) passed.

M2 completes the selected Label/TextBox/CommandButton profile above the existing
IR7 project-class contract. `vba-ide` owns design documents, source handler
validation/generation, runtime orchestration and a bounded versioned text exchange
format. `vba-ui-egui` owns reusable designer/runtime presentation. Platform IO
stays in the standalone shell or embedding host. No new shipping dependency or
Office application object store is introduced.

Form/control execution uses generated project-class support appended to user
source, preserving original handler coordinates. Controls use native fields and
source Event/WithEvents delivery. Programmatic and interactive changes therefore
share the same statement frames, aliasing, budgets, debugger and fault handling.
Generated support is compiler input, never persisted in place of authored source.

Modal display uses the ordinary host suspension protocol. A new explicitly opted-in
VM suspended-callback API retains the waiting invocation while an event runs on
ordinary VM frames. It never reenters the Rust interpreter. Saved continuations
are bounded, retain their locations/objects and must be cancelled with their host
tokens on Reset/close. Existing callback APIs remain terminal-only. Form rendering
must not repeatedly poll an unresolved modal wait or replay the suspended call.

The exchange format is a tiny-vba UTF-8 `.tvform` document, version 1, containing
validated design metadata and exact authored source. It is not a claim of binary
Office `.frm/.frx` compatibility. Unknown versions, control types, fields and
resources fail explicitly before project mutation. Hosts receive the complete
owned payload through the existing save/export request interface and acknowledge
its revision; edits since a request cannot be marked saved. Office package
persistence and binary resource import remain separately reviewed M7 work.

Named native field reference captures require IR7 and its project-class capability.
Early-bound member names, access kinds, visibility and argument signatures are
validated in semantics; late-bound Object/Variant dispatch retains runtime checks.
ByRef uses canonical field locations, with parenthesized values remaining typed
temporaries. This profile is independently tested; public documentation is not
fresh Office oracle evidence.

Embedding wait tokens are opaque and can equal modal token values. The form host
therefore wraps them in its own bounded namespace and routes each resume/cancel
back to the original token. Equal numeric values must never confer modal input
ownership on an unrelated host wait. The private gate root/handle is reserved
within this adapter; it is not part of the embedding's Office object catalogue.
