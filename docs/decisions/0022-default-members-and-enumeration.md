# ADR 0022 — Generic default members and enumeration

Date: 2026-10-28. Status: adopted; generic implementation validated.

Extend verified IR with explicit capability-gated automation operations. Default
Get/Let/Set and enumeration requests carry native arguments and no hardcoded
member name. The host selects its own default members and collection behavior.
Value-context default resolution is distinct from object-reference evaluation;
primitive values bypass the host, while object resolution obeys host budgets,
permissions, suspension and exactly-once continuation. Safe evaluation never
uses this effectful path.

For Each preserves source/Next checkpoints and typed local assignment. Host
enumeration supplies a bounded owned snapshot; native arrays use live canonical
element locations with resize protection. Verified loop-region metadata supports
cleanup on exit/jumps/return and retention across valid error-handler resumes.
Every cursor, snapshot and traversal is resource bounded. No host borrow or
external iterator token survives a step except the existing pending host wait.

Excel defaults, collection objects, Count/Item, ordering and workbook effects
belong exclusively in GridCraft. Both repositories retain separate Git histories
and pinned dependencies. Source import remains inert and no COM/DLL loader is
introduced. Details/evidence are recorded in the M1 completion work log.
