# Documentation index

Read [architecture](architecture.md), [implementation plan](implementation-plan.md),
[scope decision](decisions/0001-repository-scope.md) and [progress](implementation-progress.md) first.
[UI ownership decision](decisions/0002-editor-debugger-ui.md),
[editor/debugger UI contract](editor-debugger-ui.md), [developer guide](developer-guide.md),
[debugger](debugger.md), [conformance](conformance-methodology.md)
and [compatibility matrix](compatibility-matrix.md) describe development contracts and current limits.

[M0 contract decision](decisions/0003-m0-source-scenarios-foreign.md) and
[scenario format](scenario-format.md) document the implemented foundations and fixture validator.

[Bytecode and VM specification](bytecode-vm-specification.md) defines the adopted IR 1 instruction,
verification and execution contract; [ADR 0004](decisions/0004-bytecode-vm-contract.md) records the choices.
M2 IR/binding/lowering is implemented; the documented VM execution subset is implemented at M3.
[M2 compiler contract](compiler.md) and [ADR 0006](decisions/0006-m2-binding-and-verification.md) record the current API, budgets and evidence.

[M3 runtime/debugger contract](runtime.md) and [ADR 0007](decisions/0007-m3-execution-and-debugger.md)
record supported execution, generic host continuations and identical native/WASM acceptance.

[Agent design skills](agent-skills.md) lists the shared Codex/OpenCode skills and their task routing.

## Supplied design inputs (preserved)

- [Interpreter implementation specification](vba-interpreter-rust-implementation-spec.md), originally in tiny-vba/docs/.
- [Combined project overview](design/VBA_Combined_Project_Overview.md), copied unchanged from workspace docs/.
- [Office reference and generation plan](design/office_api_reference_and_generation.md), copied unchanged from workspace docs/.

All three available source documents were read during setup on 2026-10-26.
They contain historical repository names and proposals. Current scope is the tiny-vba engine, reusable editor/debugger UI and standalone IDE, plus
compatibility layers and document integration in the Gridcraft and Wordcraft forks. The current decision records
resolve those differences; original documents are not silently rewritten.

The inputs link `office_automation_architecture.md`, but it was absent from both supplied
docs folders. Its detailed Office design is an input gap for fork integration, not a
prerequisite for standalone interpreter development. Other relative links in original
snapshots may also refer to historical locations; use this index to locate available files.

M1 implementation: [frontend subset and API](frontend.md),
[adopted frontend contract](decisions/0005-m1-frontend.md), and
[reviewed external grammar results](m1-corpus-report.md).
