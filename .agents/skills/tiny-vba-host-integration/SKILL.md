---
name: tiny-vba-host-integration
description: Design or review tiny-vba generic foreign calls, host references, ByRef/wait ownership, source import and application integration contracts. Use when bridging the engine or IDE to external document applications.
---

# tiny-vba host integration

Read [host integration](../../../docs/host-integration.md),
[runtime](../../../docs/runtime.md), [project import](../../../docs/project-import.md)
and [forms](../../../docs/forms.md).

Keep vba-foreign host-neutral: invocation kind, opaque reference/target,
named/positional/omitted arguments, ByRef obligations and result/error/wait state.
Preserve exact tags and native calls. The host owns object identity/generation,
Office catalogue/dispatch, event order, trust, document IO/OOXML and undo/recalc.
Compiled source cannot grant roots or per-site permissions.

Dispatch once; explicit resume polls the correct pending token. Typed result and
all patches validate before commitment. Cancel/close release pending tokens;
no document borrow survives a callback/pause/wait. Suspended calls require the
exact token and keep outputs separate from the original foreign arguments.
Cumulative work and payload limits survive yields; reusable temporary host scratch
does not erase actual copying/work charges.

Decode/attach source/resources inertly and retain original bytes separately from
canonical source and authored edits. Reference paths never activate libraries.
Code names are identities, not display names. Native Save/Export and edited package
copies use revision-checked snapshots; ordinary document Save preserves originals.
Signature removal is not verification/resigning. Use generic shared-host tests;
Office-shaped contracts and actual package workflows belong to the forks.
