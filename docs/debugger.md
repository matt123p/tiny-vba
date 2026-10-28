# Debugger design contract

Status: M3 headless VM/controller implemented, extended by M8 UI and M10 advanced debugging (2026-10-28). See [runtime APIs](runtime.md), [embedding](ui/embedding.md) and [M10 acceptance](ui/m10-implementation.md).

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

DAP is optional and publishes supported capabilities explicitly. Disconnect/cancel/stale objects and
host close during pause require explicit lifecycle outcomes, with no retained document locks.

M10 adds Quick Watch, context-scoped conditional/change watches, local array
index inspection/typed element edits and runtime error-trapping controls.
Conditional evaluation is read-only and bounded. Set Next Statement supports
verified straight-line current-frame checkpoints only. Immediate calls require
an explicit policy and execute in an isolated child VM; host effects remain real.
See [ADR 0018](decisions/0018-m10-classic-tools-and-debugger-policy.md) for exact
bounds, refusals and stale-state rules. Class-specific error trapping remains M11.

Run the external single-session stdio adapter with
`cargo run -p vba-cli --features dap -- dap [source.bas]`. Launch may instead
supply explicit `sourceCode`; no ambient project discovery occurs. Supported
requests include initialize/launch/configurationDone, source breakpoints,
stack/scopes/variables, safe evaluate/setVariable, continue/step/pause, source,
all-errors filtering and disconnect/terminate. Unsupported breakpoint conditions,
hit counts and log points are unverified. Framing and real subprocess lifecycle
tests live in `crates/vba-dap/tests/protocol.rs` and `apps/vba-cli/tests/dap.rs`.
