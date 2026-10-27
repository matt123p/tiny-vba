# Plan of action

Updated 2026-10-27. This plan builds the interpreter and editor/debugger UI here; document/Office integration stays external.
Setup and the documented M0–M3 subsets are complete. The detailed [bytecode/VM specification](bytecode-vm-specification.md)
and [ADR 0004](decisions/0004-bytecode-vm-contract.md) are written before frontend work continues.
M2 binding/lowering and M3 execution/debugger are implemented. M4 now meets its original
external acceptance gate: 153 reviewed ReflexGrid pure-language cases match their published
Microsoft Excel Mac oracle expectations, alongside 44 authored scenarios. [The report](m4-oracle-report.md)
records source hashes, the upstream environment and the distinction from a fresh Office run.
The current profile is m5-host-v1 with [IR 4 host roots](decisions/0013-m5-host-contract.md) on top
of the [IR 3 language extensions](m4-language.md); unselected corpus cases and new oracle acquisition
remain M9 work.

| Stage | Work in tiny-vba | Acceptance gate |
|---|---|---|
| Setup — complete | Workspace/src layout, CLI refusal, adapted agent/skill files, docs, CI, corpus gitlinks/pins/hashes | Native scaffold checks and library WASM compilation; no application dependency |
| M0 — contract and test harness — complete | Source IDs/spans/diagnostics; versioned scenario parser/validator; at least 20 independently authored pure-language cases; foreign-call ownership prototype and deterministic double | CLI validates valid fixtures and rejects malformed ones; missing/empty/null distinctions; fake call/error/wait round trip; no silent execution stub |
| M1 — frontend — complete for reviewed subset | Lexer, source-preserving parser, .bas/.cls metadata, procedures/declarations/assignment/If/For/calls; positive/negative fixtures | Exact error spans and same-line statement IDs; bounded recovery; run a reviewed grammar-corpus subset and report unsupported syntax |
| M2 — binder and bytecode — complete for reviewed subset | Scopes, Option Explicit, locals/procedure binding, argument subset, lowering, verifier/disassembler | Compile/disassemble examples; reject invalid branches/slots/stack effects; deterministic source maps |
| M3 — executable VM + debugger — complete for reviewed subset | Minimal typed values/arithmetic, explicit frames, loops/calls, budgets; generic foreign protocol; break/step/inspect/edit/resume | Host-free Answer() returns Long 42; generic host call trace correct; pause/edit/resume without replay; infinite loop cancels; native and WASM execution smoke tests |
| M4 — semantic depth — complete | Coercions/numeric widths, special states, Currency/Date, arrays, whole-array/cell ByRef, optional/named/ParamArray arguments, Option Base, handlers, intrinsics and module/Static state | Original growing external-subset gate: 153 reviewed ReflexGrid cases with published Microsoft Excel expectations pass exact typed/error comparisons; 44 authored scenarios; negative IR/version tests and native/WASI execution; [evidence](m4-oracle-report.md) |
| M5 — fork integration contract — in-repo contract complete | Versioned foreign/value/debug APIs (`API_VERSION`/`ProtocolVersion`/`check_protocol`), hidden IR 4 host-root imports through `CompileOptions::imports`, `HostBinding::for_program`, testkit bridge driver (`run_to_boundary`/`statement`) and a fake-host break/edit/resume proof with exactly one dispatch; workbook FillCells, undo/recalc and permissions stay fork-owned | crates/vba-foreign/tests/protocol.rs, crates/vba-compiler/tests/compiler.rs, crates/vba-vm/tests/host.rs and tests/embedding.rs; [ADR 0013](decisions/0013-m5-host-contract.md) |
| M6 — project source decoding — complete | Read-only MS-OVBA decoder, encoding/attributes/references/codenames; malformed-stream and budget tests | 14 controlled native/WASI tests decode source/reference/attribute metadata, preserve module/code-name identities and diagnostic locations, reject unsupported/protected/corrupt/over-budget inputs; no auto-execution; [ADR 0014](decisions/0014-m6-project-decoding.md) |
| M7 — callbacks and lifecycle — in-repo contract complete | Bounded generic callback queue (`schedule_callback`/`run_next_callback`, FIFO, `Limits.callbacks`), static reset (`reset_state`), host root rebinding, stale-reference error propagation, cancellation/disconnect drain | Native/WASI callback tests cover reentrancy refusals, permanent closure, non-destructive delivery validation, shared payload budgets and reset/restart (crates/vba-vm/tests/callbacks.rs, [ADR 0015](decisions/0015-m7-callbacks-and-lifecycle.md)); event order and workbook partial-side-effect acceptance remain fork-owned and are not claimed here |
| M8 — editor/debugger UI and developer tools — in-repo gate complete | Toolkit-independent IDE state, reusable egui panels, standalone native/web shell, scriptable CLI debugger, executable render/browser harnesses | Real edit/run/break/step/inspect/edit/resume tests; generic-host exactly-once/cleanup proof; native/browser runs and inspected original snapshots; accessibility/contrast/layout tests and native/WASM gates; [M8 evidence and explicit broader-spec boundaries](ui/m8-acceptance.md) |
| M9 — conformance and release | Curate more library/corpus cases, oracle goldens, importer/parser/VM fuzz targets, performance and native/WASM CI | Per-feature report with exact tested versions and all status classes; prior passing tests do not regress |
| M10 — classic editor/debugger completion — planned | Remaining classic editing/layout, metadata-backed IntelliSense, advanced verified debugging, DAP and fidelity checks | Native/browser classic workflow, safety, accessibility and visual gates; [scope](ui/post-m8-roadmap.md#m10--classic-editor-code-aids-and-advanced-debugger) |
| M11 — classes and UserForm editor — planned | Prerequisite class/document semantics, shared form model, Toolbox/designer/Properties/Format and real control events | Authored form design/save/reopen/event debugging and runtime/cleanup tests; [scope](ui/post-m8-roadmap.md#m11--classdocument-support-and-userform-editor) |
| M12 — native Craft extensions and enhancements — planned | Versioned extension API/manager, opt-in non-Microsoft editor features and Python translation preview | Capability/unload/isolation, preview/undo and supported-subset conversion tests; Classic mode works with extensions disabled; [scope](ui/post-m8-roadmap.md#m12--native-craft-extensions-and-non-microsoft-enhancements) |
| M13 — optional Copilot-style assistance — planned | Opt-in inline/chat assistance, provider/privacy policy, source-aware previews and approved retrieval | Permission/offline/injection/cancellation/undo/shortcut tests; no implicit execution; [scope](ui/post-m8-roadmap.md#m13--optional-copilot-style-assistance) |
| M14 — host integration acceptance — planned, cross-repository | Shared UI in Gridcraft/Wordcraft; real document/events, trust, storage, signatures and host lifecycle | Executable evidence in each owning fork; baseline may proceed from M5/M7/M8 without waiting for AI; [scope](ui/post-m8-roadmap.md#m14--gridcraft-and-wordcraft-integration-acceptance) |

## Post-M8 scope allocation

The [M10–M14 roadmap](ui/post-m8-roadmap.md) assigns every remaining UI/extension feature family
from the supplied specification, with owners, prerequisites and acceptance gates. M8 remains the
delivered baseline; M9 releases/conforms that supported baseline. Later milestones are planned,
not implemented, and do not retroactively enlarge M8 or move Office ownership into this repository.

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

1. M4 language and external-oracle gates are implemented under ADRs 0011/0012; preserve explicit profile refusals and authored evidence.
2. M5: stabilize foreign/value/debug API versioning and reusable bridge tests; fork-owned FillCells proof. The in-repo M5 contract is implemented under [ADR 0013](decisions/0013-m5-host-contract.md); the workbook-level FillCells gate remains fork-owned.
3. M6 source decoding is complete under [ADR 0014](decisions/0014-m6-project-decoding.md) and M7 bounded generic callbacks/lifecycle under [ADR 0015](decisions/0015-m7-callbacks-and-lifecycle.md); M8 now implements the reusable graphical editor/debugger under [ADR 0016](decisions/0016-m8-embeddable-editor.md); next M9 conformance/release work and separate fork integration.
4. Extend vetted scenario/oracle coverage through M9, retaining NOT_RUN for unexecuted external material.

5. After the applicable M9 baseline, deliver M10 classic completion, M11 forms, M12 native
   extensions/translation and M13 optional AI according to their dependency gates. M14 tracks
   fork-owned integration and can progress independently of optional extensions/AI. See the
   [post-M8 roadmap](ui/post-m8-roadmap.md) for the authoritative deferred scope.

## Measurement and decisions

Maintain [compatibility](compatibility-matrix.md), [progress](implementation-progress.md) and source evidence per feature.
Use PASS, MISMATCH, UNSUPPORTED, BLOCKED_BY_POLICY, INVALID_FIXTURE, CRASH, TIMEOUT and NOT_RUN.
Initial integrated benchmark target: 100 asserted cases and 20 independent workbooks, assembled gradually.
Office-specific 40/30/15/10/5 category targets from the supplied design belong in the fork suite;
tiny-vba reports language/debugger/import results separately. No initial global pass-percentage gate.

Open decisions and selected defaults are in [ADR 0001](decisions/0001-repository-scope.md).
No substantial UserForms/COM/DLL, P-code, project writing or full Word work before the vertical slice passes.
