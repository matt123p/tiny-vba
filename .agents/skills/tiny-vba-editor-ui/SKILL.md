---
name: tiny-vba-editor-ui
description: Build or review the tiny-vba VBA editor and graphical debugger, including egui panels, standalone shell, Gridcraft embedding, design tokens, accessibility and interaction. Use for visual or IDE state/action changes.
---

# Editor and debugger UI design

Read [UI contract](../../../docs/editor-debugger-ui.md),
[UI ownership ADR](../../../docs/decisions/0002-editor-debugger-ui.md) and the implemented UI layers in
[architecture](../../../docs/architecture.md). This repository owns both reusable panels and a
standalone native/web IDE; do not delegate the editor/debugger implementation to application forks.

## Match the host while remaining standalone

Use Rust and the Gridcraft-compatible egui/eframe family. Before adopting/updating UI dependencies,
inspect Gridcraft's current manifests if available; align reviewed revisions and account for Cargo
patches being controlled by each consuming workspace root. Record unavailable integration checks
rather than adding a mandatory sibling build dependency.

Separate toolkit-independent editor/session state and actions from panel rendering and the app shell.
Embed using the caller's egui context, theme, fonts, scaling and action routing. Do not replace global
visuals or install fonts every frame. Inject theme tokens following Gridcraft's theme::Tokens and
syntax-colour conventions; provide original standalone defaults without importing Gridcraft's UI crate.
Match spacing, typography, widget and focus conventions through the host interface.

Route controls through stable action IDs, labels, shortcuts and enabled states. Panels do not mutate
the VM or workbook directly. Keep platform file IO/pickers and prompts behind shell/host interfaces;
headless/programmatic actions never open dialogs. Persist plain source without implying Office binary
project writeback. Use bounded execution/output updates so macros cannot freeze the editor.

## Deliver and inspect the actual experience

Use the contract's required-surface table: module/procedure browser, editor with line numbers and VBA
syntax, breakpoint gutter, execution highlight, run/step controls, locals, watches, call stack and output.
Preserve source versions and pause generations. Use the debugger skill for inspection and stepping.

Give controls accessible labels and keyboard access. Convey stopped/error/breakpoint state with glyphs
as well as colour. Verify text contrast at 4.5:1 and functional glyphs at 3:1 in light and dark themes.
Use original/licensed attributed assets; craft-fonts is optional, with system/egui defaults available.

Render and inspect snapshots after visual changes: light/dark, narrow width, scaling and focus. Check
an injected host theme as well as standalone defaults. Exercise keyboard actions and inspect accessible
labels. Test the real edit/run/break/step/inspect/resume workflow when implemented, native and browser
behaviour, and headless builds without GUI dependencies. Report visual evidence and any untested host
embedding; screenshots of placeholder state are not debugger execution evidence.

## Implemented M8 editor/debugger

Read [ADR 0016](../../../docs/decisions/0016-m8-embeddable-editor.md), the
[embedding/harness guide](../../../docs/ui/embedding.md) and
[acceptance matrix](../../../docs/ui/m8-acceptance.md). vba-ide is toolkit/IO-free; vba-ui-egui
mounts in caller Ui with injected tokens, independent per-view IDs and explicit commands;
vba-ide-app owns native/web IO and preferences. Run cargo xtask ui, ui-native and web plus ordinary
CI/WASM gates for affected UI changes. Keep accessibility checks enabled; render/inspect original
light/dark/narrow/scaled/host-theme/focus output. No downloaded reference evidence enters Git:
.cache/evidence/ is ignored and provenance/notices/pins are verified. Do not claim full C0/C1,
Office event/workbook, Set Next Statement or unsafe evaluation support from standalone UI evidence.
