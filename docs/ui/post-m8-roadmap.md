# Remaining UI and host scope

Updated 2026-11-04. The [remaining implementation plan](../implementation-plan.md)
is authoritative for numbering, dependencies and acceptance. This index contains
remaining UI/host work only; completed evidence is in
[implementation progress](../implementation-progress.md).

M2's selected shared class/designer/runtime/exchange profile is complete; see
[its acceptance report](../m2-acceptance.md). Remaining work below consumes that
contract in real application/package workflows. Its custom exchange-format choice
is superseded by ADR 0038; M7A removes `.tvform`.

Third-party ActiveX/OLE controls not supplied with Office applications are out of
scope for now. Do not schedule their designer, loading or execution support.
Standard Office controls remain in scope through independent implementations.

## Native Office form/document workflows — M7A

[M5 document/event/UDF acceptance](../m5-acceptance.md) and
[M6 shared/fork integration](../m6-acceptance.md) are complete for their declared
profiles. Shared code stays in tiny-vba; Office objects, trust, document state,
event ordering and undo/recalculation stay in the application forks.

[M7 authored package acceptance](../m7-acceptance.md) is complete for its historical
profile. [M7A](../implementation-plan.md) now activates native UserForms, worksheet
controls and native editing/persistence, and removes `.tvform`. The
[forms breakdown](../office-forms-controls-implementation-plan.md) retains FC task
IDs; [ADR 0038](../decisions/0038-native-office-forms-and-compatibility-only.md)
sets the compatibility-only direction. Implementation has not started.
Fresh Office comparisons remain [M8.5](../implementation-plan.md#m8--expanded-conformance-hardening-and-release).
Generic-host doubles and standalone UI images cannot establish real Office compatibility.

## Remaining specification traceability

The [public-source UI specification](craft_vba_editor_debugger_ui_spec.md) is a
requirements reference, not a commitment to every proposed extension. Current user
scope and the implementation plan take precedence over that earlier proposal.

| Remaining reference scope | Current allocation |
|---|---|
| Required native edited form/source persistence; signature invalidation | M7A.5, using the M7 reviewed writeback contract |
| Native worksheet controls, UserForms, Office semantics and removal of .tvform | M7A.1–M7A.7 / FC0–FC6 |
| Remaining lawful reference-build/fidelity observations | M8.5; NOT_RUN until observed |
| Applicable native form host/storage parts of scenarios 1–23 | M7A, preserving earlier M2/M5/M6/M7 regressions |
| Public-source/provenance review (scenario 26) | Required throughout M1–M8 |

Native Craft extensions, Python translation previews and Copilot-style assistance
are removed from the current scope. The specification's P8 and scenarios 24–25
do not create scheduled work. Exact Office geometry/ordering requires lawful
reference-build evidence; unverified items remain V, never implied pixel identity.
Use original renders and licensed assets; cache downloaded evidence only under
ignored `.cache/evidence/`.
