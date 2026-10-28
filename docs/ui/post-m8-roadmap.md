# Remaining UI and host scope

Updated 2026-10-28. The [remaining implementation plan](../implementation-plan.md)
is authoritative for numbering, dependencies and acceptance. This index contains
remaining UI/host work only; completed evidence is in
[implementation progress](../implementation-progress.md).

## Class/document runtime and standard Office forms — M2

Follow [M2](../implementation-plan.md#m2--classdocument-semantics-and-standard-office-userforms):
shared classes/properties/instances/events and error trapping before form execution,
then the shared model/designer, standard Office controls, correctly signed event
handlers and reviewed source/resource persistence. Host package acceptance is M7.

Third-party ActiveX/OLE controls not supplied with Office applications are out of
scope for now. Do not schedule their designer, loading or execution support.
Standard Office controls remain in scope through independent implementations.

## Host embedding and real document workflows — M6 and M7

Follow [M6](../implementation-plan.md#m6--remaining-gridcraft-and-wordcraft-integration)
for WordCraft objects/bridge/editor integration and new real-host workflows in
both forks. Shared code stays in tiny-vba; Office objects, trust, document state,
event ordering and undo/recalculation stay in the application forks.
[M5](../implementation-plan.md#m5--document-modules-events-and-range-aware-udfs)
owns Excel document events and reference-aware UDF behavior.

[M7](../implementation-plan.md#m7--unchanged-document-and-package-acceptance)
owns unchanged-source .xlsm/.docm package acceptance, forms/standard controls,
signature handling and reviewed writeback design. Generic-host doubles and
standalone UI images cannot establish real Office compatibility.

## Remaining specification traceability

The [public-source UI specification](craft_vba_editor_debugger_ui_spec.md) is a
requirements reference, not a commitment to every proposed extension. Current user
scope and the implementation plan take precedence over that earlier proposal.

| Remaining reference scope | Current allocation |
|---|---|
| Class/document runtime, class-specific trapping and P6 UserForms/designer | M2 |
| Office document events and Range-aware UDF context | M5; shared class/event contracts in M2 |
| P7 WordCraft/generalisation and actual host workflows | M6 |
| Form/document source and package persistence; signatures | M7 |
| Remaining lawful reference-build/fidelity observations | M6/M7, reported in M8 |
| Applicable class/form parts of scenarios 1–23 | M2, with host/storage acceptance in M6/M7 |
| Public-source/provenance review (scenario 26) | Required throughout M1–M8 |

Native Craft extensions, Python translation previews and Copilot-style assistance
are removed from the current scope. The specification's P8 and scenarios 24–25
do not create scheduled work. Exact Office geometry/ordering requires lawful
reference-build evidence; unverified items remain V, never implied pixel identity.
Use original renders and licensed assets; cache downloaded evidence only under
ignored `.cache/evidence/`.
