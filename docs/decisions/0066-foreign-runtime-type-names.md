# ADR 0066 — Foreign runtime type names

Date: 2026-11-06. Status: shared implementation and pinned fork validation passed. M7A.4 remains open.

M7A.4 includes dynamic TypeName for objects supplied by an embedding. ADR 0064
already covers ordinary project classes and native Forms classes. This increment
extends the existing host-neutral ObjectTypeResolver with an optional bounded
runtime_type_name query. It reads the embedding's existing object store, validates
full handle identity/generation/lifetime and returns its native type spelling.
No property, VBA, host invocation, IO, COM or object activation runs. The optional
default preserves the generic Object result. Nothing and scalar/array intrinsic
behavior stay with the central runtime.

The VM checks resolver identity before querying, charges lookup/string work and
result allocation, and preserves resolver errors through normal VBA handlers.
Names are nonempty, contain no control characters and fit 255 UTF-16 units;
byte length is checked before scanning a hostile result. Oversized metadata fails
the existing resource boundary. Both intrinsic encodings use the same query.
IR 12 and all static permission/execution/memory limits remain unchanged: this is
an extension of existing read-only type resolution, not a new serialized operand.

GridCraft and WordCraft own the names and object identity checks. Their nominal
assignability and runtime display names share one projection of their existing
objects, including existing feature-kind aliases. Neither creates another object
registry. Public type spelling must not be guessed from a variable's declared
interface or inferred by executing a default getter.

The descriptor audit also corrects the documented varname parameter for TypeName,
VarType and IsArray. Named arguments bind through ordinary IR 8 argument positions;
no new syntax or invocation protocol is introduced. VarType default-member
evaluation for object arguments remains a separate compatibility gap; this
decision changes its parameter spelling only.

Authored evidence covers generic aliases, no getter/host dispatch, Stop/resume,
revocation, foreign identity, optional fallback, UTF-16 bounds and malformed
metadata. Fork evidence must independently cover live Office-shaped objects and
unchanged document state. Fresh Office build/OS/locale comparisons remain NOT_RUN
under M8.5; the broader M7A.4 event/lifecycle/property/platform ledger remains open.

Primary reference: [TypeName](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/typename-function),
[VarType](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/vartype-function),
[IsArray](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/isarray-function).
