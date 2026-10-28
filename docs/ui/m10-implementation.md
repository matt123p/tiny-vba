# M10 implementation, acceptance and work log

Historical milestone IDs are retained in this acceptance record. Future work
uses the [renumbered remaining plan](../implementation-plan.md); see its
[mapping](../implementation-progress.md#plan-renumbering--2026-10-28).

Started 2026-10-28 on `feat/m10-classic-editor-completion` from `eaafa8c`.
User request: complete M8 and M10. M8's corrected baseline is already implemented;
rerun its regressions rather than relabeling planned M10 work as delivered.

## Stages

| Stage | Scope | State |
|---|---|---|
| M10.1 | Editing/project workflows: block edits, indentation/case, rename/remove/export, bookmarks/history, split/procedure views and view preferences | Complete for the delivered profile below |
| M10.2 | Metadata-backed constants, Quick/Parameter Info, definition/help navigation, references and Object Browser | Complete for declarative source/host metadata |
| M10.3 | Quick Watch, conditional/change watches, array inspection/local editing and error trapping | Complete for the supported runtime profile |
| M10.4 | Verified Set Next Statement and capability-controlled Immediate evaluation | Complete for the documented safety profile |
| M10.5 | Optional DAP frontend, protocol/transport, scripted lifecycle acceptance | Complete for the advertised single-session capabilities |
| M10.6 | Requirement-to-test mapping, M8 regression rerun, UI/native/browser/WASM/CI and original rendered inspection | Complete; exact Office geometry remains V |

Ownership: reusable behavior in tiny-vba; no GridCraft edits. Keep native calls,
explicit identities, stale-source/pause rejection and bounded work. No proprietary
Office implementation/assets or unlicensed fixtures. Public behavior references
are recorded with each implemented stage; documentation is not oracle evidence.

Completion requires evidence for all stage requirements, not only compilation.
Update this file with tests, actual limitations and next task at each checkpoint.

## Initial checkpoint — 2026-10-28 (historical)

Implemented model/UI work so far: physical-line block indent/outdent/comment,
module rename including VB_Name with stable SourceId, explicit export-before-remove
and discard choice, bookmarks/history; Enter indentation/case formatting; host
reference/constant metadata, scoped definition navigation and parameter selection;
local array inspection/typed element edits, conditional/change watches and error
trapping; safe straight-line checkpoint relocation; optional framed DAP frontend
and `vba-cli/dap` stdio shell. No completion claim yet: remaining view/customization,
Immediate policy and broader UI/runtime gates still need implementation/verification.

Evidence so far: four editing tests, four advanced debugger tests and three DAP
transcript/framing/lifecycle tests pass, alongside existing M8 model/UI suites.
Latest metadata/Enter/UI rerun is logged in ignored `target/m10-current.log`.
Set Next Statement refuses all branch/handler procedures, non-executable targets,
fault/ancestor/stale handles; it does not simulate unrestricted relocation.
Conditional predicates use a new read-only, exact-visit VM inspection API, not
host evaluation. Safe watches do not call properties/functions.

Public behavior references consulted (paraphrased; original implementation/tests):
https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/edit-menu
https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/code-window
https://learn.microsoft.com/en-us/office/vba/language/concepts/forms/add-a-watch-expression
https://microsoft.github.io/debug-adapter-protocol/overview
The DAP schema/transport is implemented independently; no upstream adapter copied.
Pinned serde_json 1.0.154 is an MIT OR Apache-2.0 JSON dependency, resolved from
the local registry cache. It is used only in the optional DAP layer, not native
debugger/host dispatch. Update architecture/provenance contracts and gates before
the stage is committed. Next: finish view/customization and explicit Immediate
evaluation policy, then expand negative/interaction/transport acceptance.

## User checkpoint and continuation — 2026-10-28

The user committed the accumulated implementation as `8dd37dc` on tiny-vba main.
It is preserved unchanged. Completion work continues on `feat/m10-completion`
from that commit; the original feature branch is historical. GridCraft is untouched.

Real interaction evidence now includes selected block indentation/Undo, procedure
editing that preserves the other procedure and Unicode/CRLF bytes, two accessible
split editors with right-docked panes, and Enter indentation/case correction.
Initial test failures included missing platform command modifiers and an incorrect
accessible label; corrected tests pass without weakening the source assertions.
Original rendered split, dark procedure, narrow/scaled split and dark options
variants are generated under ignored `reports/ui/` and being inspected.

The real Chromium harness passes M8 debugger/completion/clipboard/syntax recovery,
an exact UTF-8 source download and Unicode/CRLF `.bas` upload with no execution.
The harness clears inherited indentation when typing an explicitly indented fixture.
Browser tooling lives only in ignored `target/web-tools` and `target/browser-tools`.
Rust web-sys source transfer compiles for wasm32-unknown-unknown. Browser downloads
do not silently acknowledge a successful filesystem save or discard dirty source.

Explicit Immediate policy tests pass for private Function calls with owned local
snapshots, real host effects, denied calls and exactly-once cancellation of waits.
This is an isolated bounded child VM, not live shared-project code injection.
Watches retain a separate read-only path. Final work includes view-wide editing
behavior, scoped watch display/budgets, accurate DAP stop reasons, the complete
requirement/evidence mapping and fresh final CI/WASM/native/browser gates.

## Delivered profile — 2026-10-28

The implementation preserves the user's `8dd37dc` checkpoint. M10 completion
continues on `feat/m10-completion`; no GridCraft changes are included.
[ADR 0018](../decisions/0018-m10-classic-tools-and-debugger-policy.md) defines
ownership, safety policies, resource bounds and the optional DAP dependency.
Earlier checkpoint notes above describe intermediate states, not current gaps.

### Requirement-to-test map

All fixtures are independently authored. Paths below are relative to the repo.
The applicable P1–P5 classic flows are implemented through the native IDE model;
UI controls never synthesize successful VM operations.

| Stage / requirement | Implementation and acceptance evidence |
|---|---|
| M10.1 block indentation/comment, Enter indentation/case | `vba-ide/tests/editing.rs` checks physical lines, Unicode, all line endings and selection; `metadata.rs` checks lexical strings/comments/case; real `vba-ui-egui/tests/interaction.rs` covers selected block commands, repeated procedure edits and Enter behavior. |
| M10.1 source Undo/Redo, richer Find/Replace | Shared bounded `history.rs` replaces widget-local history across views. `editing.rs` proves bounds/redo invalidation/bookmark anchors; `search.rs` proves module/procedure/project scopes, forward/backward wrap, case/whole-word Unicode offsets, undoable replacements and all-buffer budget prevalidation. UI tests verify F3/Shift+F3, selected match Copy and existing M8 Find wrap. |
| M10.1 rename/remove/export/bookmarks/navigation | `editing.rs` verifies stable SourceId, `Attribute VB_Name`, name validation/duplicates, dirty state, explicit discard/export cancellation and bounded navigation. `(Name)` routes to validated rename. Removal clears only that module's retained state. Unchanged bookmark anchors follow edits and Undo. |
| M10.1 split/procedure-only views and layout | `views.rs` edits a projection through the original source map. Interaction tests retain CRLF/Unicode and untouched procedures, support completion/menu Copy, repeated selected block operations and cross-view Undo. Two independently labelled editors, pane docking and narrow/scaled layouts are asserted. |
| M10.1 preferences and toolbar customization | Shell `tests/source_io.rs` round-trips editing/layout settings and toolbar order, without source or Immediate execution permission. UI offers visibility/order controls, project left/right, debugger bottom/right with narrow fallback. Original options/split/procedure renders are inspected. |
| M10.1 native/browser source transfer | Shell source tests verify exact Unicode/CRLF/attributes, invalid UTF-8, oversize and Office binary-write refusal. Real Chromium downloads exact `.bas` bytes and uploads an authored Unicode/CRLF file through the chooser without execution. Export-before-remove queues an explicit request; browser downloads retain dirty state. |
| M10.2 List Constants and Quick/Parameter Info | `metadata.rs` proves scoped visibility, nested-call argument selection, string commas, inherited indentation and chained receiver metadata without evaluation. UI test inserts a real selected constant and runs the resulting source. Existing completion/signature/syntax regressions remain active. |
| M10.2 definition/help, references/Object Browser | Metadata tests prove definition/history and bounded external reference/constant/help configuration. UI filters project/library members, navigates definitions and exposes validated HTTPS help. Host metadata is declarative, not executable library loading or type binding. |
| M10.3 Quick Watch/conditional/change watches | `advanced_debug.rs` proves true/change checkpoints, selected-frame module/procedure context and nonblocking invalid conditions. A model unit test exhausts the watch budget and verifies the explicit one-time pause/disable/resume behavior. UI Quick Watch inspects a real paused expression and promotes it to Watches. |
| M10.3 safe arrays/local edits/error trapping | `advanced_debug.rs` checks bounded index inspection, typed element edits and resumed values; existing workflows cover error snapshots/trapping/invalid local edits. Options update actual runtime trapping. Class-specific trapping remains M11. |
| M10.4 Set Next Statement | IDE advanced tests prove actual skip without execution, stale-source refusal, branch and non-executable target refusal. `vba-vm/tests/inspection.rs` proves current exact-visit inspection, foreign/stale rejection, relocation generation invalidation and unchanged values until Continue. |
| M10.4 explicit Immediate policy | Advanced/embedding tests prove default denial, private Function calls, owned local snapshots, parent-local isolation, real host calls and exactly-once wait cancellation. No calls/properties enter the safe watch path; the opt-in is explicit and not persisted. |
| M10.5 optional DAP | Four `vba-dap/tests/protocol.rs` tests cover framed UTF-8, malformed/oversized input, initialize/launch/configure/break/stack/variables/evaluate/edit/step/continue, accurate stop reasons, pause/disconnect, stale IDs and unsupported breakpoint options. Two CLI subprocess tests prove real stdin/stdout lifecycle and oversized-frame failure with deadlines. Protocol tests also execute under WASI. |
| M10.6 keyboard/accessibility/theme/fidelity | 30 real egui interaction tests include 21 M8 regressions plus M10 flows; independent embedded scopes and injected theme/contrast assertions remain. Fifteen original render variants cover light/dark, narrow/scaled, host theme, focus, syntax/completion and new M10 views/options. Native Windows capture uses an actual paused VM. Real Chromium verifies edit/run/break/step/clipboard/completion/syntax recovery and source transfer. |

### Verification environment and reproduction

Local Windows, pinned Rust 1.99.1, actual WASI under Node and Chromium via the
repository Playwright harness. Required tooling lives in ignored `target/`;
no downloaded reference assets or browser binaries are committed.

| Gate | Result |
|---|---|
| `cargo xtask ci` | PASS: format, strict all-target/all-feature Clippy, **277 workspace tests**, layers, provenance and bundled evidence checks. Optional DAP process tests are included through all features. |
| `cargo xtask wasm` | PASS: headless workspace cross-compilation. |
| `cargo xtask wasm-run` | PASS: **95 tests** actually executed under WASI, including IDE/DAP and new VM inspection tests. |
| `cargo xtask ui` | PASS: **90 model/UI/shell tests** (including 30 interaction tests) and **15 original render variants**. Overlaps workspace tests; do not sum these counts. |
| `cargo xtask ui-native` | PASS: actual Windows app capture, `UI_CAPTURE_OK`, normal exit. |
| `cargo xtask web` | PASS: browser bundle built with pinned wasm-bindgen 0.2.126. |
| `cargo clippy -p vba-ide-app --target wasm32-unknown-unknown -- -D warnings` | PASS: browser-only lint gate; checks source-transfer code absent from native compilation. |
| `node tools/ui-browser/smoke.cjs` | PASS: real Chromium workflow. Set `VBA_PLAYWRIGHT_MODULE` and `PLAYWRIGHT_BROWSERS_PATH` to locally installed tooling when needed. |

Logs are ignored `target/m10-final-*.log`; original renders/captures are ignored
`reports/ui/`. Inspected dark procedure/options, narrow/scaled split and native
paused capture for readable text, distinct source panes and real statement/local
state. Existing host-theme/focus/light variants remain in the same render gate.
No fresh macOS/Linux native UI or Microsoft Office oracle run is claimed.

### Supported boundaries

- Set Next Statement is restricted to verified straight-line current-frame
  checkpoints; branches, handlers, fault/ancestor/stale targets are refused.
- Immediate calls run in an isolated child VM using local snapshots. They do not
  mutate paused local/global state; explicitly permitted host effects are real.
  Waiting children are cancelled; safe watches never invoke functions/properties.
- Source history is bounded to 256 states/8 MiB, with source-changing undo steps.
  Project-wide Replace All records each changed module as a history step.
- Find uses literal matching, ASCII case folding and Unicode word boundaries.
  Locale-specific case matching, wildcard patterns and a selected-text search
  scope are not advertised. Source byte offsets remain valid.
- Docking offers pane placement and toolbar visibility/order, not a cloned native
  VBE drag/drop docking engine. Exact VBE geometry/ordering remains **V** (unverified).
- External library metadata does not load COM/DLLs or grant executable type/constant
  binding; M15 owns that contract. Classes/forms remain M11. Office events, workbook
  packages and unchanged Excel source acceptance remain the fork/M14–M19 gates.
- This acceptance adds no external corpus executions: seven pins checked, checkout
  unavailable for 0/646 candidate hashes; the existing 153 bundled oracle cases
  retain their inherited evidence classification, not fresh Office observations.

### Public behavior sources and provenance

Public Microsoft VBA documentation informed command names and behavior; original
Rust code, tests and geometry implement the supported profile. No implementation
code, proprietary binaries/artwork or reference screenshots were imported.

- [Edit menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/edit-menu)
- [Code window](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/code-window)
- [Find dialog](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/find-dialog-box)
- [Watch expressions](https://learn.microsoft.com/en-us/office/vba/language/concepts/forms/add-a-watch-expression)
- [Public DAP transport/capabilities overview](https://microsoft.github.io/debug-adapter-protocol/overview)

Dependency rights/pins are recorded in `docs/provenance/ui-dependencies.json` and
`tools/architecture.json`. JSON is confined to optional DAP transport/metadata;
the embedded UI and host dispatch continue to use native Rust calls.

### Final closure

All gates above passed locally. Final audit fixes included shared source history,
selection restoration after procedure edits, read-only DAP stack queries and
accurate stop reasons, explicit watch-budget exhaustion, and Find selection/focus
across F3/Shift+F3/View Code. The extra browser-target Clippy run found a complex
callback field type; a named alias resolved it and the lint/build/Chromium gates
were rerun successfully. Chromium reported `BROWSER_UI_PASS`, including exact
download/upload and no page errors. Visual review also covered the original
light split/right docks, injected host theme and focused dark Immediate pane.

No unfinished M10 stage remains within the declared profile. Next smallest UI
task is M11's class/document-module prerequisite contract and its execution tests,
before building form event execution. M9 release/conformance and M15 host type
binding remain independent tracks; no merge or dependency-pin update in GridCraft
is part of this completion.
