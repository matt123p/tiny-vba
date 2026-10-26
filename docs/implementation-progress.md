# Implementation progress

Updated 2026-10-26. Current stage: setup complete; M0 harness/contract work remains.

Created a standalone Cargo workspace with 13 library crates, vba-cli and xtask, each with src/.
Added current architecture, implementation plan, decisions, adapted AGENTS/CLAUDE and repository-local skill.
Preserved the supplied interpreter spec and copied the two workspace design documents unchanged.
Added test tier/fixture/example/fuzz/report locations and native/WASM/corpus CI jobs.

Cloned six documented test repositories and Microsoft VBA-Docs as pinned submodules.
Source lock tracks notices/revisions; candidate inventory tracks SHA-256, size and NOT_RUN status.
No downloaded macros, oracle scripts, interpreters or workbook files have been executed.
Contextures is tracked as a manual download source; no Word corpus was identified in the supplied docs.
No parser, binder, runtime, VM, fake-host implementation, project decoder or scenario runner exists yet.

Scope updated on 2026-10-26: this repository owns the reusable egui VBA editor/debugger and
standalone native/web IDE. ADR 0002 and the UI contract define ownership, Gridcraft design
alignment and the required visual acceptance gate. No UI packages or working UI exist yet.
The enforced Cargo graph is unchanged because this change defines scope and implementation gates.

Added seven shared Codex/OpenCode design skills under .agents/skills: development routing,
architecture, language semantics, debugger, editor UI, host integration and conformance.
AGENTS.md routes design/implementation work through relevant skills; docs/agent-skills.md records
shared discovery and usage. These are agent guidance, not new engine/UI implementation.
Validation: all seven skill names, descriptions, bodies and local routing links passed a
dependency-free structural check; bundled quick_validate.py could not run because PyYAML is absent.
`cargo xtask ci` and `cargo xtask wasm` passed after the skill changes. Discovery paths were
verified against official documentation; live Codex/OpenCode skill activation was not exercised.

Next smallest task: source identity/span/statement diagnostic types, then M0 scenario validation and
20 independently authored pure-language reference cases. See the plan for acceptance gates.

## Setup validation

- `cargo xtask ci`: passed on Linux with Rust 1.99.1 (fmt, Clippy, workspace tests, layers and available corpus hashes).
- Two CLI integration tests passed; these check explicit refusal and help, not VBA execution.
- `cargo xtask wasm`: all 13 libraries compiled for wasm32-unknown-unknown.
- Strict corpus verification: seven Git pins and 646 candidate file hashes checked; no macros executed.
- Current documentation/skill links and unchanged source document copies checked.
- macOS/Windows jobs are configured, not run locally. No MSRV run, fuzzing or VBA runtime tests yet.
