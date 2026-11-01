# Remaining UI and host scope

Updated 2026-11-01. The [remaining implementation plan](../implementation-plan.md)
is authoritative for numbering, dependencies and acceptance. This index contains
remaining UI/host work only; completed evidence is in
[implementation progress](../implementation-progress.md).

M2's selected shared class/designer/runtime/exchange profile is complete; see
[its acceptance report](../m2-acceptance.md). Remaining work below consumes that
contract in real application/package workflows.

Third-party ActiveX/OLE controls not supplied with Office applications are out of
scope for now. Do not schedule their designer, loading or execution support.
Standard Office controls remain in scope through independent implementations.

## Real document/package workflows — M7

[M5 document/event/UDF acceptance](../m5-acceptance.md) and
[M6 shared/fork integration](../m6-acceptance.md) are complete for their declared
profiles. Shared code stays in tiny-vba; Office objects, trust, document state,
event ordering and undo/recalculation stay in the application forks.

[M7](../implementation-plan.md#m7--unchanged-document-and-package-acceptance)
is next: unchanged-source .xlsm/.docm package acceptance, forms/standard controls,
signature handling and reviewed writeback design. Fresh Excel/Word comparisons
are deferred to [M8.5](../implementation-plan.md#m8--expanded-conformance-hardening-and-release).
Generic-host doubles and standalone UI images cannot establish real Office compatibility.

## Remaining specification traceability

The [public-source UI specification](craft_vba_editor_debugger_ui_spec.md) is a
requirements reference, not a commitment to every proposed extension. Current user
scope and the implementation plan take precedence over that earlier proposal.

| Remaining reference scope | Current allocation |
|---|---|
| Form/document source and package persistence; signatures | M7 |
| Remaining lawful reference-build/fidelity observations | M8.5; NOT_RUN until observed |
| Applicable form host/storage parts of scenarios 1–23 | M7 using the completed M2/M5/M6 profiles |
| Public-source/provenance review (scenario 26) | Required throughout M1–M8 |

Native Craft extensions, Python translation previews and Copilot-style assistance
are removed from the current scope. The specification's P8 and scenarios 24–25
do not create scheduled work. Exact Office geometry/ordering requires lawful
reference-build evidence; unverified items remain V, never implied pixel identity.
Use original renders and licensed assets; cache downloaded evidence only under
ignored `.cache/evidence/`.
