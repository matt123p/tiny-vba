# 0008 — Non-reused VM instance identity

Date: 2026-10-26. Status: adopted and implemented during the M2/M3 acceptance audit.

A caller-supplied session label is insufficient to isolate inspection handles. After an old session
is cancelled/dropped, a replacement can reuse that label, frame ID and pause generation. M3's
previous handle validation then accepted the historical handle, and a retained debugger controller
could bind its old breakpoints/step target to the replacement execution.

Keep Session::identity() as the caller's routing label. Introduce opaque SessionInstance, allocated
once per Session from a non-wrapping atomic u32 sequence within the loaded engine. Exhaustion
returns a resource error; cancellation/drop never recycles an instance token. Atomic compare/exchange
uses only stable APIs available at the declared MSRV and works in native/WASM builds without clock,
randomness, IO or a platform identity service. Instance IDs are ephemeral, not compiled-code IDs.

Handles and checkpoint views carry this token alongside the existing caller label. Handle validation
checks token, label, frame and pause generation. Location owner identity derives from the token.
The debugger keys breakpoints and step targets by instance, while retaining source/revision checks
when binding. Identical compiled source, labels and frame/generation values cannot revive an old
handle or controller target in a new execution. SessionInstance's constructor remains private.

This changes the unfrozen M3 runtime API; it does not change any IR instruction or serialized ABI.
IR 1 remains unchanged. Regression tests and the shared native/WASI smoke program cover rejected
old-handle reads/edits and ignored stale breakpoint/step bindings after client-ID reuse. See
[acceptance audit](../m2-m3-acceptance-audit.md) and [runtime contract](../runtime.md).
