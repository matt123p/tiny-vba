# ADR 0019 — Remaining plan scope and numbering

Date: 2026-10-28. Status: adopted on explicit user direction.

## Decision

The implementation plan contains remaining work only and restarts at contiguous
M1–M8. Completed implementation and old milestone identities remain in progress,
acceptance reports and existing ADRs. Do not relabel historical evidence as new
milestone completion. The mapping lives in implementation progress, not the plan.

Remove the former M12 native extensions/translation and M13 AI assistance from
scheduled scope. Exclude third-party ActiveX/OLE controls not supplied with Office
applications. Retain standard Office UserForms/controls through independent shared
contracts; this does not authorize arbitrary COM/DLL loading or binary add-ins.
The earlier supplied UI specification is subordinate to this current scope.

Order remaining work as host types/default members/collections, classes/forms,
range behavior, identity/value semantics, events/UDFs, remaining host integration,
document/package acceptance, then conformance/hardening/release. State dependencies
and allowed parallel tracks explicitly. Every feature carries tests and evidence;
conformance collection, fuzzing and performance work start before final release.

## Ownership and consequences

tiny-vba retains reusable language/runtime/editor/form contracts; Office objects,
document state, trust, events, packaging and application integration remain in the
forks. No crate graph, product code, dependency or implemented capability changes.
Current documentation and skills must link to the new plan rather than reuse old
future milestone numbers. Historical logs retain their original numbers and an
explicit historical label. Removed scope needs a new user decision before planning.
