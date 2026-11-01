# ADR 0033: Intrinsic functions before package acceptance

Adopted 2026-11-01 by explicit owner direction following the missing StrComp
report: add intrinsic-function coverage as a new milestone before M7.

Insert **M6A — VBA intrinsic-function coverage** between accepted M6 and M7.
Preserve existing M7/M8 milestone and task IDs. M6 remains signed off for its
declared profile; M6A acceptance is a new prerequisite for starting M7.
This supersedes ADR 0032's next-work sequencing, not its M6 sign-off or the
deferral of fresh Excel/Word observations to M8.5.

[The intrinsic-function document](../vba-intrinsic-functions.md) defines inventory,
implementation groups, ownership, platform/host-service limits and acceptance.
Audit existing as well as absent intrinsics, starting implementation with StrComp;
worksheet coverage cannot substitute for VBA semantics. Pure language behaviour
belongs in tiny-vba, with explicit injected services for environmental behaviour.
Office objects and document workflows remain fork-owned. Existing restrictions
on ambient IO, arbitrary COM/DLL activation and unreviewed corpus execution remain.

This is a scheduling/scope decision, not implementation or compatibility evidence.
Update the plan, progress, documentation index, compatibility status and relevant
skills to identify M6A as next. No runtime capability/profile/IR version changes
are adopted by this planning-only decision.
