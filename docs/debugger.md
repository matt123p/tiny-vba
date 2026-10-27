# Debugger design contract

Status: M3 headless VM/controller implemented, 2026-10-26. See [runtime APIs, tests and limitations](runtime.md). M8 now supplies [the reusable graphical editor/debugger and CLI](ui/embedding.md); DAP remains optional/unimplemented.

The reusable graphical editor/debugger and standalone IDE belong in tiny-vba. They are required
product work under [ADR 0002](decisions/0002-editor-debugger-ui.md) and the
[UI contract](editor-debugger-ui.md). Gridcraft embeds these panels; it owns the document bridge.
The egui UI calls native debugger/IDE APIs directly. DAP is optional and does not gate the visual UI.

[IR 1](bytecode-vm-specification.md) now specifies checkpoint visits/one-visit continuation permits,
frame-local stacks, exactly-once host continuations, Stop/fault/quota/wait distinctions, step targets,
inspection generations and cancellation. The M3 subset follows this detailed contract;
On Error handler execution is reserved for a later IR version, not implemented by IR 1.

The VM owns explicit frames, instruction pointer, per-frame error-handler state, execution budgets,
source maps and statement checkpoints. Its inspection hooks are independent of vba-debug and DAP.
The debugger controller owns source breakpoints, run/pause/continue/step into/over/out, local inspection
and validated local updates. Multiple statements on one line retain distinct IDs.

M3 must prove pause/edit/resume without restarting the procedure or repeating host writes, including
nested calls and loops. Do not keep mutable host borrows across pauses. Default watch inspection must
not invoke arbitrary Property Get/functions with side effects or bypass capability checks.

DAP comes later and must publish supported capabilities honestly. Disconnect/cancel/stale objects and
host close during pause require explicit lifecycle outcomes, with no retained document locks.
