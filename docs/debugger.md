# Debugger design contract

Status: planned, 2026-10-26. vba-debug, vba-dap and the editor/debugger UI have no implementation yet.

The reusable graphical editor/debugger and standalone IDE belong in tiny-vba. They are required
product work under [ADR 0002](decisions/0002-editor-debugger-ui.md) and the
[UI contract](editor-debugger-ui.md). Gridcraft embeds these panels; it owns the document bridge.
The egui UI calls native debugger/IDE APIs directly. DAP is optional and does not gate the visual UI.

The VM owns explicit frames, instruction pointer, per-frame error-handler state, execution budgets,
source maps and statement checkpoints. Its inspection hooks are independent of vba-debug and DAP.
The debugger controller owns source breakpoints, run/pause/continue/step into/over/out, local inspection
and validated local updates. Multiple statements on one line retain distinct IDs.

M3 must prove pause/edit/resume without restarting the procedure or repeating host writes, including
nested calls and loops. Do not keep mutable host borrows across pauses. Default watch inspection must
not invoke arbitrary Property Get/functions with side effects or bypass capability checks.

DAP comes later and must publish supported capabilities honestly. Disconnect/cancel/stale objects and
host close during pause require explicit lifecycle outcomes, with no retained document locks.
