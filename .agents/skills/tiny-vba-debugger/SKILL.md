---
name: tiny-vba-debugger
description: Design, implement or review tiny-vba breakpoints, stepping, inspection, watches, debugger actions, DAP and pause/disconnect lifecycle. Use for execution-control correctness, including editor-to-debugger behaviour.
---

# Debugger execution contract

Read [debugger contract](../../../docs/debugger.md) and the state/lifecycle sections of
[editor/debugger UI](../../../docs/editor-debugger-ui.md). Inspect VM checkpoints and controller
state before adding controls. Use the editor UI skill as well when changing visual interaction.

Follow [IR 1 checkpoint/continuation semantics](../../../docs/bytecode-vm-specification.md): distinguish
normal checkpoint pauses, post-Stop pauses, fault snapshots, quota yields and host waits. Continue grants
one permit only for the paused checkpoint visit; step targets use frame identities and dynamic visits.
The M3 headless VM/controller subset is now implemented; the runtime contract records its limits.

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

## Implemented M3 contract

Read [runtime](../../../docs/runtime.md) and [ADR 0007](../../../docs/decisions/0007-m3-execution-and-debugger.md).
The m3-core-v1 runtime is intentionally narrower than structural IR. Preserve explicit frame/alias
storage, one-visit checkpoint permits, saved fault PCs, generation checks and nonreplayed continuations.
Use vba-runtime for operations/edits. VM host dispatch must use checked precommit return/writeback
validation and explicit per-site permissions; foreign aliases remain unsupported. Run cargo xtask ci,
wasm and wasm-run (native plus actual WASI execution). Runtime example results do not mark unrun M0
or external scenario assertions PASS. Graphical UI, DAP and VBA error-handler semantics remain later work.

[ADR 0008](../../../docs/decisions/0008-vm-instance-isolation.md) distinguishes the caller's routing
label from the non-reused SessionInstance. Key handles, locations, breakpoints and stepping by the
engine instance; caller-label/frame/generation equality alone cannot identify an execution. Include
the replacement-session regression in native/WASI evidence when changing these identities.
