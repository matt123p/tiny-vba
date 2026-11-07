# ADR 0079: Earlier form input precedes document activation

Status: implemented and validated in shared runtime/UI; consumer integration in progress.
Date: 2026-11-07.

## Contract and ownership

ADR 0078's document barrier must not dispatch Deactivate ahead of an earlier
native form key, text, paste or IME event in the same raw batch. The shared egui
adapter transfers that prefix to its existing native keyboard queue before
begin_pass. The document request waits for queued input, canonical keyboard
receipts, widget defaults and their ordinary callbacks. A preceding raw pointer
event gets a UI pass before deactivation; existing native mouse continuations
also have to become idle. Full mixed pointer order remains an open M01 case.

The keyboard adapter shares one bounded enqueue/normalization path for ordinary
input and transferred prefixes. It retains at most 64 raw entries (Text splits
into individual characters) and 32,768 UTF-16 units, checking before cloning.
Adjacent matching browser Text/Key normalization is limited to one raw batch.
The document queue retains its existing separate bounds. No execution, memory,
permission, callback or queue limit is raised.

Canonical values, selection, focus, completion receipts and event arguments stay
in ordinary generated VBA classes. No VM, compiler, foreign protocol, mutable
control-value store or new VBA surface is introduced. Hosts keep their existing
raw hooks and executable-command authorization paths.

## Composition and cancellation

The raw boundary tracks only whether a native composition is active, its window
and execution/document identity. It stores no composition text or control value.
Nonempty Preedit starts composition; empty Preedit and Commit end it. Legacy
Enabled/Disabled events are accepted too. A composition started before the
document press retains its subsequent IME events in the ordinary keyboard queue,
including during Break. Deactivate waits for its terminal event, default and
callbacks. Paste/IME are not converted into physical key or KeyPress callbacks.
An empty preedit followed by Commit in the same raw batch keeps both events with
the form. New composition in the visible source editor does not acquire native
form ownership or block a later document press.
The adapter does not synthesize an IME commit or infer one from a value diff.
Preedit/value/Change details continue to use the existing native text adapter;
full platform and Office comparison cases remain open.

F5/F8 remain debugger-owned during Break. Reset, changed source/execution/document,
lost authorization or changed canvas/surface geometry discard an unapplied prefix
with its later document gesture. A prefix callback that opens a modal window
instead cancels only the later document gesture: its parked keyboard receipt and
remaining form input must survive the modal callback. This distinction preserves
ADR 0076's nested input ownership.

A Change handler can also redirect to a modeless form after the original key
receipt has been consumed. During the transient no-active-window state, retain
the visible owner's raw tail through Running/Break/Waiting. On a completed
modeless transition, route the remaining queue/held-key metadata to the canonical
active window. Parked modal scopes retain their original destinations. The
regression includes Stop between Deactivate and Activate and explicit SetFocus
in the new window; KeyUp and the later document gesture each execute once.

## Evidence and remaining scope

The production raw-hook tests cover key replacement/default/Change/KeyUp before
Deactivate, Stop in KeyDown or Change, Continue/Reset, paste, coalesced and spanning
composition commit/cancel, IME continuation at Stop, skipped form UI, input budgets,
document/trust invalidation and a modal opened by the prefix callback. Consumer
canvas tests exercise stopped prefix input before any document selection/edit.
Exact final gates and pins belong in implementation progress.

This adds selected L01/L02/K02 ordering evidence. It does not complete M7A.4 or
reduce its member/resource/control/binding/platform checklist. Chrome/keyboard
activation, arbitrary mixed pointer/keyboard ordering, further reentrant focus/
binding/collection cases and platform IME behavior remain open. Fresh Office
comparisons remain M8.5 NOT_RUN; these are independently authored regressions.

References: [MSForms activation](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/activate-deactivate-events),
[physical key events](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/keydown-keyup-events),
[KeyPress](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/keypress-event),
[ADR 0078](0078-document-activation-input-barrier.md).
