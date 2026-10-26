# Implementation progress

Updated 2026-10-26. Current stage: M0–M2 reviewed subsets complete; M3 VM/debugger execution is next.

## Pre-push verification of M0–M2

On 2026-10-26, reran `cargo xtask ci` (formatting, warning-free all-target Clippy, all 54
workspace integration tests, layer checks and available corpus integrity), `cargo xtask wasm`
(all 14 libraries), and `cargo test --workspace --release --locked` (the same 54 tests in release).
All passed. Strict corpus verification checked all seven pins and 646/646 hashes; the opt-in
five-file grammar_corpus example matched all reviewed syntax classifications. `git diff --check`
passed. Corrected the fixture-schema documentation's stale claim that compile is unavailable.
These checks cover implemented M0–M2 behavior. VM/Office execution remains unavailable; WASM
runtime, other operating systems and separate MSRV testing are not claimed by these local runs.

## M2 completion

Implemented ordered module/procedure scopes and case-insensitive lookup, Option Explicit, scalar
parameters/locals/Function result slots, implicit Variant locals, literal constants, positional ByVal
and exact-type local ByRef binding. Unsupported persistent storage, class/foreign binding, deeper
argument and constant/numeric forms fail explicitly with source spans. Binding preflights AST depth
and budgets source/token/node payload and constant substitution before copying.

Added vba-compiler for parse/bind orchestration and checkpoint-based lowering, including If/ElseIf,
restricted positive integral-step For, direct nested calls, Set/Let modes, Exit and Stop. Retained
same-line/control statement identities and full statement tables after unreachable code removal.
Constants are deduplicated by exact tag/payload in first emitted-use order. Parameters/result/user/
scratch slots have explicit roles. New reviewed edges preserve IR -> diagnostics/values only;
semantics now shares ScalarKind from values. No external Rust dependency or corpus execution added.

Implemented every IR 1 instruction's structural verification, immutable opaque VerifiedProgram,
launch/signature/slot/call/foreign/source ownership checks, V/Location stack simulation, exact maxima,
iterative reachability, bounded verification and exact-tag deterministic disassembly. M2's explicit
ir1-structural capability target is not a runtime implementation. Unknown versions/features/op
capabilities, malformed IR, unreachable blocks and over-budget input cannot create VerifiedProgram.

CLI check/compile/disasm now work. Arithmetic's full reviewed golden matches the nine-instruction
IR 1 example with Integer 6/7, Long result and max_stack=3. M2Control.bas independently demonstrates
If/For/nested calls and ByRef lowering. Execution remains unavailable and all M0/external scenario
execution results remain NOT_RUN. See [compiler contract](compiler.md) and
[ADR 0006](decisions/0006-m2-binding-and-verification.md) for supported forms, budgets and limitations.

Validation: `cargo xtask ci` and `cargo xtask wasm`; compiler/IR/binder/CLI tests and documented
compile/disasm examples. The workspace now has 54 integration tests (8 compiler, 9 IR, 4 binder,
7 CLI, 13 frontend, and 13 existing M0 library tests). Native tests include malformed-IR mutations,
all instruction stack shapes, each verifier budget dimension, exact source diagnostics and bounded
constant amplification. Library WASM evidence is compilation only. No Office oracle, runtime/WASM
execution, macOS/Windows or separate MSRV run is claimed. Corpus integrity remains seven pins and
646/646 candidate hashes; the M1 five-file syntax report is unchanged.

Next smallest task: M3 VM explicit frames/typed local storage/checkpoint dispatcher with hard/slice
budgets and actual capability checks; execute Arithmetic.Answer and assert exact Long(42). Then
implement direct alias calls, debugger/host continuations and native/WASM pause/resume proofs.

## M1 completion (historical stage record)

Implemented a pure-Rust bounded lexer/parser in vba-syntax with retained source and all trivia tokens,
exact checked spans, spelling-preserving names and distinct same-line statement identities. The AST
covers scalar declarations/parameters, Sub/Function, module/class attributes/options, expressions,
assignment/Let/Set, If/ElseIf/Else, numeric For/Next, explicit/implicit calls, Exit and Stop. Named,
omitted and optional syntax is preserved without claiming binding or IR 1 execution support.
SourceFile coordinates now handle LF/CRLF/lone CR consistently. Parser recovery never reports syntax
success, and source/token/nesting/tree/argument/diagnostic limits bound adversarial input.

Added `vba-cli parse <source.bas|source.cls>` with bounded UTF-8 reads, exact source diagnostics and
nonzero rejection. Both self-authored examples and all 20 M0 VBA bodies parse, as do the new class
metadata/control fixtures. All execution results stay NOT_RUN; no binder, IR verifier, VM or UI exists.
The development CLI -> syntax edge is recorded in the reviewed DAG; no dependency was added to syntax.
See [frontend support and limits](frontend.md) and [ADR 0005](decisions/0005-m1-frontend.md).

Reviewed five directly authored MIT tree-sitter-vba examples for explicit syntax-only ingestion.
The two nominally valid modules are UNSUPPORTED (Select Case/property procedures); all three broken
examples are REJECTED at source diagnostics. Five classifications matched review; zero executable
compatibility passes are claimed. [The report](m1-corpus-report.md) records pins, hashes, rights,
capability/profile restrictions and reproducible invocation. Normal tests need no corpus checkout.
No upstream implementation, generated parser, downloaded script or macro was executed or copied.

Validation: `cargo xtask ci`, `cargo xtask wasm`, the opt-in grammar_corpus example and documented
parse commands passed. Thirty-two integration tests now cover infrastructure/frontend/CLI, including
13 frontend tests for preservation, exact source spans, control metadata, call/precedence distinctions,
unsupported/malformed input, nested control statement identities and token/tree/recovery/identifier limits.
Member identifiers enforce the same 128-byte limit as standalone identifiers. The deterministic malformed/truncated
stress check is not a fuzzing campaign. Corpus integrity remains seven pins and 646/646 hashes.
WASM evidence is compilation only; macOS/Windows and a separate MSRV run were not executed locally.

Next smallest task: M2 bound module/procedure scopes and Option Explicit/local/result/parameter binding
against the M1 AST, then the IR 1 program layouts, verifier/disassembler and real compiler/lowering crate.
Numeric semantic legality, property/Select Case/array syntax and the broader grammar remain unsupported
as documented; M1 completion covers its reviewed subset, not the full supplied grammar wishlist.

## Bytecode/VM contract prepared before M1 (historical stage record)

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
