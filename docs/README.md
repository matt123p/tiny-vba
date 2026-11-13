# Documentation

Start with the [root README](../README.md) to run the editor or CLI.
These guides describe the implemented system and its compatibility boundaries.

## Understand the system

| Guide | What it explains |
|---|---|
| [Architecture](architecture.md) | Pipeline, crate ownership, dependencies and host/UI boundaries |
| [Compatibility](compatibility-matrix.md) | Supported profiles, explicit limitations and evidence status |
| [Host integration](host-integration.md) | Native calls, references, events, trust and application ownership |
| [Native forms](forms.md) | Controls, designer/runtime state, native saving and edited package copies |
| [Editor/debugger UI](editor-debugger-ui.md) | Shared model, renderer, accessibility and interaction contracts |
| [Embedding](ui/embedding.md) | Run the shell, mount panels and handle source IO/host requests |

## Language and execution contracts

| Reference | Focus |
|---|---|
| [Frontend](frontend.md) | Source identity, parsing, grammar and diagnostics |
| [Compiler](compiler.md) | Binding, capabilities, lowering and verification |
| [Bytecode and VM specification](bytecode-vm-specification.md) | Foundational IR 1 instruction/stack/continuation contract; versioned extensions live in the decisions |
| [Language profile](m4-language.md) | Scalar/coercion, array, argument, handler and persistent-state contracts |
| [Runtime](runtime.md) | Sessions, host lifecycle, inspection and resource accounting |
| [Debugger](debugger.md) | Checkpoints, stepping, watches, Immediate and DAP |
| [Intrinsic inventory](vba-intrinsic-functions.md) | Functions, semantic differences and injected services |
| [Intrinsic design](intrinsic-design.md) | Descriptor catalogue, argument plans and IR 8 call verification |
| [Project import](project-import.md) | Inert MS-OVBA/native form decoding, identity and bounds |
| [Module contexts](vba_module_types_and_execution_contexts.md) | Module kinds, code names, scope and lifetime |
| [Name and object resolution](vba_excel_context_and_object_resolution.md) | Host/source lookup precedence and object context |
| [Attributes and code projection](vba-attributes-and-editor-projection.md) | Lossless source/metadata ownership and editor presentation |

## Develop and verify

- [Developer guide](developer-guide.md): toolchain, commands and contribution conventions.
- [Testing and evidence](testing.md): gates, immutable selections/receipts and provenance.
- [Scenario format](scenario-format.md): typed fixture schema and outcomes.
- [Release comparison protocol](m8-office-comparison.md): fresh Office capture and validation.
- [Historical inherited-results report](m4-oracle-report.md): source rights and exact comparison basis.
- [Corpus policy](../tests/corpora/README.md), [fuzzing](../fuzz/README.md) and [examples](../examples/README.md).
- [Agent skills](agent-skills.md): focused repository development guidance.

## Architectural decisions and historical evidence

[Decisions by subject](decisions/README.md) retain rationale and versioned contracts.
Their milestone labels and dated evidence describe their original context, not
current task status. The compatibility matrix is the maintained support summary.

Implementation plans, progress diaries, task ledgers and duplicated supplied-design
snapshots have been removed; their history remains in Git. Two hash-pinned scope
snapshots remain solely as [acceptance evidence](testing.md#selections-and-recorded-results).
They are not maintained work plans. License notices, provenance manifests and raw
fixture/receipt data remain alongside their owning artifacts.
