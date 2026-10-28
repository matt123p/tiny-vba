---
name: tiny-vba-architecture
description: Design or review tiny-vba crate boundaries, public APIs, dependencies and cross-layer changes. Use when choosing where a feature belongs or changing the architecture; covers headless engine and repository-owned IDE layering.
---

# Architecture decisions

The remaining plan now uses new M1–M8 numbers under ADR 0019. Implemented contract
IDs in this skill and earlier ADRs are historical; use the progress mapping when
planning future work. Office-supplied standard controls remain in scope, third-party
ActiveX/OLE controls do not. Native extensions/translation and AI are not scheduled.

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
- VM inspection hooks support vba-debug; debug controls VM; optional DAP wraps the headless IDE. Never reverse
  those dependencies or put egui/DAP types into the VM.
- This repository owns reusable editor/debugger panels and the standalone IDE. Implemented vba-ide
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
or external scenario assertions PASS. M8/M10 graphical UI and optional DAP are implemented above
the headless engine; use ADR 0018 for their policy. Current handlers belong to the M4 profile.

[ADR 0008](../../../docs/decisions/0008-vm-instance-isolation.md) distinguishes the caller's routing
label from the non-reused SessionInstance. Key handles, locations, breakpoints and stepping by the
engine instance; caller-label/frame/generation equality alone cannot identify an execution. Include
the replacement-session regression in native/WASI evidence when changing these identities.

The current scalar consumer is m4-scalars-v1 under [ADR 0010](../../../docs/decisions/0010-m4-profile-and-verification-hardening.md).
Compile against current VM capabilities; legacy M3-tagged programs require recompilation. Inspect
the xtask workspace banner before trusting gate evidence from a shared target cache.

Current M4 support supersedes earlier first-slice limits: read [M4 language/IR 2](../../../docs/m4-language.md)
and [ADR 0011](../../../docs/decisions/0011-m4-language-extension.md). Compile against m4-language-v1
VM capabilities. Preserve the IR 1 structural target and explicit IR 2 feature/version gates.
The authored inventory is 20 M0 plus 24 M4 PASS fixtures; no Microsoft oracle observation is claimed.

M4 external acceptance is now implemented under [ADR 0012](../../../docs/decisions/0012-m4-oracle-acceptance.md).
Read [the report](../../../docs/m4-oracle-report.md) and current language contract: 44 authored
plus 153 inherited upstream-reported Excel Mac scenarios PASS. Schema 2 and the reviewed allowlist
preserve raw results/environment/hashes; no fresh Office run is claimed. Use m5-host-v1
and explicit IR 3/4 capabilities, retaining IR 1/2 refusal rules. Unselected external cases remain NOT_RUN.

M6 is implemented under [ADR 0014](../../../docs/decisions/0014-m6-project-decoding.md).
Read [the import contract](../../../docs/project-import.md) before changing vba-project.
Keep extraction read-only/inert, project-local ModuleId separate from caller-owned SourceId,
UTF-8 attribute spans and code names distinct from stream/display names. Preserve explicit
corrupt/unsupported/protected/limit refusals and container/decompression work caps. cfb/encoding_rs
are reviewed pinned external dependencies; package preservation and trust UX remain fork-owned.
The 14 controlled importer tests run natively and under WASI through cargo xtask wasm-run.

## Implemented M8 editor/debugger

Read [ADR 0016](../../../docs/decisions/0016-m8-embeddable-editor.md), the
[embedding/harness guide](../../../docs/ui/embedding.md) and
[acceptance matrix](../../../docs/ui/m8-acceptance.md). vba-ide is toolkit/IO-free; vba-ui-egui
mounts in caller Ui with injected tokens, independent per-view IDs and explicit commands;
vba-ide-app owns native/web IO and preferences. Run cargo xtask ui, ui-native and web plus ordinary
CI/WASM gates for affected UI changes. Keep accessibility checks enabled; render/inspect original
light/dark/narrow/scaled/host-theme/focus output. No downloaded reference evidence enters Git:
.cache/evidence/ is ignored and provenance/notices/pins are verified. Do not claim full C0/C1,
Office event/workbook, Set Next Statement or unsafe evaluation support from standalone UI evidence.
