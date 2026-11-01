# M6 integration acceptance audit

Updated 2026-11-01. **Local implementation is complete for the declared profile;
M6 is not yet fully accepted.** Fresh Microsoft Office reference observations
remain NOT_RUN and have not been waived. The shared changes also require
publication and an immutable dependency pin in both application forks. M7 has
not started.

## Implemented scope

| Task | Result and evidence |
|---|---|
| M6.1 | WordCraft's bounded native Application/Document/Selection/Range profile, nominal catalogue, live UTF-16 ranges and stale-handle refusals; real document/session tests |
| M6.2 | Shared editor embedding, per-document trust/source state, native/browser source routing; actual Chromium inert import and byte-identical CRLF/Unicode export; asynchronous source-save success/cancel/failure/stale-completion tests; Alt+F11 full-frame regression |
| M6.3 | WordCraft live WithEvents Application BeforeClose/BeforeSave/WindowSelectionChange routing; all-sink signature validation before effects, connection order, ByRef propagation, debugger pause/reset, lifecycle cancellation and native undo grouping |
| M6.4 | Authored fork workflows pass; **fresh Word/Excel observations NOT_RUN** |
| M6.5–M6.8 | Existing [per-layer/context acceptance](m6-context-resolution-check.md): staged resolution, With, fork globals/implicit Me and read-only receiver completion |
| M6.9 | Owned module/member metadata, exact procedure-local invocation attribute, signed IDs, known-value validation and bounded class export envelope; metadata has no executable checkpoints |
| M6.10 | Canonical raw source plus reversible code projection; raw/view coordinates, newline/Unicode preservation, hidden-record edits/history/search/export (including a temporarily erased procedure name), mapped debugger gutters and procedure views; read-only raw inspector |
| M6.11 | Attribute-only document module has blank code, code-name explorer/Properties and separate source filename; descriptions/Instancing read-only; host-validated, revision-checked rename acknowledgement API |
| M6.12 | Shared native/WASM/WASI/UI checks and actual WordCraft/GridCraft metadata workflows; actual browser acceptance tracked below; Office observations remain separate |

[ADR 0029](decisions/0029-host-event-subscriptions.md) owns generic subscription
mechanics; [ADR 0030](decisions/0030-attribute-metadata-and-code-projection.md)
owns attributes and the editor projection. Office names/signatures/policy and
live document/workbook effects remain in the forks.

## Verification

Shared implementation:

- `cargo xtask ci`: PASS, 416 workspace tests, formatting, strict Clippy,
  layers, assets, source/provenance and corpus checks.
- `cargo xtask wasm`: PASS.
- `cargo xtask wasm-run`: PASS, 205 actual WASI tests across 31 groups,
  including syntax attributes, attribute execution, host events and projection.
- `cargo xtask ui`: PASS; `xvfb-run -a cargo xtask ui-native`: PASS;
  `cargo xtask web`: PASS.
- Playwright/Chromium `tools/ui-browser/smoke.cjs` and `forms.cjs`: PASS for the
  editor/debugger, visible-code clipboard with retained hidden header, exact
  Unicode/CRLF transfers and modal-form breakpoint/resume. The recorded-member
  attribute case also passes: inert import, explicit Macro2 execution, hidden
  metadata in the code pane and byte-identical raw export.
- Inspected original light/dark blank-document, paused recorded-macro and raw
  metadata screenshots under ignored `reports/ui/m6-*.png`.

Forks are currently verified using a temporary local Cargo patch, not a published
shared revision. No machine-specific patch should remain in the deliverable.

- WordCraft: 3 position-map tests and **25 actual document workflows PASS**;
  optional VBA UI suite **1035 PASS, 2 existing ignored**. New cases include
  Application events, source-save completion, metadata-preserving real execution,
  resumed selection notifications and multi-slice native undo. Full CI and the
  rebuilt VBA-enabled browser application are in progress.
- GridCraft: complete VBA adapter suite PASS, including **5 context-resolution
  workflows**. The new workflow imports blank ThisWorkbook metadata, runs the
  exact authored recorded-attribute shape with explicit consent, checks A1=42,
  retains raw source and undoes the workbook change. Full CI is being rerun with
  the existing optional craft-fonts checkout to resolve the earlier font-dependent
  environment failures.

## Explicit boundaries and remaining acceptance

The owner has no Word or Excel installation; Document Control lists no connected
sessions. Public Microsoft documentation and inherited oracle results are useful
specification/evidence, but do not substitute for the M6.4 fresh-reference gate.
Do not mark M6 accepted or begin M7 without actual observations or an explicit
owner decision changing that requirement.

Document code names are host-owned. Both current forks refuse rename requests
without changing raw metadata or host identity; the acknowledgement API does not
claim package writeback. Excel Macro Options/shortcut assignment, project
activation/enumerator semantics, default fields/functions, arbitrary designer
resources/third-party controls, automatic Word package Open/New, Application.Run,
add-ins and chart sheets are outside this declared profile or explicitly refused.
A supported Property Get default member remains executable.

Binary VBA project writeback, signatures and unchanged-package round trips belong
to M7. Source import remains inert. The README describes tiny-vba as shared
ArtCraft Office infrastructure; the standalone shell exists only for development
and testing.
