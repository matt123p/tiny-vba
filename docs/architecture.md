# Architecture

Tiny-vba owns the reusable VBA language engine, debugger, editor and standalone
application. GridCraft and WordCraft own their Office-compatible object models
and document integration. All ordinary macro-to-host calls are native Rust calls
inside the application's process.

## Compilation and execution

```text
source → lexer/parser → semantic binding → verified bytecode → explicit-state VM
                                                              ↑
                                                        debugger controller
                                                              ↑
                                                     IDE model → egui panels
```

This is the data flow, not a dependency graph. `vba-ir` is independent of the
binder and executor. Compilation selects explicit capabilities; verification
rejects unsupported versions and malformed programs before execution. The current
VM supports IR through version 14, retaining refusal rules for older targets.
See [compiler](compiler.md), [bytecode](bytecode-vm-specification.md) and
[runtime](runtime.md) for the detailed contracts.

## Crate ownership

| Crate | Responsibility | Direct normal dependencies |
|---|---|---|
| vba-diagnostics | Source identities, spans, diagnostic codes | none |
| vba-values | Typed Variant, arrays, references and l-values | vba-diagnostics |
| vba-syntax | Lexer/parser and source-preserving AST | vba-diagnostics |
| vba-semantics | Scope/type/call binding | vba-diagnostics, vba-foreign, vba-runtime, vba-syntax, vba-values |
| vba-ir | Bytecode, verifier, constants and source maps | vba-diagnostics, vba-values |
| vba-compiler | Parse/bind coordination and deterministic verified IR lowering | vba-diagnostics, vba-foreign, vba-ir, vba-semantics, vba-syntax, vba-values |
| vba-runtime | Shared coercion/operators/builtins/error rules; bounded encoding_rs code-page conversion for string/byte intrinsics | encoding_rs, vba-diagnostics, vba-values |
| vba-foreign | Generic host call/result/wait protocol; protocol version 1 and validated source imports | vba-diagnostics, vba-values |
| vba-vm | Frames, instruction pointer, budgets, inspection hooks; host-root binding and protocol checks | vba-diagnostics, vba-foreign, vba-ir, vba-runtime, vba-values |
| vba-debug | Breakpoint/step/inspection controller | vba-diagnostics, vba-vm |
| vba-dap | Optional bounded M10 Debug Adapter Protocol frontend | serde_json, vba-ide |
| vba-project | Bounded MS-OVBA/native form decoding, source-only encoding and inert raster resources | cfb, encoding_rs, image, vba-diagnostics |
| vba-testkit | Deterministic generic foreign host, trace support and bounded embedding driver | vba-debug, vba-diagnostics, vba-foreign, vba-ir, vba-values, vba-vm |
| vba-conformance | Scenario validation, typed expectations and outcome judgment | vba-diagnostics, vba-testkit, vba-values, vba-vm |
| vba-cli | Source parsing, compilation/disassembly/execution, fixture validation and optional DAP stdio | vba-compiler, vba-conformance, vba-dap, vba-diagnostics, vba-ide, vba-ir, vba-syntax, vba-values, vba-vm |
| vba-ide | Toolkit-independent source buffers, commands, compiler/debug-session orchestration and safe watches | vba-compiler, vba-debug, vba-diagnostics, vba-foreign, vba-ir, vba-project, vba-runtime, vba-syntax, vba-values, vba-vm |
| vba-ui-egui | Reusable Classic editor/debugger rendering; explicit commands and scoped theme/IDs | egui, vba-ide |
| vba-ide-app | Native/browser shell, source IO, preferences and capture harness | eframe, image, js-sys, vba-ide, vba-ui-egui, wasm-bindgen, wasm-bindgen-futures, web-sys |
| xtask | Maintenance/build/render commands | none |


[tools/architecture.json](../tools/architecture.json) is the reviewed dependency
graph; `cargo xtask layers` checks it against Cargo metadata. Shipping libraries
may use testkit/conformance only as development dependencies. The CLI has a
reviewed normal conformance dependency. No crate automatically loads corpus code.

## Values, calls and state

`vba-values` preserves exact VBA tags, array bounds, canonical locations and
object identity. `vba-runtime` centralizes conversion, operators and intrinsic
semantics, including conversions used by binding and debugger edits.

The VM uses explicit frames, operand stacks and statement checkpoints. VBA calls
do not recurse through the Rust stack. Session identity is non-reused and separate
from the host's routing label. Source revisions, pause generations and frame
identity prevent stale debugger handles from accessing a different execution.

Host waits retain a continuation: dispatch happens once, and an explicit resume
polls the correct token. Pauses and yields preserve lifetime counters. Cancellation
releases pending work through the host protocol. Completed document effects remain
observable after errors or Reset; the VM does not provide document rollback.

## Host boundary

`vba-foreign` defines generic invocation, reference, permission, result, patch and
wait contracts. It contains no Excel/Word object catalogue or document storage.
The host supplies imports, immutable reference/type metadata and object roots;
source code cannot grant itself capabilities. Object references remain opaque,
non-owning tokens whose generations and validity are checked by the host.

Forks own Office dispatch, workbook/document objects, event ordering, trust,
protection, OOXML packaging, undo and recalculation. There is no parallel Office
object registry in the VM. Host borrows must end before callbacks, waits or pauses.
See [host integration](host-integration.md) for embedding and persistence boundaries.

## Editor and debugger

`vba-debug` controls the VM; the VM never depends on the debugger, DAP or UI.
`vba-ide` owns toolkit-independent project/source/session state and commands.
`vba-ui-egui` renders it inside a caller-owned `Ui`, with independent widget IDs
and injected themes. It must not replace global fonts, scaling or window settings.
`vba-ide-app` supplies native/browser IO and preferences. DAP is an optional
frontend above the IDE model.

Inspection is bounded and does not call arbitrary getters/functions. Immediate
function evaluation requires explicit policy and an isolated VM; permitted host
effects remain real. [Debugger](debugger.md) and [UI](editor-debugger-ui.md)
document these policies.

## Native projects and forms

Project decoding is inert and bounded. Original binary resources, canonical
extracted source, authored editor state and live runtime values have separate
ownership. Native form behavior uses ordinary generated VBA classes and canonical
VM fields; rendering reads inert snapshots rather than another writable model.

Shared codecs handle supported MS-OVBA/native form records. The IDE owns snapshot
and revision checks; shells own file commits; forks own package commits and trust.
Ordinary document Save retains imported VBA bytes. Explicit edited-copy actions
encode supported source/forms and remove invalidated signatures. See
[project import](project-import.md) and [forms](forms.md).

## Safety and portability

The core has no ambient filesystem, network, shell or COM authority. Time,
randomness, locale and prompts are explicit services. Input-derived allocations,
recursion and work are bounded. Production Rust forbids unsafe code, unchecked
input indexing, panic placeholders and unwrap/expect.

Headless libraries build without UI toolkits, application siblings or reference
checkouts. Native, browser WASM and actual WASI execution are separate checks;
compilation alone does not establish runtime compatibility. See [testing](testing.md).

## Design rationale

The [decision index](decisions/README.md) groups adopted architectural decisions
by subject. Decision numbers are stable historical identifiers; their milestone
labels describe the context when written. Current support belongs in the
[compatibility matrix](compatibility-matrix.md), not in a chronological work log.
