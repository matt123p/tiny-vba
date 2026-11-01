# Tiny VBA

Tiny VBA is the shared VBA implementation for the ArtCraft Office applications:
GridCraft, WordCraft and DeckCraft. Written in Rust for native and WebAssembly use,
it provides the language engine, runtime, editor and debugger for embedding in those
applications.

The standalone graphical shell and command-line tools exist only to develop and test
the shared components without launching a host application. A standalone VBA editor
is not the project's goal.

**Current status:** the shared engine, editor/debugger and read-only project importer
are implemented, with verified IR 1–7 and the `m5-host-v1` runtime profile.
The current M6 work adds host-event subscriptions, VBA attribute metadata and a
lossless code view for embedding in GridCraft and WordCraft. See the
[M6 acceptance record](docs/m6-acceptance.md) for tested behavior and remaining gates.
Fresh Microsoft Office reference observations remain NOT_RUN; M7 has not started.

All **44 authored scenarios and 153 reviewed ReflexGrid cases pass**. The latter match published
Microsoft Excel Mac oracle expectations; this is inherited evidence, not a new local Excel run.
See [the acceptance report](docs/m4-oracle-report.md) and [language scope](docs/m4-language.md).
[Read-only project import](docs/project-import.md) extracts bounded source/metadata without execution.
The [embeddable editor/debugger and standalone test tools](docs/ui/embedding.md) are implemented, with
[corrected M8 completion audit](docs/ui/m8-completion-audit.md) and
[classic tools/DAP acceptance](docs/ui/m10-implementation.md).
The [remaining plan](docs/implementation-plan.md) uses new M1–M8 numbering for
host/language compatibility, standard Office forms, document integration and release.
Historical reports retain their original milestone IDs; see the
[numbering map](docs/implementation-progress.md#plan-renumbering--2026-10-28).


## Shared components and application integration

This repository owns language semantics, bytecode, execution, debugging, the reusable egui
editor/debugger UI, a generic foreign-call contract and read-only VBA project decoding.
The ArtCraft applications own their Office compatibility objects, member catalogues,
document state, host adapters, storage, trust, events, undo/recalculation and application UI.
The Excel and Word compatibility layers live in GridCraft and WordCraft respectively.

Applications embed the shared editor/debugger in their existing UI stack and supply their
own host integration. The standalone shell exercises the same shared components for testing.
See the [editor/debugger UI contract](docs/editor-debugger-ui.md) and
[embedding guide](docs/ui/embedding.md).

## Development and standalone testing

The commands below exercise the shared engine and UI in isolation. Microsoft Office and
the ArtCraft host applications are not required for these tests.

Rust is pinned by `rust-toolchain.toml`; Python 3.11+ and Git support repository checks only.
Project decoding uses pinned cfb and encoding_rs dependencies; Office is not required. Node 18+ runs the WASI smoke test.

```sh
cargo run -p vba-ide-app
cargo run -p vba-cli -- debug examples/pure-vba/Arithmetic.bas --entry Answer
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
git submodule update --init --checkout --depth 1
cargo xtask corpora
python3 tools/check_repository.py corpora --require-checkouts
```

Submodules are pinned by Git gitlinks and [the source lock](references/sources.lock.json).
Fast builds/tests work with no submodules checked out. Do not use `--remote` in reproducible builds.
Corpus/reference submodules use `update = none` so Cargo Git consumers do not
download unrelated test data. `--checkout` explicitly opts in when verifying corpora.
See [corpus policy and inventory](tests/corpora/README.md) before selecting any fixture.
Two workbook sources have unresolved licensing; every specimen still requires classification.

## Layout

```text
apps/vba-ide-app/     standalone native/web shell for development and testing
apps/vba-cli/src/     command-line development and test frontend
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

## UI, embedding and provenance

The [M8 plan](docs/ui/m8-implementation-plan.md), [embedding guide](docs/ui/embedding.md) and
[acceptance/compatibility report](docs/ui/m8-acceptance.md) describe the shared Classic editor/debugger and its embedding interfaces.
`cargo xtask ui` generates original screenshots in ignored `reports/ui/`; `ui-native` runs a real
window/capture and `web` builds the browser bundle. The optional Chromium harness is in
`tools/ui-browser`. Core and IDE model consumers remain free of GUI dependencies.

UI implementation follows public functional documentation with independently authored code/layout
and original markers. [Source provenance](docs/provenance/vbe_sources.yaml) references ignored
`.cache/evidence/` downloads; no reference HTML, screenshots, VBE artwork or clone code is checked
in. [Dependency/font notices](docs/provenance/ui-dependencies.json) are tracked separately.
