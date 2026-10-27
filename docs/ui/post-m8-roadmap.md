# Post-M8 UI and extension milestones

Updated 2026-10-27 on explicit user direction. All milestones below are **planned, not implemented**.
M8 remains the delivered embeddable editor/debugger baseline recorded in
[its acceptance report](m8-acceptance.md). M9 remains conformance and release of the supported
baseline; its release gate does not imply completion of later UI features.

The [public-source UI specification](craft_vba_editor_debugger_ui_spec.md) is the requirements
reference. Its P0–P8 phases and C0/C1/X labels describe feature families, not additional M8 scope.
The 2026-10-27 correction request explicitly includes working scoped word/member completion and
live syntax feedback in M8 (specification section 16). This document assigns the remaining work to repository milestones and defines completion evidence.
Numbering identifies scope; dependencies below determine sequencing. Host integration can proceed
without waiting for optional extensions or AI.

## M10 — Classic editor, code aids and advanced debugger

Owner: tiny-vba editor/model/compiler/debugger layers. Depends on M8 and the applicable M9 baseline.

- Complete remaining classic editing/project workflows: whole-block indent/comment commands, automatic indentation/case correction, split and procedure-only code views,
  docking and toolbar customisation, validated module renaming, export-before-remove, bookmarks,
  navigation history, richer editor options/preferences and browser source-file transfer.
- Extend M8 scoped word/member completion and project signature help with List Constants, richer
  Quick/Parameter Info, definition/help navigation, external reference metadata and Object Browser.
  Use actual compiler/host catalogue metadata; external library loading needs an explicit host policy.
- Add Quick Watch, conditional/value-change watches, fuller safe expression/index inspection,
  expanded local editing and error-trapping controls supported by the runtime. Class-specific
  trapping belongs to M11. Existing M8 stepping and basic run-to-cursor remain delivered.
- Design and verify Set Next Statement before exposing instruction-pointer relocation. Expand
  Immediate calls/property evaluation only behind explicit capabilities and side-effect policy;
  retain default safe evaluation and pause/source-generation checks.
- Add the optional DAP adapter and its scripted breakpoint/stack/variables/step/disconnect gate.
- Resolve remaining keyboard, focus, accessibility and reference-fidelity verification items using
  public documentation and lawful black-box observations where available. Keep unverified exact
  VBE geometry/ordering marked V; do not promise pixel identity without evidence.

**Acceptance:** committed requirement-to-test mapping for the remaining applicable P1–P5 flows;
module edit/import/export/rename/remove round trips; deterministic metadata-backed code aids;
positive/negative relocation and watch-policy tests; DAP session cleanup; native/browser keyboard,
accessibility, theme and visual-harness checks. No debugger command may simulate unsupported VM work.
Exact Office behaviour claims require separately recorded reference-build evidence and M14 workflows.

## M11 — Class/document support and UserForm editor

Owner: tiny-vba reusable language/runtime, form model and designer UI; host adapters/storage stay
in the application forks. Depends on M8; coordinate shared commands/code aids with M10.

- Establish the prerequisite class/document-module, property, instance/event and class error-trapping
  contracts needed for forms. Do not enable a designer Run action before its runtime is real.
- Implement a shared form model and visual designer: Toolbox, control selection/drag/resize,
  grid/snap, Properties, tab order, Format alignment/spacing/grouping/z-order and designer zoom.
- Generate/navigate correctly signed control event handlers and debug actual form events through
  the existing controller. Keep form design mode separate from execution reset.
- Implement reviewed form source/resource import/export and round-trip fixtures. Define the host
  persistence interface and preserve opaque/unsupported control data or require an explicit loss decision.
- Keep third-party ActiveX/COM execution a separately reviewed capability. Cross-platform forms
  do not imply arbitrary Windows controls or native VBE add-ins work unchanged.

**Acceptance:** independently authored forms with Label, TextBox and CommandButton can be designed,
saved/exported, reopened/imported and edited without loss; control events really run, pause, inspect
and resume; class/error-mode and reset/cleanup tests pass; keyboard/Properties/tab-order and original
native/browser snapshots pass. Host Office-package form persistence is verified under M14.

## M12 — Native Craft extensions and non-Microsoft enhancements

Owner: tiny-vba extension interfaces and reusable UI; platform/service adapters belong above the
model. Depends on M8; use M10/M11 capabilities only when an extension needs them.

- Define a versioned native Craft extension API with discovery, enable/disable, command/pane
  registration, capability declarations, bounded work, error isolation and unload cleanup.
- Add an extension manager and distinguish these extensions from binary VBIDE/COM add-ins.
  Loading an extension must not implicitly run a macro or grant document/network/file access.
- Deliver opt-in enhanced-editor features such as richer diagnostics, refactoring/code actions,
  search/navigation improvements and extension-owned panes while preserving Classic mode.
- Provide the non-Microsoft Python translation/preview extension with a declared supported subset,
  clear diagnostics for unsupported constructs, reviewable output and explicit export. VBA remains
  the original source; preview/conversion must not silently execute or claim universal equivalence.

**Acceptance:** independently authored extension fixtures register/use/unload commands and panes;
permission denial, failure and cleanup paths preserve source/debug sessions; conversion fixtures
verify the declared subset and unsupported cases; source-changing actions are previewed and undoable;
Classic mode remains usable with all extensions disabled. Public-source/licence and cache rules apply.

## M13 — Optional Copilot-style assistance

Owner: an opt-in assistant extension and shell/host provider adapters. Depends on M12's extension
and capability boundary and the applicable M10 metadata/undo interfaces. No AI dependency enters
the VM, compiler or default Classic workflow.

- Add labelled inline suggestions and an optional explain/fix/generate-tests/chat pane. Keep
  deterministic completion separate: classic popups and existing shortcuts take precedence.
- Apply accepted suggestions as one undoable edit with a diff/preview; cancel stale suggestions
  after source or context changes. Use the M12 conversion preview if offering AI-assisted translation.
- Keep assistance off by default. Distinguish local/remote providers, scope what may be sent and
  honour host/workspace permissions, cancellation and offline operation.
- Treat project source, comments, cells, filenames and retrieved content as untrusted data.
  Restrict retrieval to approved sources, keep provenance and never give generated text implicit
  authority to run macros, write files, access services or export data.
- Record provider/retrieval limitations honestly; provenance controls on supplied evidence are not
  a guarantee about a general-purpose model's entire training history.

**Acceptance:** opt-in/permission-denial/offline/cancellation tests; stale-response and single-undo
edit tests; classic completion/shortcut precedence; prompt-injection and exfiltration regressions;
no execution or export without a separate authorised action. Fake/local provider fixtures keep
normal CI independent of paid services, credentials and network access. Validate native/browser UI.

## M14 — Gridcraft and Wordcraft integration acceptance

Owner: the application forks for Office objects, document UI/storage/trust/events/undo/recalc;
tiny-vba owns the shared embedding APIs. Baseline integration depends on M5/M7/M8 and can run in
parallel with later UI milestones. Form-specific workflows additionally depend on M11; AI is optional.
The user authorized the GridCraft baseline on 2026-10-27; its implementation and
fork-owned evidence are recorded in [the integration work log](../gridcraft-integration.md).
Broader event, form, signature and WordCraft acceptance below remains planned.

- Embed the same UI in both applications with real document projects, themes, fonts, shortcuts,
  source save/return routing, catalogue metadata and host event signatures.
- Verify actual workbook/document mutations, waits, pause/inspect/edit/resume, event ordering,
  reset/abnormal-stop cleanup and the host's undo/recalc/partial-side-effect policy without replay.
- Verify controlled `.xlsm`/`.docm` attachment/import/save, code names, trust/no untrusted auto-run,
  opaque forms/controls, signature validity/loss notices and promised payload preservation.
- Add reference-build observations and independently authored real document workflows. Preserve
  evidence distinctions between public documentation, generic doubles and actual Office observations.

**Acceptance:** executable tests and evidence in each owning fork, including real document/event
workflows and package round trips; both hosts reuse the editor implementation. Nothing becomes PASS
from a tiny-vba-only snapshot or generic-host test. Protected-project/P-code bypass and unrestricted
binary add-in compatibility remain excluded without a separate reviewed scope decision.

## Specification traceability

| Reference scope | Milestone allocation |
|---|---|
| P0 provenance/fidelity verification and remaining V items | M10 for editor reference checks; M14 for host/reference-build checks; retain the implemented M8 provenance baseline |
| Remaining P1–P3 shell/editing/code aids and P4/P5 debugger features | M10; delivered M8 features stay in its acceptance record |
| Class/document runtime and P6 UserForms/designer | M11; host document events/persistence also need M14 |
| P7 Wordcraft/generalisation and actual host workflows | Shared UI already delivered in M8; application integration/evidence in M14 |
| P8 non-AI enhancements, native add-ins and translation preview | M12; classic compatibility completion stays in M10 |
| P8 / section 9 AI inline/chat and provider/privacy controls | M13; native extension/translation infrastructure in M12 |
| Mandatory acceptance scenarios 1–22 | Remaining reusable classic features M10, class-specific pieces M11, real-host/storage portions M14; retain M8's existing partial evidence |
| Scenario 23 | M11, plus M14 for Office-package/host persistence |
| Scenarios 24–25 | M13, with M12 extension isolation |
| Scenario 26 public-source/provenance review | Existing M8 checks continue; mandatory in M10–M14 |

All later milestones retain independent code/artwork, licensed dependencies and original-render
snapshot requirements. Download evidence only into ignored `.cache/evidence/`; track URLs, hashes,
observations and redistributable notices. Planned scope does not change current capabilities.
