# M8 implementation and verification plan

Started 2026-10-27. Input: the user-supplied
[public-source UI specification](craft_vba_editor_debugger_ui_spec.md), the
[repository UI contract](../editor-debugger-ui.md), and ADR 0002.

## Deliverable and scope

Implement one host-neutral VBA editor/debugger, usable independently and embedded in an existing
egui application. Preserve the classic menu names, project/properties/code arrangement, procedure
selector, breakpoint margin, debugger controls and inspection windows. Connect every enabled
execution action to the actual compiler, VM and debugger. Keep unsupported specification features
visible as disabled commands with explanations and record them in an acceptance matrix. M8 is
this repository's editor/debugger gate; the new document additionally describes host integration,
UserForms and later extensions, which require separate acceptance evidence.

## Ordered work

1. **Plan and provenance.** Inspect current contracts and sibling dependency manifests without
   copying sibling implementation. Record public documentation URLs, retrieval dates and factual
   observations. Cache downloaded evidence beneath ignored `.cache/evidence/`; never commit it.
   Use original drawn glyphs and egui's licensed default fonts. No VBE resources or clone source.
2. **IDE model (`vba-ide`).** Add toolkit-independent source buffers, stable IDs, revisions,
   multi-module compilation, procedure navigation, command metadata/enabled states, host requests,
   diagnostics, statement breakpoints, bounded execution and cleanup. Integrate real stepping,
   stack/local inspection, validated active-frame edits, safe watches and bounded output.
3. **Reusable rendering (`vba-ui-egui`).** Build the classic shell and panes above the model, with
   injected theme tokens, host-owned egui context/fonts/scaling and explicit shortcut routing.
   Add syntax highlighting, editor selection/undo, line numbers, execution/breakpoint markers,
   find/replace, project properties, object/symbol browsing and inspection interactions.
4. **Standalone native/web shell (`vba-ide-app`).** Provide the host-free authored demo and plain
   source open/save/import/export. Keep platform persistence outside the model and renderer.
   Include a CLI debugger using the same model. Supply a minimal browser launcher and documented
   embedding example; no Office application sibling is a build dependency.
5. **Executable UI harness.** Render actual panels with actual paused VM state; drive keyboard
   and pointer commands. Produce original light/dark, narrow, scaled, focus and injected-theme
   PNGs in ignored report output. Add semantic layout/accessibility and contrast assertions;
   inspect generated images and correct clipping/overlap. Run a native app smoke when a virtual
   display is available and browser interaction when browser tooling is available.
6. **Acceptance and documentation.** Run model/debugger/embedding/file/interaction regressions,
   strict Clippy/format/layer checks, native and WASM gates. Record exact commands, results and
   limitations; update architecture/DAG, dependencies/licences, compatibility, progress and skills.
   Commit implementation and user-supplied specification; exclude downloaded evidence and generated
   reports. M9 conformance/release remains next after this gate.

## Test matrix

- Edit/add/select/import/export/save modules; Unicode, CRLF, source limits and dirty revisions.
- Compile success/failure and exact diagnostic navigation; reject nonexecutable breakpoints;
  preserve multiple statements on one line and stable source identities.
- Run/continue/pause/reset/step into/over/out; nested calls, runtime errors, infinite-loop budgets,
  source changes while paused and project close/disconnect. Count host dispatches to prevent replay.
- Inspect stack frames and typed values; edit valid/invalid locals; safe expression watches refuse
  arbitrary calls/property gets and show stale/out-of-scope states without executing host code.
- Independently mount the panels in an existing context with a caller theme and host action handler.
- Keyboard F5/F8/F9/Shift+F8/Ctrl+Shift+F8, pane shortcuts and find/save; widget accessible names.
- Render snapshots in light/dark/host themes, narrow width and high DPI with a real paused session.
  Treat our PNGs as original output, not proof of pixel identity with a Microsoft reference build.
- Build/run native UI and compile/browser-run the shared web UI; separately verify headless engine
  dependencies do not acquire egui, eframe or platform IO.

## Evidence policy

Track URLs, hashes, observation summaries, dependency licences and test results. Do not check in
third-party HTML, screenshots, documentation copies, proprietary artwork or reference binaries.
No licensed Office installation has been supplied: do not claim live Excel/VBE observations,
workbook event parity, signature preservation, or exact current-release geometry. All implementation
and demo/test source is independently authored against public contracts.

## Progress

- [x] Repository contracts and new UI specification reviewed; plan written before implementation.
- [x] Provenance and dependency review.
- [x] IDE model and debugger integration.
- [x] Reusable editor/debugger rendering.
- [x] Standalone/native/web/CLI harnesses.
- [x] Tests, rendered review and acceptance results.

## Completion record

The implemented layers and reproducible commands are recorded in [ADR 0016](../decisions/0016-m8-embeddable-editor.md).
[Acceptance and compatibility status](m8-acceptance.md) separates the repository gate from the
broader C0/C1 programme and fork-owned workflows. [Progress](../implementation-progress.md) records
exact test counts and executed native/browser/WASM checks. Original PNGs were rendered and visually
inspected; downloaded references remain in the ignored cache, with only provenance summaries and
required licensed font notices tracked.

## Scope after M8

The [post-M8 roadmap](post-m8-roadmap.md) now owns the remaining work: M10 classic editor/debugger
completion, M11 forms/class runtime, M12 native extensions/non-Microsoft enhancements and translation,
M13 optional Copilot-style assistance, and M14 cross-repository host acceptance. These are planned
milestones with their own gates. The completed checkboxes above apply only to the delivered M8
baseline; they do not mark these later features complete. M9 conformance/release remains next.
