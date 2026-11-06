# ADR 0070 — Native keyboard events and default input

Date: 2026-11-06. Status: adopted; shared validation passed.

The shared IDE form provider owns native KeyDown/KeyPress/KeyUp dispatch. Emit
ordinary typed events on each eligible generated control class and typed bridges
for private authored handlers. ByVal MSForms.ReturnInteger retains its mutable
Value through normal event frames; Shift is the 1/2/4 modifier mask. Specific
MSForms interfaces expose these events to static and Controls.Add WithEvents
sinks. Generic Control retains its existing event exposure. No Office name enters
the generic VM: its inert bounded metadata queries reuse the existing RaiseEvent
subscription selector and never execute code or activate objects.

The egui adapter queues raw input before widgets process it. An ordinary form
method publishes a completion receipt in canonical VBA fields. Only after the
callback completes may the original or replaced input reach the focused widget.
Zero cancels input. Stop preserves the pending receipt; Continue completes the
same frame; Reset/source changes invalidate its object identity. Hidden, detached
or unloaded/reloaded receivers cannot replay edits. UI storage contains raw input,
identity and receipt sequence only, never another copy of control values. Pending
input is bounded to 64 events and 32,768 UTF-16 units across windows. Overflow
reports an error and discards the batch before further default processing.

Text commits are distinct from physical keys. Use the existing injected Asc/Chr
locale services when an ANSI KeyPress handler is connected. With no such handler,
preserve Unicode text without requesting an ANSI conversion. Paste, copy, cut and
IME input retain distinct adapter events; do not infer KeyPress from Value/Change.
Backspace/Escape require a character phase; Delete does not. Existing eligible
Default/Cancel commands bypass Enter/Escape physical handlers. Unknown physical
key/replacement mappings fail explicitly rather than guessing a platform code.

This is a reference-based implementation, not fresh Office evidence. K01/K02
remain PARTIAL until the full checklist passes, including platform mapping,
control-character shortcuts, root-window focus, IME/locale behavior, focus transfer and repeat edge
cases. Accelerators and mouse contracts remain separate open M7A.4 work. Existing
host permissions, execution/memory limits, binary import/writeback boundaries and
the owner's 4,096 static permission allowance are unchanged.

References:

- [Microsoft Forms KeyDown/KeyUp](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/keydown-keyup-events)
- [Microsoft Forms KeyPress](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/keypress-event)
- [M7A.4 completion checklist](../m7a4-completion-checklist.md)

Shared validation PASS: full CI, WASM compilation, actual WASI suite and the
final eight-case keyboard target, all 24 native form UI cases, UI/native/web
gates and an actual Chromium stopped-KeyPress/replacement workflow. Paused and
resumed captures were inspected. Host pins and acceptance are recorded separately.

Host integration found UTF-16 context incorrectly feeding its stored bytes into
Asc as a DBCS character (ASCII a became 24832). The runtime now treats UTF-16
storage as lacking an ANSI code page: Asc/Chr preserve ASCII and refuse non-ASCII
ANSI conversion until a genuine code page is configured. AscB/ChrB/LenB and
byte-string storage retain their existing representation contract; configured
Windows-1252 and Shift-JIS behavior stays covered by the runtime suite. This
shared intrinsic correction applies equally inside and outside keyboard handlers.
See the primary [Asc](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/asc-function)
and [Chr](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/chr-function) references.
