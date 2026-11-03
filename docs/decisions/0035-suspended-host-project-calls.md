# ADR 0035 — bounded host calls into a waiting project

Accepted 2026-11-03 for generic host integration. Office dispatch and file operations remain fork-owned.

A foreign invocation already returns `Reply::Wait(token)`. A host sometimes needs a public
procedure or cancellable document/publisher callback to finish before answering that token.
Starting a fresh interpreter loses persistent globals, object identities and cumulative limits.

The IDE now exposes `SuspendedCall` and `start_suspended_call(token, call, host)` over the
existing explicit suspended-callback frames. It requires the exact current wait token, matching
host, current source, an empty callback queue and bounded nesting (eight). Calls run only through
normal debugger ticks. `take_host_call_result` returns the token and a typed result/error with
callback ByRef outputs. The host owns its effects and must answer the original wait separately;
callback outputs are not writebacks to the original foreign call's argument slots.

The VM resolves unambiguous standard-module procedures, optionally module-qualified. The IDE
also checks public visibility against current source metadata. Document events retain native
owner binding; host publisher events retain live subscription identity. Runtime VBA errors can
be returned to the parked caller; resource, protocol and cancellation failures remain fail-closed.
Parked frames count toward call depth and share all lifetime work/payload/callback limits.
Reset/close cancel every parked token once. Existing modal-form suspension remains supported.
`prepare_host_events` prepares idle callback context without executing user code.

IR 8 gains `host-procedure-defaults-v1`: standard optional parameters retain literal default /
required metadata and a trailing ParamArray flag for runtime host dispatch. Older targets keep statically expanded arguments
and required standard parameter metadata. Verifiers refuse the extension without both IR 8
and its capability; no opcode or foreign protocol variant is added. Qualified type members may
use contextual keyword spellings (`External.Name`); bare reserved identifiers remain invalid.

Generic tests live in `vba-ide/tests/suspended_calls.rs`, the compiler capability regression,
and syntax qualified-type regression. Workbook trust, macro names, event signatures/order,
consent, prompts, persistence, undo and active/code-owner routing are tested in GridCraft.

Hosts may select a fatal policy for a nested runtime error with `Ide::abort_host_wait`.
It validates the exact current token, cancels all parked and modal-form waits through
their owning host and publishes a terminal runtime diagnostic without invoking a caller
error handler. Generic result/error transport does not prescribe an application's policy.
The generic host regression verifies wrong-token refusal, no caller resumption and one
cancellation. GridCraft selects this policy for an unhandled macro error in Application.Run.
