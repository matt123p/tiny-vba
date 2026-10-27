# Running and embedding the VBA editor

The same `vba-ide` model and `vba-ui-egui` renderer power the standalone native app, browser app and
render harness. Neither Gridcraft nor Wordcraft is a build dependency.

## Standalone commands

```sh
cargo run -p vba-ide-app
cargo run -p vba-ide-app -- --open examples/pure-vba/Arithmetic.bas
cargo run -p vba-ide-app -- --dark
cargo run -p vba-cli -- debug examples/pure-vba/Arithmetic.bas --entry Answer
```

The demo starts with `Main`. F9 toggles a statement breakpoint, F5 runs/continues, F8 steps into,
Shift+F8 steps over and Ctrl+Shift+F8 steps out. Ctrl+G focuses Immediate, Ctrl+L shows the stack,
Ctrl+R shows Project Explorer, F4 shows Properties, F2 shows the symbol browser, Ctrl+F opens find,
and Ctrl+S requests source save. UI Reset stops execution and discards runtime state; it retains
host side effects. Native import/export/save accepts bounded UTF-8 `.bas` source and preserves
line endings and attributes. Browser transfer currently uses explicit copy/paste source panels.
Closing modified source requires save or explicit discard. No binary Office project is rewritten.

The CLI also accepts `--script commands.txt` with commands `break <line>`, `run`, `continue`,
`step`, `next`, `out`, `locals`, `stack`, `print <expression>`, `set <name = expression>`,
`watch <expression>`, `reset`, and `quit`.

## Embed in a caller-owned Ui

Declare path dependencies on `vba-ide` and `vba-ui-egui`; align the complete egui family and winit
patches with [ADR 0016](../decisions/0016-m8-embeddable-editor.md). Cargo patches are controlled by
the consuming workspace root. There must be a single egui API family so contexts and Ui types agree.

```rust,ignore
use vba_ide::{Host, HostRequest, Ide};
use vba_ui_egui::{EditorUi, Theme, apply};

struct VbaEditor {
    ide: Ide,
    view: EditorUi,
    theme: Theme, // map the host's theme tokens once, not global styles each frame
}

impl VbaEditor {
    fn show(&mut self, ui: &mut egui::Ui, host: &mut impl Host)
        -> vba_ide::Result<Vec<HostRequest>>
    {
        for command in self.view.show(ui, &self.ide, self.theme) {
            apply(&mut self.ide, command, host)?;
        }
        self.ide.tick(host)?; // one bounded slice or one wait-token poll
        Ok(self.ide.take_requests())
    }
}
```

Mount this inside an existing panel, tab, window or child Ui. The renderer does not replace global
visuals, install fonts or change context scaling. Supply the host's fonts/scaling and map light/dark
colours to `Theme`. Stable `Action::id`, `label`, `shortcut` and `Ide::enabled` allow routing through
host menus/commands. The host decides when this Ui owns keyboard shortcuts; draw it only in its
active editor surface. Host toolbars can instead call `Ide::dispatch` directly.

Populate source with `Ide::add_module(name, text)` and retain the returned `SourceId` mapping to
the host's project/module identities. These are plain source buffers. Preserve the original Office
project parts and opaque unsupported components in the host; adding source never executes it.
Current execution accepts standard modules; classes, document events and UserForms need later
language/host contracts. Unsupported structures must not be silently converted or discarded.

Call `Ide::configure_host(imports, roots, catalog)` before launch. `Import` and native object roots
use the M5 contract; the host's own identity/generation model remains authoritative. Optional
`ApiMember` entries feed Object Browser and qualified completion; absent metadata yields no invented
Office APIs. Actual host calls are normal in-process `vba_foreign::Host` calls. A slow operation
must return `Reply::Wait` and resolve later, preserving responsiveness.

Handle `HostRequest::Save`, `Import`, `Export`, `ReturnToHost` and `ConfirmClose` through the existing
application pipeline. A save request carries a source ID and revision; call `mark_saved(id, revision)`
only after successful persistence. A later edit causes an obsolete save acknowledgement to be
rejected. Host save does not imply tiny-vba binary project writeback. On project close, finish any
save/discard decision then call `Ide::close(host, discard_modified)` to cancel pending work and
release execution. No document borrow survives a pause, inspection or render call.

[Generic embedding tests](../../crates/vba-ide/tests/embedding.rs) demonstrate a real native host
write, break/edit/resume without replay, exactly-once wait completion and cancellation without
rollback. They use generic objects, not a second Office object model.

## Reproduce UI evidence

```sh
cargo xtask ui                    # headless workflows, interactions, original rendered PNGs
cargo xtask ui-native             # real native window, capture then exit; requires a display
xvfb-run -a cargo xtask ui-native  # Linux CI/virtual display
cargo xtask web                   # dist/ui/; requires matching wasm-bindgen CLI
```

For web setup, install `wasm32-unknown-unknown` and `wasm-bindgen-cli` version `0.2.126`. Serve
`dist/ui` over HTTP. The optional browser smoke harness requires Node 20+:

```sh
npm ci --prefix tools/ui-browser
cd tools/ui-browser
npx playwright install --with-deps chromium
npm test
```

It starts its own loopback server, drives real F8/F5 debugger interaction, asserts output and writes
browser PNGs. `VBA_PLAYWRIGHT_MODULE` can point to an external cached Playwright installation.
`VBA_UI_REPORT_DIR` chooses the generated PNG directory; the default is ignored `reports/ui/`.
The committed tests assert commands, geometry, accessibility and contrast; PNGs are reproducible
inspection output, not a checked-in Microsoft or cross-platform pixel baseline.

## M8 code-assistance corrections

Completion now works from design-time source declarations and supplied `ApiMember` entries, including
simple typed receivers. Typing a word/period or Ctrl+Space/Ctrl+J opens a caret-anchored list; arrows,
Enter/Tab, Escape and pointer selection work in the source editor. Basic project signature help is
nonexecuting metadata. Live parser failures are separate `Module::syntax_diagnostics`; they render
red/underlined affected lines and clear on correction without Compile or diagnostic navigation.
Use the model's checked editor/source byte projection when implementing another frontend; original
source bytes/line endings remain authoritative. Compatible same-host reruns retain Static values;
Reset clears the invocation/project state. Host identity changes prevent state transfer.

`cargo xtask ui` also writes `light-completion`, `dark-completion`, `light-syntax-error` and
`dark-syntax-error` PNGs. Chromium uses real keyboard source replacement, member completion and
execution (`8`), then syntax error/correction, and captures `browser-completion`/`browser-syntax-error`.
The standalone page's hidden source mirror and live feedback status are harness observations; they
do not introduce web APIs into the embeddable model/panels. All images stay ignored under `reports/ui/`.
See specification section 14 and the provenance manifest for public reference examples/cache hashes.

## Ordinary editor commands

| Operation | M8 support and verification |
|---|---|
| Undo / Redo | Source shortcuts Ctrl+Z / Ctrl+Y or Ctrl+Shift+Z (Command equivalents on macOS); functional Edit menu entries; keyboard and menu regression tests. History uses the toolkit's bounded, grouped undo behavior. |
| Cut / Copy / Paste | Selection-aware keyboard clipboard operations; Copy/Cut produce real clipboard output; multiline Paste updates source through the same revision/size checks. Native Edit-menu Paste requests the platform clipboard. Chromium verifies the real clipboard round trip. |
| Browser Paste menu | Disabled with a shortcut explanation: use Ctrl+V / Cmd+V or the browser paste command. Browser clipboard permission/user-gesture policy belongs to the browser shell. |
| Selection/navigation/deletion | Pointer selection, Shift selection, Select All, arrows/Home/End and word/document navigation, Backspace/Delete and selection replacement use the source widget; regression tests cover Select All, Shift+End, deletion and Unicode source. |
| New lines / indentation | Enter inserts new lines; Tab inserts indentation at the caret and Shift+Tab removes caret indentation. Untouched LF/CRLF/CR bytes remain intact. This does not implement automatic indentation or whole-block indent/outdent. |
| Find / Replace | Literal, case-sensitive current-module search; Find Next advances then wraps; bounded Replace All. |
| Remaining editing workflows | Whole-block indentation/comment commands, automatic indentation/case correction, bookmarks, richer find options, split/procedure views and docking/customisation remain M10. M8 does not claim every conventional editor feature. |

Selection and undo/redo survive temporary Edit-menu focus. Escape cancels completion while keeping
source focus; accepting completion and immediately typing within the same rendered frame preserves
the new text. These interaction details have permanent regressions rather than relying on the
presence of toolkit keyboard bindings alone.
