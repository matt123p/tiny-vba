# 0007 — M3 explicit execution and generic host continuations

Date: 2026-10-26. Status: adopted and implemented for the M3 subset.

Implement ADR 0004's IR 1 execution model in vba-vm, shared scalar operations in vba-runtime and
source breakpoint/stepping policy in vba-debug. The VM depends on no compiler/debug/UI type.
Add CLI -> vm/values for `run`; debugger/foreign example support stays in CLI dev-dependencies.
VM acceptance tests use compiler/debug/testkit as dev-dependencies only. No external Rust crate added.

Publish m3-core-v1 separately from M2's structural verification profile. Compile against its actual
capabilities and recheck immutable VerifiedProgram at launch. Implement a narrow tested numeric/
conversion matrix, not all operation combinations described by the IR enums. Unknown combinations
fail Unsupported. Error numbers 6/11 cover checked integer overflow/division by zero; host errors
preserve their supplied numbers. Broader coercion and VBA handler state remain M4.

Use explicit frames, owned typed locations and canonical direct ByRef aliases. Capture call fault PCs
and advance caller continuations once. Checkpoints have fresh visit IDs and one-visit continuation
permits. Keep quota yields, ordinary pauses, post-Stop pauses, waiting, fault pauses and terminal states
separate. Breakpoint binding includes session/revision/statement identity; step targets use frame IDs.
Inspection handles expire on transitions; edits are validated through runtime rules and cannot touch
hidden or fault-paused storage. Truncate snapshot locals explicitly under a bounded copy budget.

Extend vba-foreign with precommit validation hooks for return values and every patch, while retaining
the original M0 wrapper API. The VM uses only checked hooks; the controller remains the sole writeback
committer. Refuse foreign alias snapshots before dispatch; exact-tag typed writes and validated Variant
writes are the supported subset. Explicit host identity/site permissions, poll limits, cancellation and
lost-host reporting preserve the M0 ownership boundary. Generic Object-local member source binding
adds Call/Get/Let/Set without Office names or default-member inference.

Use the same Rust acceptance example for native and wasm32-wasip1 execution under a Node WASI runner
with no preopened directories/environment. Keep wasm32-unknown-unknown library checks separately.
The runner is test infrastructure; product execution remains Rust and host services remain explicit.
[Runtime contract](../runtime.md) records exact APIs, capabilities, charges, runnable commands, tests
and limitations. This completes the headless M3 proof, not the required M8 graphical editor/debugger,
Office bridge, DAP, scenario assertion runner or broad VBA error/coercion support.
