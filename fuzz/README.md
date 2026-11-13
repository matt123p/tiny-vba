# Robustness testing

M8 supplies three dependency-free Rust fuzz entrypoints under `targets/`, built as
examples of their owning crates. `cargo xtask m8` runs all three deterministic
campaigns plus the reviewed library comparisons; `cargo xtask m8-wasm` runs the
same code under Node 22 WASI. Native CI and `wasm-run` include them.

```sh
cargo run -p vba-syntax --example fuzz_parser --locked
cargo run -p vba-project --example fuzz_importer --locked
cargo run -p vba-vm --example fuzz_vm --locked
M8_FUZZ_SEED=123 M8_FUZZ_CASES=4096 cargo run -p vba-vm --example fuzz_vm --locked
cargo run -p vba-project --example fuzz_importer --locked -- --stdin < minimized.bin
```

Default seed is 19768, with six original seeds for parser/importer, seven for VM, and 256 mutations per target.
Campaign counts are bounded to 1..4096; stdin is bounded to 64 KiB. Only valid UTF-8
enters parsing; all three module kinds are checked with bounded diagnostics and
valid spans. Importer targets inert CFB/MS-OVBA decoding and capped decompression.
VM input is capped at 4 KiB, compiled and verified, then executed with 2,048
instructions, 16 frames, 128 locations, 256 KiB cumulative payload and bounded
slices. Infinite loops, recursion, errors, Stop and array seeds are included.
VM seed entrypoints are preflight-compiled, including real infinite-loop and
recursive calls. The harness cancels valid pause-heavy inputs when its own
128-slice quota is exhausted; that is separate from a VM crash or timeout.
No host capabilities or upstream scripts are available.

These are deterministic mutation campaigns and reusable stdin targets, not
coverage-guided libFuzzer runs or a long-duration fuzzing claim. No new external
fuzz dependency is introduced. Minimized accounting regressions live under
`crates/vba-vm/tests/host.rs`; runtime type/palette regressions live in their owning
crate tests. Preserve any future discovered crash input and add a regression
before claiming a fix. Eight asserted native/WASI performance cases run through
`cargo xtask m8-bench`; see [M8 evidence](../docs/testing.md).
