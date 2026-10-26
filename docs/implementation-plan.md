# Plan of action

Prepared 2026-10-26. This plan builds the interpreter and editor/debugger UI here; document/Office integration stays external.
Setup and the documented M0–M3 subsets are complete. The detailed [bytecode/VM specification](bytecode-vm-specification.md)
and [ADR 0004](decisions/0004-bytecode-vm-contract.md) are written before frontend work continues.
M2 binding/lowering and M3 execution/debugger are implemented. M4 is complete for the reviewed
[m4-language-v1 subset](m4-language.md), including IR 2, arrays/arguments, coercions, handlers,
intrinsics and persistent state. The 20 original and 24 M4 authored scenarios are asserted PASS.
External corpus/Microsoft oracle observations remain M9 work and are not claimed here.

| Stage | Work in tiny-vba | Acceptance gate |
|---|---|---|
| Setup — complete | Workspace/src layout, CLI refusal, adapted agent/skill files, docs, CI, corpus gitlinks/pins/hashes | Native scaffold checks and library WASM compilation; no application dependency |
| M0 — contract and test harness — complete | Source IDs/spans/diagnostics; versioned scenario parser/validator; at least 20 independently authored pure-language cases; foreign-call ownership prototype and deterministic double | CLI validates valid fixtures and rejects malformed ones; missing/empty/null distinctions; fake call/error/wait round trip; no silent execution stub |
| M1 — frontend — complete for reviewed subset | Lexer, source-preserving parser, .bas/.cls metadata, procedures/declarations/assignment/If/For/calls; positive/negative fixtures | Exact error spans and same-line statement IDs; bounded recovery; run a reviewed grammar-corpus subset and report unsupported syntax |
| M2 — binder and bytecode — complete for reviewed subset | Scopes, Option Explicit, locals/procedure binding, argument subset, lowering, verifier/disassembler | Compile/disassemble examples; reject invalid branches/slots/stack effects; deterministic source maps |
| M3 — executable VM + debugger — complete for reviewed subset | Minimal typed values/arithmetic, explicit frames, loops/calls, budgets; generic foreign protocol; break/step/inspect/edit/resume | Host-free Answer() returns Long 42; generic host call trace correct; pause/edit/resume without replay; infinite loop cancels; native and WASM execution smoke tests |
| M4 — semantic depth — complete for reviewed profile | Centralized coercions/all scalar widths, special states, Currency/Date, canonical ByRef/temporaries, bounded arrays, optional/named calls, handlers/intrinsics and module/Static state | 44 authored typed scenario assertions; negative IR2 and error-number tests; handler propagation/resume and explicit state restore/reset; native/WASI evidence; [scope](m4-language.md) |
| M5 — fork integration contract | Stabilize public foreign/value/debug APIs and version compatibility table; offer reusable tests | Fork-owned bridge can run FillCells against its own fake Office layer, pause/edit/resume and validate results; core still builds alone |
| M6 — project source decoding | Read-only MS-OVBA decoder, encoding/attributes/references/codenames; malformed-stream and budget tests | Decode controlled project bytes; preserve module identities; reject unsupported/protected/corrupt inputs; no auto-execution |
| M7 — callbacks and lifecycle | Bounded callbacks/events, static reset, stale-reference handling, cancellation/disconnect cleanup | Generic-host reentrancy tests pass; fork-owned event order and partial-side-effect tests pass |
| M8 — editor/debugger UI and developer tools (required) | Implement toolkit-independent IDE state, reusable egui panels and standalone native/web shell; wire CLI debugger; optional DAP | Edit and run a self-authored module; use gutter breakpoints, execution highlight, continue/pause/stop and step into/over/out; inspect/edit locals, stack, safe watches and output; pause/resume without replay; embedded and standalone UI checks plus native/WASM gates |
| M9 — conformance and release | Curate more library/corpus cases, oracle goldens, importer/parser/VM fuzz targets, performance and native/WASM CI | Per-feature report with exact tested versions and all status classes; prior passing tests do not regress |

## Editor/debugger UI commitment

The visual editor/debugger is a required deliverable in this repository, not deferred to Gridcraft
or satisfied by a CLI/DAP adapter. Follow [the UI contract](editor-debugger-ui.md). Use the same
reviewed egui/eframe stack and design conventions as Gridcraft; deliver embeddable panels and a
standalone shell. Implement the first executable visual slice after the M3 pause/resume proof;
M8 completes the UI acceptance gate. Host-free editor/debugger use must work without an Office layer.
Gridcraft embedding is a separate cross-repository acceptance gate.

## Cross-repository gates (owned by the forks)

Gridcraft implements the Office metadata/schema/catalogue, object model and bridge. Its first gate is
`FillCells` through fake Office dispatch with a breakpoint; then real Session edits with recalc/undo checks.
A runtime error after a successful write must follow the chosen compatibility policy rather than
implicitly rolling back the whole macro. Project attachment/trust and .xlsm load/run/save are host tasks.
Verify original project bytes/relations, code names, ThisWorkbook versus ActiveWorkbook and no untrusted auto-run.
Gridcraft embeds tiny-vba's editor/debugger panels with its own theme and command routing.
Verify a real workbook macro can pause, inspect and resume through that UI without duplicate writes
or retained document locks; the fork owns this integration, not a second editor implementation.

Wordcraft later implements Word objects, live ranges/character units, its bridge and .docm preservation/events.
Python/JavaScript bind directly to fork-owned Office dispatch; they do not pass through the VBA engine.
Do not duplicate those implementations in tiny-vba to make a local demonstration pass.

The [M2/M3 acceptance audit](m2-m3-acceptance-audit.md) maps M2 gates to tests and records the
M3 replacement-session isolation correction under ADR 0008.

## Immediate task sequence

1. M4 language gates are implemented under ADR 0011; preserve explicit profile refusals and authored evidence.
2. M5: stabilize foreign/value/debug API versioning and reusable bridge tests; fork-owned FillCells proof.
3. Start the reusable graphical editor/debugger slice on proven VM APIs; M8 completes its acceptance gate.
4. Extend vetted scenario/oracle coverage through M9, retaining NOT_RUN for unexecuted external material.

## Measurement and decisions

Maintain [compatibility](compatibility-matrix.md), [progress](implementation-progress.md) and source evidence per feature.
Use PASS, MISMATCH, UNSUPPORTED, BLOCKED_BY_POLICY, INVALID_FIXTURE, CRASH, TIMEOUT and NOT_RUN.
Initial integrated benchmark target: 100 asserted cases and 20 independent workbooks, assembled gradually.
Office-specific 40/30/15/10/5 category targets from the supplied design belong in the fork suite;
tiny-vba reports language/debugger/import results separately. No initial global pass-percentage gate.

Open decisions and selected defaults are in [ADR 0001](decisions/0001-repository-scope.md).
No substantial UserForms/COM/DLL, P-code, project writing or full Word work before the vertical slice passes.
