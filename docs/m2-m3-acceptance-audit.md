# M2/M3 acceptance audit

Audited 2026-10-26 after local checkpoint 4247b5c. The accepted scope remains IR 1 and the
m3-core-v1 subset described in the compiler/runtime contracts; this does not broaden VBA syntax,
coercions, error handlers or Office support.

## M2 gate review

| Obligation | Evidence | Audit result |
|---|---|---|
| Scopes, Option Explicit, locals/result/parameters, supported argument binding | vba-semantics/tests/binding.rs; compiler negative binding and module visibility cases | Covered |
| Compile examples and deterministic disassembly | compiler answer_golden_is_verified_and_deterministic, if_for_byref_and_hidden_scratch_lower_to_verified_blocks | Covered |
| Exact constant tags, slot/site order, same-line and nested-call source maps | compiler nested_calls_and_colon_statements_map_to_their_own_checkpoint and first_emitted_constant_order_preserves_exact_tags_and_deduplicates | Covered |
| Invalid constant/local/callee/site/branch operands and entry into expressions | IR malformed_indexes_blocks_stacks_and_metadata_are_rejected and direct-call/foreign metadata mutation tests | Covered |
| Missing terminators, falloff, unreachable code, V/Location mismatches and exact maximum stack | IR malformed/CFG/stack cases; compiler unreachable source retention test | Covered |
| Version/feature/op/conversion/profile refusal and metadata/work/payload budgets | IR feature_profile_version_opcode_operator_and_conversion_gates_are_checked and all_verifier_resource_dimensions_are_enforced; compiler custom targets | Covered |
| Valid reachable self-loop; VerifiedProgram only through bounded verification | IR reachable_infinite_cycle_is_valid_and_unreachable_block_is_rejected; private verified-program constructor, public verify/recheck | Covered |

M2 is complete for its adopted, explicitly limited IR 1 subset. No M2 code change was needed by this
audit. Arrays, persistent module storage, optional/named direct-call binding and later IR features
remain explicit unsupported cases under ADR 0004/0006, not missing IR 1 implementations.

## M3 continuation

Re-ran the existing execution/debugger/host acceptance suite, then added a session-reuse regression.
The new case failed before the fix: a handle from a cancelled/dropped session was accepted by a new
session with the same caller label, frame ID and pause generation. Old controller step/breakpoint
state also used only the caller label as its execution discriminator.

Implemented [ADR 0008](decisions/0008-vm-instance-isolation.md): each execution receives a fresh opaque
SessionInstance that is never recycled within the loaded engine. Handles, location ownership,
breakpoints and step targets use that instance. The caller label remains available for routing.
The regression checks stale reads/edits and stale controller bindings; identical assertions run in
the shared native/WASI smoke program. Numeric instance values are deliberately not golden output.

Verification commands:

```sh
cargo xtask ci
cargo test --workspace --release --locked
cargo xtask wasm
cargo xtask wasm-run
```

All 75 integration tests pass in debug and release. The shared smoke executes exact Long(42),
If/For/calls Long(18), host pause/edit/resume Long(41), one invocation/two polls, stale handles,
cancellation and replacement-session isolation both natively and in wasm32-wasip1 under Node.
Core libraries compile for wasm32-unknown-unknown. Local toolchain: Rust 1.99.1, Node 18.19.1.
Native CI also checks format, strict Clippy, reviewed crate layers and seven corpus pins/646 hashes.
These are local checks, not a new macOS/Windows, separate MSRV or Office compatibility result.

M3's reviewed headless subset remains complete with this correction. The next milestone remains
M4's scenario assertion runner and semantic/error-handler work; graphical debugger/DAP and Office
integration retain their separate acceptance gates. M0/external execution manifests stay NOT_RUN.
