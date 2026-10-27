# 0015 — M7 bounded generic callbacks and lifecycle

Adopted for implementation, 2026-10-27.

## Delivery model

The VM remains single-ownership and non-reentrant (bytecode specification: "Running is not an
externally reentrant session state"). A host that needs to call VBA code — an event sink, a queued
notification, a deferred action — schedules a callback by name; the VM executes it as a fresh
top-level invocation on the same session after the current program reaches a terminal state. There
is no mid-run injection, no nested `run_slice`, and no second VM. This matches the supplied
composition design: host borrows are short-lived, callbacks execute at safe points, and a paused
debugger never retains a mutable host borrow.

`Session::schedule_callback(procedure, arguments)` validates the name against the compiled program
(case-insensitive, any visibility), the argument count against the procedure signature, the payload
against the session budget and the queue against `Limits.callbacks` (default 256; FIFO). Scheduling
is refused once the session is terminal. `Session::run_next_callback(host, quota, predicate)`
executes one queued callback: fresh frames on the same session instance, sharing module/static
state, host-root bindings and output; counters report `callbacks_run`; the checkpoint predicate
and breakpoints apply exactly as in `run_slice`; host waits inside a callback use the existing
exactly-once `resume_host` path; a host-root that the host has invalidated surfaces the host's
reference error (91), like any Invoke, with no repeated dispatch. `cancel`/`disconnect`/`host_lost`
drain the queue without executing pending callbacks.

## Static reset and stale references

`Session::reset_state()` clears persistent/static storage and captured output and bumps the
generation; it is refused unless the session is terminal, so reset ownership is explicit rather
than a side effect of dropping the session. A fresh `run_slice` afterwards behaves like a fresh
session over the same compiled revision and identity.

Host references stay the host's to validate. The VM re-checks binding identity when delivering a
callback; object lifetime/generation remains host-owned, and a stale reference is a VBA-visible
numbered error, never a silent success or an automatic rebind.

## Non-goals

Excel event ordering, `Application.EnableEvents`, document/event macros with cancellable decisions,
UDF evaluation and any Office-shaped event catalogue are fork-owned: a fork implements its event
sink over this generic queue and owns ordering/reentrancy policy above it. No Send/Sync or worker
thread is introduced. Callbacks while paused mid-macro and recursive execution remain rejected.
