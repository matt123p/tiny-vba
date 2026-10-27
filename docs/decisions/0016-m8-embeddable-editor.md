# 0016 — Embeddable Classic editor/debugger and executable UI harness

Adopted and implemented, 2026-10-27. Implements ADR 0002 and the repository M8 gate;
[the implementation plan](../ui/m8-implementation-plan.md) was written before code.
The user-supplied [public-source specification](../ui/craft_vba_editor_debugger_ui_spec.md)
is the functional/visual direction. Its full C0/C1 and Office integration programme is larger than
M8; [acceptance and feature status](../ui/m8-acceptance.md) distinguish the delivered surfaces.

## Ownership and execution

`vba-ide` owns buffers/source identities/revisions, multi-module compilation, statement breakpoints,
command metadata, source maps, debugger session state and bounded update actions. It has no toolkit,
filesystem, network or document-model dependency. `vba-ui-egui` renders those views inside a
caller-owned `egui::Ui` and returns explicit commands. `vba-ide-app` owns the independent native/web
shell and source IO. `vba-cli debug` uses the same model. No runtime/compiler edge points upward.
The reviewed graph and provenance checks enforce these boundaries.

An embedding supplies generic source imports, native object roots and optional public API metadata;
it retains ownership of host lifetime, document mutations, trust, storage, undo and events. Calls
borrow the host only for dispatch/tick/cancel; no host/document borrow is stored across a pause.
Each tick executes at most 256 VM instructions or polls one host continuation. The host itself
must return a deferred wait rather than block on slow IO. UI Reset cancels and destroys execution,
returning to Design without undoing successful host effects. It is distinct from M7's explicit
`Session::reset_state` restart API. Editing compiled source marks it stale; continue and inspection
are refused until explicit reset/recompile. Source edits clear affected breakpoint locations with
feedback rather than moving them silently. Multiple statements on one line retain distinct spans.

Locals/stack refer to real pause handles. Older frames can be inspected but only the active frame
is editable. Safe watch/Immediate evaluation parses a bounded expression with the existing parser,
looks up only paused-frame names and uses the shared runtime's coercions/operators. Calls, property
gets, indexing and arbitrary procedure execution are refused. Immediate permits `? expression`
and validated active-local assignment. Watch failures remain explicit, with no host dispatch.

## UI stack and assets

Use egui/eframe/egui_kittest 0.37.0 at the exact reviewed public fork revision
`6d518fc799681571963842f95b55a6b68431b15a` and winit
`a517b3846ee8457d84ee4975110bfda6797c44dc`, matching the inspected Gridcraft manifest.
Both pins are necessary: the GUI fork's native clipboard/event interface extends upstream winit.
The standalone workspace owns patches; consuming workspaces own theirs. No sibling source is
copied and no sibling checkout is required to compile or test this project.

Themes are plain injected tokens. Rendering scopes visuals to its local Ui, preserves the caller's
global context/fonts/scaling, and caches source highlighting using the existing lexer. Eframe owns
window persistence; the shell persists font size and pane visibility without storing project source
in preferences. Code buffers stay separate from document/Office binary storage. Native IO preserves
UTF-8 source bytes/line endings; browser source transfer uses explicit copy/paste without ambient IO.

Layout, markers and text indicators are original. Licensed egui default fonts are upstream package
inputs with [redistributable notices](../../licenses/fonts/); no proprietary VBE artwork, font,
resource dump or clone implementation was used. [Provenance](../provenance/vbe_sources.yaml) records
public URLs/date/hash/observations; downloaded HTML remains only in ignored `.cache/evidence/`.
[Dependency records](../provenance/ui-dependencies.json) record reviewed pins/licences/notices.
These records describe the implementation process and inputs; they are not a claim of legal advice
or verified pixel identity with Microsoft products.

## Acceptance

`cargo xtask ui` runs model, interaction, embedding and shell tests and renders original light/dark,
narrow, scaled, injected-theme and focused-pane PNGs from real paused execution. Accessibility checks
remain enabled. `cargo xtask ui-native` runs/captures the actual standalone native app;
`cargo xtask web` builds the shared Rust/WASM UI, with a separate Chromium keyboard smoke harness.
Image outputs are ignored local reports. Geometry, semantic interaction and contrast assertions are
committed; no proprietary screenshot baseline or downloaded reference artifact enters Git.
The ordinary CI gate also compiles/lints every UI target, while WASI executes the headless IDE tests
alongside the earlier language/import/callback suites. No Office/workbook or physical-device parity
claim follows from these standalone tests.

## Subsequent milestone allocation

On explicit user direction, the remaining UI specification work is assigned to
[planned M10–M14 milestones](../ui/post-m8-roadmap.md): classic completion, forms/class runtime,
native extensions/non-Microsoft enhancements and translation, optional AI, and fork integration
acceptance respectively. This scheduling amendment preserves the M8 baseline and layer/ownership
contracts above; planned scope is not an implemented capability.

## M8 correction request — 2026-10-27

The user explicitly includes working IntelliSense and syntax-error feedback in M8. The toolkit-free
model caches parser diagnostics and scoped declarations with each source revision; it resolves
simple receiver types only against supplied supported metadata. The renderer owns the anchored
completion list and keyboard/pointer focus. Parser messages/red underlines update without Compile,
without navigating away from the typing caret; compiler/runtime diagnostics remain explicit actions.
Basic project signature help is source metadata, never an evaluator/host call. Advanced class/library
resolution, constants and richer Quick Info remain M10. Specification section 16 defines examples.

The editor uses an LF display projection with checked byte mappings and a bounded source diff to
retain untouched mixed CR/CRLF/LF bytes. Compiler/debugger coordinates always refer to original source.
Compatible terminal invocations transfer VM ProjectState within the same host identity; explicit
Reset/source revision/host replacement prevents reuse. No GUI or new dependency is added below
vba-ide. The standalone browser exposes source/feedback mirrors only for its accessible test harness;
embedded hosts still consume Rust model state/commands directly.

The follow-up ordinary-editor audit verifies keyboard and menu Undo/Redo, selection-aware clipboard
commands and deletion. Native menu Paste uses the shell's viewport clipboard request; browser Paste
uses browser keyboard/user-gesture policy with an explicit menu explanation. Renderer-local selection
and a menu-opening undo snapshot preserve editor history through temporary menu/IME focus changes.
This does not add IO to the model or claim automatic/whole-block indentation (M10).
