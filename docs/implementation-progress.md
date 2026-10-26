# Implementation progress

Updated 2026-10-26. Current stage: M0 complete; M1 frontend is next.

## Bytecode/VM contract prepared before M1

Authored the adopted IR 1 bytecode/VM implementation specification and ADR 0004. The document
specifies exact instruction operands/stack effects, program/constants/signatures/slots, source
checkpoint mapping, canonical direct ByRef, restricted M0 host adaptation and single-use host
continuations. It defines verifier/disassembly rules, explicit execution states and errors,
checkpoint visit permits, stepping/inspection generations, resource defaults, lowering examples
and M2/M3 positive/negative acceptance obligations. A planned upper-layer vba-compiler owns
compilation/lowering when implemented; no package/dependency graph change is made now.

Updated architecture, debugger, plan, compatibility, development entry points and relevant skills.
This is documentation only: no instruction enum, verifier, compiler, VM, debugger or runtime
behavior was added. All M0 VBA fixtures and external corpus candidates retain their prior status.
M2/M3 remain unimplemented. M1's bounded lexer remains the next smallest implementation task.

Validation: documentation links and instruction/reference consistency reviewed; `cargo xtask ci`
and `cargo xtask wasm` passed after the documentation change. Existing 16 integration tests still
cover M0 only; no bytecode acceptance case is claimed to have run.

## M0 completion

- Implemented SourceId, per-source StatementId, bounded SourceFile, checked UTF-8 byte spans and
  attached diagnostics, including Unicode/CRLF/identity/bounds tests.
- Implemented scalar Value states/widths, scaled Currency/serial Date, exact UTF-16 strings and
  bounded, invalidatable owner-scoped locations. No runtime coercion or arrays yet.
- Implemented bounded schema-1 fixture parser/validator and 20 independently authored MIT pure-language
  cases with typed expectations, rationale, requirements and explicit unmeasured evidence. All remain NOT_RUN.
- Added `vba-cli conformance validate <scenario>`; CLI tests validate all 20 fixtures and reject malformed
  input. Execution commands still fail explicitly; fixture validation is not VBA syntax validation.
- Proved generic Call/Get/Let/Set, omitted/named/ByRef snapshots and ordered delayed writebacks against
  a deterministic scripted host. Tested errors, repeated waits without replay, cancel, stale locations/
  references, wrong host identity, invalid/non-ByRef patches, alias order and resource limits.
- Recorded adopted source/scenario/foreign prototype contracts, lifecycle requirements and M4/M5
  limitations in ADR 0003; updated architecture, reviewed dependency graph, layer checker and focused skills.
  The development CLI's conformance dependency is explicitly allowed; shipping libraries remain isolated.
- No third-party Rust dependencies, downloaded macro execution, Office observations, parser, VM or UI added.

Verification: `cargo xtask ci` passed (format, warning-free Clippy, workspace tests, layers and corpus
integrity); `cargo xtask wasm` passed for all 13 libraries. Sixteen Rust integration tests cover the
M0 infrastructure and CLI, including validation of all 20 fixtures. Corpus integrity remains seven
pins and 646/646 candidate hashes; this is not a corpus compatibility result. WASM runtime behavior,
Office compatibility, macOS/Windows execution and a separate MSRV run were not tested locally.

Next smallest task: M1 bounded lexer for Arithmetic.bas with exact spans and positive/negative token
fixtures, including comments, CRLF/continuations and colon-separated statements. Then implement the
source-preserving parser and record exact unsupported syntax; M0's cases stay NOT_RUN until execution exists.

## Setup history (pre-M0)


Created a standalone Cargo workspace with 13 library crates, vba-cli and xtask, each with src/.
Added current architecture, implementation plan, decisions, adapted AGENTS/CLAUDE and repository-local skill.
Preserved the supplied interpreter spec and copied the two workspace design documents unchanged.
Added test tier/fixture/example/fuzz/report locations and native/WASM/corpus CI jobs.

Cloned six documented test repositories and Microsoft VBA-Docs as pinned submodules.
Source lock tracks notices/revisions; candidate inventory tracks SHA-256, size and NOT_RUN status.
No downloaded macros, oracle scripts, interpreters or workbook files have been executed.
Contextures is tracked as a manual download source; no Word corpus was identified in the supplied docs.
At setup, no parser, binder, runtime, VM, fake-host implementation, project decoder or scenario runner existed.

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

The setup's next tasks (source identities, M0 fixture validation and 20 independent cases) are now
complete as recorded above. See the plan for remaining milestones.

## Setup validation

- `cargo xtask ci`: passed on Linux with Rust 1.99.1 (fmt, Clippy, workspace tests, layers and available corpus hashes).
- Two CLI integration tests passed; these check explicit refusal and help, not VBA execution.
- `cargo xtask wasm`: all 13 libraries compiled for wasm32-unknown-unknown.
- Strict corpus verification: seven Git pins and 646 candidate file hashes checked; no macros executed.
- Current documentation/skill links and unchanged source document copies checked.
- macOS/Windows jobs are configured, not run locally. No MSRV run, fuzzing or VBA runtime tests yet.
