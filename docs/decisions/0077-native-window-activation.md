# ADR 0077: Native UserForm window activation

Status: implemented and validated in shared runtime/UI and both pinned consumers.
Date: 2026-11-06.

## Contract and ownership

The native form provider in `vba-ide` owns application window activation through
ordinary generated VBA classes. The UserForms provider retains the currently
active window and bounded scalar ordering/transition tokens. The VM, compiler,
foreign protocol and authored source gain no Office-specific feature. Rendering
only inspects canonical fields; it never activates a form or invokes a getter.
No runtime value is duplicated in a mutable host registry.

Show activates a visible native window; switching windows deactivates the former
window before activating the next. Repeated activation is inert. Hide deactivates
and restores the most recent eligible visible window. Unload retires ownership
without raising Deactivate, then restores another visible window. Visible modal
windows prevent modeless activation. Existing modal waits and permissions remain
authoritative. Retained modal rank also covers an Activate handler paused before
Show registers its foreign wait.

Transition tokens reject obsolete outer activation/Show continuations after a
handler redirects focus, hides/re-shows or unloads a requested window. Losing
window activation invalidates in-progress control focus defaults. Hide clears
its display reference only if a callback has not shown the same instance again.
Reset and source changes retain their existing whole-session invalidation.

## Input and embedding

The shared egui renderer orders modeless windows by canonical activation order.
A physical header/body press produces an executable InstanceWindowFocus command;
normal control mouse/key/focus entry uses ordinary activation frames. Passive
hover and external application focus loss do not synthesize Deactivate.

Keyboard continuations retain their executing receipt owner separately from the
window receiving subsequent defaults/input. After an authored callback activates
another modeless window, queued text/releases follow that window exactly once.
Input waits until the destination widget exists, including after Stop. Existing
raw queue, callback, VM, memory and authorization budgets remain unchanged.

`document_window_activated` returns an executable command for embedders. A host
must dispatch it through its existing trust/authorization gate before document
input. Automatic document-canvas raw-input buffering/replay is **not implemented**
in either consumer by this change; its integration remains L01/L02 work. Modal
RefEdit document selection must preserve modal ownership.

## Evidence and limits

Thirteen authored runtime tests in `vba-ide/tests/form_window.rs` cover lifecycle
ordering, inert queries, retained alias/display lifetime, focus redirection,
QueryClose cancellation, Stop/resume and modal ownership before the foreign wait.
Four actual egui tests in `vba-ui-egui/tests/form_windows.rs` cover header presses,
executable document activation, external viewport focus and modeless key transfer
with/without Stop. These are authored expectations, not fresh Office traces.

M7A.4 remains IN PROGRESS. The completion checklist remains authoritative;
remaining document input barriers, broader lifecycle/binding/collection/platform
and property contracts are not reclassified as complete. M7A.5/.6 remain native
edited persistence and legacy format removal. M8.5 observations remain NOT_RUN.

## Primary references

- [MSForms Activate/Deactivate](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/activate-deactivate-events)
- [Show](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/show-method)
- [Hide](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/hide-method)

The references establish application-internal activation and the absence of
Deactivate on unloading. Exact reentrant event traces remain authored inferences
pending the separate fresh Office comparison gate.

Shared full CI/strict Clippy, WASM/WASI, UI/native/web and actual Chromium
activation/nested-input/editor/modal-form checks pass. Both consumers pin product
`a7d9d3122c42ca602e14118da48477ea02522f06`. GridCraft full CI (10 steps), strict
optional VBA/UI Clippy, four form UI cases, web and 28 native/28 Node-WASM package
cases pass. WordCraft full CI (8 steps, isolated serial tests), strict optional VBA/UI
Clippy, 13 VBA UI cases, the hidden-editor form, web and 23 native/23 Node-WASM
package cases pass. The native nested-frame UI regression waits for its canonical
Tab KeyUp continuation before requesting the next pointer action; broader mixed
input queuing while callbacks run remains part of the open M01/L02 acceptance.
These results accept this change, not the complete M7A.4 checklist.
