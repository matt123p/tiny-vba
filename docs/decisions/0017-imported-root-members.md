# 0017 — Preserve dispatch on imported object roots

Date: 2026-10-27. Status: adopted during GridCraft integration.

M5 imports distinguish a fixed-member alias (`member: Some`) from a root object
(`member: None`). Binding a member on a root object must preserve the actual
member name, invocation kind and arguments, with the imported root as target.
The previous alias shortcut also consumed root-object member expressions: it
could discard a no-argument call or reject a valid argument/property assignment.

Restrict that shortcut to fixed-member aliases. Their documented behavior stays
unchanged. Root assignments themselves remain forbidden; properties on the root
use ordinary native Get/Call/Let/Set dispatch and existing permission checks.
No protocol version, IR shape or dependency graph changes. This is a binder fix,
not an Office catalogue in the language engine.

The VM embedding regression verifies nested returned objects, named arguments,
Let/Set and Get with the exact four host calls and typed result. GridCraft owns
the corresponding real-workbook tests.
