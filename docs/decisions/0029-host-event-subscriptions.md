# ADR 0029: explicit delivery to host-object WithEvents subscriptions

Status: adopted for M6 implementation, 2026-11-01. Extends ADR 0023 and ADR 0028.

## Problem and ownership

Word Application events need to reach live class/document instances whose
`WithEvents` fields contain the actual host Application object. The existing
project-class event path and document callbacks cannot establish that subscription.
Office event names, signatures, authorization and document lifecycle stay in the
application forks. The shared engine supplies host-neutral subscription mechanics.

## Contract

- Scalar `WithEvents` fields may use an explicitly registered nominal host object
  type. Untyped objects and arrays remain refused. Foreign subscriptions require
  the new `host-events-v1` capability as well as IR 7 project-class support;
  older consumers refuse the program. Project-class event verification is unchanged.
- `Session::host_event_handlers` snapshots actual live field connections for an
  explicitly supplied host publisher and event name, in connection order. It
  requires a current terminal session and matching host identity. Event names,
  parameter counts, subscription counts, scanning work and allocations are bounded.
- The embedding declares each parameter's scalar kind, ByRef mode and optional
  nominal type using `HostEventParameter`. Every matching handler must be a Sub
  with the exact required scalar/nominal/mode signature. All handlers are checked
  before any handler runs; mismatches return errors rather than skipping a sink.
- Opaque `HostEventHandler` snapshots retain the real receiver and procedure.
  `schedule_host_event_handler` uses the ordinary bounded callback path, including
  session/generation checks, host permissions, debugger frames, pause/resume and
  one-shot ByRef output retrieval. Reset/close invalidate old instance handles.
- `vba-ide` exposes the same discovery/delivery path above the VM. A callback is
  delivered only from a completed current invocation. Import never connects or
  executes an event handler; authored code must assign the WithEvents field.
- The host delivers snapshot members sequentially, carries completed ByRef outputs
  into the next handler and defines failure/cancellation policy. No mutable host
  borrow, VM location or callback frame is retained by the host. A snapshot is a
  connection-list snapshot, not a new ambient event source or a COM connection.

## WordCraft acceptance target

WordCraft owns DocumentBeforeClose, DocumentBeforeSave and WindowSelectionChange
signatures, the live document/selection arguments and trust checks. Its delivery
state must preserve the connection list and ByRef values through pauses, cancel
remaining work on Reset/trust loss, and finish application handlers before the
existing document lifecycle tail. Cancellation prevents the save/close operation;
prior edits remain in normal undo history. Fresh Office observations remain a
separate NOT_RUN requirement when Office is unavailable.

## Validation

`crates/vba-ide/tests/host_events.rs` exercises connection order, propagated ByRef
outputs, disconnection, all-before-execution signature refusal, host mismatch,
pause/resume and Reset. The existing project-class suite remains required. The
shared tests run natively and through the existing WASI IDE gate. Fork tests must
prove actual document effects and save/close decisions; generic tests do not
establish Microsoft Word compatibility.
