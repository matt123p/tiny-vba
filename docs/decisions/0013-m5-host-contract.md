# ADR 0013: M5 native host integration contract

Adopted for implementation, 2026-10-26.

vba-foreign owns protocol version 1 and generic source imports (a named root object or a member
of a named root). Compiler/binder consume only these declarations; no Office member catalogue
or application state enters the engine. Source global names resolve only when explicitly supplied
by the embedding application. Nested foreign result targets use existing explicit Invoke
continuations, preserving exactly-once dispatch and statement debugger boundaries.

IR 4 adds hidden Object HostRoot local bindings; earlier IR targets remain closed. HostBinding
supplies the root references and explicit site allowlist. Missing/mismatched roots and incompatible
protocols fail before dispatch. The VM remains independent of compiler, debugger and applications.
Root references use the host's identity/generation; the application validates its own object lifetime.

Version the foreign/value/debug public contracts and publish supported IR/profile/feature tables.
This is a native Rust contract, not a stable C ABI or serialized bytecode promise. JSON is not
the host member/value ABI. The testkit supplies bounded run/inspection helpers reusable by forks.

Gridcraft owns its integration package and Office-shaped fake/real workbook proofs. Its existing
Session command/edit/recalc/history mechanisms remain authoritative. A typed edit entry point may
reuse the same command execution guard, journal and edit transaction; workbook values never pass
through a JSON member-dispatch shim. Active and explicitly qualified objects have distinct target
identities. Successful writes survive later errors; no implicit whole-macro rollback is introduced.
Pause, cancellation, close and host loss must release borrows and pending tokens explicitly.

M5 acceptance requires fake and real FillCells, breakpoint/edit/resume without repeated writes,
active versus qualified targets, stale refs, permissions/protocol refusal, partial effects and
undo/recalc integrity. Core native/WASM builds must not require Gridcraft or its dependencies.
The existing broad Office-model/import/UI/event roadmap remains outside this bounded bridge.
