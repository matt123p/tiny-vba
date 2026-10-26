# Architecture

Status: source scaffold, 2026-10-26. This is the current repository layout; APIs remain to be designed.

The current scaffold and future headless engine can compile and test without Office, UI toolkits,
application forks or corpus downloads. This repository also owns an optional editor/debugger UI
and standalone IDE; those upper layers may depend on UI toolkits.
VBA source flows through syntax → semantics → verified IR → explicit-state VM.
The dependency graph is separate from this data flow: IR does not need a dependency on the binder.

| Crate | Responsibility | Direct normal dependencies |
|---|---|---|
| vba-diagnostics | Source identities, spans, diagnostic codes | none |
| vba-values | Typed Variant, arrays, references and l-values | diagnostics |
| vba-syntax | Lexer/parser and source-preserving AST | diagnostics |
| vba-semantics | Scope/type/call binding | diagnostics, syntax |
| vba-ir | Bytecode, verifier, constants and source maps | diagnostics, values |
| vba-runtime | Shared coercion/operators/builtins/error rules | diagnostics, values |
| vba-foreign | Generic host call/result/wait protocol | diagnostics, values |
| vba-vm | Frames, instruction pointer, budgets, inspection hooks | diagnostics, ir, values, runtime, foreign |
| vba-debug | Breakpoint/step/inspection controller | diagnostics, vm |
| vba-dap | Later DAP adapter | debug |
| vba-project | Read-only source/project decoder | diagnostics |
| vba-testkit | Deterministic generic foreign host and trace support | diagnostics, values, foreign |
| vba-conformance | Scenario validation, typed comparison, reports | diagnostics, testkit, vm |
| vba-cli | Standalone command frontend | diagnostics (compiler/VM wiring later) |
| xtask | Maintenance commands | none |

These dependencies reserve boundaries, not finished APIs. Most library crates contain only module documentation.
`tools/architecture.json` is the enforced graph. New external dependencies require explicit graph review.
`vba-testkit` and `vba-conformance` are development packages; production crates can use them only as dev-dependencies.
All packages are currently unpublished. No crate automatically loads corpus code.

## Ownership boundary

The generic foreign protocol carries invocation kind, target/reference, named/positional/omitted arguments,
ByRef obligations and explicit result/error/wait state. Its concrete Rust API must be proven by tests at M0/M3.
It must not know Excel/Word names, Range behavior, Office enums or document storage.
The applications implement a bridge to their own Office compatibility layers. The VM owns VBA class instances;
external object references preserve host identity/lifetime without defining a parallel Office registry.

No `vba-office-bridge`, `vba-excel-host`, `office-*` or `automation-*` crate belongs in this workspace.
The forks own Office dispatch/catalogues, object handles, document trust/lifecycle, OOXML,
undo/recalc and document UI. tiny-vba owns the reusable VBA editor/debugger UI and standalone IDE.
The host supplies document integration and prompts through explicit interfaces.
The interpreter owns VbaValue and source decoding; conversions to Office AutomationValue occur in fork bridges.
A generic foreign-host double here proves interpreter mechanics; an Office-shaped fake belongs with the Office layer.

Use synchronous native Rust calls for ordinary host work. Release mutable host borrows before callbacks,
pause or yield. Budget execution and allocations. The VM never calls DAP or the UI and never performs ambient IO.
Inspection hooks and statement safe points belong to VM; debug controls them, DAP wraps debug.

## Files and portability

`vba-project` consumes project bytes/source and returns module metadata. It does not own OOXML import/export,
project trust or macro attachment. P-code, protected-project bypass and binary source writeback are excluded.
The scaffold has no platform-specific library dependencies. `cargo xtask wasm` compiles all libraries for
wasm32-unknown-unknown; runtime WASM tests will be added when execution exists.

## Planned editor/debugger UI layers

Adopted scope: [ADR 0002](decisions/0002-editor-debugger-ui.md). Product contract:
[editor/debugger UI](editor-debugger-ui.md). These packages are planned, not implemented:

| Package | Responsibility | Allowed direction |
|---|---|---|
| vba-ide | Toolkit-independent editor/project/session state and actions | diagnostics, syntax/semantics, debug, project; no UI toolkit or Office types |
| vba-ui-egui | Embeddable editor/debugger panels | ide, egui and compatible UI support; no Gridcraft/Wordcraft dependency |
| vba-ide-app | Standalone native/web application shell | ui-egui, ide; eframe/platform facilities at this layer |

Gridcraft embeds vba-ui-egui in its UI composition layer and supplies a host bridge.
The core, VM and debugger controller never depend on these packages. The embedded UI accepts
a caller-owned egui context and style; it must not replace global fonts, theme or window settings.
Native debugging uses in-process Rust APIs; DAP remains an optional adapter for external editors.

Keep tools/architecture.json as the graph of actual Cargo packages. Add these nodes and reviewed
external dependencies when their implementation lands; do not add placeholder APIs or manifests.
Headless and UI checks must be separate so engine consumers do not pull GUI dependencies.
