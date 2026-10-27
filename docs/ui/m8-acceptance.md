# M8 acceptance and compatibility status

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
| Editing | Monospace editor, lexer-based VBA colours, line numbers, scrolling, selection, keyboard editing/undo, indentation, literal find/replace with allocation bounds, diagnostic navigation |
| Breakpoints | Actual verified visible statement spans; F9/gutter actions; rejection of blank/comment/declaration lines; distinct colon-separated statements; explicit invalidation after edits |
| Execution | Real run, pause, continue, UI Reset and into/over/out stepping; bounded ticks; stale-source refusal; run-to-cursor in break mode and Show Next Statement |
| Inspection | Real frame stack and selected-frame locals; validated active-frame edits; typed values and bounded array previews; old-frame editing refused |
| Watches/Immediate | Safe scalar expressions using parser/runtime rules; out-of-scope/error states; explicit refusal of calls/getters/indexing; `? expression` and active-local assignment |
| Symbols | Actual parsed procedure signatures and injected host API catalogue; prefix completion; qualified host names require real supplied metadata |
| Embedding | Caller-owned Ui/theme/fonts/scaling; explicit commands and source/host requests; no toolkit/IO in IDE model; generic-host tests count dispatch/resume/cancel effects |
| Shell/lifecycle | Host-free authored demo; CLI debugger; native and browser shells; dirty-close confirmation; native cleanup and font/pane preference persistence |
| Visual/accessibility | Named widgets with accessibility checking enabled; original markers plus colour; contrast assertions; actual paused-state light/dark/narrow/1.5x/host-theme/focused-pane snapshots |

## Explicit boundaries of the broader UI specification

| Specification area | Current status |
|---|---|
| `.xlsm` / `.docm` source attachment, trust, package/signature preservation, host save/return | Supplied by embedding host. Native standalone saves `.bas` only. No binary writeback or macro auto-run on import. |
| Worksheet/workbook events, Excel/Word object models, real workbook edits and undo/recalc | Fork-owned integration acceptance; not proven by standalone snapshots or generic doubles. |
| Class/document execution, UserForms/designer/Toolbox/Format, ActiveX and binary add-ins | Not implemented by this runtime/UI profile; relevant commands are disabled with explanations. |
| Set Next Statement; conditional/value-change watches; arbitrary Immediate calls/getters; class error-trapping mode | Unavailable without additional verified VM/evaluation contracts; not simulated. |
| Split code windows, procedure-only view, full docking/custom toolbar designer, module renaming/removal | Deferred broader-spec features. Current panels are resizable; current editor is Full Module View. Metadata is read-only. |
| Full classic IntelliSense, parameter info, host event stub generation, references loading | Partial: real project procedure/host catalogue browsing and prefix completion only. No fabricated libraries/members. |
| Exact Windows VBE pixels, accelerator ordering, all platform shortcuts, physical-device/Office parity | Unverified. Layout is an original functional interpretation of public docs, with named classic commands and tested core shortcuts. |
| AI assistance and Python translation | Not included; the editor/debugger operates offline without either. |

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
and `browser-completed`. These are product output, distinct from the ignored downloaded reference
cache. Visual review corrected missing font glyphs, unnamed selectors, narrow-header clipping and
internal frame/value presentation. Native capture runs a real eframe window under Xvfb; Chromium
verifies F8 reaches Break, F5 reaches Completed and output contains `10`, with no page errors.

Final gate results and exact counts are recorded in [implementation progress](../implementation-progress.md).
No Office installation, other operating system, hardware accessibility device, fresh Microsoft
oracle run or external-workbook integration is claimed by these local checks.

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
