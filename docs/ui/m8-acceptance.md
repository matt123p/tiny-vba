# M8 acceptance and compatibility status

**Audit status: M8 complete for the defined standalone/embeddable scope; product corrections and local/remote acceptance pass.** See
[the completion audit](m8-completion-audit.md) for historical failures and their permanent regressions.

Date: 2026-10-27. Scope: the repository's reusable editor/debugger gate under ADRs 0002/0016.
The [new UI specification](craft_vba_editor_debugger_ui_spec.md) supplies the public-source Classic
layout/command direction and also describes a broader product roadmap. This implementation does
not assert completion of every C0/C1 scenario in that roadmap or equivalence with a licensed Office
reference build. No such reference build was supplied or observed.

## Delivered surfaces

| Surface | Implemented behaviour / evidence |
|---|---|
| Classic shell | Eleven named menus, compact run/step toolbar, Project Explorer and Properties on the left, object/procedure selectors above code, bottom debugger panes, status and window/module switching |
| Source/project | Multiple stable-ID standard modules; insertion, selection, procedure navigation; native bounded UTF-8 source import/export/save; browser copy/paste source transfer; dirty state and obsolete-save refusal |
| Editing | Monospace editor, lexer-based VBA colours, line numbers, scrolling, selection, keyboard editing/undo/redo, functional Edit-menu clipboard/selection/delete actions, caret indentation, advancing/wrapping literal find/replace with allocation bounds, live red/underlined parser feedback and explicit compile/runtime diagnostic navigation |
| Breakpoints | Actual verified visible statement spans; F9/gutter actions; rejection of blank/comment/declaration lines; distinct colon-separated statements; explicit invalidation after edits |
| Execution | Real run, pause, continue, UI Reset and into/over/out stepping; bounded ticks; Static state retained across compatible reruns until Reset; stale-source refusal; run-to-cursor in break mode and Show Next Statement |
| Inspection | Real frame stack and selected-frame locals; validated active-frame edits; typed values and bounded array previews; old-frame editing refused |
| Watches/Immediate | Safe scalar expressions using parser/runtime rules; out-of-scope/error states; explicit refusal of calls/getters/indexing; `? expression` and active-local assignment |
| Symbols | Design-time scoped declarations/parameters/procedures; caret-anchored automatic and Ctrl+Space/Ctrl+J completion, arrows/Enter/Tab/Escape/pointer handling; parsed project signature help; typed receiver members use supplied metadata |
| Embedding | Caller-owned Ui/theme/fonts/scaling; explicit commands and source/host requests; no toolkit/IO in IDE model; generic-host tests count dispatch/resume/cancel effects |
| Shell/lifecycle | Host-free authored demo; CLI debugger; native and browser shells; dirty-close confirmation; native cleanup and font/pane preference persistence |
| Visual/accessibility | Named widgets with accessibility checking enabled; original markers plus colour; contrast assertions; actual paused-state light/dark/narrow/1.5x/host-theme/focused-pane snapshots |

## Work assigned to later milestones

Remaining work is assigned to the [M10–M14 roadmap](post-m8-roadmap.md), with explicit owners,
dependencies and acceptance gates. These milestones are planned; current unsupported states remain.

| Specification area | Planned milestone / current boundary |
|---|---|
| Split/procedure views, docking/toolbars, rename/remove, full classic code aids and options | **M10** — classic editor completion; current Full Module View and scoped keyboard/pointer completion and live syntax feedback remain delivered M8 scope. |
| Set Next Statement, conditional/value-change watches, expanded evaluation policies and DAP | **M10** — verified debugger contracts required before enabling commands; default safe evaluation remains. |
| Class/document execution, class trapping and UserForms/designer/Toolbox/Format | **M11** — real class/event runtime is a prerequisite for running forms. Unsupported controls need safe preservation/loss handling; no implied arbitrary ActiveX support. |
| Native Craft add-ins, non-Microsoft enhanced-editor features and Python translation preview | **M12** — opt-in extension API/manager with capability and cleanup tests; binary VBIDE/COM add-in compatibility is not promised. |
| Copilot-style inline/chat assistance and AI provider/privacy/retrieval controls | **M13** — optional, off by default, with preview/undo and explicit permissions; M8 stays usable offline without AI. |
| Actual Excel/Word objects/events, host save/return, trust, package/signature preservation and undo/recalc | **M14**, owned by the forks — shared UI is reused; standalone `.bas` editing/generic doubles do not establish real-host acceptance. |
| Exact-reference shortcuts, ordering, geometry and physical/Office parity checks | **M10** for editor fidelity; **M14** for real-host/reference-build evidence. Unverified claims remain marked V. |

## Reproducible checks and visual review

Run [the harness commands](embedding.md#reproduce-ui-evidence). Model tests exercise actual compiled
execution, source lifetimes, compiler/runtime errors, nested stepping, typed inspection, invalid edits,
watch refusal, cancellation, project close and save races. Generic-host tests assert exactly-once
writes and wait-token cleanup. UI tests drive actual keyboard/pointer commands, source edit/undo,
Immediate evaluation, named menus/panes, injected-context isolation, narrow/scaled geometry and
contrast. Shell tests cover byte-preserving source round trips, invalid UTF-8, size bounds and binary
writeback refusal. The CLI has a scripted break/inspect/edit/continue regression.

The original PNG variants live in ignored `reports/ui/`: `light-paused`, `dark-paused`,
`narrow-paused`, `scaled-paused`, `host-theme`, `focus-immediate`, `dark-object-browser`, `native-paused`, `browser-break`
`browser-completed`, `light-completion`, `dark-completion`, `light-syntax-error`,
`dark-syntax-error`, `browser-completion` and `browser-syntax-error`. These are product output, distinct from the ignored downloaded reference
cache. Visual review corrected missing font glyphs, unnamed selectors, narrow-header clipping and
internal frame/value presentation. Native capture runs a real eframe window under Xvfb; Chromium
verifies F8 reaches Break, F5 reaches Completed and output contains `10`; the extended
typing workflow accepts Debug.Print and executes output `8`, then introduces/corrects a syntax error
without Compile. Both paths require no page errors.

Final gate results and exact counts are recorded in [implementation progress](../implementation-progress.md).
[Remote CI](https://github.com/matt123p/tiny-vba/actions/runs/40552813785) also passes Windows, Linux and macOS native checks, WASM, UI and corpus verification for implementation commit `e24d07a`. No Office installation, hardware accessibility device, fresh Microsoft oracle run or external-workbook integration is claimed.

## Public-source process

Implementation and authored fixtures were written independently from public functional requirements.
No proprietary source, VBE clone implementation, extracted resource, copied screenshot or tutorial
artwork is included. Public pages are referenced by URL/date/hash and factual observation in
[the source manifest](../provenance/vbe_sources.yaml). Downloads remain only in `.cache/evidence/vbe/`;
Git ignores that directory and CI rejects tracked cache content. The provenance checker also checks
cached hashes when present, exact GUI/event bridge pins and licensed font notices. A checkout without
the cache still runs all tests offline after normal dependency acquisition.

Fonts come from the reviewed egui dependency package; required redistributable font notices are in
`licenses/fonts/`. They are licence/attribution material, not downloaded UI-reference evidence.
No Microsoft branding or VBE resource artwork is used. [Dependency provenance](../provenance/ui-dependencies.json)
records source pins/licences; it does not claim that public visibility grants unrestricted asset reuse.
