# ADR 0021 — Nominal host object types

Date: 2026-10-28. Status: adopted; core native/WASI acceptance passes. Fork
acceptance is recorded separately in the M1 log.

## Ownership and representation

`vba-values::NominalType` is a bounded canonical library/version/type identity.
It contains no Office names or objects. The host registers descriptors in its
explicit reference snapshot and supplies an `ObjectTypeResolver` backed by its
existing object store. The resolver is a bounded read-only query: no properties,
user code, VBA reentry, IO or COM activation. tiny-vba owns no parallel registry.

Host-supplied assignability includes supported interfaces. Resolver identity must
match the session's host; full object handle identity/generation and lifetime
remain host-owned. Wrong type yields VBA error 13, stale/foreign objects error 91.
Nothing is valid for object slots. Untyped Object/Variant late binding remains.
Nominal code aids also use the registered library namespace. Catalogue owners
must be fully qualified or paired with the matching `ApiMember.library`; display
labels and same-named classes from another library do not establish identity.

IR 5 adds verified per-procedure/local nominal metadata with an explicit
`nominal-objects-v5` capability. Old IR 1–4 consumers reject it. Parameter/result,
persistent and array metadata retain identity; ByRef requires matching declared
identity. Compiler-owned argument temporaries retain the callee's nominal type.
Object arrays carry their declared identity through Variant copies, resize and
Erase; checking only the current elements would incorrectly accept empty arrays
of another class. IR verification rejects malformed metadata and persistent
aliases that drop/change identity.
The VM checks values on typed storage/read/return, array elements/whole arrays,
entry/callback arguments, host writeback and debugger edits. Foreign writeback
validates the entire patch before committing locations. Type queries have a
per-session work budget bounded by the instruction limit.

Metadata counts/payload and derived location copies are charged to verification/
session budgets. Nominal error/availability checks occur before location mutation;
queued callbacks revalidate before delivery, and persistent snapshots revalidate
before restoration. The resolver's read lock, if any, must not outlive a query.

`Session::new` remains an empty-resolver entry point. Embeddings opt into
`new_with_type_resolver`; the IDE passes its explicitly configured resolver to
ordinary and isolated Immediate sessions. Configuration requires Reset.

## Acceptance and exclusions

Generic authored tests belong to tiny-vba. Excel descriptors, object identity
queries and unchanged Worksheet/Range source tests belong to GridCraft. The
reviewed graph remains acyclic and no fork dependency enters the engine.
Default members, collections, Office event semantics and third-party controls
are outside M1.1; nominal declarations must not silently enable them.
