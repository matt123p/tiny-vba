# ADR 0071 — Native keyboard focus and default event order

Date: 2026-11-06. Status: adopted; shared validation passed.

The shared form renderer owns raw keyboard arbitration before editor shortcuts
or source editing. A native window's function keys must not restart a macro via
VBE shortcuts. Paused execution retains the debugger's existing F5/F8 commands;
their releases belong to those commands even if Continue returns focus to a form.

Canonical focus, handler arguments and edits stay in ordinary VBA objects and
frames. The UI retains stable root/control widget IDs and bounded physical-key
ownership, not another copy of control values. A root focus target exists for
empty forms and forms whose effective visible controls are disabled. The normal
VBA input method also checks root eligibility, so omitting TabStop alone does not
make a form eligible to receive root keystrokes. Inspection never calls getters.

Retain the raw triggering character/release while a keyboard handler is stopped.
Run no new callback in break mode. Ignore additional repeats of that held key
while stopped and leave subsequent debugger/editor typing to its actual focus.
Source edits, Reset, stale objects and hidden windows discard pending ownership.
The existing 64-event/32,768-UTF-16-unit barrier remains; held key identities are
also capped at 64 and debugger shortcut identities at F5/F8. No execution,
memory, host-call or static-permission budget increases are introduced.

Clear egui's precomputed keyboard focus navigation when intercepting a native
key. Replay the approved default input against current canonical focus, then
let ordinary focus/validation callbacks finish before the queued release.
A conservative generation-keyed authored-procedure hint keeps the barrier active
when the old receiver has no keyboard handler but the new receiver does. Actual
event matching and ANSI conversion remain normal provider decisions, not this
hint. Escape's early toolkit focus clearing is recovered only for an identified
previously focused native widget.

For Space on a non-text receiver, dispatch the ANSI character phase before the
widget's default action. Suppress a paired raw space text event so KeyPress is
not delivered twice. Zero cancels before Click/Change. Preserve the same mutable
ReturnInteger and ordinary callback/wait/debugger frames as ADR 0070. Editable
text keeps its separate physical-key/text-commit path; paste/IME stay distinct.

The reviewed eframe web backend emits a text event before the physical key
when its text agent is unfocused. Normalize a matching adjacent single ASCII
text/key pair within that captured input batch. Do not reorder unrelated text,
IME, multi-character commits or events across frames. A regression supplies the
exact web-style batch independently of the native event ordering.

This closes authored root-focus, Tab/SetFocus continuation, Space cancellation and
editor-shortcut isolation regressions. Full keyboard/platform/focus/IME, mouse,
accelerators, lifecycle and the other M7A.4 checklist rows remain open. Fresh
Office build/OS/locale comparisons remain M8.5 NOT_RUN. No new format, language
feature, VM Office special case or parallel control store is added.

Primary references:

- [MSForms physical key events](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/keydown-keyup-events)
- [MSForms ANSI character event](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/keypress-event)
- [Existing input barrier](0070-native-keyboard-events.md)

Shared validation PASS: full CI, WASM compilation, actual WASI, all nine runtime
keyboard cases, 11 new UI keyboard cases and all 24 existing native form UI cases.
UI/native/web gates pass. Chromium proves stopped Tab routing and cancelled
button Space without duplicate KeyPress or a leaked debugger KeyUp. Paused and
cancelled form captures were inspected. Host integration is being validated.
