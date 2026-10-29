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

M10's implemented classic tools, advanced debugging and optional DAP are defined by
[ADR 0018](../../../docs/decisions/0018-m10-classic-tools-and-debugger-policy.md) and the
[acceptance map](../../../docs/ui/m10-implementation.md). Preserve shared bounded source history,
literal scoped search, view selection/focus and explicit browser transfer. Set Next Statement
supports verified straight-line checkpoints only; watch evaluation stays read-only; Immediate
calls require explicit isolated-evaluation policy. Metadata does not imply library/type binding.
The [remaining plan](../../../docs/implementation-plan.md) now uses new M1–M8
numbering. Milestone IDs in the implemented contracts above are historical; see
the [mapping](../../../docs/implementation-progress.md#plan-renumbering--2026-10-28).
New M1 owns host type binding, M2 shared classes/standard Office forms, M5 Excel
events/UDFs, and M6/M7 remaining host/package acceptance. Third-party ActiveX/OLE
controls not supplied with Office are out of scope. Native extensions/translation
and AI assistance are removed from planned scope; do not schedule them from the
earlier supplied specification. Preserve fork ownership and existing UI evidence.

The 2026-10-27 user correction includes working scoped word/member completion, source signature help
and live red/underlined parser diagnostics in historical M8 (UI specification section 16).
Declarative reference metadata/List Constants belong to the historical M10 implementation;
executable external type/reference binding is new M1. Preserve original CR/CRLF source
bytes through the editor view mapping and Static state across compatible same-host reruns until Reset.

M2's selected class/standard-form profile is implemented under ADR 0023/0024.
Read docs/m2-acceptance.md for current scope, workflow, exchange format and gates;
docs/m2-classes-and-forms.md retains historical incomplete checkpoints. Preserve
IR7 class/event identity, canonical field ByRef, static visibility/signature checks,
property value ByVal semantics, inert safe inspection and all three error modes.

vba-ide owns the bounded shared design, authored source, .tvform codec and form
orchestration; vba-ui-egui renders the same designer/runtime in native/browser
hosts. Generated native control classes use ordinary Event/WithEvents frames.
Explicit suspended callbacks park a modal caller; existing callbacks remain
terminal-only. Retain parked locations, shared budgets, input ownership and host
tokens through pause/resume. Cancel every token on Reset/close; do not execute
user cleanup during abnormal shutdown. Design/history must survive VM Reset.

Form import is inert and validates before project mutation. Save/export preserve
exact authored source and design; revision acknowledgments must reject intervening
source/design edits. Unknown formats, controls and binary resources fail explicitly.
The selected Label/TextBox/CommandButton profile does not authorize COM/DLL or
third-party controls, binary Office form/resource compatibility or package writeback.
Office document bindings/events, storage, trust and package persistence stay in forks.
Run CI/WASM/WASI plus UI/native/web and both Chromium smoke/form acceptance gates
when changing these surfaces. M3 range/worksheet work is GridCraft-owned.
