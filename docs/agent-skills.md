# Agent design skills

The canonical skills live in `.agents/skills/<name>/SKILL.md`. Codex and OpenCode both discover
this repository location; maintain one set rather than copies in tool-specific directories.
Launch the agent in this repository or a subdirectory so repository discovery applies.

| Skill | Use for |
|---|---|
| [tiny-vba-development](../.agents/skills/tiny-vba-development/SKILL.md) | Entry point, task routing, design decision workflow and delivery gates |
| [tiny-vba-architecture](../.agents/skills/tiny-vba-architecture/SKILL.md) | Ownership, crate layering, dependencies and public contracts |
| [tiny-vba-language-semantics](../.agents/skills/tiny-vba-language-semantics/SKILL.md) | Source identity, binding, typed values, coercion and execution |
| [tiny-vba-debugger](../.agents/skills/tiny-vba-debugger/SKILL.md) | Statement stepping, pause/resume, watches and session lifecycle |
| [tiny-vba-editor-ui](../.agents/skills/tiny-vba-editor-ui/SKILL.md) | Gridcraft-compatible egui design, reusable panels and standalone IDE |
| [tiny-vba-host-integration](../.agents/skills/tiny-vba-host-integration/SKILL.md) | Generic foreign protocol, application boundaries and source import |
| [tiny-vba-conformance](../.agents/skills/tiny-vba-conformance/SKILL.md) | Fixtures, corpus policy, oracle evidence and honest support claims |

`AGENTS.md` directs agents to the entry point and relevant focused skills before design changes.
Each skill links to the authoritative contracts; skills do not replace decision records or introduce
new product scope. Current user direction and adopted decisions take precedence over historical
supplied specifications. Record material contract changes and update affected skills together.
Skills guide agent judgement; dependency checks and behavioural/visual evidence still verify the work.

## Using the skills

Both tools can choose skills from their descriptions. For explicit use, ask, for example:

- Codex: “Use $tiny-vba-editor-ui to implement the breakpoint gutter.”
- OpenCode: “Use the tiny-vba-editor-ui skill to implement the breakpoint gutter.”
- Either: “Review this new dependency using tiny-vba-architecture.”

If skills are absent from the tool's list, verify the project working directory, inspect local skill
permissions/overrides and restart the session if necessary. No user-level installation, symlinks,
plugin connection or tool-specific config changes are required for this repository layout.

Discovery references checked 2026-10-26:
[Codex skill discovery](https://learn.chatgpt.com/docs/build-skills) and
[OpenCode skill discovery](https://opencode.ai/v2/docs/skills).
