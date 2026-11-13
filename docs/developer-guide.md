# Developer guide

Use the Rust toolchain pinned in [rust-toolchain.toml](../rust-toolchain.toml) and
the checked-in Cargo.lock. Product code is Rust; maintenance checks require Python
3.11+. The declared minimum Rust version is 1.95; the pinned toolchain is the
verification environment, not evidence of a separate MSRV run.

## Build and run

```sh
cargo run -p vba-ide-app
cargo run -p vba-cli -- parse examples/pure-vba/Arithmetic.bas
cargo run -p vba-cli -- run examples/pure-vba/Arithmetic.bas --entry Answer
cargo run -p vba-cli -- debug examples/pure-vba/Arithmetic.bas --entry Answer
cargo run -p vba-cli -- --help
```

For an external debugger, enable the optional adapter:
`cargo run -p vba-cli --features dap -- dap examples/pure-vba/Arithmetic.bas`.
See [embedding](ui/embedding.md) for host mounting and native/browser source transfer.

## Validate changes

```sh
cargo xtask ci
rustup target add wasm32-unknown-unknown wasm32-wasip1
cargo xtask wasm
cargo xtask wasm-run
```

Use Node 22 for reproducible WASI/host/browser checks. UI work also needs
`cargo xtask ui`, `cargo xtask ui-native` (a display or `xvfb-run -a`) and
`cargo xtask web`. Browser packaging requires wasm-bindgen-cli 0.2.126; the
maintenance browser harness uses its separately locked Playwright tooling.
[Testing](testing.md) explains optional host, fuzz, benchmark and corpus gates.

Run checks from the intended checkout. xtask logs its discovered workspace rather
than trusting a path compiled into a cached executable. WASI artifact lookup honors
`CARGO_TARGET_DIR`. Tests belong under the owning crate/app's `tests/`; root `tests/`
holds shared fixtures. Locate fixtures through `CARGO_MANIFEST_DIR`.
Generated reports and downloaded evidence belong in ignored `reports/` and
`.cache/evidence/`, with provenance/notices tracked separately.

## Make a change

Start with [architecture](architecture.md), the relevant contract in the
[documentation index](README.md), and [AGENTS.md](../AGENTS.md). Preserve the
reviewed dependency graph and headless build boundary. Align the egui family and
patch revisions with the consuming workspace; Cargo patches belong to its root.

Update the owning contract and compatibility claims when behavior changes. Add an
architectural decision for a material contract/ownership change, rather than
appending a progress diary to multiple guides. Keep plans, task checklists and
session handoffs out of maintained reference documentation; Git history retains
removed implementation narratives. Immutable evidence selections/receipts must
retain their identities and must not be presented as fresh results.

External source updates require an explicitly chosen commit, notice/specimen
review, and matching source-lock/inventory hashes. Stage the gitlink and metadata
together, then run strict corpus checks. Do not execute upstream scripts/macros
or import implementation code. See [corpus policy](../tests/corpora/README.md).
