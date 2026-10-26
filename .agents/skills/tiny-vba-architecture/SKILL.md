---
name: tiny-vba-architecture
description: Design or review tiny-vba crate boundaries, public APIs, dependencies and cross-layer changes. Use when choosing where a feature belongs or changing the architecture; covers headless engine and repository-owned IDE layering.
---

# Architecture decisions

Read [architecture](../../../docs/architecture.md), the [actual dependency graph](../../../tools/architecture.json),
[ADR 0001](../../../docs/decisions/0001-repository-scope.md) and
[ADR 0002](../../../docs/decisions/0002-editor-debugger-ui.md).
Inspect the affected Cargo manifests and callers before designing an API.

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
- Testkit/conformance packages are not shipping dependencies. Keep native hot-path host calls
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
