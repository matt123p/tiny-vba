# ArtCraft VBA Compatibility — Combined Architecture and Implementation Plan

**Status:** Proposed consolidated architecture and executable roadmap  
**Date:** 26 October 2026  
**Primary target:** Existing Excel `.xlsm` workbooks running against GridCraft  
**Secondary target:** Word `.docm` projects running against WordCraft  
**Implementation:** Rust, native and WebAssembly where supported  
**New-code licence target:** `MIT OR Apache-2.0`, subject to dependency/source review  
**Audience:** Human maintainers and implementation agents

## 1. Executive decision

The project contains **two independent products** that should be built together but not conflated:

1. **Craft Office Automation (`craft-automation`)** — the language-independent implementation of the Microsoft Excel/Word automation object model. It includes Office object definitions, dispatch, host capabilities, object identity, events, macro-project attachment and integration adapters to GridCraft/WordCraft. It does **not** parse or execute VBA.
2. **Rust VBA Interpreter (`vba-rs`)** — a clean-room VBA compiler and pausable interpreter with Microsoft-compatible VBA semantics, project-source extraction and a debugger. It knows nothing about GridCraft/WordCraft internals; a thin integration bridge connects it to `craft-automation`.

The existing **GridCraft** and **WordCraft** repositories remain the owners of document state, user-interface integration, undo, recalculation, editing and OOXML read/write. They must not become dependent on the VBA parser or value semantics in their low-level crates.

**Decisions to freeze before implementation:**

- In-process **typed Rust traits and object handles**, not JSON-RPC/HTTP/MCP, for the hot interpreter-to-document path.
- **One canonical Office member registry and one canonical `AutomationValue`/`ObjectHandle` definition** in `craft-automation`. The interpreter has its *own* `VbaValue` because VBA types/coercion differ, and a single explicit mapping at the bridge.
- **One Office host contract**, owned by `craft-automation`; no independent Excel object system or parallel member catalogue in `vba-rs`.
- **A host-independent bytecode VM**, with an explicit execution stack and debug checkpoints from the beginning. DAP is a later adapter to that debugger, not the debugger itself.
- **Excel first**. Prove `Range`, `Cells`, collections and `.xlsm` under GridCraft before spending substantial effort on Word coverage, UserForms, COM or broad API completeness.
- **Test-observed compatibility**, with pinned Microsoft specifications/reference data, source provenance and a feature-by-feature coverage report. Unsupported features fail explicitly.
- **Fresh source implementation. Do not reuse or derive code from `craft-vba`.** Other interpreters may inform comparative research, subject to licence/clean-room rules, but are not implementation bases.

**Product promise (qualified):** opening an `.xlsm` in GridCraft should eventually allow an ordinary, supported VBA macro to run against the actual workbook, subject to **explicit macro permission**. A valid file does not imply unconditional execution; platform-specific COM, UserForms, Windows API declarations and unsupported object members remain visible limitations.

## 2. Review of the three existing designs

This overview consolidates these project design files:

- [`office_automation_architecture.md`](office_automation_architecture.md) — shared Office object model, dynamic dispatch, typed host contracts, events, document lifecycle, security and application adapters.
- [`office_api_reference_and_generation.md`](office_api_reference_and_generation.md) — Microsoft source references, versioned object-model schema, reviewed metadata, code generation and conformance scaffolding.
- [`vba-interpreter-rust-implementation-spec.md`](vba-interpreter-rust-implementation-spec.md) — VBA parser/binder, VM, runtime types, debugger, `.xlsm` project parser and differential test strategy.

### 2.1 What is already consistent

The documents agree on Rust; an independent implementation; native calls rather than internal RPC; separation of Excel/Word behavior from VBA language behavior; a shared Office model usable by Python/JavaScript; a host-independent interpreter; GridCraft first; a debugger designed into the VM; strict build/layering constraints; and testing against Microsoft behavior.

### 2.2 Necessary reconciliations

| Topic | Potential inconsistency in separate plans | Consolidated decision |
|---|---|---|
| Host interface | Interpreter proposes `vba-host::AutomationHost`; Office layer already specifies dispatcher/host traits | **Office layer owns the Office API**. VBA declares only its minimal generic *foreign-call* consumer interface; `vba-office-bridge` adapts it to Office dispatch. |
| Values | Both plans define general-purpose value/handle types | `AutomationValue` and Office `ObjectHandle` live only in `automation-types`; `VbaValue`, VBA references and VBA class instances stay in `vba-values`. Convert once, explicitly. |
| Member catalogue | VBA binding might independently hard-code Excel member signatures | Single reviewed catalogue and generated registry in `craft-automation`. VBA binder may consume a versioned **read-only metadata view** for early binding and named arguments. |
| Macro projects | Office design and interpreter both discuss `vbaProject.bin` | `office-project` handles opaque attachment, identity, trust and OOXML package coordination; `vba-project` handles **read-only decoding into module source and VBA metadata**. Actual OOXML readers/writers stay in Craft hosts. |
| Debugger dependency | Debugger and VM could depend on each other | Put inspection hooks and pause/step protocol into a low-level VM-facing interface. `vba-debug` controls VM execution; `vba-dap` wraps `vba-debug`. No cycle. |
| Events | Host emits events but needs to call code defined in interpreter | Office layer defines a language-neutral event sink and scheduler contract. The composition layer registers VBA callbacks; Office crates never import the VBA VM. |
| Undo | A GridCraft grouped command may roll back edits on failure | Distinguish **atomic mutation**, **undo grouping** and **macro failure policy**. Excel-compatible default preserves completed edits on an ordinary VBA error; rollback is a separate optional mode. |
| Roadmap | One design prioritises broad API generation while another prioritises VM | **Thin vertical slice first**: versioned schema foundation, minimal dispatch/host, minimal compiler/VM/debugger, fake-host integration; expand API generator and language semantics in parallel afterward. |
| Tooling | Python/JS might be routed through VBA VM | Both bind **directly to Office dispatch**; never through the VBA compiler or `VbaValue` conversions. |

### 2.3 Implementation risks requiring explicit design

1. **File round-trip is not the same as executable VBA.** Current Craft documentation reports preservation of macro payloads in macro-enabled formats, but verify with real and synthetic fixtures. Supporting an `.xlsm` also requires source decoding, codenames, events, reference binding, project lifetime and trust.
2. **Office semantics are more than a command registry.** `Range.Value` scalar/matrix behavior, default `Item` members, named/omitted arguments, live proxy objects and Excel error conditions should reside in the Office facade. Command IDs are not a replacement for that facade.
3. **Re-entrancy and host borrowing can cause subtle bugs.** Do not hold a mutable GridCraft or WordCraft session borrow across a VM pause, event callback, dialog, or yield. Host calls acquire/release access for one operation; events are queued and delivered at safe points.
4. **Type fidelity matters from the outset.** `Empty`, `Null`, missing argument, `Nothing`, `CVErr`, dates, Currency, arrays with lower bounds, named arguments and property Let/Set cannot be collapsed into JSON `null` or a single number/string type.
5. **The debugger cannot be retrofitted safely onto arbitrary recursive evaluation.** Stable source spans/statement IDs, explicit frames, by-reference locations and stoppable bytecode execution are prerequisites.
6. **Security and Microsoft-compatible execution are distinct modes.** A Rust VM is memory-safe, not automatically sandboxed. Capabilities, user consent, cancellation, resource budgets, and stronger isolation for truly untrusted macros must be planned separately.
7. **The reference catalogue needs review.** Microsoft's documentation and .NET interop signatures are complementary, not interchangeable. Generated APIs must never report false success for an unimplemented member.
8. **WASM is a portability goal, not evidence of safe multitasking.** In-browser execution needs instruction budgets and cooperative yields; OS-dependent functions belong behind optional backends.

## 3. End-to-end architecture

```text
                 .xlsm or .bas/.cls               .docm (later)
                        |                              |
          GridCraft OOXML importer          WordCraft OOXML importer
                        |                              |
              office-project attachment/trust/lifecycle
                        |                              |
            vba-project source extraction (.xlsm/.docm)
                        |
             vba-syntax -> vba-semantics -> vba-ir
                        |
                 vba-values + vba-runtime
                        |
                 pausable vba-vm <----> vba-debug <----> vba-dap / IDE
                        |
                 vba-office-bridge  [only conversion/interop boundary]
                        |
       +----------------+------------------------------------------+
       |          craft-automation shared libraries                |
       | automation-types / metadata / dispatch / lifecycle       |
       | office-excel / office-word / security / project           |
       +----------------+------------------------+-----------------+
                        |                        |
             typed SpreadsheetHost           typed DocumentHost
                        |                        |
              GridCraft adapter             WordCraft adapter
                        |                        |
              GridCraft Session             WordCraft Session
                actual workbook               actual document

          Python binding -------+ 
          JavaScript binding ---+---> craft-automation Office dispatch
                                 (neither depends on the VBA VM)
```

### 3.1 Ownership of each layer

| Concern | Authoritative owner |
|---|---|
| XLSX/DOCX/OOXML package and host-side document storage | GridCraft / WordCraft |
| Workbook values, formulas, calculation, selection, undo and formatting | GridCraft engine/model |
| Word document stories, editing, layout, selection and undo | WordCraft engine/doc |
| Office API names, signatures, defaults, enums and member IDs | `craft-automation` metadata/schema |
| Office `Application`/`Workbook`/`Range`/`Document` object behavior | `office-excel` / `office-word` |
| Document host capability traits, mutation/event protocol | `automation-host` / `automation-lifecycle` |
| Language-neutral automation values and live Office handles | `automation-types` |
| VBA syntax, scoping, types, coercion, functions, error handlers | `vba-rs` compiler/runtime |
| Executable bytecode, frames, safe points and step state | `vba-vm` |
| Debugger controls/inspection | `vba-debug`, with optional `vba-dap` |
| VBA-to-Office value/object/exception mapping | `vba-office-bridge` |
| `vbaProject.bin` preservation/trust | Craft import/export + `office-project` |
| `vbaProject.bin` read-only decoding into source modules | `vba-project` |
| Editor UX / macro picker / consent screens | Craft applications |

**Dependency rule:** source code in shared layers must never depend on a higher layer merely to call back into it. Use registrations, callbacks or narrow traits supplied by the top-level application composition crate.

## 4. Recommended repository strategy

**Use two new repositories plus small adapter changes in the two existing application repositories.** This preserves independent testing/release, avoids one giant Cargo dependency cycle, and allows the Office automation project to serve Python/JS or other ArtCraft applications. A *development integration workspace* can pin all four projects together for cross-repository CI without becoming a shipping library.

### 4.1 Repository A — `craft-automation`

```text
craft-automation/
├── Cargo.toml                         # Rust workspace, pinned MSRV, lint policy
├── LICENSE-MIT / LICENSE-APACHE
├── README.md                          # overall contract and quick start
├── AGENTS.md                          # clean-room, crate layering, PR gates
├── docs/
│   ├── architecture.md                # canonical combined Office-layer design
│   ├── host-contract.md               # frozen low-level host traits
│   ├── dispatch-and-values.md         # identity, args, conversions, errors
│   ├── events-and-lifecycle.md
│   ├── project-attachment.md
│   ├── security.md
│   ├── office-api-coverage.md          # generated from verified tests
│   ├── decisions/                     # ADRs, one per nontrivial decision
│   ├── office_automation_architecture.md
│   └── office_api_reference_and_generation.md
├── crates/
│   ├── automation-types/              # neutral values, IDs, handles, errors, args
│   ├── automation-metadata/           # generated stable Office member descriptions
│   ├── automation-host/               # typed SpreadsheetHost/DocumentHost + extensions
│   ├── automation-dispatch/           # resolve/get/let/set/invoke/enumerate
│   ├── automation-lifecycle/          # contexts, event queue, undo/batch policies
│   ├── automation-security/           # permissions, budgets, consent contracts
│   ├── office-common/                 # shared Office enums/types and semantics
│   ├── office-excel/                  # Excel Application/Workbook/Worksheet/Range...
│   ├── office-word/                   # Word Application/Document/Range... (later)
│   ├── office-project/                # opaque VBA attachment + document lifecycle
│   ├── automation-testkit/            # fake hosts and reusable contract tests
│   └── automation-wit/                # OPTIONAL future sandbox/component boundary
├── schema/
│   ├── schema-version.toml
│   ├── reviewed/{excel,word,office}/  # reviewed source-controlled YAML
│   ├── overrides/                     # defaults, signature fixes, event notes
│   └── review-queue.yaml
├── references/
│   ├── vba-docs.lock.toml             # pinned upstream SHA + licences
│   └── NOTICE-reference-sources.md
├── tools/
│   ├── office-api-import/             # dev-only Markdown/interop ingestion
│   └── office-api-codegen/            # deterministic registry/test generation
├── examples/
│   ├── fake-excel/
│   └── fake-word/                      # later
├── tests/
│   ├── values-and-dispatch/
│   ├── excel-object-model/
│   ├── events-and-lifecycle/
│   ├── security/
│   └── generated-metadata/
└── .github/workflows/ci.yml
```

**Build dependency shape:** `automation-types` is foundational; `automation-host` and `automation-metadata` depend on neutral types; dispatch depends on values + metadata; Office object-model crates depend on dispatch and host contracts; lifecycle/security are reusable services. `automation-testkit` may depend on all public contracts but no production crate depends on testkit. The importer/code generator is development-only; normal builds never download docs or require Office installed.

The `office-word` crate and a wide catalog of source entries may exist initially with explicit `not_implemented` status. Do not develop the Word facade at parity with Excel during the first Excel milestone.

### 4.2 Repository B — `vba-rs`

```text
vba-rs/
├── Cargo.toml
├── LICENSE-MIT / LICENSE-APACHE
├── README.md
├── AGENTS.md
├── docs/
│   ├── architecture.md
│   ├── language-semantics.md
│   ├── bytecode-and-debugger.md
│   ├── office-bridge-contract.md
│   ├── macro-project-import.md
│   ├── conformance-methodology.md
│   ├── compatibility-matrix.md
│   ├── decisions/
│   └── vba-interpreter-rust-implementation-spec.md
├── crates/
│   ├── vba-diagnostics/               # source IDs, spans, structured failures
│   ├── vba-syntax/                    # lexer, VBA parser, syntax tree
│   ├── vba-semantics/                 # symbols, scope, calls, name/type binding
│   ├── vba-ir/                        # verified IR/bytecode and source maps
│   ├── vba-values/                    # VBA Variant, arrays, l-values, VBA objects
│   ├── vba-runtime/                   # coercions, operators, builtins, Err
│   ├── vba-foreign/                   # MINIMAL host-neutral VM call/suspend protocol
│   ├── vba-vm/                        # explicit frames, scheduler, bytecode runner
│   ├── vba-debug/                     # pause/step/breakpoint/inspection controller
│   ├── vba-dap/                       # optional DAP frontend, no VM dependency on DAP
│   ├── vba-project/                   # read-only MS-OVBA module/project extraction
│   ├── vba-office-bridge/             # sole dependency on craft-automation APIs
│   └── vba-conformance/              # scenario runner, oracle diffs, reports
├── apps/
│   └── vba-cli/                       # parse/check/compile/run/debug/inspect/test
├── examples/
│   ├── pure-vba/
│   └── fake-excel/
├── tests/
│   ├── syntax/
│   ├── binder/
│   ├── runtime/
│   ├── debugger/
│   ├── fake-host/
│   ├── project-import/
│   ├── oracle-goldens/
│   └── regression/
├── fuzz/
└── .github/workflows/ci.yml
```

**Crucial naming choice:** `vba-foreign` is **not a second Office `AutomationHost`**, member registry, Excel facade or object model. It declares only how the VM requests a foreign method/property, supplies arguments and receives a value/error/optional wait. The actual implementation lives in `vba-office-bridge`, which uses the public `automation-dispatch` contract and generated metadata. Other possible foreign hosts can implement the same protocol without changing VM instructions.

**Value separation:** `vba-values::VbaValue` preserves VBA `Integer`, `Long`, `Currency`, `Date`, `Decimal`, `Empty`, `Null`, `Missing`, `CVErr`, `Nothing`, class instances and ByRef locations. `automation-types::AutomationValue` is the **language-neutral call boundary**. The bridge maps them using member signatures. It references Office `ObjectHandle` rather than inventing another registry for foreign Office objects; internal VBA class instances remain VM-owned.

### 4.3 Existing application repositories

```text
gridcraft/
  crates/automation-adapter/           # SpreadsheetHost implemented over real Session
  crates/.../                          # small neutral edit/event hooks if required
  apps/gridcraft/                       # composition, macro picker, enablement, debugger
  apps/gridcraft-cli/                   # headless macro invocation, test entry points
  tests/automation-integration/         # workbook, recalc, undo, .xlsm regressions

wordcraft/                               # implement after Excel-first gates
  crates/automation-adapter/           # DocumentHost implemented over Session/doc
  crates/.../                          # narrow text-range/edit-event hooks if needed
  apps/wordcraft/                       # composition and macro UI
  tests/automation-integration/
```

The adapter should sit at a **permitted upper-level integration layer**; neither `gridcraft-core/model/calc` nor `wordcraft-doc` should depend on `office-excel`, `office-word` or `vba-vm`. Use `cargo xtask layers` / the current repository's layering validator before every merge.

### 4.4 Optional development integration workspace

```text
artcraft-vba-integration/             # non-shipping CI/workbench repo; optional
├── Cargo.toml                         # local paths in dev, pinned Git revisions in CI
├── Cargo.lock
├── tests/end-to-end/
├── fixtures/self-authored/
├── scripts/check-cross-repo.sh
└── docs/compatibility-dashboard.md
```

This is useful **after** the initial host and VM boundaries stabilise. A single Cargo workspace containing all crates from all projects is not required, and may clash with the Craft projects' own workspace/layer checks. If needed, use a top-level integration crate that composes **published or git-pinned dependencies** and runs tests; do not restructure the applications for tooling convenience.

## 5. Stable contracts to define first

The following are conceptual interfaces, not frozen Rust code. Exact lifetimes/ownership must be proven with compiling tests before publishing them.

### 5.1 Office dispatch boundary (canonical)

```rust
pub trait OfficeDispatcher {
    fn resolve_member(&self, object: ObjectHandle, name: &str)
        -> Result<MemberId, AutomationError>;
    fn dispatch(&mut self, request: Invocation<'_>, ctx: &CallContext<'_>)
        -> Result<AutomationValue, AutomationError>;
    fn enumerate(&mut self, object: ObjectHandle, ctx: &CallContext<'_>)
        -> Result<Box<dyn AutomationEnumerator>, AutomationError>;
}
```

Required operations: `Get`, `PropertyLet`, `PropertySet`, `Invoke`, indexed/default member invocation and enumeration. Member resolution should be case-insensitive as VBA expects, with named/positional/omitted arguments distinguished. `MemberId`s are stable within versioned schema releases. Add object retain/release, stale-handle invalidation, host yields/callbacks and capabilities as explicit parts of the final contract.

### 5.2 Typed document-host boundary (below Office objects)

`SpreadsheetHost` and `DocumentHost` are **small sets of typed primitives**, not thousands of Office methods. Examples: `read_cells`, `write_cells` (rectangular/bulk), `read_formula`, `calculate`, `read_format`, `apply_format`, workbook/sheet identity and selection; for Word, story-aware text ranges, text replacement, styles and selection. Return a structured mutation receipt so recalc, events and undo can be coordinated. Host errors stay language-neutral.

Large operations must not be expanded into a JSON command per cell. Normal interactions stay native; IPC/WIT/JSON belongs only to separately designed sandbox or remote frontends.

### 5.3 VBA foreign-call adapter (above Office dispatch)

`vba-foreign` describes a generic VM action such as `InvokeForeign { target, member, invoke_kind, args }`. `vba-office-bridge` performs:

1. Resolve VBA global names such as `Application`, `ThisWorkbook`, `ActiveWorkbook`, `ActiveSheet`, `Range` or `Cells` against the **correct project and workbook context**.
2. Convert `VbaValue` arguments to `AutomationValue` while preserving `Missing`, named arguments, ByRef obligations and `PropertyLet` vs `PropertySet`.
3. Dispatch using the canonical metadata registry and Office object proxies.
4. Convert return values, `Nothing` references, `CVErr` values and host failures to VBA-compatible results or raised `Err` state.
5. Map Office enumeration to `For Each` and keep object handles valid across VM pause/resume.
6. Route capability checks and cancellation to the host. Never bypass Office dispatch for convenient, undocumented access to GridCraft internals.

The low-level VM and parser must not import `gridcraft-*`, `wordcraft-*`, `office-excel` or `office-word` types. The Office bridge is the only deliberate dependency crossing.

### 5.4 Context, events and script callbacks

- `AutomationContext`: document/project identity, revision, object registry, event state, permissions and lifecycle.
- `ExecutionSession`: compiler program revision, frames, module/static state, heap, Err/error-handler state and VM pause status.
- A **language-neutral event sink** supports event delivery, including cancellable `BeforeSave`/`BeforeClose` events with an explicit synchronous decision point. The application registers an implementation that invokes an appropriate VBA handler.
- Mutations from either user commands or macro calls must emit compatible events, with a known queue/reentrancy policy; no UI-specific event pathway.
- Host borrows are short lived; callbacks execute only at safe points. A paused debugger cannot retain `&mut GridCraftSession` or an open unbounded edit lock.
- `Application.EnableEvents` suppresses eligible Excel event execution, not workbook mutation or recalculation.

### 5.5 Undo and failure behavior

Do not treat VBA execution as a database transaction. Separate atomic host writes, undo grouping and failure handling. Ordinary Excel-like macro runtime errors should preserve earlier completed edits where Excel does; optional preview/rollback modes may exist but must be explicitly described as different semantics. Always unwind edit grouping and release resources on cancellation, debugger disconnect, workbook close, timeout or panic containment.

### 5.6 Source-package boundary

- `office-project`: opaque bytes, document attachment, reference identity, signatures/trust status, host-side load/save integration.
- `vba-project`: read-only `.bas`/`.cls`/project extraction from `vbaProject.bin` according to MS-OVBA, including code-name/module references and line/source metadata.
- GridCraft/WordCraft readers and writers: OOXML packaging, relationships, content-types and original payload preservation.
- No P-code execution; no source writeback into signed binary projects in the MVP. Changes to workbook content and to VBA binary/source are separate operations with separate signature implications.

## 6. Implementation order — one integrated roadmap

The work should proceed as **small vertical slices**, not three isolated giant implementations. Independent tasks may proceed in parallel after the interface gates below. Every phase has explicit output, tests and a stop/go criterion.

| Phase | Main owner | Goal and output | Exit criterion |
|---|---|---|---|
| **0. Contract and baseline** | Both | Pin source references; inspect actual GridCraft/WordCraft engine signatures; establish licence rules; write crate dependency DAG, fixture schema and core interface. | Both Rust workspaces compile; **no duplicate Office host/value definitions**, no cycles; 20 independently specified VBA/oracle scenarios inventoried. |
| **1. Automation foundation** | Office | `AutomationValue`, object handles, host errors, `SpreadsheetHost` fake, invocation kinds, `OfficeDispatcher` skeleton; **hand-reviewed narrow Excel schema** for `Application`/`Workbook`/`Worksheet`/`Range`. | A pure Rust example can write/read fake `Sheet1!A1` by Office dispatch; default/indexed properties and explicit unsupported errors tested. |
| **2. VBA compiler foundation** | VBA | Lexer, parser, AST/source spans, simple semantic binder, bytecode/IR verifier; executable expressions, declarations, assignments, `Sub`, `If`, `For`, calls. | `vba-cli` compiles/disassembles positive and negative fixture sources with statement-level source maps; invalid code/bytecode never crashes. |
| **3. First vertical slice** | Both | Minimal `VbaValue`/operator subset, explicit-stack `vba-vm`, `vba-foreign`, `vba-office-bridge`, fake Excel object facade; **breakpoint, step, inspect and resume**. | `FillCells` macro populates fake `A1:A10`; pause before an assignment, change `i`, resume, inspect resulting cells. This is the first major proof. |
| **4. Real GridCraft integration** | GridCraft + Office | `SpreadsheetHost` adapter over actual GridCraft engine; values/formulas, change receipts, recalc, selection, grouped history semantics; headless macro command. | One `.bas` macro changes a live workbook; values/formulas/recalc and undo invariants correct, existing `cargo xtask ci` still green. |
| **5. VBA semantic depth** | VBA | Strong Variant/coercion, integer widths, date/currency, arrays/bounds, ByRef, named/optional args, standard functions, project/static state, `On Error`/`Err`/`Resume`. | Growing Excel-oracle-golden tests compare **type, value, error number, side effects**, not just success. No silent unsupported semantics. |
| **6. OOXML `.xlsm` execution** | VBA project + GridCraft | Read attached VBA project, module and document code names, macro catalogue, explicit trust/enable, load and execute extracted source in real workbook. | Self-authored `.xlsm` opens, is inert until consent, selected macro executes; open/save checks retain macro bytes/relationships where promised. |
| **7. API metadata pipeline and useful Excel breadth** | Office | Pinned VBA-Docs importer, reviewed YAML/overrides, deterministic codegen, constants and member coverage; `Value/Value2`, formulas, collections, Offset/Resize, formatting and common operations. | `--check` codegen reproducible; supported members have host- and oracle-tested behaviors; pending signatures remain marked unresolved. |
| **8. Event correctness and project persistence** | Both | `Workbook_Open`, `Worksheet_Change`, `EnableEvents`, guarded nested events, statics/reset, macro errors vs partial writes, lifecycle closure, workbook save hooks. | Differential event-order tests pass for target Excel profile; no stuck undo groups/re-entrant mutable borrows; no auto-run for untrusted files. |
| **9. Debugger UX / DAP** | VBA + app | DAP frontend, breakpoint mapping in imported modules, variable/call-stack inspection, debugger UI/CLI; safe watch/Immediate evaluation. | Scripted DAP round-trip (`setBreakpoints`, `continue`, `stackTrace`, `variables`, `next`, `disconnect`); pause/resume survives host object calls. |
| **10. Excel conformance/release** | Both + GridCraft | Curated `.xlsm` corpus, fuzzing, native/WASM CI, performance work, bug-fixing, compatibility report. | Feature-by-feature release matrix; a pinned benchmark suite does not regress; unacceptable crash/freeze/security classes covered. |
| **11. Word and other languages** | Office + WordCraft | Word `Document`/`Range`/`Selection` host and facade, then VBA document events; separate Python/JS language adapters to dispatch. | Real WordCraft story-aware range editing and `.docm` basic macros; Python/JS can automate Excel/Word **without** a VBA VM. |

**Parallel work:** after phase 1, the Office metadata importer and the VBA compiler can be developed independently. After phase 3, GridCraft integration, VBA semantics and wider Office member implementations can proceed in parallel against a versioned contract. **Do not wait for the entire Microsoft Office catalogue before executing a 10-line VBA macro.**

### 6.1 Minimal demonstration (phase 3)

```vb
Public Sub FillCells()
    Dim i As Long
    For i = 1 To 10
        Cells(i, 1).Value = i * 10
    Next i
End Sub
```

Test flow: create fake workbook/worksheet; compile; set a breakpoint on the assignment line; run; inspect local `i`; change it at the debugger; resume; inspect exact cell results, host call sequence and absence of stale handles. Breakpoint/resume must retain VM frames and already committed writes, not re-run code from the beginning. This demonstration exercises the contract and interpreter simultaneously.

### 6.2 First useful `.xlsm` gate (phase 6)

Use a self-authored workbook containing both a standard module and a worksheet/document module; keep the initial executable macro small. Validate: OOXML package parts and code-name mapping, source extraction, visible macro list, explicit trust prompt, `ThisWorkbook` versus `ActiveWorkbook`, `Range.Value2`, formula recalc, a VBA runtime error after an earlier write, and save/reopen macro preservation. Test untrusted opening separately: **no execution**.

### 6.3 MVP scope versus later expansion

**MVP included:** `.bas` and common `.xlsm` standard modules; core flow control, procedures, basic Variant/coercion, simple ByRef and arrays where required by selected tests; Excel `Application`, `Workbook`, `Worksheet`, `Range`/`Cells`, `Value`/`Value2`, basic formula interaction, collections, macro enablement, debugger core, and stable headless tests.

**Later:** complete language edge cases; classes/default members, advanced error/event semantics, UserForms/ActiveX, COM, DLL calls, arbitrary external IO, chart/pivot automation, broad Word object model, macro editing and VBA binary rebuild. Some are inherently OS-dependent or security-restricted; compatibility should be reported, not promised.

## 7. Work breakdown for implementation agents

### Workstream A — Contract, object values and fake host

Deliver one source-controlled `automation-types` package; stable IDs/handles, error taxonomy, missing/empty/null/object distinctions, argument descriptors, `SpreadsheetHost` read/write/recalc traits and `automation-testkit` fake. Define context generation/invalidation and ownership. Publish compiling examples and ABI/metadata compatibility tests. **No GridCraft adapter yet.**

### Workstream B — Office metadata and dispatch

Define canonical schema version and exact statuses; manually encode the first Excel object slice, implement dispatch and default/member argument handling; then add pin/ingest/reconcile/codegen pipeline from Microsoft VBA-Docs with reviewed overrides. Output member IDs, constants, schema validation, coverage and test skeletons. Unimplemented methods must error explicitly and code generation must be deterministic.

### Workstream C — VBA language frontend

Implement lexer/parser with case-insensitive name lookup but source spelling preservation; distinct statements with spans (including multiple statements on one line); compile-time symbols, argument binding, `Option Explicit`, typed declarations and bytecode lowering/verification. Use invalid syntax suites from the outset. Keep locale/conditional compilation profile explicit; do not inherit the executing machine's OS architecture as the simulated VBA target.

### Workstream D — VM, runtime and debugger core

Implement a bounded bytecode interpreter with explicit frames and instruction pointer; typed VBA values, l-value/ByRef representation, normal and error-handler state, stop/resume checkpoints, step modes and watch inspection. Mock clock/random/MsgBox. Build `vba-debug` as a controller over VM inspection hooks. No UI/IDE dependencies.

### Workstream E — VBA ⇄ Office bridge

Consume `vba-foreign` from the VM and `OfficeDispatcher` from `craft-automation`. Own all cross-value/error/argument conversions and resolution of root Office objects. Implement exact property-get/let/set, collection iteration and default member behavior using metadata. This is the integration seam that both teams should test jointly. It must **not** add a second object store or reroute through command JSON.

### Workstream F — GridCraft adapter and file integration

Map typed host operations onto the **actual** GridCraft `Session`/engine API; preserve recalculation, protection, validation, dirty-state and history. Add neutral document edit notifications where the existing engine lacks them. Integrate project source decoding with package preservation; wire macro consent and run commands in UI/CLI at an upper layer. Confirm original package relations and project bytes with fixture tests. Do not trigger macros simply because the file opens.

### Workstream G — Validation/compatibility

Own manifest schema, fixtures, oracle result format, differential comparer, WASM/native/test CI and fuzz targets. Start alongside phase 0; every implemented Office member and VBA semantic feature needs a test and source evidence. Split host/API mismatches from language/runtime failures so regressions have a responsible owner.

### Workstream H — WordCraft and Python/JavaScript (after Excel proof)

Implement real story-aware `DocumentHost` and Word compatibility objects; preserve live range/selection semantics and Office event lifecycle. Separately implement Python/JS binding crates that convert their language values to `AutomationValue`. No dependency on `vba-runtime`/`vba-vm` is permitted.

## 8. Cross-project engineering rules

### 8.1 Dependency diagram (must remain acyclic)

```text
vba-syntax -> vba-semantics -> vba-ir ----+
                                          |
vba-values -> vba-runtime ---------------+--> vba-vm <-- vba-foreign
                                               ^
                                           vba-debug -> vba-dap

vba-foreign + vba-values + automation-dispatch
                      |
              vba-office-bridge
                      |
                 OfficeDispatcher
                      |
    automation-types + automation-metadata + automation-host
                      |
               office-excel / office-word
                      |
             typed host trait impls
                      |
         GridCraft / WordCraft top-level integration
```

*Arrows are illustrative of information/dependency flow, not every exact Cargo import; prevent a VM↔debugger or application↔shared-library cycle in the checked-in crate graph.*

### 8.2 Performance and state consistency

- Bulk rectangular range IO and explicit cell-block types; avoid a separate host call for each value when VBA semantics permit batching.
- Reads after writes must observe committed state; never defer a host edit past a VBA-observable read or event boundary.
- Cache resolved member metadata and compiled programs by **schema version/project source revision**; invalidate staleness predictably.
- Do not expose direct mutable workbook or document pointers to a language runtime.
- Distinguish formula text, cached result, raw scalar value and displayed formatted text.
- Define numerical/error conversion per `Value` vs `Value2`; dates and Currency must not be silently flattened.
- Word text offsets require an explicitly tested conversion policy (Word-compatible character units versus UTF-8 bytes); this is not a UI-caret simulation.

### 8.3 Safety, portability and code quality

- `#![forbid(unsafe_code)]` where compatible with Craft policies; avoid `unwrap`, `expect`, `panic`, `todo`, `unimplemented` in production execution paths.
- Bounds for source/project extraction, bytecode, arrays, calls, host operations, event nesting, memory and execution steps. Fuzz importers, parser and VM decoder.
- Explicit macro consent and least-privilege capability checks at **host** boundary. File/network/shell/COM APIs are restricted and platform-specific, not unconditionally exposed by a friendly VBA spelling.
- Ordinary macros should **preserve already completed cell edits** after a VBA exception unless a tested Microsoft behavior dictates otherwise; transactional preview is separately named.
- Stronger isolation (helper process or sandboxed component) is future work; do not mistake in-process memory-safe Rust for a complete untrusted-code sandbox.
- Use target profile such as `Excel365Windows64` for conformance while keeping host architecture independent and clearly labelling Mac/browser differences.
- Both shared crates and application adapters must pass native CI, wasm smoke checks, formatting, Clippy and the existing Craft layering gates.

### 8.4 Clean-room and licence

New source must be independently authored. **No `craft-vba` code reuse**, no copying MPL/GPL implementation material into dual-licensed code. Prefer authoritative Microsoft specifications, verifiable behaviors, self-authored fixtures and permissively licensed tooling after inspection. MicrosoftDocs/VBA-Docs uses CC BY 4.0 for docs and MIT for code samples (verify current terms); track attribution and avoid redistributing proprietary PIA/type-library binaries. Inspect each external VBA workbook's rights and security requirements before putting it into a public test corpus.

## 9. Test programme and CI release gates

### 9.1 Shared scenario format

Use one versioned, independently maintained scenario model across repos, with:

- Scenario ID, provenance/source licence, hash, target Office product/version/build, OS/locale/profile.
- Source/entry procedure or workbook artifact, setup, call sequence, capability policy, expected VBA return type/value/error and expected workbook/document diff.
- Explicit arrays (dimensions/lower bounds), object/collection shapes, date/Currency type distinctions, formula text versus evaluated values and exact error numbers where known.
- Expected status classification: `PASS`, `MISMATCH`, `UNSUPPORTED`, `BLOCKED_BY_POLICY`, `INVALID_FIXTURE`, `CRASH`, `TIMEOUT`, `NOT_RUN`.
- Versioned golden oracle output; reproducible harness under Microsoft Excel where available; tests must not silently rewrite expected outputs.

### 9.2 Test layers

| Level | Runs against | Key assertions |
|---|---|---|
| Syntax/binder | VBA source only | Grammar, names, error spans, call/argument rules |
| VM semantics | No Office host | Operators, typed results, ByRef, arrays, `Err`, deterministic exceptions |
| Fake Office | `automation-testkit` | Dispatch, object identity, defaults, collection iteration, events |
| GridCraft integration | Real engine with synthetic workbook | Cell values/formulas, recalc, undo, mutation permissions |
| `.xlsm` integration | Full import/execution/save pipeline | `vbaProject.bin`, relationships/codenames, trust and macro results |
| Excel differential oracle | Pinned Excel version | Type/value/error/event/workbook-state parity |
| Debugger | VM/fake + real host | Breakpoints, stepping, stack, watch edits, pause after host calls |
| Robustness | Fuzzed/malformed/untrusted inputs | No hangs, panics, bypassed permissions or unbounded allocations |

A reasonable **initial target** is 100 asserted cases and 20 representative independent `.xlsm` files, **not** a claim these tests already exist. Do not gate phase 1 on collecting all 120 specimens: curate at least 20 small reference cases immediately and expand alongside implementation. Track pass rates by supported feature/API, not a single global score.

### 9.3 CI contracts

**Each new repository:** fmt, Clippy `-D warnings`, Rust tests, generated-schema up-to-date check, no forbidden dependencies/code patterns, no circular crate references, native and WASM compilation/smoke (platform-gated dependencies clearly excluded), fuzz regressions and test fixture provenance verification.

**Each Craft integration PR:** existing `cargo xtask ci`, layering checker, macro package round trip, history/recalc/selection tests, cross-project pinned-contract test and safe untrusted-macro no-execute test.

**Compatibility governance:** a member is only `verified` with attributable behavior tests against an identified reference profile. Every regression from a prior passing assertion must be fixed or explicitly marked with an explanation; support count must not rise because a no-op stub exists.

## 10. Git/project collaboration conventions

- Assign **one source-of-truth repo per contract**; API signatures change via reviewed ADR plus cross-repo compatibility tests.
- Use `schema_version`, Rust crate semver, `Cargo.lock` for binary apps and pinned Git revisions/workspace dependencies in cross-repository CI. Keep a compatibility table of validated combinations.
- Avoid circular PR dependency by merging pure interfaces/contract tests first, then fake-host/bridge, then actual application adapter.
- Make each task buildable and independently testable. Prefer separate PRs for values, metadata, parser, VM debugger, GridCraft IO, event integration, and external corpus ingestion.
- Share one `office-api-coverage.md` generator and one feature/test ID namespace. Avoid two incompatible compatibility dashboards.
- Keep fast tests self-authored and repository-local; pin/download large or restrictive workbook corpora separately with documented permissions and hashes.
- Establish a public `vba-cli`: `parse`, `check`, `compile`, `disasm`, `run --fake-excel`, `inspect-project`, `test`, and later `debug`/`dap`.
- Document supported/unsupported cases rather than hiding limitations behind success/error suppression.

## 11. Exact first PR sequence

This is the **starting task list** an implementation agent can follow immediately.

| PR | Repo | Small deliverable | Must pass |
|---|---|---|---|
| 1 | `craft-automation` | Workspace, licence/AGENTS, `automation-types`, `ObjectHandle`, value/argument/error tests | Native/WASM builds, handle generation and Missing/Null distinction |
| 2 | `craft-automation` | `automation-host`, fake worksheet and rectangular read/write interface | Read/write scalar and 2-D block, errors on bounds/protection |
| 3 | `craft-automation` | Narrow reviewed Excel member YAML + dispatcher and default member tests | `Range`, `Cells`, `Item`, `Value2`, named/omitted arguments; unsupported errors |
| 4 | `vba-rs` | Workspace, `vba-diagnostics`, lexer/parser and source-span/statement-ID tests | Compile and reject simple procedures predictably |
| 5 | `vba-rs` | Binder + checked IR/bytecode + disassembly | Resolved locals/procedures, verified branches/source map |
| 6 | `vba-rs` | `vba-values`, core arithmetic, explicit-state VM loop/calls | Pure `For`/`If`/`Sub` fixture results and budget cancellation |
| 7 | `vba-rs` | VM inspection hooks + `vba-debug`, break/resume/step tests | Pause on same-line statements, inspect/change local, resume reliably |
| 8 | Both | `vba-foreign` + `vba-office-bridge` + fake Excel integration | **`FillCells` end-to-end fake workbook demo with breakpoint** |
| 9 | GridCraft | Minimal `SpreadsheetHost` adapter + `vba-cli`/app composition test | Real `Range.Value2` writes, recalc/undo invariant tests |
| 10 | VBA + GridCraft | `office-project` attachment + read-only `vba-project` extraction + macro trust | `.xlsm` load/explicit execute/save test; no auto-execution |
| 11+ | Both | Deeper VBA semantics, Office API generator, events, DAP, larger corpus | Independent conformance gates per capability |

**Rule for autonomous agents:** Do not begin large-scale API generation, UserForms, COM emulation, full Word support or UI-based debugger work until PR 8's paused/resumed fake-host macro passes. Do not claim `.xlsm` compatibility until PR 10's self-authored file passes a real engine test.

## 12. Reference sources and useful external material

### Authoritative documentation

- Microsoft **VBA language specification [MS-VBAL]**: <https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/d5418146-0bd2-45eb-9c7a-fd9502722c74>
- Microsoft **Office VBA file format [MS-OVBA]**: <https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/575462ba-bf67-4190-9fac-c275523c75fc>
- Microsoft **Office VBA reference source**: <https://github.com/MicrosoftDocs/VBA-Docs>
- Microsoft **Excel VBA object model**: <https://learn.microsoft.com/en-us/office/vba/api/overview/excel/object-model>
- Microsoft **Word VBA object model**: <https://learn.microsoft.com/en-us/office/vba/api/overview/word/object-model>
- Microsoft **Excel interop API**: <https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.interop.excel?view=excel-pia>
- Microsoft **Word interop API**: <https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.interop.word?view=word-pia>
- Microsoft **Debug Adapter Protocol**: <https://microsoft.github.io/debug-adapter-protocol/>

### Host project integration references

- GridCraft: <https://github.com/storytold/gridcraft>
- GridCraft architecture: <https://github.com/storytold/gridcraft/blob/main/docs/architecture.md>
- GridCraft file-format parity: <https://github.com/storytold/gridcraft/blob/main/docs/file-format-parity.md>
- GridCraft contributor rules: <https://github.com/storytold/gridcraft/blob/main/AGENTS.md>
- WordCraft: <https://github.com/storytold/wordcraft>
- WordCraft architecture: <https://github.com/storytold/wordcraft/blob/main/docs/architecture.md>
- WordCraft file-format parity: <https://github.com/storytold/wordcraft/blob/main/docs/file-format-parity.md>
- WordCraft contributor rules: <https://github.com/storytold/wordcraft/blob/main/AGENTS.md>

### Comparative test and tooling leads (licence audit before use)

- ReflexGrid VBA/Excel comparisons: <https://github.com/milochen0418/reflex_grid>
- ReflexGrid compatibility documentation: <https://milochen0418.github.io/reflex_grid/compatibility.html>
- VBA-JSON (`.bas` and test/spec workbook): <https://github.com/VBA-tools/VBA-JSON>
- Tree-sitter VBA grammar: <https://github.com/harumiWeb/tree-sitter-vba>
- VBA workbook examples: <https://www.contextures.com/excelfiles.html>
- `ovba` Rust source-extraction library (evaluate current licence/API): <https://github.com/ekuurila/ovba>
- Other interpreters **for comparison only, never copied as this implementation**: <https://github.com/Ryujiyasu/oxi/tree/main/crates/oxivba-core>, <https://github.com/kent-tokyo/elixcee>, <https://github.com/scriptandcompile/vb6>

## 13. Definition of complete for the first integrated release

The first release is acceptable when:

1. A user can open a supported `.xlsm` in GridCraft without macros executing until authorised.
2. Source is extracted with correct module and workbook/sheet code-name associations.
3. A documented subset of standard-module VBA is compiled and run in a pausable VM, with real cell/range operations and correct observed type/error semantics.
4. The debugger can break, inspect, step, change a local and resume without replaying mutations, including across a host method call.
5. GridCraft retains authoritative workbook state, formula recalculation and valid undo/edit history.
6. Saving a supported macro-enabled file preserves the expected project bytes and OOXML relations; unsupported packages/signed-project changes have explicit diagnostics.
7. All attempted unsupported Office members and VBA constructs fail visibly, never with fabricated success.
8. The project publishes a reproducible, pinned conformance report with known limits.
9. Native and supported WASM builds pass core tests; malicious/malformed inputs have bounded failure paths.
10. Python/JS and Word integrations have documented extension contracts, without being prerequisites for the Excel-first release.

---

**Immediate next action:** assign the first eight PRs and freeze the minimal `AutomationValue`/handle/member-dispatch contract and fake Excel test schema. The fastest route to useful compatibility is **a 10-line VBA macro with a working debugger and real Office dispatch**, not a huge untested catalogue or a language engine tied directly to GridCraft.
