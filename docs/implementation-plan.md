# Plan of action

Updated 2026-10-28. This plan builds the interpreter and editor/debugger UI here; document/Office integration stays external.
The GridCraft M14 baseline and exposed-profile audit are implemented in the fork; follow the
[staged integration work log](gridcraft-integration.md) to resume or verify it.
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
| M8 — editor/debugger UI and developer tools — complete for the defined M8 scope | Toolkit-independent IDE state, reusable egui panels, standalone native/web shell, scriptable CLI debugger, executable render/browser harnesses | Real edit/run/break/step/inspect/edit/resume tests; generic-host exactly-once/cleanup proof; native/browser runs and inspected original snapshots; accessibility/contrast/layout tests and native/WASM gates; [M8 evidence and explicit broader-spec boundaries](ui/m8-acceptance.md); [completion-audit corrections and verification](ui/m8-completion-audit.md) |
| M9 — conformance and release | Curate more library/corpus cases, oracle goldens, importer/parser/VM fuzz targets, performance and native/WASM CI | Per-feature report with exact tested versions and all status classes; prior passing tests do not regress |
| M10 — classic editor/debugger completion — implemented profile | Classic editing/layout, metadata-backed code aids, bounded advanced debugging, browser file transfer and optional DAP | [Requirement-to-test map and limits](ui/m10-implementation.md); native/browser/WASM, keyboard, accessibility and original visual gates; [ADR 0018](decisions/0018-m10-classic-tools-and-debugger-policy.md) |
| M11 — classes and UserForm editor — planned | Prerequisite class/document semantics, shared form model, Toolbox/designer/Properties/Format and real control events | Authored form design/save/reopen/event debugging and runtime/cleanup tests; [scope](ui/post-m8-roadmap.md#m11--classdocument-support-and-userform-editor) |
| M12 — native Craft extensions and enhancements — planned | Versioned extension API/manager, opt-in non-Microsoft editor features and Python translation preview | Capability/unload/isolation, preview/undo and supported-subset conversion tests; Classic mode works with extensions disabled; [scope](ui/post-m8-roadmap.md#m12--native-craft-extensions-and-non-microsoft-enhancements) |
| M13 — optional Copilot-style assistance — planned | Opt-in inline/chat assistance, provider/privacy policy, source-aware previews and approved retrieval | Permission/offline/injection/cancellation/undo/shortcut tests; no implicit execution; [scope](ui/post-m8-roadmap.md#m13--optional-copilot-style-assistance) |
| M14 — host integration acceptance — GridCraft baseline implemented; broader cross-repository scope planned | Shared GridCraft UI and bounded workbook adapter implemented; WordCraft, broader events/forms/signatures remain planned | [GridCraft baseline evidence and limits](gridcraft-integration.md); [broader scope](ui/post-m8-roadmap.md#m14--gridcraft-and-wordcraft-integration-acceptance) |
| M15 — host types, default members and collections — planned | Generic host type/reference/constant binding, default-member dispatch and enumeration; fork supplies Excel metadata and collections | Unchanged typed declarations and collection/default-property source execute in GridCraft; generic native/WASM tests remain host-neutral |
| M16 — Excel range and worksheet coverage — planned, fork-owned | Lightweight references separate from materialization; Cells/Rows/Columns/Item, relative and named ranges, navigation | Unchanged authored range workflows pass with bounds, protection, undo/recalc and native/WASM checks |
| M17 — Excel identity, values and member semantics — planned | Stable host identities, active versus code-owner roots, Value/Value2, Address, formatting and application settings | Exact typed results, signatures, side effects and lifetime behavior match the declared Excel profile; no success stubs |
| M18 — document modules, events and Range-aware UDFs — planned | Reuse M11 language/class prerequisites and M7 callbacks; fork supplies codenames, event ordering, controls and UDF context | Authored unchanged workbook/worksheet handlers and Range-aware Functions run, pause/resume and clean up safely |
| M19 — unchanged-workbook and package acceptance — planned | Reviewed licensed workbook suite, load/compile/run/save reporting, controls/package preservation and separately reviewed VBA writeback | Per-workbook source identity, exact effects/results/events and package evidence; no general compatibility claim from subset tests |

## Excel source-compatibility milestones (M15–M19)

These milestones allocate the remaining work from the 2026-10-28
[GridCraft audit](https://github.com/matt123p/gridcraft/blob/97123e580deb309329668a994ebc6004030e1a86/docs/vba-excel-compatibility.md).
All are **planned, not implemented**. M14.10 already corrected member ownership,
two-endpoint Worksheet.Range, ownership links and Range.Row/Column/Count; those
fixes remain delivered. M14 is the host-integration umbrella; M15–M19 make its
remaining Excel compatibility work independently implementable and testable.
WordCraft acceptance remains in M14 and is not implied by Excel results.

### M15 — Host types, default members and collections

Owner: tiny-vba generic language/foreign contracts; GridCraft Excel metadata and objects.
Depends on M5/M7 and the audited M14 baseline; coordinate metadata UI with M10.

- **M15.1:** Specify and implement host-supplied type namespaces, reference and
  constant binding so declarations such as `As Worksheet` and `As Excel.Range`
  compile unchanged. Preserve Object/Variant late binding and explicit missing-reference
  diagnostics; no Excel names or dependencies enter the shared engine.
- **M15.2:** Implement default Get/Let/Set and parameterized Item dispatch,
  including fixed-member import chaining, named/omitted arguments, ByRef
  obligations, return types and exact errors. Do not discard member applications.
- **M15.3:** Add bounded host collection enumeration and the language constructs
  needed by selected unchanged collection workflows. GridCraft supplies actual
  Worksheets/Sheets collections, Count/Item and supported sheet kinds.

Acceptance: generic fake-host regressions plus fork tests for typed declarations,
`Worksheets.Count`, indexed/default Item and collection loops; exact values,
argument/error behavior, stale handles and cancellation in native and actual WASM.

### M16 — Excel range and worksheet coverage

Owner: GridCraft; tiny-vba changes only for generic mechanisms proven necessary.
Depends on M15 for typed/default-member/collection source acceptance.

- **M16.1:** Separate range-reference construction from cell materialization
  budgets. Implement Cells zero/one/two-index forms and column letters, Rows,
  Columns, CountLarge and Item, with Excel argument names and coercion rules.
- **M16.2:** Implement relative Range.Range, Offset/Resize, local/qualified and
  named references, and explicit multi-area behavior. Preserve sheet ownership
  and checked arithmetic without allocating whole-sheet cell arrays.
- **M16.3:** Add UsedRange, End and Find/navigation workflows; route structural
  edits through native engine commands, including protection and undo/recalc.

Acceptance: authored unchanged source covers common last-row, iteration,
relative/named range and boundary workflows; exact addresses/types, overflow/errors,
resource refusals and mutation atomicity pass natively and in WASM.

### M17 — Excel identity, values and member semantics

Owner: GridCraft document identity, conversion and command layers; shared VBA
coercion stays in tiny-vba. Depends on M15 and applicable M16 reference support.

- **M17.1:** Supply stable worksheet identities and dynamic roots; distinguish
  ThisWorkbook from ActiveWorkbook and support active sheet/cell/selection without
  retargeting stale handles after sheet rename/reorder or document switching.
- **M17.2:** Match Value versus Value2 Date/Currency behavior, typed errors,
  scalar/array assignment and formula/text conversion. Complete Address's optional
  arguments and supported A1/R1C1, relative and external forms.
- **M17.3:** Implement required formatting objects, application settings and
  calculation/member signatures with real engine behavior. Catalogue completion
  must describe actual support; unsupported operations retain explicit failures.

Acceptance: exact value tags, array bounds, signatures/named arguments, owner
identity, selection and calculation side effects; document/sheet lifecycle and
undo/protection regressions. Record public documentation versus observed evidence.

### M18 — Document modules, events and Range-aware UDFs

Owner: tiny-vba reusable class/document semantics under M11 and callback lifecycle
under M7; GridCraft document bindings, events, controls and calculation context.
Depends on those prerequisites and relevant M15–M17 capabilities; do not duplicate M11.

- **M18.1:** Execute supported workbook/worksheet modules with code-name roots,
  correct signatures and code-owner context while keeping import inert.
- **M18.2:** Deliver Workbook_Open/Worksheet_Change and selected document/control
  events with defined ordering, EnableEvents, reentrancy/recursion budgets,
  cancellation and partial-side-effect policy. Connect M11 forms where required;
  arbitrary ActiveX/COM remains a separately reviewed capability.
- **M18.3:** Add Range/reference-aware UDF arguments and Caller/Volatile/context
  behavior, persistent state and recalculation rules where supported; enforce the
  declared UDF mutation policy without silently substituting value arrays.

Acceptance: authored unchanged event and UDF source executes against real
GridCraft state; pause/inspect/resume, exactly-once dispatch, reset/close/trust
revocation and calculation dependencies pass. No untrusted automatic open execution.

### M19 — Unchanged-workbook and package acceptance

Owner: GridCraft acceptance/package layer, coordinated with tiny-vba M9 evidence
and source-import contracts. Depends on the capabilities required by each specimen,
normally M15–M18 and M11 for forms; optional extensions/AI are not prerequisites.

- **M19.1:** Curate licensed, reviewed workbooks with declared language/object/
  reference/control requirements and independently established expected results.
  Use public documentation and permitted observations, never proprietary code or
  unlicensed fixtures. Keep candidate/unrun material separate from executable cases.
- **M19.2:** Report load, compile, run and save separately for unchanged source;
  compare source identity, exact typed values/errors, effects/events and package
  relationships/payloads. Cover controls/forms, trust and lifecycle explicitly.
- **M19.3:** Design binary project writeback and signature preservation/invalidation
  as a separate reviewed contract before implementation. Original-byte preservation
  is not edited-source writeback; no protection/P-code bypass or unrestricted add-ins.

Acceptance: per-workbook feature/profile/version and evidence records with PASS,
MISMATCH, UNSUPPORTED and NOT_RUN distinguished. Run both repositories' required
CI/WASM gates and actual fork workflows. Publish remaining incompatibilities;
subset success never establishes universal Excel compatibility.

## Post-M8 scope allocation

The [M10–M14 roadmap](ui/post-m8-roadmap.md) assigns every remaining UI/extension feature family
from the supplied specification, with owners, prerequisites and acceptance gates. M8 remains the
delivered baseline, with [completion-audit corrections](ui/m8-completion-audit.md); M9 releases/conforms that supported baseline. Later milestones are planned,
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
3. M6 source decoding is complete under [ADR 0014](decisions/0014-m6-project-decoding.md) and M7 bounded generic callbacks/lifecycle under [ADR 0015](decisions/0015-m7-callbacks-and-lifecycle.md). M8 corrections are complete; M10 extends the reusable graphical IDE under [ADR 0018](decisions/0018-m10-classic-tools-and-debugger-policy.md). Preserve their regression gates while proceeding with M9 conformance/release work and separate fork integration.
4. Extend vetted scenario/oracle coverage through M9, retaining NOT_RUN for unexecuted external material.

5. M10's implemented profile is documented in its acceptance map. Next UI work is M11 forms, M12 native
   extensions/translation and M13 optional AI according to their dependency gates. M14 tracks
   fork-owned integration and can progress independently of optional extensions/AI. See the
   [post-M8 roadmap](ui/post-m8-roadmap.md) for the authoritative deferred scope.

6. For unchanged Excel source, start M15.1's generic host-type contract and typed
   declaration proof, then M15.2/M15.3. Progress through M16–M19 by their dependencies;
   keep the [integration work log](gridcraft-integration.md) current at each stage.
   This compatibility track can advance independently of optional M12/M13 work.

## Measurement and decisions

Maintain [compatibility](compatibility-matrix.md), [progress](implementation-progress.md) and source evidence per feature.
Use PASS, MISMATCH, UNSUPPORTED, BLOCKED_BY_POLICY, INVALID_FIXTURE, CRASH, TIMEOUT and NOT_RUN.
Initial integrated benchmark target: 100 asserted cases and 20 independent workbooks, assembled gradually.
Office-specific 40/30/15/10/5 category targets from the supplied design belong in the fork suite;
tiny-vba reports language/debugger/import results separately. No initial global pass-percentage gate.

Open decisions and selected defaults are in [ADR 0001](decisions/0001-repository-scope.md).
No substantial UserForms/COM/DLL, P-code, project writing or full Word work before the vertical slice passes.
