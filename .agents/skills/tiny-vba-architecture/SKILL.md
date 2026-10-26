---
name: tiny-vba-architecture
description: Design or review tiny-vba crate boundaries, public APIs, dependencies and cross-layer changes. Use when choosing where a feature belongs or changing the architecture; covers headless engine and repository-owned IDE layering.
---

# Architecture decisions

Read [architecture](../../../docs/architecture.md), the [actual dependency graph](../../../tools/architecture.json),
[ADR 0001](../../../docs/decisions/0001-repository-scope.md) and
[ADR 0002](../../../docs/decisions/0002-editor-debugger-ui.md).
Inspect the affected Cargo manifests and callers before designing an API.
Read the adopted [IR 1 contract](../../../docs/bytecode-vm-specification.md) and
[ADR 0004](../../../docs/decisions/0004-bytecode-vm-contract.md) for bytecode/VM work.
Compilation/lowering belongs in the implemented upper-layer vba-compiler;
do not add documentation-only graph nodes or put reusable lowering inside the CLI/VM.

## Place responsibility before adding code

- Syntax preserves source; semantics binds names/calls; IR verifies bytecode; runtime centralizes
  language rules; VM owns explicit execution state. Keep these separable and testable headlessly.
- VM inspection hooks support vba-debug; debug controls VM; optional DAP wraps debug. Never reverse
  those dependencies or put egui/DAP types into the VM.
- This repository owns reusable editor/debugger panels and the standalone IDE. Planned vba-ide
  holds toolkit-independent state/actions, vba-ui-egui renders them and the app owns its shell.
  UI packages are optional for headless consumers; the visual product remains a required deliverable.
- Office catalogues, document storage, undo/recalc, OOXML and application bridges stay in the forks.
  vba-foreign is a minimal generic protocol. Neither Office types nor application sibling paths
  belong in core or reusable UI dependencies.
- Testkit/conformance packages are not shipping library dependencies. ADR 0003 permits the
  development CLI to depend on conformance for fixture validation; keep that exception out of engine/UI libraries.
  Keep native hot-path host calls
  in Rust; JSON belongs to manifests, reports and optional debugger transport.

For a new contract, describe the owner, data/identity semantics, errors, cancellation and resource
lifetime. Prefer the smallest compiling proof against a deterministic generic host before expansion.
Do not invent APIs for Gridcraft; inspect its actual contract when integration is in scope.

## Make a decision reviewable

Compare the proposal against the relevant milestone and existing interfaces. State the concrete
reason for a boundary change and its effect on headless builds, WASM, embedding and compatibility.
Record material changes in an ADR and update the architecture alongside the implementation.
Keep tools/architecture.json synchronized with actual packages/dependencies; do not insert planned
packages merely to reserve names. External dependency adoption includes licence and portability review.

Run `cargo xtask layers`, then repository CI/WASM gates. Demonstrate headless consumers do not
pull UI/platform dependencies when UI packages are introduced. Report planned versus working APIs.

M1 implements vba-syntax's lexer/parser/AST and adds CLI -> syntax for source parsing under ADR 0005.
Its only library dependency is diagnostics. Syntax success, binding validity and execution results
remain distinct. See [frontend](../../../docs/frontend.md) before consuming recovered trees.

## Implemented M2 contract

Read [compiler](../../../docs/compiler.md) and [ADR 0006](../../../docs/decisions/0006-m2-binding-and-verification.md).
Reuse vba-values ScalarKind, vba-semantics bound symbols and vba-compiler lowering. The compiler always
passes vba-ir verification; only vba-ir constructs VerifiedProgram. Keep IR independent of binding and
VM independent of compilation/debug/UI. M2 ir1-structural capabilities describe verification shapes,
not runtime support. Actual execution must check the M3 runtime matrix. Preserve exact constant tags,
ordered slots/sites, complete nonexecutable statement metadata and explicit unsupported diagnostics.

## Implemented M3 contract

Read [runtime](../../../docs/runtime.md) and [ADR 0007](../../../docs/decisions/0007-m3-execution-and-debugger.md).
The m3-core-v1 runtime is intentionally narrower than structural IR. Preserve explicit frame/alias
storage, one-visit checkpoint permits, saved fault PCs, generation checks and nonreplayed continuations.
Use vba-runtime for operations/edits. VM host dispatch must use checked precommit return/writeback
validation and explicit per-site permissions; foreign aliases remain unsupported. Run cargo xtask ci,
wasm and wasm-run (native plus actual WASI execution). Runtime example results do not mark unrun M0
or external scenario assertions PASS. Graphical UI, DAP and VBA error-handler semantics remain later work.

[ADR 0008](../../../docs/decisions/0008-vm-instance-isolation.md) distinguishes the caller's routing
label from the non-reused SessionInstance. Key handles, locations, breakpoints and stepping by the
engine instance; caller-label/frame/generation equality alone cannot identify an execution. Include
the replacement-session regression in native/WASI evidence when changing these identities.
