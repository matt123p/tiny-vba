# tiny-vba

**A VBA engine, editor and debugger written in Rust, for native and WebAssembly applications.**

tiny-vba lets applications embed a familiar VBA development environment: write a
macro, set a breakpoint, inspect typed values and step through execution. It is an
independent, clean-room implementation built for the ArtCraft applications,
including GridCraft and WordCraft.

![The VBA editor paused at a breakpoint with typed locals](docs/images/debugger.png)

## Try it

With the Rust toolchain in [rust-toolchain.toml](rust-toolchain.toml) installed,
run these commands from this checkout:

```sh
# Open the standalone editor and debugger.
cargo run -p vba-ide-app

# Run a small VBA function that returns 42.
cargo run -p vba-cli -- run examples/pure-vba/Arithmetic.bas --entry Answer

# Open a native UserForm example.
cargo run -p vba-ide-app -- --open examples/forms/CustomerForm.frm
```

In the editor, **F5** runs or continues, **F9** toggles a breakpoint and **F8**
steps into the next statement. The default demo starts at `Main`. The standalone
application is a development host; workbook and document macros need an
application's object adapter.

See [examples](examples/README.md) for source samples, or run
`cargo run -p vba-cli -- --help` for parsing, compilation and debugging commands.

## What it provides

- **VBA language and runtime:** typed values, checked coercion, `ByRef`, arrays
  with lower bounds, error handlers, classes, properties, events and intrinsic
  functions within the [documented compatibility profile](docs/compatibility-matrix.md).
- **An editor:** project navigation, syntax colouring, diagnostics, completion,
  signature help, Find/Replace and an Object Browser.
- **A debugger:** breakpoints, stepping, Locals, Watches, Call Stack, Immediate
  and an optional Debug Adapter Protocol frontend.
- **Native forms:** `.frm`/`.frx` import, a shared designer, modal/modeless runtime
  interaction and explicit native save/export workflows.
- **Embedding:** a toolkit-independent IDE model and reusable egui panels, with
  native Rust calls to the host's live objects.

![Member completion in the light-theme editor](docs/images/completion.png)

The engine preserves distinctions such as `Empty`, `Null`, `Missing`, `Nothing`,
Currency and Date. The host application owns its Office-compatible objects,
document state, trust, undo and recalculation. This keeps macros connected to the
actual document rather than a separate model.

## Compatibility

High Excel/Word VBA compatibility is the goal. **Universal Office compatibility
is not a claim about this implementation.** Unsupported operations fail explicitly.
The compiler/runtime and native forms support finite, tested profiles; arbitrary
COM/DLL activation, P-code execution and third-party ActiveX controls are excluded.

Local tests and inherited reference results are available. Fresh Excel/Word
comparison remains **NOT_RUN**, so the release comparison gate is still open.
See the [compatibility matrix](docs/compatibility-matrix.md) for supported behavior
and [testing and evidence](docs/testing.md) for what those results establish.

## Use it in an application

The maintainer's forks embed tiny-vba through opt-in features. These integrations
are on feature branches and are not part of official upstream builds:

| Application | Integration branch | Run from the fork checkout |
|---|---|---|
| [GridCraft](https://github.com/matt123p/gridcraft/tree/feat/tiny-vba-integration) | `feat/tiny-vba-integration` | `cargo run -p gridcraft --features vba` |
| [WordCraft](https://github.com/matt123p/wordcraft/tree/feat/vba-host-integration) | `feat/vba-host-integration` | `cargo run -p wordcraft --features vba` |

Each fork pins its own tiny-vba revision, which determines its available features.
DeckCraft is an intended host; no integration is claimed here.

For application authors, start with [host integration](docs/host-integration.md)
and [embedding the editor](docs/ui/embedding.md).

## Develop

Product code is Rust. Repository checks use Python 3.11+; WASI checks use Node.
Core checks work without Microsoft Office, application siblings or corpus downloads.

```sh
cargo xtask ci
rustup target add wasm32-unknown-unknown wasm32-wasip1
cargo xtask wasm
cargo xtask wasm-run
```

The [developer guide](docs/developer-guide.md) covers prerequisites, focused checks
and UI/browser validation. Read the [architecture](docs/architecture.md) before
changing crate boundaries. The [documentation index](docs/README.md) links the
maintained contracts and architectural decisions.

## License and credits

tiny-vba is independently authored using public specifications, documentation and
permitted black-box observations. It includes no Microsoft implementation code,
proprietary Office binaries or vendor artwork.

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
Third-party materials retain their own notices; see [source provenance](references/sources.lock.json)
and [UI dependency/font notices](docs/provenance/ui-dependencies.json).
The screenshots are [original renders of tiny-vba](docs/images/README.md).
VBA, Excel, Word and PowerPoint are Microsoft trademarks. This project is not
affiliated with or endorsed by Microsoft.
