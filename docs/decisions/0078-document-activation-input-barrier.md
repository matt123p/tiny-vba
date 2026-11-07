# ADR 0078: Document activation before document input

Status: implemented and validated in shared runtime/UI and both pinned consumers.
Date: 2026-11-07.

## Ownership and contract

ADR 0077 supplies ordinary native UserForm activation and executable window-focus
commands. Its document activation helper alone does not order document input.
The shared egui view now retains bounded raw document gestures at the embedding's
raw-input hook, before begin_pass computes pointer state. A document press requests
an authorized InstanceWindowFocus(false); the document receives the original input
only after the ordinary Deactivate callback completes. Stop/Continue preserve that
continuation; no alternate interpreter, document model or control-value store is
introduced. The compiler, VM and foreign protocol gain no Office behavior.

The IDE exposes inert execution_identity using the existing opaque, non-reused
VM SessionInstance. The caller-supplied Session.identity label can be reused and
is deliberately not the invalidation key. Hosts supply a document/window lifetime
identity, authorized canvas bounds and whether the canvas is actually visible.
Reset, changed source, a replaced execution/document/window, lost authorization,
changed canvas/surface bounds or an authored window activation discard stale input.
An original held release stays with its discarded gesture.

Raw input retains its original batches, button/modifier/position data, text, paste
and IME distinctions. At most 64 queued events and 32,768 UTF-16 text units are
retained. Overflow discards remaining input and reports explicitly; already-applied
document actions are not rolled back.
F5/F8 remain available during Break; their releases are not replayed into the
document. New debugger pointer/source input stays available while the document
is hidden. No runtime instruction, memory, static permission or callback limit
is increased.

## UI pass and host integration

The raw hook runs after synthetic input is offered. The original synthetic batch
can be acknowledged once its events belong to the barrier; the barrier then owns
its bounded raw continuation. A replayed batch is offered once per UI pass and
removed only after the document's completed, non-discarded root UI pass acknowledges
it. A hidden logic tick does not append it again. A consumed pass that skipped the
document (e.g. fonts warm-up) causes a fresh acknowledged canvas paint before replay,
so input cannot target missing/stale widget hit areas.

GridCraft dispatches through dispatch_editor_command/Bridge.begin; WordCraft uses
prepare_ui_action and its existing trusted command dispatcher. Both raw hooks run
before canvas input. Continue returns a pending original document activation from
the debugger to its document view, paints the canvas, then releases the gesture.
The debugger is not hidden while its callback remains paused. Modal forms retain
their input ownership; RefEdit's explicit worksheet-selection lending bypasses
this modeless deactivation path. External application focus is not document input.

## Evidence and limits

Eleven shared actual egui/raw-input tests cover callback Stop/Continue, one document
action, source preservation, cancellation on Reset/source/document/trust changes,
non-reused replacement executions, reentrant activation, hidden/unapplied passes,
held releases, stale geometry, event/text budgets and modal ownership. Host tests
use real document/canvas input and the existing authorization path. Final test
receipts and pinned revisions are recorded in implementation progress.

This implements selected L01/L02 document input ordering; it does not finish the
broader M7A.4 control/member, binding, collection, mixed-input and platform ledger.
Fresh Office traces remain M8.5 NOT_RUN. Expectations are independently authored
against the published MSForms contract, not measured Office observations.

References: [MSForms activation](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/activate-deactivate-events),
[Hide](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/hide-method),
[ADR 0077](0077-native-window-activation.md),
[instance identity](0008-vm-instance-isolation.md).

If activation invalidates a partially replayed document gesture, the view also
cancels its own egui pointer/drag capture before discarding the queued release.
The capture regressions reactivates a form after a replayed press, proves no stuck
button or ghost click, and verifies a fresh document gesture still works.

Remaining mixed-input acceptance includes a coalesced earlier form-key/IME event
before the document press, focus transferred through other chrome/keyboard routes,
and further pointer/collection/binding redirection. These are open checklist cases;
the canvas-gesture regressions do not establish complete input or lifecycle parity.

Canonical window ownership also gates native keyboard fallback and rejects stale
egui form focus after document activation. Pending key receipts and retained modal
ownership keep their existing exceptions. The eleventh regression injects an old
native focus bookmark after the document click and verifies no native text edit
or window reactivation. Existing modeless transfer and keyboard suites still pass.

Final shared CI/WASM/WASI/UI/native/web and Chromium checks pass. Both hosts pin
`ce6482640332cb67ccfe3372886a4039c832437e`. GridCraft full CI (10 steps), strict
optional VBA/UI Clippy, five form UI cases, web and 28 native/28 Node-WASM package
cases pass. WordCraft full CI (8 steps, isolated serial tests), strict optional
VBA/UI Clippy, 13 VBA UI cases, the hidden-editor and document-focus/Reset cases,
web and 23 native/23 Node-WASM cases pass. Host document views and five native
views were inspected. This accepts the canvas continuation change; the complete
M7A.4 checklist remains open.
