# tiny-vba

A new, standalone VBA interpreter, editor and debugger in Rust, intended to run natively and in WebAssembly.
Develop, run and test the language engine without Gridcraft, Wordcraft, Microsoft Office or a graphical interface.

**Current status:** M0–M3 complete for the documented [frontend](docs/frontend.md) and
[compiler subset](docs/compiler.md): source-preserving parsing, scalar name/signature binding,
checkpoint-based bytecode lowering, bounded IR 1 verification, explicit-frame execution and a headless debugger.
The CLI implements `parse`, `check`, `compile`, `disasm`, `run` and fixture validation.
See the [M3 runtime matrix](docs/runtime.md) for supported execution and debugger/host APIs.
The graphical editor/debugger and DAP remain planned. Fixture validation still reports NOT_RUN;
there is no automatic scenario assertion runner.


The Excel/Word compatibility layers, Office metadata and application adapters belong in the
Gridcraft and Wordcraft forks. This repo owns language semantics, bytecode, execution,
debugging, a reusable egui editor/debugger UI, a standalone IDE application, a generic foreign-call
contract, and read-only VBA project decoding. Gridcraft embeds this UI using its existing UI stack
and design conventions. See the [editor/debugger UI contract](docs/editor-debugger-ui.md).

## Start here

Rust is pinned by `rust-toolchain.toml`; Python 3.11+ and Git support repository checks only.
No third-party Rust crates or Office installations are needed for M0–M3. Node 18+ runs the WASI smoke test.

```sh
cargo run -p vba-cli -- --help
cargo run -p vba-cli -- parse examples/pure-vba/Arithmetic.bas
cargo run -p vba-cli -- run examples/pure-vba/Arithmetic.bas
cargo run -p vba-cli --example m3_smoke
cargo run -p vba-cli -- disasm examples/pure-vba/Arithmetic.bas
cargo run -p vba-cli -- compile examples/pure-vba/M2Control.bas --entry Answer
cargo run -p vba-cli -- conformance validate tests/fixtures/m0/arithmetic.vbas
cargo test --workspace --locked
cargo xtask ci
rustup target add wasm32-unknown-unknown
cargo xtask wasm
rustup target add wasm32-wasip1
cargo xtask wasm-run
```

`cargo xtask ci` checks formatting, Clippy, Rust tests, dependency boundaries and source pins.
It verifies corpus hashes when checkouts are present; it never runs downloaded VBA.
WASM library compilation and identical native/WASI execution are separate gates.

## Optional external test material

```sh
git submodule update --init --depth 1
cargo xtask corpora
python3 tools/check_repository.py corpora --require-checkouts
```

Submodules are pinned by Git gitlinks and [the source lock](references/sources.lock.json).
Fast builds/tests work with no submodules checked out. Do not use `--remote` in reproducible builds.
See [corpus policy and inventory](tests/corpora/README.md) before selecting any fixture.
Two workbook sources have unresolved licensing; every specimen still requires classification.

## Layout

```text
apps/vba-cli/src/     standalone command-line frontend
crates/*/src/        compiler, values/runtime, VM, debugger, foreign protocol, importer
crates/*/tests/      owning crate's Rust integration tests as they are implemented
apps/vba-cli/tests/  executable CLI checks
examples/            self-authored pure VBA and foreign-call acceptance examples
tests/               shared scenario/fixture data by test tier
tests/corpora/       six external test-data submodules and hashed candidate inventory
references/          official VBA-Docs submodule, source lock and reference notices
docs/                architecture, plan, decisions, progress and original design inputs
.agents/skills/      shared Codex/OpenCode development and design skills
fuzz/                reserved standalone fuzz workspace (not implemented)
reports/             ignored generated output
xtask/ and tools/    quality commands and offline repository checks
```

There is deliberately no root `src/`: Cargo builds `src/` inside each focused crate.
See [architecture](docs/architecture.md) for crate responsibilities and dependencies,
[bytecode/VM specification](docs/bytecode-vm-specification.md) for the adopted IR 1 implementation contract,
[the implementation plan](docs/implementation-plan.md) for gated work,
and [decisions](docs/decisions/0001-repository-scope.md) for choices and open questions.

## Project documents

[Documentation index](docs/README.md) · [Compatibility matrix](docs/compatibility-matrix.md) ·
[Development guide](docs/developer-guide.md) · [Agent skills](docs/agent-skills.md) ·
[Progress](docs/implementation-progress.md)

The supplied design documents remain preserved. Current user direction and the repository
scope decision take precedence over their older proposals for a shared automation repository
or an Office bridge inside the interpreter workspace.

## Licence

[MIT](LICENSE), preserving the existing repository licence. The design proposes MIT OR Apache-2.0;
that change remains a maintainer decision. External submodules retain their own licences and notices.
