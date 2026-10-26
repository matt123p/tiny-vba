# Developer guide

Use the commands in [README](../README.md). Product code is Rust; maintenance checks require Python 3.11+.
`rust-toolchain.toml` pins the compiler; Cargo.lock records the workspace resolution. MSRV is declared as 1.95;
current verification uses the pinned toolchain, not a separate MSRV CI run.

`cargo xtask ci` is the native gate. `cargo xtask wasm` checks all libraries for wasm32-unknown-unknown;
it requires `rustup target add wasm32-unknown-unknown`. Run `cargo xtask wasm-run` for identical native/WASI acceptance; it additionally needs
Node 18+ and `rustup target add wasm32-wasip1`.
`cargo xtask layers` reads Cargo metadata and compares normal dependencies to the reviewed acyclic graph.
`cargo xtask corpora` validates Git/source pins and available hashes. The strict Python form in README also
requires all submodules to be initialized. CI tests native Linux/macOS/Windows, library WASM, and strict corpus integrity separately.

Rust tests live under `crates/<owner>/tests/` or `apps/vba-cli/tests/`, not root tests/.
Root tier directories hold shared input/expected data; use CARGO_MANIFEST_DIR to locate fixtures reliably.
Keep generated artifacts under ignored reports/. External macros never run as part of native checks.
For M0 fixtures, run `cargo run -p vba-cli -- conformance validate tests/fixtures/m0/arithmetic.vbas`.
Validation succeeds for well-formed [schema 1](scenario-format.md), reports NOT_RUN, and never executes VBA.
See [ADR 0003](decisions/0003-m0-source-scenarios-foreign.md) for source/location identities and explicit
pending-host cancellation obligations. M1 implements a bounded source-aware lexer/parser:
`cargo run -p vba-cli -- parse examples/pure-vba/Arithmetic.bas`. See [frontend](frontend.md) for
the subset, diagnostics, AST and [syntax-only corpus report](m1-corpus-report.md).
Before compiler/VM work, read the adopted
[bytecode/VM specification](bytecode-vm-specification.md) and [ADR 0004](decisions/0004-bytecode-vm-contract.md).
They define the IR 1 layout/verifier/continuation contracts and M2/M3 acceptance cases.
M3 now executes the [runtime subset](runtime.md) via `vba-cli run`; use the Rust m3_smoke example
for host wait/debugger/cancel acceptance. M2 now implements vba-compiler, binding, IR verification and disassembly; see [the compiler contract](compiler.md).

To update a source pin, fetch/checkout an explicitly chosen commit in that submodule, inspect upstream notices
and specimen changes without running it, then update the source lock and affected inventory hashes together.
Stage the gitlink and metadata together; review the diff and run strict corpus verification. Do not use moving
branches or silently rewrite expected outputs. Do not commit modified upstream source as interpreter code.

The app forks should consume a published or revision-pinned tiny-vba dependency at their upper composition layer.
No local sibling path or all-project workspace is needed. A separate integration workbench may be added later.

The editor/debugger UI and standalone IDE are developed here, following
[the UI contract](editor-debugger-ui.md). Gridcraft consumes the embeddable UI package.
When UI packages land, add separate UI build/snapshot/accessibility checks and explicit headless
package checks; the existing workspace-wide WASM library gate must be reviewed for the new layout.
Use the same reviewed egui family revisions as the embedding host: Cargo patches are applied
by the consuming workspace root, so standalone and embedding workspaces must align their pins.

Validation must run from the intended checkout: xtask now discovers its runtime workspace and logs
`xtask workspace: ...`. It does not trust the executable's compiled-in source path, which can refer
to a different clone in a shared build cache. The cross-checkout integration test covers this case;
wasm-run also honors CARGO_TARGET_DIR when finding the built artifact. Current execution uses
m4-scalars-v1; recompile legacy M3 programs instead of relabeling their verification target.
