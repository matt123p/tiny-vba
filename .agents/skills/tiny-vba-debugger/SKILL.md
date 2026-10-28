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
  must obey the same capabilities as macros. M10 Immediate calls require explicit policy and an
  isolated bounded child VM; real host effects must remain visible and permission-checked.
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
or external scenario assertions PASS. M8/M10 implement the graphical UI and optional DAP;
current handler semantics are defined by the M4 runtime profile.

[ADR 0008](../../../docs/decisions/0008-vm-instance-isolation.md) distinguishes the caller's routing
label from the non-reused SessionInstance. Key handles, locations, breakpoints and stepping by the
engine instance; caller-label/frame/generation equality alone cannot identify an execution. Include
the replacement-session regression in native/WASI evidence when changing these identities.

The current scalar consumer is m4-scalars-v1 under [ADR 0010](../../../docs/decisions/0010-m4-profile-and-verification-hardening.md).
Compile against current VM capabilities; legacy M3-tagged programs require recompilation. Inspect
the xtask workspace banner before trusting gate evidence from a shared target cache.

Current M4 support supersedes earlier first-slice limits: read [M4 language/IR 2](../../../docs/m4-language.md)
and [ADR 0011](../../../docs/decisions/0011-m4-language-extension.md). Compile against m4-language-v1
VM capabilities. Preserve the IR 1 structural target and explicit IR 2 feature/version gates.
The authored inventory is 20 M0 plus 24 M4 PASS fixtures; no Microsoft oracle observation is claimed.

M4 external acceptance is now implemented under [ADR 0012](../../../docs/decisions/0012-m4-oracle-acceptance.md).
Read [the report](../../../docs/m4-oracle-report.md) and current language contract: 44 authored
plus 153 inherited upstream-reported Excel Mac scenarios PASS. Schema 2 and the reviewed allowlist
preserve raw results/environment/hashes; no fresh Office run is claimed. Use m5-host-v1
and explicit IR 3/4 capabilities, retaining IR 1/2 refusal rules. Unselected external cases remain NOT_RUN.

## Implemented M8 editor/debugger

Read [ADR 0016](../../../docs/decisions/0016-m8-embeddable-editor.md), the
[embedding/harness guide](../../../docs/ui/embedding.md) and
[acceptance matrix](../../../docs/ui/m8-acceptance.md). vba-ide is toolkit/IO-free; vba-ui-egui
mounts in caller Ui with injected tokens, independent per-view IDs and explicit commands;
vba-ide-app owns native/web IO and preferences. Run cargo xtask ui, ui-native and web plus ordinary
CI/WASM gates for affected UI changes. Keep accessibility checks enabled; render/inspect original
light/dark/narrow/scaled/host-theme/focus output. No downloaded reference evidence enters Git:
.cache/evidence/ is ignored and provenance/notices/pins are verified. Do not claim full C0/C1,
Office event/workbook, Set Next Statement or unsafe evaluation support from standalone UI evidence.

M10 is implemented under [ADR 0018](../../../docs/decisions/0018-m10-classic-tools-and-debugger-policy.md).
Use the [acceptance map](../../../docs/ui/m10-implementation.md) for exact restrictions.
Checkpoint inspection must reject foreign/stale/permitted visits; conditional watches are read-only
and visibly disable on budget exhaustion. Relocation rejects branches/handlers/faults/ancestor frames
and invalidates the old pause without execution. DAP wraps vba-ide, not a second session controller;
JSON is optional transport only. Run native/WASI protocol tests and CLI subprocess lifecycle tests.

ADR 0022 adds IR 6 generic default invocation/value contexts and bounded For Each.
Read docs/m1-defaults-and-collections.md and the current compiler/runtime contract.
Office defaults, collections and signatures remain fork-owned. Preserve safe
watch evaluation, canonical array aliases, resize locks and wait/cancel semantics.
