---
name: tiny-vba-language-semantics
description: Implement or review VBA syntax, binding, values, coercion, bytecode and runtime behaviour in tiny-vba. Use for language correctness and execution changes rather than Office object-model implementation.
---

# tiny-vba language semantics

Read [frontend](../../../docs/frontend.md), [compiler](../../../docs/compiler.md),
[runtime](../../../docs/runtime.md), [bytecode](../../../docs/bytecode-vm-specification.md)
and the relevant versioned [decisions](../../../docs/decisions/README.md).

Preserve exact VBA value tags, numeric widths, Currency/Date, UTF-16 code units,
array lower bounds and canonical ByRef aliases. Use shared runtime coercion for
binding/defaults and debugger edits; do not create parallel conversion rules.
Evaluate arguments once in source order; omitted metadata is not Empty/Null/Missing.
Honor declaration shadowing, component kind, nominal identity and host reference
snapshots. See [module contexts](../../../docs/vba_module_types_and_execution_contexts.md)
and [resolution](../../../docs/vba_excel_context_and_object_resolution.md).

Keep bytecode version/capability refusal explicit through IR 14. Lazy As New scalar
and array elements construct through ordinary frames; declaration, Set targets,
whole-array inspection and safe watches remain inert. Failed constructors retain
retry and Stop/Continue semantics. No arbitrary external activation is added.

Use [intrinsic design](../../../docs/intrinsic-design.md) and the
[inventory](../../../docs/vba-intrinsic-functions.md): one descriptor catalogue,
argument plan and context/service contract. Locale/code-page/RNG services are
explicit; never fabricate missing behavior. VarType returns the enum's Long
representation; QBColor uses the formal numeric palette. Prove exact types/errors,
alias effects, bounds and continuations with owning-crate regressions and WASI
execution when changing runtime behavior.
