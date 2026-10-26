# Editor and debugger UI contract

Status: required and planned, 2026-10-26. No UI implementation exists yet.
Ownership: [ADR 0002](decisions/0002-editor-debugger-ui.md).

## Product and ownership

tiny-vba delivers a reusable VBA editor/debugger UI and a standalone IDE. A developer must be able
to edit and debug a host-free VBA module here without launching Gridcraft or installing Office.
Gridcraft embeds the same components for workbook macros and supplies its Office/document bridge.
The standalone IDE initially runs pure language programs and deterministic generic-host examples;
it does not promise Excel objects without an application adapter.

## Gridcraft compatibility and visual rules

- Use Rust, egui and eframe; align the full egui family and reviewed fork revisions with Gridcraft.
  Avoid distinct egui versions whose context/widget types cannot be shared by the embedding host.
  The standalone workspace owns its Cargo patches; embedding applications own theirs.
- Offer embeddable panels independently of the eframe shell. Use the host's egui context, style,
  theme tokens, fonts, icons and scaling. Do not reset global visuals or install fonts every frame.
- Follow Gridcraft's theme::Tokens conventions for light/dark colours and syntax highlighting.
  An injected theme mapping lets Gridcraft pass its actual token values without a dependency here
  on Gridcraft's UI crate. Provide original standalone defaults and use tokens rather than ad hoc colours.
- Match host spacing, typography, focus behaviour, keyboard navigation and widget conventions.
  Actions have stable IDs, labels, enabled states and shortcuts; panels dispatch actions rather than
  mutate the VM or workbook directly. Headless/programmatic actions never open dialogs.
- Give controls accessible labels. Distinguish breakpoints, paused execution and errors with glyphs
  as well as colour. Check text contrast at 4.5:1 and functional glyph contrast at 3:1 in both themes.
- Use original or appropriately licensed assets with attribution. Follow Gridcraft's font policy:
  craft-fonts is an optional external input; system/egui defaults work without it. Never ship vendor
  artwork or fonts. Follow conventions without copying application source.
- Preserve native and browser support. Platform file pickers, source persistence and host prompts
  belong behind shell/host interfaces; core editor/session logic has no ambient filesystem access.

## Required editor and debugger surfaces

| Surface | Required behaviour |
|---|---|
| Project/module browser | Select modules and procedures; show modified-source state; preserve source identities |
| Source editor | VBA syntax colours, line numbers, indentation, selection, undo/redo, find, diagnostics with source spans |
| Breakpoint gutter | Toggle and display source-statement breakpoints; distinguish unresolved breakpoints and multiple statements per line |
| Execution view | Highlight the current statement and navigate from diagnostics and stack frames to its source |
| Debug controls | Run, pause, continue, stop, step into/over/out; show session state and disable invalid actions |
| Locals and stack | Inspect typed values and frames; validated supported local edits while paused |
| Watches | Inspect supported expressions with explicit unsupported/side-effect status; no implicit arbitrary function execution |
| Output | Display Debug.Print, runtime errors and diagnostics without freezing the UI |

Opening/editing/saving plain source belongs here. Original Office project bytes, .xlsm/.docm
packaging and binary VBA project writeback remain outside this initial UI contract. Editing paused
source must mark compiled code stale; require an explicit stop/recompile/restart before running
changed code. Do not imply live code replacement. Immediate Window execution is a later extension
with explicit evaluation policy; read-only watches are part of the initial debugger UI gate.

## State and host interfaces

vba-ide owns stable source/module identities, editor buffers, diagnostics, debug action routing and
session state. vba-ui-egui presents that state; vba-debug remains the toolkit-independent execution
controller. UI updates consume bounded execution slices and output batches so an infinite VBA loop
cannot block repaint, pause or cancellation. Source maps, breakpoint IDs and watch results refer to
the compiled source version and pause/session generation; stale inspection handles are rejected.

A host integration supplies project attachment, native foreign calls, document lifecycle and trust
prompts through explicit interfaces. The embedding host controls window placement, shortcuts and
prompt presentation. Closing a project, stopping execution or disconnecting releases host references
and resources according to the host policy. Source import never auto-runs macros. Debugger
expressions obey the execution capability policy. No mutable host borrow survives a pause.

## Delivery and acceptance gates

1. After the M3 executable debugger proof, add IDE state/actions, embeddable panels and a standalone
   shell with a self-authored pure-language program. Show real source, breakpoints and stepping;
   unavailable operations must be disabled or return an explicit unsupported diagnostic.
2. Complete M8 with all required surfaces above. Edit/run a module, stop at a breakpoint, step through
   nested calls, inspect the stack, edit a supported local and resume without replaying mutations.
   Verify compile errors, runtime errors, cancellation, source changes and project close.
3. Render and inspect native snapshots for light/dark themes, narrow layouts, scaling and focus.
   Exercise keyboard controls, accessible labels, token contrast and a caller-supplied host theme.
   Add checks for attributed assets/fonts when those inputs are introduced.
4. Verify standalone native and browser builds, browser debugger interaction, and separate headless
   builds that do not pull egui/eframe. Keep UI tests independent of corpus downloads and Office.
5. In Gridcraft, embed these panels and run a real workbook-editing macro through pause/inspect/resume.
   Verify theme/shortcut integration, recalc, undo policy and cleanup without duplicate writes.

Steps 1–4 belong to tiny-vba; step 5 belongs to the Gridcraft integration. A CLI or DAP demonstration
alone does not satisfy the visual UI acceptance gate. Current packages and tools/architecture.json
remain the scaffold until implementation lands; planned package names are recorded in architecture.md.
