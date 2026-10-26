# Rust VBA Interpreter for ArtCraft — Architecture and Implementation Specification

> **Status:** Proposed implementation specification / agent hand-off  
> **Date:** 26 October 2026  
> **Primary target:** Microsoft Excel VBA inside GridCraft (`.xlsm`)  
> **Secondary target:** WordCraft, and other ArtCraft applications, via separate host adapters  
> **Language:** Rust; native desktop and WebAssembly targets  
> **Proposed licence:** `MIT OR Apache-2.0`, consistent with GridCraft and WordCraft  
> **Audience:** An implementation LLM / engineering agents working in small, testable increments

## 0. Executive instruction to the implementing agent

Build a **new, modular, clean-room VBA interpreter in Rust**. It must parse, bind, compile, and execute VBA code with **Microsoft-compatible semantics** over a **host-independent virtual machine (VM)** that supports pausing, resuming, breakpoints, stepping, inspection, and cancellation. Its primary host is the existing **GridCraft Excel object-model compatibility layer**, not a new or duplicated spreadsheet model. WordCraft is a future host, not a reason to compromise the Excel-first design.

**Do not** adopt `oxivba-core`, `elixcee`, or a VB6 interpreter as the implementation. They are useful architectural comparisons and behavioural test sources only. Do not copy MPL/GPL code into the new dual-licensed runtime. Use Microsoft's public specifications, public documentation, independent black-box observations, and properly licensed tests. Carefully evaluate, rather than automatically adopt, permissively licensed libraries for VBA project extraction and source grammar.

**Do not attempt full VBA compatibility in one change.** Start with a small vertical slice which runs a procedure, modifies mock Excel cells, hits a breakpoint, inspects/modifies a local variable, and resumes. Grow compatibility via differential tests against Microsoft Excel. A feature is *implemented* only when it has positive, negative, and edge-case tests, and its expected behaviour is independently grounded. Unsupported constructs must produce explicit structured errors, never silent no-ops.

Before coding, inspect the **current** repositories and read their contributor rules. The repository architecture can change; this document records the inspected design at the date above, not a licence to assume that APIs remain unchanged:

- [GridCraft](https://github.com/storytold/gridcraft), [architecture](https://github.com/storytold/gridcraft/blob/main/docs/architecture.md), [AGENTS.md](https://github.com/storytold/gridcraft/blob/main/AGENTS.md), [command/control protocol](https://github.com/storytold/gridcraft/blob/main/docs/control-protocol.md).
- [WordCraft](https://github.com/storytold/wordcraft), [README and architecture](https://github.com/storytold/wordcraft#architecture), [AGENTS.md](https://github.com/storytold/wordcraft/blob/main/AGENTS.md).
- The project's pre-existing **shared Office compatibility-layer design document** (if present). This specification is for the interpreter and its integration contract: **do not duplicate that layer**.

### Decisions already made

| Decision | Requirement |
|---|---|
| Implementation | New Rust source, modular crates, no Rust-to-Python/JS interpreter dependency |
| Licensing | New project code `MIT OR Apache-2.0`; assess third-party licence and provenance per dependency and fixture |
| Primary application | Excel/GridCraft first; Word/WordCraft only through host abstractions |
| Interpreter model | Parse → semantic binding → inspectable IR/bytecode → explicit-state VM |
| Debugger | Part of initial VM design, with core debug API and a later Debug Adapter Protocol (DAP) adapter |
| Host calls | Native, in-process Rust interfaces; **not JSON-RPC** for per-cell/per-member calls |
| Browser | Avoid platform-native dependencies in core crates; compile/test `wasm32-unknown-unknown` |
| Standards | [MS-VBAL](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/d5418146-0bd2-45eb-9c7a-fd9502722c74) is the normative VBA language reference; Excel behaviour is the practical compatibility oracle |
| Imports | Read embedded source from `.xlsm`; do not implement P-code execution |
| Safety | Explicit user consent, runtime budgets, capability restrictions, predictable errors |
| Compatibility claims | Evidence-based, differentiated into parser / runtime / Excel API / file import / debugger |

### Non-goals for the first release

Full COM/ActiveX automation, arbitrary native DLL `Declare` calls, arbitrary filesystem or network access, Microsoft P-code execution, compiling native machine code/LLVM, a complete VBA IDE, signed VBA project creation, a full UserForms designer, Word-specific object coverage, and pixel-exact Excel UI. Architectural extension points **must** exist for these, but they must not block basic spreadsheet macros.

---

## 1. Product requirements and success criteria

### 1.1 User scenarios

1. **Existing file:** A user opens an ordinary `.xlsm` workbook in GridCraft. Workbook cells, formulas, formats, relationships and original VBA project are retained to the degree documented by GridCraft. They can inspect available macros and explicitly enable/run selected macros. Opening a document by itself must not silently execute untrusted code.
2. **Excel automation:** A VBA macro reads and writes `Range`, `Cells`, `Worksheet`, `Workbook`, `Application` and collection members, using syntax and semantics compatible with Excel.
3. **Event macros:** Once explicitly enabled, relevant worksheet/workbook events run with correct code-name/module binding, event order, guards and reentrancy policy.
4. **Debugging:** A developer sets a breakpoint, runs the macro, steps into/over/out, reads locals and the call stack, changes a local while paused, inspects host objects, uses `Debug.Print`, and continues or aborts.
5. **Headless test:** A CLI executes a selected macro using a fake host or loaded GridCraft workbook, prints machine-readable diagnostic/results, and exits with a non-zero status on failure.
6. **Browser:** Where supported, the same VM runs in a WASM build without blocking the UI indefinitely; execution and interactions can yield and resume.
7. **Future extensibility:** Python and JavaScript bindings may invoke the shared Office layer directly. They must *not* be forced through the VBA parser, VBA VM, or VBA-specific values unless explicitly needed for compatibility.

### 1.2 Quality attributes

- **Correctness over feature-count marketing:** preserve Microsoft-observable semantics for implemented constructs.
- **Maintainability:** semantics, parsing, bytecode, debugger and host binding live in separable units with narrow dependencies; avoid giant interpreter source files.
- **Diagnosability:** every stage emits structured diagnostics with module, project, source range, code/category and optional underlying cause.
- **Robustness:** malformed source, invalid file bytes, hostile sizes, cyclic references, bad host handles, or excessive recursion cannot crash the application.
- **Determinism:** tests control clock, randomness, locale, user prompts and host responses.
- **Performance:** compile once per project revision; cache resolved symbols/member metadata; batch range operations; avoid per-cell serialisation/IPC.
- **Portability:** core Rust works on supported native OSs and wasm; optional OS/platform backends sit behind features.
- **Isolation:** one workbook's code/state must not leak into another without an explicitly permitted cross-workbook operation.

### 1.3 Precise meaning of “compatibility”

Report separate dimensions rather than a single misleading percentage:

- **Parsing:** syntactically accepted VBA files and source modules.
- **Binding:** correct symbols, calls, implicit/default members and diagnostic resolution.
- **Language runtime:** value types, operators, statements, errors, classes and standard library.
- **Excel object model:** object/method/property/event conformance for the implemented surface.
- **Document:** `.xlsm` project extraction, preservation, macro execution and round-trip.
- **Debugging:** breakpoints, step semantics and state inspection.
- **Restricted-platform exclusions:** deliberately blocked capabilities, clearly separated from bugs.

Each test has a status: `PASS`, `MISMATCH`, `UNSUPPORTED`, `BLOCKED_BY_POLICY`, `INVALID_FIXTURE`, `CRASH`, `TIMEOUT` or `NOT_RUN`. Never count an expected refusal as a passing compatibility test unless the requirement is specifically security behaviour.

---

## 2. Existing ArtCraft integration: respect the actual applications

As inspected on 26 October 2026, GridCraft is an **engine-first Rust Cargo workspace**:

- `gridcraft-core`: addresses, values, Excel-like coercion and date handling.
- `gridcraft-formula` / `gridcraft-functions` / `gridcraft-calc`: spreadsheet formula parsing, function execution and dependency-driven recalculation.
- `gridcraft-model`: workbook, worksheet, cell data and features.
- `gridcraft-xlsx`: workbook file parsing/serialization.
- `gridcraft-engine`: `Session`, selections, history and **command registry**; commands drive the UI, CLI and MCP entry points.
- `gridcraft-ui-egui`: native/browser UI.

The inspected GridCraft engine uses `Session::execute(id, params)` for application commands; its architecture document describes `Session::grouped`, `group_begin`, `group_end`, `group_rollback`, and `abort_groups`, along with document trust/provenance and explicit trust for existing stored automation scripts. **Use and extend these mechanisms rather than inventing independent workbook storage or a second undo system.** Read [the latest architecture](https://github.com/storytold/gridcraft/blob/main/docs/architecture.md) before changing the engine.

### 2.1 Three separations to enforce

```
                     .xlsm / .bas / .cls
                              |
                     [Project loader]
                              |
                  [VBA lexer / parser]
                              |
                    [Semantic binder]
                              |
                    [IR / bytecode]
                              |
                  [Pausable Rust VBA VM] <--> [Debugger core] <--> [DAP / IDE]
                       |           |
              [VBA runtime]    [Host bridge]
                                    |
                  [Shared Office compatibility layer]
                               /           \
                      Excel adapter     Word adapter (later)
                           |                   |
                    GridCraft APIs        WordCraft APIs

   Python/JavaScript -> their own bindings -> shared Office compatibility layer
```

1. **Language and VM are host-agnostic.** No dependency on `gridcraft-*` or `wordcraft-*` crates below the host-adapter boundary.
2. **Office compatibility model is language-agnostic.** Excel object semantics, member signatures, collection/default member behaviour and event vocabulary are exposed via an abstraction usable by other scripting languages.
3. **Document engine remains authoritative.** VBA operations must mutate the *actual* GridCraft workbook through the supported model/engine APIs. They must not maintain a parallel cell store.

### 2.2 Preserve existing project layering

Do not insert a dependency cycle into the GridCraft Cargo workspace. Prefer a separate VBA workspace during initial development, then add lightweight adapters at an appropriate layer. If the compatibility layer already defines automation values, reconcile rather than duplicate them. Use a neutral host bridge crate that the language VM and GridCraft can both depend on without lower-layer crates depending on higher ones.

Suggested integration sequence:

1. Compile/test `vba-*` crates as a standalone workspace.
2. Run against a fake in-memory Excel host.
3. Integrate via a GridCraft host adapter and explicit macro command(s), not UI-only APIs.
4. Add project metadata and macro attachment to the document loading/save path.
5. Add user-facing macro enablement and debugger surfaces last.

### 2.3 Undo, error and macro semantics: avoid a false shortcut

GridCraft can group commands into one undo unit and roll back failed groups. **Do not assume a failed VBA macro should atomically roll back all previous cell edits:** in Microsoft Excel, earlier completed edits can remain when execution stops with a runtime error. Choose explicit policies:

- **Compatibility execution:** preserve effects up to the error/stop, consistent with observed Excel behaviour; group for Undo when useful but do not inadvertently revert successful prior statements.
- **Preview/sandbox mode:** optionally run transactionally with explicit rollback, but label it as a separate product mode rather than Excel-equivalent behaviour.
- **Aborted/abandoned debugger:** close/abort outstanding groups without leaving Undo unusable; define whether partially completed edits remain or are restored as a documented policy.
- **Programmatic errors:** turn host errors into VBA errors with the correct number/message where supported; do not swallow them or make every failure a transaction panic.

Events caused by macro edits must follow a separately tested reentrancy model (`Application.EnableEvents`, nested callbacks, event ordering). Avoid running a second VM recursively without a defined scheduler/call-frame policy.

---

## 3. Proposed Rust crate boundaries

Use names as a guide; change exact layout to fit the owning repository, but keep the dependency rules.

```text
vba/
├── Cargo.toml
├── crates/
│   ├── vba-diagnostics/   # source files/spans, errors, diagnostic codes
│   ├── vba-syntax/        # lexer, parser, syntax tree, exported module formats
│   ├── vba-semantics/     # symbol tables, scopes, compile-time binding, types
│   ├── vba-ir/            # bytecode/instructions, constants, source maps
│   ├── vba-values/        # Variant, scalar/array, object handles, byref locations
│   ├── vba-runtime/       # coercions, operators, intrinsics, error semantics
│   ├── vba-host/          # host interfaces and metadata, no application dependency
│   ├── vba-vm/            # execution state, frames, scheduler, interpreter
│   ├── vba-debug/         # in-process breakpoint/step/inspection API
│   ├── vba-dap/           # optional Debug Adapter Protocol adapter
│   ├── vba-project/       # .bas/.cls and vbaProject.bin import, metadata
│   ├── vba-excel-host/    # GridCraft Office-layer adapter; integration-only crate
│   └── vba-conformance/   # fixture formats, test comparison and reporting
├── apps/
│   └── vba-cli/           # parse, check, run, debug, inspect-project, tests
├── tests/
│   ├── syntax/
│   ├── semantics/
│   ├── runtime/
│   ├── debugger/
│   ├── excel-host/
│   ├── reference-oracle/
│   └── workbooks/
├── fuzz/
└── docs/
    ├── architecture.md
    ├── compatibility-matrix.md
    ├── diagnostics.md
    ├── debugger.md
    └── developer-guide.md
```

**Dependency direction:** diagnostics/values/host contracts are low-level; syntax does not depend on runtime; semantics depends on syntax, not GridCraft; IR depends on semantics metadata; VM depends on IR/values/runtime/host, not on DAP or UI; DAP depends on debugger; Excel adapter depends on GridCraft compatibility interfaces, not vice versa. Avoid a circular dependency between `vba-debug` and `vba-vm`: put shared debug hooks/events into a low-level module or make the debugger a controller over a VM inspection trait.

### 3.1 Architecture guardrails

- Normal code must not use `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, unreachable assumptions on user input, or unchecked indexing; follow GridCraft's [`AGENTS.md`](https://github.com/storytold/gridcraft/blob/main/AGENTS.md).
- Forbid `unsafe` in new interpreter crates unless a rare dependency boundary is explicitly approved and documented. Prefer `#![forbid(unsafe_code)]`.
- No UI toolkit types, Excel data-model types or OS resources in the VM/value crates.
- Use small modules and focused traits, not monolithic 800 KB–1.5 MB runtime files.
- Make external state/clock/random/UI prompts injectable.
- Ensure all user-derived allocations, nesting, recursion, iterations, compiled code sizes and host calls can be bounded.
- Do not use JSON as the **runtime host dispatch transport**; DAP, CLI diagnostics and test manifests may use JSON.

---

## 4. Source, project and compilation pipeline

### 4.1 VBA project model

Represent an Office VBA project as an explicit semantic object, not merely a bag of strings:

```text
Project {
  project_id, name, references, platform_profile, compile_constants,
  modules: [
    {kind: Standard, name, source, attributes},
    {kind: Class, name, source, attributes},
    {kind: Document, name, code_name, host_object_binding, source, attributes},
    {kind: UserForm, name, source, metadata/binary design, attributes},
  ],
  original_binary(optional), signatures_metadata(optional),
  source_encoding, source_revision, validation_report
}
```

Preserve module order, names, code names, declarations, class metadata, VB attributes (`VB_Name`, `VB_PredeclaredId`, `VB_Exposed`, `VB_UserMemId`), and reference ordering where they are meaningful. Track project revision independently of workbook content revision so compiled code is invalidated only when required. Document-module code names (`Sheet1`, `ThisWorkbook`) are not worksheet display names.

### 4.2 Lexer and parser

Required support, incrementally:

- Case-insensitive identifiers while preserving original spelling for debugging/editor rendering.
- VBA logical/physical lines, colon-separated statements, comments (`'` and `Rem`), continuations, strings and escaped quotes, named arguments (`:=`), literal/type suffixes, date literals, bracket expressions where applicable, and compound member access.
- Modules, declarations, `Option Explicit/Base/Compare/Private Module`, Subs/Functions, properties, class/event declarations, arrays/UDTs/enums, conditional compilation and external declaration syntax (even when invocation is blocked).
- Statements including `If`, `Select Case`, `For`, `For Each`, `Do`, `While`, `With`, labels, `GoTo`, `GoSub`, `Return`, `On Error`, `Resume`, `ReDim`, `Erase`, `Set`, and `Mid` assignment.
- Distinguish syntactically important VBA forms such as `Foo x`, `Call Foo(x)`, `Foo (x)`, member `!` access and default-member calls. Do not normalize these before the binder has resolved their semantics.
- Preserve **module ID, byte/character offsets, line/column and statement identity**. The same line can contain several debug-steppable statements.
- Reject invalid source with structured recoverable diagnostics instead of panicking.

Implementation options:

- **Preferred:** pure-Rust lexer + parser under project control, informed by MS-VBAL and independent tests. Hand-written Pratt parsing for expressions and recursive descent for declaration/statement syntax are reasonable but not compulsory.
- **Optional accelerator:** [harumiWeb/tree-sitter-vba](https://github.com/harumiWeb/tree-sitter-vba) is MIT and useful for corpus/grammar and editor syntax. Its parser is generated in C and explicitly targets tooling; do not assume its permissive tree is a validated compilation AST. If strict pure-Rust and `unsafe_code = forbid` are non-negotiable, use it only as a reference/test corpus rather than linking it into the compiler.

Do not confuse parsed-with-recovery with valid VBA. Maintain `syntax_valid`, `semantically_valid` and `executable` as distinct states.

### 4.3 Semantic analysis and binding

Implement a distinct pass for:

- Module and project symbol tables; public/private symbols, implicit names, shadowing and conflicts.
- `Option Explicit`, `Option Base`, `Option Compare`, default procedure visibility and project references.
- Standard, class, document and (later) form module resolution.
- ByVal / ByRef parameters; implicit ByRef; `Optional`, default expressions, omitted/Missing and `ParamArray`.
- Named arguments, param ordering, call parentheses, default members and properties (`Get`, `Let`, `Set`).
- Type declarations/suffixes and enum/UDT types; `As New` and object instantiation.
- Compile-time constants and conditional compilation (`#If VBA7`, `Win64` etc.) according to an **explicit target platform profile**, not whichever OS happens to run GridCraft.
- Early-bound known Office members through type/member metadata, and late-bound Variant/Object invocation through a runtime dispatcher.
- Compile errors assigned to the correct module/source range.

Separate compile-time binding from runtime object/member dispatch. Record whether a member was statically identified and cache stable dispatch descriptors, but allow late binding where VBA requires it.

### 4.4 Platform and locale profile

Create a profile such as `Excel365Windows64` as the initial differential-test target, but make the following explicit and configurable:

- VBA dialect/conditional compilation symbols (`VBA7`, `Win32`, `Win64`, `Mac`).
- Pointer width (`LongPtr`), `LongLong` availability and declared external API restrictions.
- Locale-dependent parsing/formatting, date and decimal conventions, case comparisons, character conversions.
- Clock/timezone, deterministic `Rnd` seed, host's Excel version.
- Office library/type metadata version.

**Never** set contradictory compile constants (for example, blindly setting both `Win32` and `Win64`) without direct evidence. Define and test the selected Office/OS profile. Existing workbooks may target different profiles; unsupported targets should produce warnings or failures rather than miscompile silently.

---

## 5. VBA values, coercion and references

This subsystem deserves disproportionate testing. Do not reuse `gridcraft-core::Value` directly as the VBA `Variant`: worksheet cell types and VBA runtime types have different semantics. Convert at the Office host boundary using explicit rules.

### 5.1 Value representation

Use an appropriately factored representation, conceptually:

```rust
// Design sketch, not guaranteed compile-ready.
pub enum VbaValue {
    Empty,
    Null,
    Missing,               // omitted optional argument when observable
    Boolean(bool),         // representation must preserve VBA True = -1 in conversions
    Byte(u8),
    Integer(i16),          // VBA Integer is signed 16-bit
    Long(i32),             // VBA Long is signed 32-bit
    LongLong(i64),         // when supported by profile
    Single(f32),
    Double(f64),
    Currency(i64),         // fixed 4-decimal scale
    Decimal(VbaDecimal),
    Date(VbaDate),         // OLE/VBA-compatible serial semantics
    String(VbaString),
    Error(VbaErrorValue),  // CVErr value != raised runtime exception
    Array(VbaArray),
    Record(VbaRecord),
    Object(ObjectHandle),  // including typed Nothing / null object state
}
```

`Nothing` is conceptually an absent object reference, not interchangeable with `Null`, `Empty`, or `Missing`. Decide and document whether the internal representation uses a special variant or a nullable object handle. Object identity must be separate from copied scalar values. A record/UDT has value semantics. Arrays need element type, dimension lengths and lower bounds, and copying/reference semantics matching their contexts. Multidimensional ordering matters, particularly for `Range.Value`.

### 5.2 Coercion and operators

Create individually testable functions for:

- `+`, `-`, `*`, `/`, `\`, `Mod`, `^`, `&` and comparisons.
- Bitwise/logical `And`, `Or`, `Xor`, `Not`, `Eqv`, `Imp`, including operand evaluation rules and `Null` propagation.
- Explicit `CStr`, `CLng`, `CInt`, `CDbl`, `CDec`, `CCur`, `CDate`, `CBool` and implicit assignment coercion.
- Overflow, rounding and data-type promotion, including overflow/error numbers for `Integer` and `Long`.
- `Is`, `TypeOf ... Is`, `Like`, locale/string comparisons and `Option Compare`.
- `Empty` numeric/string coercion, `Null` propagation, `CVErr` values, `Nothing` dereference errors.
- Fixed-length strings, Unicode and VBA string indexing conventions.
- Excel cell API conversions (`Value` vs `Value2`, errors and date/currency types).

Do not hard-code the expected outputs of every example. Define general semantic rules from the specification, implement them centrally, then check against diverse oracle fixtures.

### 5.3 L-values and ByRef

A **ByRef argument is a writable location**, not simply a copied `VbaValue`. Model stable, validated locations such as local/frame slots, static/module slots, array elements, record fields and qualified property l-values where VBA permits them. Alias-aware writes must be observable by the caller. Ensure lifetime/invalidation checks prevent a saved reference from outliving its frame or backing object.

Tests must distinguish:

```vb
Sub Increment(ByRef x As Long)
    x = x + 1
End Sub

Sub Demo()
    Dim a As Long
    a = 4
    Increment a
    Debug.Assert a = 5
End Sub
```

Also test arrays, optional parameters, expression arguments, parenthesized calls, property values and type mismatches. Follow MS-VBAL/Excel rather than naïve borrow/move semantics.

### 5.4 Heap, object lifecycle and collections

Support stable handles, host/object ownership, collection enumeration and an explicit object identity model. Track handles with a generation or equivalent so stale handles are diagnosed. Separate VBA class instances and external host objects. If employing `Rc`/`RefCell` in a single-threaded VM, carefully handle cycles, teardown and reentrant host calls; never expose a Rust panic from borrow conflicts as a VBA error strategy. Class initialisation/termination and reference lifetime need conformance tests before claiming class compatibility.

---

## 6. IR / bytecode and pausable VM

### 6.1 Why bytecode

Do not build a long-lived recursive AST evaluator using the native Rust call stack. A small explicit IR/VM gives stable pause/resume semantics, debugging, bounded execution, more predictable reentrancy and possible future optimisation. A tree-walking prototype is acceptable **only if it is immediately superseded by the explicit-state VM before feature expansion**.

Example lowering (illustrative instruction names):

```vb
Cells(i, 1).Value = i * 10
```

```text
LOAD_GLOBAL        Cells
LOAD_LOCAL         i
PUSH_CONST         1
CALL_DEFAULT       2
LOAD_LOCAL         i
PUSH_CONST         10
MULTIPLY_VBA
PROPERTY_LET       Value
```

Each statement has a source-map entry (module, line, span, statement ID) and a clear semantic checkpoint. Do not assume each instruction equals a VBA source statement.

### 6.2 VM state

```text
ExecutionSession {
  project_revision,
  compiled_program,
  frames: [Frame { procedure, ip, locals, temporaries, error_handler_state, ... }],
  module_and_static_storage,
  vba_object_heap,
  host_context,
  event_queue,
  debug_state,
  execution_budget,
  state: Ready | Running | Paused | WaitingForHost | Completed | Failed | Cancelled
}
```

The VM must support a *single-instruction or bounded-instruction* run method, a safe-point event loop and an explicit outcome:

```rust
pub enum VmOutcome {
    Progress,
    Paused(PauseReason),
    Waiting(HostRequestId),
    Completed(VbaValue),
    Failed(VbaRuntimeError),
    Cancelled,
}
```

`HostRequestId` and suspend/resume are illustrative abstractions; synchronous in-process host calls remain the normal fast path. Yield only when a UI dialog, async host operation, event dispatch or browser time slice genuinely requires it. Never hold mutable workbook borrows across a yield or debugger pause.

### 6.3 Required VM instruction families

1. Constants, slots and l-value loads/stores.
2. Conversions, unary/binary operators and comparisons.
3. Conditional/unconditional branches, statement enter/leave and loop control.
4. Procedure call/return, argument binding, external member invoke.
5. Arrays, records, indexing, enumeration, `With` target management.
6. VBA error-handler setup, raise/propagate, `Resume`/`Resume Next`, `Err` state.
7. Object create/reference handling and property Get/Let/Set.
8. Debug source boundaries, `Stop`, cancellation, instruction budgets and optional tracing.

### 6.4 VBA error handling

Model `On Error GoTo label`, `On Error Resume Next`, `On Error GoTo 0`, `On Error GoTo -1` where applicable, `Resume`, `Resume Next`, `Err.Clear`, `Err.Raise` and `Erl` according to MS-VBAL. Distinguish disabled/enabled/active handlers and resumption locations; a generic Rust `Result` and an exception catch are not enough. Runtime errors should not panic or leak host/internal errors. Attach source locations, stack frames and mapped VBA error numbers. Test nested calls, errors raised within handlers, line labels, and host method errors.

### 6.5 Scheduling, cancellation and browser support

- Budget by executed operations, wall/CPU time as appropriate, call depth, memory, string/array dimensions, host calls, event nesting and total VBA project size.
- Use bounded cooperative time slices for browser integration; never freeze the UI for an unbounded loop.
- Long-running host work should have explicit cancellation checkpoints; a blocked synchronous host operation must not be falsely advertised as interruptible.
- Distinguish cancellation, timeout and ordinary VBA run-time error; platform policy can map these to error handling only if explicitly designed.
- Standard library clock/random/file/dialog interfaces are injected and mockable.

---

## 7. Host automation boundary and Excel integration

### 7.1 Native Rust dispatch model

Use a **stable semantic interface**, not a transport-specific one. The code below is **conceptual**; choose implementable ownership types and avoid borrowing pitfalls:

```rust
pub trait AutomationHost {
    fn resolve_global(&mut self, name: &str) -> Result<HostValue, HostError>;
    fn invoke(
        &mut self,
        target: ObjectHandle,
        member: MemberRef,
        kind: InvokeKind,       // Method / PropertyGet / PropertyLet / PropertySet
        args: &mut [HostArg],   // positional/named/missing and writable ByRef
    ) -> Result<HostValue, HostError>;
    fn begin_enumeration(&mut self, object: ObjectHandle)
        -> Result<EnumeratorHandle, HostError>;
    fn next_item(&mut self, enumerator: EnumeratorHandle)
        -> Result<Option<HostValue>, HostError>;
}
```

A real contract must additionally address object retention/release, runtime object vs host object IDs, event subscriptions, invoking a VBA callback, host yields/dialogs, cancellation, metadata and error mapping. Distinguish missing arguments from `Empty`, `Nothing` from `Null`, and property Let from Set. Preserve named argument identity and ordering. Prefer metadata member IDs and cached dispatch after resolution; dynamic name lookup remains available for late binding.

### 7.2 Contract principles

- **VBA runtime owns VBA semantics**, including its operator conversions and argument mechanics.
- **Office layer owns Office semantics**, including Excel default properties, object hierarchy, collection indexing, event definitions, member signatures and Excel-specific errors.
- **GridCraft owns spreadsheet state**, commands, recalculation, clipboard, undo and persistence.
- Member and type metadata should be generated/maintained from an explicitly versioned catalogue inspired by [Microsoft's Excel object-model reference](https://learn.microsoft.com/en-us/office/vba/api/overview/excel/object-model), not hand-coded repeatedly at call sites.
- Do not serialize an entire `Range` into JSON to read a cell. Use native typed values and batch operations.
- Do not treat every unknown method/property as success or `Empty`; return an explicit documented VBA-compatible error.
- Support host object identity and stable handles; workbook closing, sheet deletion and undo can invalidate objects.
- Put Python/JavaScript language bindings **beside** the VBA adapter on the Office layer rather than routing them through the VM.

### 7.3 Excel object coverage prioritisation

**P0 (vertical slice):** `Application`, active workbook/worksheet context, `ThisWorkbook`, `ActiveWorkbook`, `ActiveSheet`, `Worksheets`, `Sheets`, `Range`, `Cells`, `Value`, `Value2`, `Count`, `Item`, basic iteration, `Offset`, `Resize`, `Debug.Print` and a host-provided `MsgBox` response.

**P1 (useful macro coverage):** `Workbooks`, worksheet activation, `Rows`, `Columns`, `UsedRange`, `End`, `CurrentRegion`, copy/paste, clear, insert/delete, basic formatting, `Names`, `Find`, `Formula`, `FormulaR1C1`, calculated value readback, `.Address`, `Selection`, `Application.EnableEvents` and relevant events.

**P2:** filters, sorting, tables/ListObjects, named ranges, workbook/sheet-level protection, calculation mode, pivot interaction, chart objects and more Excel enumerations, tracking GridCraft's actual engine capabilities.

**P3:** UserForms, external COM, ActiveX, cross-workbook references/add-ins, advanced chart automation and Excel-specific platform integrations.

Coverage is **per member and behaviour**, not just object existence. Example: `Range.Value` may return a scalar for a single cell but a multidimensional Variant array for a rectangular range; `Value2` differs in Date/Currency handling; `.Text` depends on format/layout. Verify using [Range.Value](https://learn.microsoft.com/en-us/office/vba/api/excel.range.value), [Range.Value2](https://learn.microsoft.com/en-us/office/vba/api/excel.range.value2), and real Excel.

### 7.4 Spreadsheet/VM interoperability

- Map GridCraft cell errors (`#N/A`, `#VALUE!`, etc.) to VBA error variants when read through the Excel object model, not indiscriminately to thrown exceptions.
- `Range.Value`, `.Value2`, `.Formula` and `.FormulaR1C1` must observe correct dimensions, typed values and formula behaviour.
- Respect workbook/sheet-qualified and unqualified references (active context, `With`, document code name, `Application` defaults).
- Support efficient rectangular reads and writes, avoiding one engine command/undo snapshot per cell where bulk operations are possible.
- Integrate with dependency recalculation, Excel calculation state and volatile functions without inventing a second evaluator.
- Decide and test the UDF evaluation contract early: worksheet formula calls into VBA may re-enter the VM while GridCraft's calculation graph is active. Avoid cycles/deadlocks and enforce function-side-effect restrictions consistent with the chosen Excel reference profile.
- Host UI dialogs are requests through host facilities, not synchronous native OS dialogs in core crates. Headless mode supplies scripted responses or a predictable policy error.

### 7.5 Event dispatch and macro lifecycle

Implement a dedicated event binding/queue layer. Bind procedures in document modules to their workbook/worksheet instances by **code name**, not just `Module1` procedure strings. Test `Workbook_Open`, `Worksheet_Change`, worksheet calculation/selection events as supported, `Application.EnableEvents`, event callback stack traces and event suppression. Define handling of an event that modifies the sheet and fires additional events. Keep user trust gates distinct from compile/language errors.

---

## 8. Debugger: design it with the VM

### 8.1 Core debugger API (mandatory in first executable VM)

Debugger APIs are in-process Rust APIs independent of VS Code, egui and DAP. Required operations:

- Create/delete/enable/disable source breakpoints by `{project, module, line, statement_id?}`; return bound/verified source locations.
- Run, pause, continue, step into, step over, step out, terminate.
- List frames with procedure/module/source location; inspect locals, parameters, statics, module globals and `Err` state.
- Read and update local variables safely when paused; reject illegal assignments with VBA-compatible errors.
- Read debugger outputs (`Debug.Print`), `Stop` statements, runtime exceptions and stack traces.
- Report state transitions and correlate VM sessions to documents/macro invocations.
- Support breakpoint-on-error policy (all, unhandled, selected numbers), optionally conditional breakpoints and watches.

### 8.2 Correct step semantics

Breakpoints attach to **source statement boundaries**; one source line can contain multiple statements. `Step Over` must execute nested function calls without pausing inside them unless another breakpoint/error occurs; `Step Into` should enter resolvable VBA calls; `Step Out` should stop in the caller. Host calls should appear as coherent source actions unless explicitly opted into host tracing. Handling loops and `On Error` jumps correctly matters more than instruction-level debugging UI.

### 8.3 Inspection/evaluation and safety

Expression evaluation may have side effects in VBA (`Property Get`, default members, functions). Default debugger watch evaluation should use a **read-only/pure inspection mode** where possible, or clearly mark evaluation unsafe/unsupported. Implement an Immediate Window in later milestones using the same parser/binder and paused-frame context. Preserve temporary object handles only while the pause/session remains valid. Never allow a debugger expression to bypass macro capability restrictions.

### 8.4 DAP adapter (after stable core)

Implement a separate [Microsoft Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/) bridge; use the [protocol specification](https://github.com/microsoft/debug-adapter-protocol/blob/main/specification.md). Support `initialize`, `launch`/`attach`, `setBreakpoints`, `configurationDone`, `threads`, `stackTrace`, `scopes`, `variables`, `continue`, `next`, `stepIn`, `stepOut`, `pause`, `evaluate`, `setVariable`, `disconnect`, and stopped/output/terminated events as support matures. Publish an explicit capabilities list rather than pretending every optional request exists.

DAP uses JSON for **debugger transport only**. That does not violate the requirement that VBA-to-Office member calls use native Rust APIs. A simple CLI debugger should work before introducing DAP or integrating the visual IDE.

---

## 9. Importing and preserving Excel `.xlsm`

### 9.1 File pipeline

A macro-enabled workbook is an OOXML/ZIP package; the macro project is normally an embedded `vbaProject.bin` compound-file part. Use package relationships/content types, **not merely filename guesses**, to locate its VBA project and associated metadata. The binary project's format is specified by [MS-OVBA](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/575462ba-bf67-4190-9fac-c275523c75fc), and the underlying compound file by [MS-CFB](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b).

Proposed sequence:

1. GridCraft's existing file loader opens the workbook data and detects macro-enabled content.
2. Validate OOXML relationships, size limits and compression ratio; locate VBA project part.
3. Read original opaque `vbaProject.bin` bytes and any relevant macro relationships/signature parts; retain these for unchanged-file preservation.
4. Extract project/modules/source/attributes/references/declared code page using a vetted reader; decode carefully.
5. Bind standard, class and document modules, preserving code names and source location maps.
6. Compile on demand or according to a documented policy; compile errors should be visible and should not corrupt the workbook.
7. When a user explicitly executes/enables a macro, start a bounded VM session with the correct host/document context.
8. On save, preserve original macro parts **if no VBA-source/project-affecting edit occurred**, subject to compatibility of the workbook changes and the package's relationships; do not silently claim signature validity after changes.
9. Project editing/export/rebuilding into an Excel-loadable binary is a later, separately tested milestone.

### 9.2 Reusable library

Consider [tim-weis/ovba](https://github.com/tim-weis/ovba) (MIT) for **read-only** project/source extraction. It does **not** extract the ZIP/OOXML part itself and does **not** write a project. It is a pre-1.0, partial implementation, so keep an adapter boundary and supply malformed-file tests. Inspect its current code/licence before vendoring or depending on it. Do not equate extraction of source with execution of P-code.

### 9.3 Security and unsupported file features

- File import must never auto-execute a macro or trust a workbook because it has a valid signature.
- Handle encrypted/protected/unsupported projects explicitly; do not bypass project protections.
- Detect suspicious source/cache inconsistencies and malformed metadata; report that source execution and Microsoft cached compiled code may not correspond. Do not attempt P-code execution.
- Preserve existing macro bytes where safe, but detect situations where unrelated parts or relationships will be dropped by GridCraft's serializer and report fidelity limitations.
- A binary `.xlsm` could contain UserForms, ActiveX and signatures that are *preserved* but not *executable*. Track these statuses separately.
- Apply decompression-bomb, stream-size, depth, count and source-length budgets.
- Observe security restrictions and clean-room rules on external fixture usage.

---

## 10. Standard library, external facilities and security model

### 10.1 VBA runtime functions

Prioritise a registry with explicit signatures, conversions, locale dependency, errors and purity metadata. Initial functions should include `Abs`, `Int`, `Fix`, `Sgn`, `Round`, `Len`, `Left`, `Right`, `Mid`, `Trim`, `LCase`, `UCase`, `InStr`, `Replace`, `Split`, `Join`, `IsEmpty`, `IsNull`, `IsError`, `IsObject`, `IsMissing`, `VarType`, `TypeName`, `CVar`, conversion functions, `LBound`, `UBound`, `Array`, and `Debug.Print`. Extend to dates, formatting, financial functions, `Like`, file APIs (capability gated), `Collection`, and `CreateObject` allowlisted substitutes only as justified by tests.

All builtins must use shared coercion/error helpers, not locally invented rules. Keep language-standard functions distinct from **Excel worksheet functions** (`Application.WorksheetFunction`) and from host commands.

### 10.2 Threat model

Assume document-provided VBA can be malicious or simply buggy. GridCraft supports desktop, CLI and browser scenarios, potentially with different trust boundaries. A Rust VM prevents some memory-safety defects but **is not a sandbox** when it can access unrestricted host APIs.

- Macro execution opt-in; retain user/workbook provenance and consent state.
- Define capability policy per execution: workbook mutation, read-only workbook, dialogs, file I/O, network I/O, shell, clipboard, native DLLs, COM automation and cross-workbook access.
- No raw OS filesystem/network/shell hooks in core VBA crates.
- Disallow or safely emulate unsupported `Shell`, `CreateObject`, `Declare`, `FileSystemObject`, `Open`, `Kill`, `Environ`, etc. according to explicit policy; raise meaningful errors rather than ignoring calls.
- Bound VM execution, recursion, array allocation, strings, open handles, output logs, event storms and pending dialogs.
- Do not allow late-bound host members to circumvent capability checks.
- Debugger evaluation, user-defined worksheet functions and event handlers obey the same policy.
- For server-hosted execution, security boundaries may require process isolation, not merely a capability trait.
- Log metadata, not workbook contents or secret arguments, by default.

### 10.3 Debugger pause and persistence

A paused VM retains workbook object references and potentially holds resources. Define session lifetime, timeouts, workbook-close behaviour, debugger disconnect policies and how outstanding group/undo state is released. A macro debugger **must not** leave GridCraft in a permanently locked group if the client disconnects.

---

## 11. Conformance and test design

**Test-driven compatibility is a deliverable, not optional infrastructure.** This project must be able to say precisely what matches Microsoft Excel and what does not.

### 11.1 Test tiers

| Tier | Inputs | Checks | Runs without Excel? |
|---|---|---|---|
| T0 syntax | `.bas`, `.cls`, snippets | AST, spans, diagnostics, grammar failures | Yes |
| T1 semantic | snippets/modules | symbol binding, types, calls, compile errors | Yes |
| T2 runtime | pure VBA procedures | typed results, mutations, error numbers, output | Yes |
| T3 fake host | deterministic Excel object double | dispatch, named/default members, events | Yes |
| T4 real GridCraft | loaded/generated workbook + macro | resulting cells, formulas, format, recalc, undo | Yes |
| T5 Excel oracle | same macro/scenario in Microsoft Excel | differential results, types, errors, state | Requires separately available Excel environment |
| T6 corpus | independent `.xlsm` workbooks | import, execution, preservation, round-trip | Usually yes; author oracle expected values separately |
| T7 debugger | deterministic test program | breakpoints, stepping, frames, watch safety | Yes |
| T8 robustness | fuzz/malformed/huge inputs | no crash, bounded allocations/runtime | Yes |

### 11.2 Reference harness

Define a canonical `TestScenario` with at least:

```yaml
id: excel-range-value-001
category: excel-host
source: self-authored
profile: Excel365Windows64
vba: |
  Public Sub RunTest()
      Worksheets("Sheet1").Range("A1").Value = 42
  End Sub
entrypoint: Module1.RunTest
setup:
  workbook_fixture: blank_book
  active_sheet: Sheet1
expected:
  workbook:
    Sheet1!A1:
      value: 42
      type: Number
  error: null
requirements:
  - vba-procedures
  - excel-range
  - excel-property-let
capabilities:
  - workbook-write
```

This is a **proposed manifest format**, not an existing executable schema. Implement a versioned schema with parse/validation, source provenance, reference Office version/build, test data/expected values, and diagnostic difference classification. Represent complex arrays with explicit bounds, dimensions and element types. Capture values, type names/`VarType`, errors, output, call traces when needed, and workbook changes (formula vs evaluated value, formatting and structural edits). Normalise only fields whose nondeterminism is documented (timestamps, paths), never silently erase differences.

### 11.3 Microsoft Office differential oracle

Run the same controlled VBA code under a real, isolated Microsoft Excel installation, preferably a known Windows Excel 365 build as the default profile. Use its automation facilities or a dedicated macro harness to record results, without inspecting Office binaries. Maintain the precise Office version, locale, OS architecture, calculation mode and security settings for each run. Some source corpora use Excel for Mac; keep the Mac results as a **different profile**, not an identical reference for every Windows behaviour.

When Excel is unavailable in CI, compare against vetted, versioned expected data generated/validated under the oracle. Observe GridCraft's clean-room restriction that test corpora and Excel-authored files are external and pinned; do not blindly commit generated Office binaries to the application repository. Verify permissible treatment of oracle-result data under the repository's current contributor rules.

### 11.4 Initial tests that MUST exist

Pure VBA:

1. Uninitialised `Variant` is `Empty`; `Null`, `Missing`, `Nothing` and `CVErr` are differentiated.
2. `Integer` (16-bit), `Long` (32-bit), `Currency`, `Date`, `Boolean` type preservation and overflow.
3. Unary operators, precedence, integer division, rounding, `&` vs `+`, `Like`, comparisons, `Null` logic.
4. `ByRef` local mutation; ByVal does not mutate; array element/UDT field and parenthesised argument behaviour.
5. Optional/named arguments, omitted values, `IsMissing`, `ParamArray`, `Call` syntax.
6. `Option Explicit`, module scope, static locals, project state across macro runs and reset.
7. `For`, `Do`, `For Each`, `Select Case`, labels, single-line `If`, `With` and nested calls.
8. `On Error Resume Next`, `On Error GoTo`, nested handler activation, `Resume` and `Erl`.
9. Arrays with explicit/nonzero lower bounds, multidimensional indexing, `ReDim Preserve`, `Erase` and enumeration.
10. Class instances, property Get/Let/Set, default member and object identity (when those features ship).

Excel host:

11. `Cells(i, 1).Value = i * 10` populates `A1:A10` correctly.
12. `Range("A1:B2").Value` vs one cell returns correct array/scalar shape and VBA element types.
13. `Range.Value2` conversions and cell errors; reading/writing formulas and subsequent recalculation.
14. `ThisWorkbook` vs `ActiveWorkbook`, sheet code name vs visible sheet name, qualified vs unqualified `Range`.
15. `For Each ws In Worksheets` and collection default member/index behavior.
16. `Worksheet_Change` event invocation, suppression (`EnableEvents`) and nested changes.
17. Macro error after an earlier successful cell write preserves/restores state according to documented compatibility policy.
18. `.xlsm` import retaining opaque VBA bytes across workbook edits where supported.

Debugger:

19. Break on a statement in a loop, inspect/update `i`, step and continue to expected state.
20. Step Into/Over/Out through procedures and errors; same-line multiple statements.
21. Pause/resume without losing object handles, workbook context, statics or call stack.
22. Graceful cancellation/time budget on an infinite loop, with an unlocked session/undo state.

Robustness:

23. Malformed tokens, deep expressions, infinite recursion, huge array bounds, oversized `.xlsm` and decompression bombs produce bounded errors, not crashes.
24. Invalid or stale handles, workbook close during pause, failed property writes and events cannot violate memory or document invariants.

### 11.5 Public test suites and workbook corpora

**A. ReflexGrid — primary source of Microsoft-Excel-comparison scenarios**

- [Repository](https://github.com/milochen0418/reflex_grid)
- [VBA interpreter and comparison methods](https://milochen0418.github.io/reflex_grid/vba.html)
- [Compatibility report](https://milochen0418.github.io/reflex_grid/compatibility.html)
- [VBA Excel cases](https://github.com/milochen0418/reflex_grid/blob/main/tests/vba_excel_cases.py)
- [Excel expected results](https://github.com/milochen0418/reflex_grid/blob/main/tests/vba_excel_expected.json)
- [Excel object cases](https://github.com/milochen0418/reflex_grid/blob/main/tests/vba_object_cases.py)
- [Excel object expected results](https://github.com/milochen0418/reflex_grid/blob/main/tests/vba_object_expected.json)
- [Excel oracle script](https://github.com/milochen0418/reflex_grid/blob/main/scripts/excel_vba_oracle_mac.py)

Its public report describes hundreds of cases for VBA language and Excel objects against recorded Excel outcomes. **Use the test *behaviours* and oracle methodology**, after checking licence and clean-room/corpus rules; do not transplant an unrelated interpreter's implementation or claim every expected case is portable to your chosen Windows profile.

**B. Independent VBA library integration corpus**

- [VBA-JSON](https://github.com/VBA-tools/VBA-JSON) — inspect the `specs/` workbook and `JsonConverter.bas`; exercises Variant, class/collection, arrays, string processing and library calls. External dependencies and OS-specific APIs must be identified.
- [VBA Project Starter](https://github.com/jlandgre/VBA_Project_Starter) — `DemoStudy.xlsm` and `tests_DemoStudy.xlsm`, including assertions and cross-workbook dependencies; advanced test, not initial benchmark.

**C. Real `.xlsm` workbooks**

- [Contextures Excel sample files](https://www.contextures.com/excelfiles.html) — macro-driven workbooks and UserForms; select representative samples and record test setup, inputs and expected workbook states yourself.
- [chapmjs/vba-samples](https://github.com/chapmjs/vba-samples) — `.xlsm` examples involving variables, objects, data, forms and file access. Some are **unsafe/unavailable under restricted macro policies**; classify them before execution.
- [dtripathi18/Excel-VBA-Projects](https://github.com/dtripathi18/Excel-VBA-Projects) — complete VBA automation examples; assess dependencies/permissions/availability before adopting fixtures.

**D. Grammar and comparative implementations**

- [harumiWeb/tree-sitter-vba](https://github.com/harumiWeb/tree-sitter-vba) — MIT grammar/corpus for syntax coverage; check licences of bundled third-party fixtures independently.
- [Oxi `oxivba-core`](https://github.com/Ryujiyasu/oxi/tree/main/crates/oxivba-core) — MPL-2.0, host-independent interpreter, useful as a design comparison. Not to be copied into dual-licensed files.
- [elixcee](https://github.com/kent-tokyo/elixcee) — MIT, Excel-oriented headless runtime and useful test/diagnostics organisation, but intertwined with its workbook implementation.
- [scriptandcompile/vb6](https://github.com/scriptandcompile/vb6) — MIT, modular VB6 parse/semantic/runtime/interpreter crates, inspect debugger techniques but do not confuse VB6 semantics with VBA compatibility.
- [RDCore](https://github.com/rubberduck-vba/RDCore) — a specification-driven .NET VBA runtime project; architecture reference only, audit GPL/project-licensing boundaries.
- [LibreOffice](https://www.libreoffice.org/) — behavioural comparison through normal application interfaces, **not source for copying** into a clean-room MIT/Apache project.

**Fixture policy:** inventory each specimen (`source URL`, upstream commit/version, licence, hash, required capabilities, host profile, safe/unsafe classification, expected behaviour). Check redistribution permissions. Prefer an external pinned corpus and self-authored programmatically generated fixtures. Never run arbitrary downloaded macros with unrestricted filesystem/network/COM rights.

### 11.6 Recommended initial benchmark composition

As a practical first cross-implementation benchmark, curate **100 automatically asserted scenarios plus 20 independently sourced `.xlsm` workbooks** (these are proposed targets, not a claim that the corpus has already been assembled). A useful initial distribution is 40 language/coercion, 30 Range/Cells/Worksheet, 15 workbook/events, 10 classes/collections/errors, and 5 import/round-trip scenarios. Classify the workbooks by dependency and difficulty: simple recorded macros, ordinary data manipulation, classes/collections, events, and UserForms/external APIs (expected to remain unsupported initially). Track the proportion of applicable tests passing within each category. Select a small executable subset for early milestones; do not make the first compiler milestone wait for all 100 tests to pass.

### 11.7 Test report artefacts

Build a CLI report with:

- counts by test tier, requirement, fixture source and outcome status;
- mismatch details including expected/actual *type*, value, error number and workbook diffs;
- coverage by VBA feature and Excel member rather than crude total lines;
- reproducible minimal source fixture/seed;
- regression links/issue IDs;
- oracle version/profile and exact tested commit SHA.

Have CI gate *regressions in previously passing tests* and the absence of crashes/panics; do not initially gate on an arbitrary overall pass percentage across unsupported features.

---

## 12. Implementation roadmap and executable acceptance gates

Each milestone must add code, tests, documentation and an independently runnable example. Do not claim completion merely because APIs/types exist. Update the compatibility matrix with `Implemented`, `Partial`, `Not implemented`, `Blocked by policy` and the supporting tests.

### M0 — Repository audit, reference lock and test harness

**Deliver:** dependency/layering graph; agreed host interface with existing compatibility layer; reference list; test manifest schema; at least 20 pure-VBA reference test cases transcribed as independently authored fixtures with traceable expected outcomes; CLI skeleton and CI workflow.

**Gate:** a fake-host test can run from the CLI (initially a stub with an explicit not-implemented diagnostic); fixture parser validates/rejects malformed manifests; no circular dependencies or prohibited licences.

### M1 — Source frontend

**Deliver:** `.bas` input, lexer, parser, source ranges, diagnostics, key expressions, declarations, Subs/Functions, simple statements, compile errors.

**Gate:** parse known valid snippets and reject known invalid ones with precise source location; error recovery does not panic; initial external grammar corpus subset evaluated and results reported. Keep a matrix of unsupported grammar features.

### M2 — Semantic binder and bytecode

**Deliver:** scopes, symbols, `Option Explicit`, variables/types, procedure signatures, argument binding subset, deterministic lowering, IR verifier/disassembler, source maps.

**Gate:** compile sample code to bytecode; verify jump targets/stack balance/slot references; source-map tests for colon-separated statements and nested calls; invalid bytecode cannot crash the VM.

### M3 — VM + debugger vertical slice **(first meaningful demonstration)**

**Deliver:** explicit stack frames/program counters, locals, simple arithmetic, loops, calls, execution budgets, fake Excel host, breakpoints, pause/resume, stepping, variable inspection/mutation.

**Gate:** run the demonstration below; pause on cell assignment; modify `i` at a breakpoint; continue; verify fake worksheet values, correct stack, no frozen host state; pass native + WASM smoke tests.

```vb
Public Sub FillCells()
    Dim i As Long
    For i = 1 To 10
        Cells(i, 1).Value = i * 10
    Next i
    MsgBox "Finished"
End Sub
```

`MsgBox` is a scripted host response at this stage. Breakpoint stepping must not require restarting the macro or replaying earlier mutations.

### M4 — VBA values and language semantics

**Deliver:** typed Variant data, conversions, dates/currency, arrays, ByRef l-values, named/optional arguments, strings, builtins and error handling. The precise order should be driven by conformance dependencies, not necessarily by the order above.

**Gate:** a growing subset of ReflexGrid and Microsoft-verified pure-language scenarios passes with exact error/type/value comparisons; no unimplemented instruction silently succeeds. Implement `On Error` as explicit VM state, not catch-all recovery.

### M5 — GridCraft host adapter

**Deliver:** adapter to the shared Office compatibility layer; `Application`, workbook/worksheet/range/collection object hierarchy; reading/writing; bulk arrays; formula/recalc semantics; robust host errors.

**Gate:** real GridCraft workbook edits from VBA match deterministic expected values and maintain undo/recalc invariants; both active and qualified references work. Existing engine tests continue to pass.

### M6 — `.xlsm` project import/preservation and trust

**Deliver:** ZIP/project extraction, module/code-name binding, original macro part preservation, macro command, explicit enablement UX hook, source diagnostics for imported files.

**Gate:** load/import/execute a self-authored `.xlsm` and at least five independently sourced workbooks under controlled policy; a no-op open/save preserves macro payload where promised; a macro cannot execute on untrusted file-open alone.

### M7 — Events, state and more Excel operations

**Deliver:** document module events, application-level enable/disable, reset/static state, sort/copy/insert and other priority object model members, event reentrancy limits.

**Gate:** event-order differential tests and at least ten real workbook workflows; abnormal stop does not corrupt GridCraft undo/history or leave an unclosable macro session.

### M8 — Developer experience / DAP

**Deliver:** DAP adapter, debugger CLI and editor integration path, conditional breakpoints, watch/immediate support with side-effect controls, call stack and diagnostic improvements.

**Gate:** automated DAP integration test with scripted `setBreakpoints` → `continue` → `stackTrace` → `variables` → `next` → `disconnect`; no state leak after disconnect.

### M9 — Extended Excel corpus and conformance

**Deliver:** broad VBA and Excel model coverage, source fidelity, interop with independent self-test workbooks (e.g. VBA-JSON), round-trip tests, performance benchmarks.

**Gate:** publish a truthful feature-by-feature report including unsupported platform features; all prior passing tests continue passing; no fuzz-induced crashes in committed regression corpus.

### M10 — Advanced features (separate plans)

UserForms and MS-OFORMS parsing, class/event refinements, VBA project editing/writing, cross-workbook automation, allowlisted external objects, WordCraft adapter, richer debugger visual UI. Each advanced feature gets its own design and security review. Do not silently fold them into the MVP.

---

## 13. CLI and developer workflow

Provide commands equivalent to the following (names are proposed; final implementation may adjust them):

```bash
vba-cli parse --source samples/Module1.bas --diagnostics json
vba-cli check --project samples/project/ --profile excel365-windows64
vba-cli compile --project samples/project/ --dump-ir
vba-cli run --source samples/Module1.bas --entry Module1.Demo --host fake-excel
vba-cli inspect-project --input samples/example.xlsm
vba-cli run --workbook samples/example.xlsm --macro Module1.FillCells \
  --capabilities workbook-read,workbook-write --out result.xlsm
vba-cli conformance --suite tests/reference-oracle --report out/report.json
vba-cli debug --source samples/Module1.bas --entry Module1.Demo
```

For incomplete commands, return `UNSUPPORTED` or an explicit feature error, not a mock success. No executable should save over source/input workbooks without explicit user instruction and safe staging.

Standard CI and engineering gates:

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- `cargo test` for each crate and integration workspace.
- WASM build/smoke test; follow repository `cargo xtask wasm` when integrated.
- Architecture/layering checks, crate licence/SPDX checks, dependency audit as appropriate.
- Property-based/fuzz tests for syntax, coercion and hostile files.
- Deterministic compatibility report from pinned fixtures.
- Documentation refresh at every milestone, including test coverage and what remains unsupported.

For development by multiple agents, split work by crate and interface. Stabilise types/contracts before parallel implementation. Require small changesets with a passing subset of tests and a written rationale for any semantic rule that diverges from the reference behaviour.

---

## 14. Detailed implementation rules for the coding LLM

1. **Read the codebase first.** Inspect actual GridCraft/Office-layer API, current `AGENTS.md` and workspace layers. Do not code against imagined traits from this document.
2. **Start with the smallest end-to-end slice.** Parser → binder → bytecode → VM → fake host → breakpoint. Only after it works, extend features.
3. **Preserve modular boundaries.** Keep frontend, runtime, VM, debugger, file import and app integration separate; define trait objects/ownership sensibly.
4. **Never silently approximate a core VBA behaviour.** Return compile errors, VBA runtime errors or `UNSUPPORTED` as appropriate. Add tests for unsupported cases too.
5. **Centralise semantics.** One coercion implementation, one ByRef argument mechanism, one error-state model, one member dispatch path, one test manifest schema.
6. **Use real reference behaviour.** Cite MS-VBAL sections or an Excel differential fixture in code review/tests when resolving tricky semantics.
7. **Do not claim full compatibility.** State the exact test suite size/version and pass/fail/unsupported categories.
8. **No code copied from incompatible licences.** MPL/GPL projects can inform requirements or be used for independent testing, not copied into MIT/Apache source. Third-party data licences can differ from code licences.
9. **Do not implement another workbook engine.** If GridCraft cannot perform an Excel operation, add the capability at the appropriate existing GridCraft/compatibility layer or mark it unsupported.
10. **Do not use JSON-RPC for hot-path host member calls.** Use Rust-native direct dispatch and batch range operations; JSON/DAP can be used for test/UI control planes.
11. **Compile for native and WASM early.** Do not postpone browser compatibility until the VM depends on threads/OS-only facilities.
12. **Respect error/undo/event semantics.** Test partial side effects, event nesting and pausing across calls, rather than treating every macro as an atomic database transaction.
13. **Build security into host APIs.** Per-execution capability policy and trust checks must apply to all paths, including late binding, callbacks and debugger evaluation.
14. **End each milestone with a usable result.** Record exact commands to run, sample output and known gaps. No placeholder methods disguised as implementation.
15. **Ask only for genuine decisions that block correctness.** Otherwise use the defaults in this spec, document assumptions and proceed with working code/tests.

### Required first implementation PR / change set

The first meaningful code change should contain:

- Workspace/crates and architecture boundary validation.
- Minimal parser and compiler for `Sub`, `Dim`, arithmetic, assignment, `For`, calls/property operations.
- Fake-host `Cells`, `Value`, basic `MsgBox` callback, deterministic workbook state.
- Bytecode VM with stack frames, source spans and step/pause/resume.
- Command-line execution and breakpoint test.
- Test fixtures covering the vertical example and at least five failures/edge cases.
- Explicit roadmap file with subsequent features and a compatibility report showing what does *not* work yet.

---

## 15. Authoritative and useful references

These links were checked as of 26 October 2026; version numbers and repository structures can change. Use public HTML spec landing pages and their latest downloadable normative text rather than depending on a deep link to a specific PDF revision.

### 15.1 Microsoft normative specifications

| Reference | How to use it |
|---|---|
| [MS-VBAL — VBA Language Specification](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/d5418146-0bd2-45eb-9c7a-fd9502722c74) | Language grammar, binding, expressions, semantics and runtime compatibility. Primary normative reference. Published version 2.4 (20 May 2025) on the checked page. |
| [MS-OVBA — Office VBA File Format Structure](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/575462ba-bf67-4190-9fac-c275523c75fc) | Macro project storages, source compression, project/module metadata. Published version 15.0 (19 May 2026) on the checked page. |
| [MS-CFB — Compound File Binary Format](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b) | Compound-file storage containing the VBA project binary. |
| [MS-OFORMS — Office Forms Binary File Formats](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/9c79701a-8c3e-4429-a139-b60ac3a1d50a) | Future UserForms/forms binary import. |
| [SpreadsheetML document structure](https://learn.microsoft.com/en-us/office/open-xml/spreadsheet/structure-of-a-spreadsheetml-document) and [Office file format reference](https://learn.microsoft.com/office/compatibility/office-file-format-reference) | OOXML workbook packaging and `.xlsm` file-format classification; use MS-OVBA for project internals. |

### 15.2 Microsoft behaviour and API documentation

- [Excel object model](https://learn.microsoft.com/en-us/office/vba/api/overview/excel/object-model) — properties, methods, events, enumerations.
- [Excel VBA reference](https://learn.microsoft.com/en-us/office/vba/api/overview/excel) — object model and conceptual documentation.
- [VBA language reference](https://learn.microsoft.com/en-us/office/vba/api/overview/language-reference) — functions, syntax and user-facing semantics.
- [Variant data type](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/variant-data-type) — Variant, Empty, Null and Error.
- [Lifetime of variables](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/understanding-the-lifetime-of-variables) — initialisation, static/module/procedure lifetimes.
- [Range.Value](https://learn.microsoft.com/en-us/office/vba/api/excel.range.value) and [Range.Value2](https://learn.microsoft.com/en-us/office/vba/api/excel.range.value2) — essential API conversion tests.
- [Debug Adapter Protocol website](https://microsoft.github.io/debug-adapter-protocol/) and [complete spec](https://github.com/microsoft/debug-adapter-protocol/blob/main/specification.md) — IDE debugger integration.

### 15.3 Rust libraries and comparative engines

| Repo | Licence / role | Recommendation |
|---|---|---|
| [tim-weis/ovba](https://github.com/tim-weis/ovba) | MIT; read-only `vbaProject.bin` source/metadata extraction | Evaluate as a dependency behind your own importer trait. |
| [harumiWeb/tree-sitter-vba](https://github.com/harumiWeb/tree-sitter-vba) | MIT; VBA/VB6 syntax grammar and corpus (third-party fixtures retain separate licences) | Use as parser/corpus reference; linking generated C parser may contradict pure Rust goal. |
| [Ryujiyasu/oxi — oxivba-core](https://github.com/Ryujiyasu/oxi/tree/main/crates/oxivba-core) | MPL-2.0 source; Rust VBA interpreter and host trait | Compare API/behaviour. **Do not adopt/copy** for this from-scratch dual-licensed implementation. |
| [kent-tokyo/elixcee](https://github.com/kent-tokyo/elixcee) | MIT; spreadsheet-oriented Rust macro interpreter | Benchmark supported scenarios; its VM is closely integrated with its own spreadsheet implementation. |
| [scriptandcompile/vb6](https://github.com/scriptandcompile/vb6) | MIT; modular Rust VB6 toolchain | Study semantic/runtime/debug design; VB6 is not VBA. |
| [rubberduck-vba/RDCore](https://github.com/rubberduck-vba/RDCore) | .NET specification-driven VBA compiler/runtime project, licensing restrictions | Study public high-level design only; not a runtime dependency. |

### 15.4 Corpus, oracle and integration references

- [ReflexGrid repository](https://github.com/milochen0418/reflex_grid), [VBA architecture](https://milochen0418.github.io/reflex_grid/vba.html), [compatibility results](https://milochen0418.github.io/reflex_grid/compatibility.html); [language tests](https://github.com/milochen0418/reflex_grid/blob/main/tests/vba_excel_cases.py), [expected outputs](https://github.com/milochen0418/reflex_grid/blob/main/tests/vba_excel_expected.json), [object tests](https://github.com/milochen0418/reflex_grid/blob/main/tests/vba_object_cases.py), [object expected](https://github.com/milochen0418/reflex_grid/blob/main/tests/vba_object_expected.json).
- [VBA-JSON](https://github.com/VBA-tools/VBA-JSON) — actual VBA integration/spec workbook and source.
- [Contextures Excel example files](https://www.contextures.com/excelfiles.html) — real-world `.xlsm` examples, many with UserForms; expected outputs must be curated.
- [chapmjs/vba-samples](https://github.com/chapmjs/vba-samples) — downloadable macro-enabled workbook specimens.
- [dtripathi18/Excel-VBA-Projects](https://github.com/dtripathi18/Excel-VBA-Projects) — larger VBA apps.
- [jlandgre/VBA_Project_Starter](https://github.com/jlandgre/VBA_Project_Starter) — separate workbook with assertion tests and project dependencies.

### 15.5 Host application architecture / process standards

- [GridCraft README](https://github.com/storytold/gridcraft), [architecture](https://github.com/storytold/gridcraft/blob/main/docs/architecture.md), [AGENTS.md](https://github.com/storytold/gridcraft/blob/main/AGENTS.md), [control protocol](https://github.com/storytold/gridcraft/blob/main/docs/control-protocol.md), [file-format parity](https://github.com/storytold/gridcraft/blob/main/docs/file-format-parity.md).
- [WordCraft](https://github.com/storytold/wordcraft) — future integration via host adapter only.
- [Mozilla MPL-2.0 FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/) — relevant when comparing oxivba-core; the new project intentionally avoids incorporating MPL-covered source.

---

## 16. Completion definition and reporting format

The project is **not finished** merely because it can parse `.bas` or display a macro list. The first *usable* Excel-VBA milestone is reached when an imported or directly supplied VBA procedure can reliably edit a real GridCraft workbook and be interactively debugged, with documented unsupported constructs and safe macro trust controls. Broad `.xlsm` compatibility remains a longer-term corpus-measured objective.

For every implementation change, the coding agent must report:

```text
Changed crates/files:
Supported VBA features added:
Supported Excel members added:
Tests added and their expected reference source:
Commands/tests actually executed and results:
Native/WASM build status:
Behaviour that remains unsupported/known mismatches:
Security or licensing implications:
Next smallest task:
```

For the overall effort, maintain `docs/compatibility-matrix.md`, `docs/debugger.md`, and `docs/implementation-progress.md`, updated from actual tests, with stable identifiers for gaps. Prioritise the next work item using real corpus failures, not feature-count estimates.

**Final guiding principle:** the language engine should be replaceable independently of the Excel compatibility layer, the Excel compatibility layer should be usable by other scripting languages, and the entire VBA execution stack should remain testable and debuggable without booting a graphical spreadsheet application.

---

## 17. Risks and required architectural responses

| Risk | Failure mode | Mitigation / proof |
|---|---|---|
| VBA syntax ambiguity | Real macros parse incorrectly despite simple tests | Preserve source positions and raw syntax; use actual `.bas`/`.cls` corpora, negative tests and differential compilation |
| Incorrect `Variant` semantics | Macros run but silently produce wrong results | Shared coercion/operation helpers; type-aware Excel oracle assertions; never represent all numbers as `f64` |
| Default members / ByRef | Common Excel idioms mutate the wrong object or slot | Explicit l-values and invoke kinds; exact property signatures; aliasing and late-binding tests |
| Wrong error handlers | `On Error` changes macro control flow or swallows failures | Explicit per-frame error states, source-aware `Resume` targets, nested-handler tests |
| Runtime cannot pause | Debugger requires interpreter rewrite | Bytecode/explicit frames and source boundary stepping in M3 |
| Duplicating GridCraft | Cell values and undo/recalc drift | Host adapter delegates to existing engine; no VBA-owned workbook store |
| Undo semantics | Failed macros lose earlier valid changes | Distinguish compatibility and transactional preview modes; differential side-effect tests |
| Callback reentrancy | Events/UDFs deadlock or panic due to mutable borrow overlap | Explicit scheduler/session context; short-lived host borrows; event-cycle limits |
| `.xlsm` corruption | Saving destroys project/signatures/unknown package parts | Opaque preservation when possible, relationship checks, round-trip fixtures and explicit fidelity warnings |
| License contamination | Cannot dual-license project as intended | Dependency/provenance inventory and licence audits; no MPL/GPL copied implementation code |
| Untrusted macros | Filesystem/network/shell compromise or infinite loops | Opt-in trust; host capability boundaries; budgets and optionally process isolation |
| WASM divergence | Runtime depends on desktop-only OS behaviour | Test wasm at M3, inject host/platform facilities, no UI or filesystem in core |
| Over-scoped agent work | Broad “VBA supported” claims with placeholder handlers | Milestone gates, feature matrix, executable tests and explicit unsupported errors |

The implementing agent should turn each material risk into a regression test or design-review checklist item early enough to avoid architectural rework.
