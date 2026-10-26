---
name: tiny-vba-development
description: Implement or review tiny-vba changes using the project design contracts. Use as the entry point for repository development and route to the focused architecture, semantics, debugger, UI, host or conformance skill.
---

# tiny-vba development

Read [AGENTS.md](../../../AGENTS.md), [progress](../../../docs/implementation-progress.md)
and the relevant [implementation gate](../../../docs/implementation-plan.md). Inspect actual code
before choosing a change; planned crates and APIs are not implemented capabilities.

## Select the relevant design skill

Load only the skills needed for the task; a change crossing boundaries may need more than one.

| Task | Skill |
|---|---|
| Crate ownership, dependencies, public contracts, design proposals | [architecture](../tiny-vba-architecture/SKILL.md) |
| Parser/binder, values, bytecode, runtime and language behaviour | [language semantics](../tiny-vba-language-semantics/SKILL.md) |
| Pause/resume, stepping, watch evaluation, debugger/DAP lifecycle | [debugger](../tiny-vba-debugger/SKILL.md) |
| Editor panels, theme, interaction, standalone/embedded IDE | [editor UI](../tiny-vba-editor-ui/SKILL.md) |
| Foreign calls, host lifetime/capabilities, project import | [host integration](../tiny-vba-host-integration/SKILL.md) |
| Fixtures, oracle evidence, corpora, compatibility reports | [conformance](../tiny-vba-conformance/SKILL.md) |

For each material design choice, identify its owning layer and applicable contract. Follow current
user direction and adopted decisions over historical supplied design snapshots. Resolve routine
implementation choices within that scope; record material contract changes in docs/decisions and
update affected architecture, plan, compatibility and progress documents. Explain any tradeoff
against an existing invariant rather than silently changing it.

Product code is Rust. Follow the root safety/layering rules and keep failures explicit. Preserve
headless native/WASM use and the required editor/debugger UI owned by this repository.

Run the relevant checks plus the repository gates `cargo xtask ci` and `cargo xtask wasm`.
Report exact verification, unavailable checks and remaining gaps; compilation is not proof of
VBA compatibility. Leave the next smallest implementable task recorded in progress.

[ADR 0010](../../../docs/decisions/0010-m4-profile-and-verification-hardening.md) publishes
m4-scalars-v1; do not silently expand m3-core-v1. Keep failure classifications and nonzero
CRASH/TIMEOUT/MISMATCH exits intact, compare exact payloads, and check narrowing bounds before casts.
Confirm xtask's logged workspace matches the intended checkout; shared binaries may originate elsewhere.
