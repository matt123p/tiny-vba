---
name: tiny-vba-debugger
description: Design, implement or review tiny-vba breakpoints, stepping, inspection, watches, debugger actions, DAP and pause/disconnect lifecycle. Use for execution-control correctness, including editor-to-debugger behaviour.
---

# Debugger execution contract

Read [debugger contract](../../../docs/debugger.md) and the state/lifecycle sections of
[editor/debugger UI](../../../docs/editor-debugger-ui.md). Inspect VM checkpoints and controller
state before adding controls. Use the editor UI skill as well when changing visual interaction.

## Required invariants

- Pause at source statement boundaries with explicit frames and instruction state. Keep separate
  statement identities on a shared line; preserve compiled-source versions in breakpoint mapping.
- Continue/step must resume the paused state without restarting or repeating earlier host writes.
  Step into/over/out must respect call depth, loops, breakpoints and error-handler jumps.
- Release mutable host borrows before pause/yield/callback. Stop, cancel, disconnect and host close
  must have explicit outcomes and release retained references/resources under the host policy.
- Inspection handles belong to a pause/session generation; reject stale frames, values and objects.
  Validate supported local edits using shared language rules before changing execution state.
- Watches default to read-only inspection. Property Get, default members and functions may cause
  effects; report unsupported/unsafe evaluation rather than silently executing them. Evaluation
  must obey the same capabilities as macros. Immediate execution is a later explicit-policy feature.
- Editing paused source marks compiled code stale. Require stop/recompile/restart for changed code;
  do not imply live code replacement. Keep diagnostics and execution highlights version-correct.
- Bound execution slices and output batches so UI repaint, pause and cancellation remain responsive.
  Controller/VM APIs stay toolkit independent; native UI calls Rust directly. DAP is optional and
  advertises only capabilities actually implemented.

Prove break/inspect/edit/resume with deterministic cases that count host writes, plus nested calls,
loops, runtime errors and cancellation. Exercise stale handles and disconnect/close during pause.
Keep UI state/actions consistent with the controller; unsupported actions are disabled or return
explicit diagnostics. A CLI or DAP test does not satisfy the required visual debugger milestone.
