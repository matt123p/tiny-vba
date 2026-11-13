---
name: tiny-vba-architecture
description: Design or review tiny-vba crate boundaries, public APIs, dependencies and cross-layer changes. Use when choosing where a feature belongs or changing the architecture; covers headless engine and repository-owned IDE layering.
---

# tiny-vba architecture

Read [architecture](../../../docs/architecture.md), the owning contract and
[decisions by subject](../../../docs/decisions/README.md).

Keep syntax/source, binding, verified IR, shared language rules and VM execution
separate. Only IR constructs VerifiedProgram; structural verification is not a
runtime capability. The VM never depends on compilation/debug/DAP/UI.
`vba-ide` is toolkit/IO-free; panels render it; the app owns shell/platform IO.
Office catalogues, object handles, document trust/storage, OOXML and undo/recalc
belong in forks. Native hot-path host calls remain Rust; JSON is for manifests,
reports and debugger transport. Testkit/conformance are not shipping library
normal dependencies; the development CLI has a reviewed exception.

For boundary changes state the owner, identity, errors, cancellation and resource
lifetime, and assess headless/native/WASM/embedding effects. Update the reviewed
`tools/architecture.json` with actual dependencies, never speculative packages.
External dependencies require license and portability review. Run layers plus
the implementation gates in AGENTS.md. Report implemented and unavailable surfaces
separately; do not infer a host API from an Office-shaped mock.
