# Developer guide

Use the commands in [README](../README.md). Product code is Rust; maintenance checks require Python 3.11+.
`rust-toolchain.toml` pins the compiler; Cargo.lock records the workspace resolution. MSRV is declared as 1.95;
current verification uses the pinned toolchain, not a separate MSRV CI run.

`cargo xtask ci` is the native gate. `cargo xtask wasm` checks all libraries for wasm32-unknown-unknown;
it requires `rustup target add wasm32-unknown-unknown`. No runtime WASM tests exist until M3.
`cargo xtask layers` reads Cargo metadata and compares normal dependencies to the reviewed acyclic graph.
`cargo xtask corpora` validates Git/source pins and available hashes. The strict Python form in README also
requires all submodules to be initialized. CI tests native Linux/macOS/Windows, library WASM, and strict corpus integrity separately.

Rust tests live under `crates/<owner>/tests/` or `apps/vba-cli/tests/`, not root tests/.
Root tier directories hold shared input/expected data; use CARGO_MANIFEST_DIR to locate fixtures reliably.
Keep generated artifacts under ignored reports/. External macros never run as part of native scaffold checks.

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
