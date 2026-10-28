# Documentation index

The remaining plan uses new M1–M8 numbering. Earlier implementation reports,
specifications and ADRs retain historical IDs; see the
[mapping](implementation-progress.md#plan-renumbering--2026-10-28).

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

[M2/M3 acceptance audit](m2-m3-acceptance-audit.md) records the gate review and
[VM instance isolation](decisions/0008-vm-instance-isolation.md) correction for reused caller labels.

[M4 profile and verification hardening](decisions/0010-m4-profile-and-verification-hardening.md)
records the current consumer profile, truthful fixture outcomes and corrected checkout selection.

[M4 language profile and IR 2](m4-language.md) specifies current semantic support, feature gates,
error handlers, arrays and persistent state. [ADR 0011](decisions/0011-m4-language-extension.md)
records the extension and shared default/constant conversion dependency.

[M4 oracle acceptance report](m4-oracle-report.md) and [ADR 0012](decisions/0012-m4-oracle-acceptance.md)
record the completed external gate, schema 2 provenance and IR 3 extension.

[M5 native host contract](decisions/0013-m5-host-contract.md) records the versioned
foreign/value/debug APIs, hidden IR 4 host-root imports and the reusable bridge driver;
[M7 bounded callbacks and lifecycle](decisions/0015-m7-callbacks-and-lifecycle.md) records the
generic callback queue, static reset and cleanup ownership; [runtime](runtime.md) and
[the compatibility matrix](compatibility-matrix.md) carry the implemented surface.

[M6 read-only project import](project-import.md) and [ADR 0014](decisions/0014-m6-project-decoding.md)
record the bounded decoder, reviewed external dependencies, source identities, protection refusals
and controlled native/WASI acceptance.

## M8 UI implementation

- [User-supplied public-source UI specification](ui/craft_vba_editor_debugger_ui_spec.md)
- [Implementation plan](ui/m8-implementation-plan.md), [ADR 0016](decisions/0016-m8-embeddable-editor.md)
- [M10 implementation and acceptance map](ui/m10-implementation.md)
- [ADR 0018: classic tools and debugger policy](decisions/0018-m10-classic-tools-and-debugger-policy.md)
- [Remaining UI/host scope: standard Office forms and integration](ui/post-m8-roadmap.md)
- [ADR 0019: remaining plan scope and numbering](decisions/0019-remaining-plan-scope.md)
- [New M1 host-reference implementation log](m1-host-references.md)
- [M1.2/M1.3 default members, collections and acceptance](m1-defaults-and-collections.md)
- [ADR 0022: generic defaults and enumeration](decisions/0022-default-members-and-enumeration.md)
- [ADR 0020: explicit host reference snapshots](decisions/0020-explicit-host-reference-snapshots.md)
- [ADR 0021: nominal host object identity and validation](decisions/0021-nominal-host-object-types.md)
- [Run, embed and reproduce visual evidence](ui/embedding.md)
- [Acceptance and broader-spec feature status](ui/m8-acceptance.md)
- [M8 completion audit: corrections, regression coverage and verification](ui/m8-completion-audit.md)
- [Public source manifest](provenance/vbe_sources.yaml), [dependency/font provenance](provenance/ui-dependencies.json)

Downloaded reference evidence is ignored under `.cache/evidence/`; original rendered PNGs are
ignored under `reports/ui/`. Neither is required for the ordinary offline test suite.
