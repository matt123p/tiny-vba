---
name: tiny-vba-development
description: Implement or review tiny-vba changes using the project design contracts. Use as the entry point for repository development and route to the focused architecture, semantics, debugger, UI, host or conformance skill.
---

# tiny-vba development

Read [AGENTS.md](../../../AGENTS.md), [architecture](../../../docs/architecture.md)
and [compatibility](../../../docs/compatibility-matrix.md), then inspect the actual
code. Historical milestone labels are not a task queue or a support guarantee.

Select only the focused skills needed:

| Task | Skill |
|---|---|
| Ownership/dependencies/contracts | [architecture](../tiny-vba-architecture/SKILL.md) |
| Parser/binder/values/IR/runtime | [language semantics](../tiny-vba-language-semantics/SKILL.md) |
| Stepping/inspection/DAP/lifecycle | [debugger](../tiny-vba-debugger/SKILL.md) |
| Panels/theme/input/native/browser | [editor UI](../tiny-vba-editor-ui/SKILL.md) |
| Foreign calls/references/project import | [host integration](../tiny-vba-host-integration/SKILL.md) |
| Fixtures/oracles/corpus/evidence | [conformance](../tiny-vba-conformance/SKILL.md) |

Identify the owning layer and current contract before a material choice. Preserve
existing supported workflows and explicit refusals. Add a decision for material
contract changes and update the owning guide/compatibility claims; do not append
milestone diaries to multiple files. Follow [development](../../../docs/developer-guide.md)
and [testing](../../../docs/testing.md) for the applicable gates.
