# 0015 — M7 bounded generic callbacks and lifecycle

Adopted and implemented; acceptance audit completed 2026-10-27.

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
is refused once the session lifecycle is closed. Normal completion and VBA runtime failure
remain callback delivery windows. Argument conversion and array element types are checked before
enqueue; invalid requests leave the queue unchanged. Queue payload and metadata consume the
shared cumulative session payload budget, including repeated enqueue/drain cycles. Top-level
ByRef parameters receive owned argument locations; direct VBA calls retain canonical aliasing.

`Session::run_next_callback(host, quota, predicate)` executes one queued callback: fresh frames on the same session instance, sharing module/static
state, host-root bindings and output; counters report `callbacks_run`; the checkpoint predicate
and breakpoints apply exactly as in `run_slice`; host waits inside a callback use the existing
exactly-once `resume_host` path; a host-root that the host has invalidated surfaces the host's
reference error (91), like any Invoke, with no repeated dispatch. `cancel`/`disconnect`/`host_lost`
drain the queue without executing pending callbacks and permanently close scheduling, delivery,
root rebinding and reset, even if the last invocation was already terminal. Cleanup on an already
terminal invocation preserves its result/failure; lifecycle closure is tracked independently.

Delivery validates host identity/protocol, quota, source freshness and lifecycle before removing
anything from the queue or changing the current outcome. Each delivered callback starts with a
fresh Err state; module/static values and captured output persist. Frame-start resource failures
produce a structured terminal failure and clean partial allocations.

## Static reset and stale references

`Session::reset_state()` clears persistent/static storage and captured output and bumps the
generation. Reset requires a terminal, open session with current source. It prepares the original
entry procedure with the original bound launch arguments and sets the outcome to Yielded; the
next `run_slice` executes it against fresh project state. Err state and pending pause requests
are cleared. Queued callbacks survive but cannot deliver until the restarted entry terminates.
Compiled revision, session instance, frame/visit identity sequences, debugger breakpoints and
cumulative instruction/host/allocation/location budgets remain unchanged. Reset cannot reopen a
closed lifecycle or replenish exhausted work budgets. If entry-frame preparation exhausts a
budget, reset returns an error and leaves a clean structured terminal failure.

Host references stay the host's to validate. The VM re-checks binding identity when delivering a
callback; object lifetime/generation remains host-owned, and a stale reference is a VBA-visible
numbered error, never a silent success or an automatic rebind.

## Non-goals

Excel event ordering, `Application.EnableEvents`, document/event macros with cancellable decisions,
UDF evaluation and any Office-shaped event catalogue are fork-owned: a fork implements its event
sink over this generic queue and owns ordering/reentrancy policy above it. No Send/Sync or worker
thread is introduced. Callbacks while paused mid-macro and recursive execution remain rejected.

## Acceptance audit

All 17 tests in the native and WASI callback integration suite pass and cover FIFO/state sharing, queue/payload limits,
argument conversion and top-level ByRef ownership, nonterminal/empty/invalid-quota/stale-source
refusals, exactly-once host wait/resume, stale roots, callback breakpoints, runtime-failure recovery,
closure from runnable/paused/completed/failed states, and reset/restart with original arguments,
cleared output/Err/statics and retained lifetime budgets. `cargo xtask wasm-run` executes this
same suite in addition to the language smoke and M6 import suite. Office event-order and real
workbook workflows remain separate fork acceptance gates; they are not established here.
