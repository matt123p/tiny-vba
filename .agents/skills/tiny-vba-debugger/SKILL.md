---
name: tiny-vba-debugger
description: Design, implement or review tiny-vba breakpoints, stepping, inspection, watches, debugger actions, DAP and pause/disconnect lifecycle. Use for execution-control correctness, including editor-to-debugger behaviour.
---

# tiny-vba debugger

Read [debugger](../../../docs/debugger.md), [runtime](../../../docs/runtime.md)
and [the debugger policy](../../../docs/decisions/0018-m10-classic-tools-and-debugger-policy.md).

Pause at statement checkpoints with explicit frame/instruction state. Statements
on the same source line retain distinct IDs. Continue permits exactly one dynamic
visit; stepping respects call depth, loops, explicit breakpoints and handlers.
Never restart a procedure or replay host effects to resume.

Inspection handles carry VM instance, frame, source revision and pause generation.
Reject stale handles and source changes. Validate local/array edits through shared
coercion before mutation. Watch evaluation is read-only and bounded; getters,
default members and functions may cause effects. Immediate calls need explicit
policy and an isolated bounded child VM; host effects stay real and authorized.
Set Next Statement is restricted to verified straight-line current-frame targets.

Release host borrows before waits/pauses. Cancel/disconnect/host loss have explicit
outcomes and pending-token cleanup; Reset is not document rollback. Keep cumulative
budgets through slices and continuations. DAP stays optional above the headless
IDE and advertises only implemented capabilities. Test nested calls/loops,
break-edit-resume with write counts, errors, stale handles and close during pause.
