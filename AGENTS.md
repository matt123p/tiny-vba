# tiny-vba — instructions for agents

Adapted from Gridcraft's AGENTS.md and CLAUDE.md for this standalone interpreter.
Read `docs/implementation-progress.md`, `docs/implementation-plan.md`, then the relevant
architecture/decision document. The supplied design specifications are indexed in `docs/README.md`.
Keep plans and progress committed and usable from this checkout.

## Design skills for Codex and OpenCode

Before implementation or a design review, read the repository-local
[development skill](.agents/skills/tiny-vba-development/SKILL.md) and the focused skills it maps
to the task. Both tools use this canonical `.agents/skills` tree. The
[skill catalogue](docs/agent-skills.md) covers architecture, language semantics, debugger,
editor UI, host integration and conformance. Load only the relevant skills.

For each material design choice, identify its owning layer and current contract. Follow current
user direction and adopted ADRs over historical supplied documents. Record material contract
changes in docs/decisions and update affected architecture, progress and skills together.
Routine implementation choices within the agreed scope do not require another approval step.

## Scope and layering

- Implement a fresh Rust VBA engine. Never reuse `craft-vba` or copy another interpreter's implementation.
- Gridcraft/Wordcraft forks own Office compatibility objects, member catalogues, document state,
  host adapters, workbook/document UI, undo/recalc and OOXML packaging.
- tiny-vba owns the reusable VBA editor/debugger UI and its standalone application. Follow
  `docs/editor-debugger-ui.md` and ADR 0002; use Gridcraft-compatible egui/eframe and UI conventions.
  Keep UI dependencies above compiler/runtime/VM/debugger-controller crates.
- `vba-foreign` is a minimal host-neutral protocol, not a second Office AutomationHost.
  Test doubles can record generic calls; Office-shaped contract tests belong in the forks.
- Follow `docs/bytecode-vm-specification.md` and ADR 0004 for IR 1 operands, verification, calls,
  checkpoint/continuation semantics and budgets; the specification is not implemented support.
- The VM uses explicit frames/bytecode and statement checkpoints. Debugger hooks exist before feature expansion.
  `vba-debug` controls the VM; the VM never depends on debugger/DAP/UI crates.
- Follow `docs/architecture.md`; update the reviewed DAG in `tools/architecture.json` when justified.
  `cargo xtask layers` enforces it. Test support stays out of shipping library dependencies.

## Correctness and safety

- Rust only for interpreter/product code. Python is limited to repository maintenance checks.
- No unsafe, unwrap/expect, panic, todo/unimplemented or unchecked input indexing in production code.
  Inherit workspace lints; return structured errors. Bound input-derived allocations, recursion and VM work.
- Centralize VBA coercion, ByRef locations and error state. Preserve Empty/Null/Missing/Nothing/CVErr,
  numeric widths, Currency/Date and array lower bounds. Do not flatten to JSON or f64.
- Use native in-process calls on the host path. JSON is suitable for manifests, reports and DAP only.
- Inject time, randomness, locale and prompts; no ambient filesystem/network/shell/COM in the VM.
- Office project source extraction is read-only. Import must never execute macros.
  No P-code execution or binary VBA project writeback yet; plain-source editing belongs to the IDE.
- Never report success for unsupported syntax, instructions or host operations.
- No vendor artwork or proprietary Office binaries. Independent code and properly licensed fixtures only.
  Observe Office as a black box with permitted fixtures; do not inspect Office implementation binaries.

## Test material and evidence

- Read `tests/corpora/README.md` before corpus work. Pins/notices are in `references/sources.lock.json`;
  specimen hashes and NOT_RUN status are in `tests/corpora/inventory.json`.
- Do not edit upstream submodules, import implementation code, run downloaded scripts/macros,
  or adopt unlicensed samples without an explicit reviewed change in policy/selection.
- Rights, capability needs, host profile and expected outcomes must be established per selected specimen.
  Pinning is not permission, safety review or a compatibility result.
- Rust integration tests live under their owning crate's tests/; root tests/ holds shared data.
- Test correct output types, error numbers, side effects, negative paths, budgets and pause/resume.
  Fixes for crashes need minimized regression tests. Do not invent Microsoft oracle observations.

## Development gates

Run `cargo xtask ci`, `cargo xtask wasm` and `cargo xtask wasm-run`. Install wasm32-unknown-unknown and wasm32-wasip1 as needed; Node 18+ runs the WASI smoke gate.
Local core tests must work without corpus checkouts or application siblings; corpus verification is separate.
Use `cargo run -p vba-cli -- --help` for the current interface. M0 fixture validation, M1 subset parsing and M2 binding/verified IR compilation and the M3 runtime/debugger subset are implemented.
See `docs/frontend.md` for syntax and `docs/compiler.md` for the narrower binding/lowering subset, IR verification and budgets; `docs/runtime.md` defines actual execution and host/debugger capabilities.
Update affected architecture, compatibility, progress and decision documents with changes.
Report exact tests run, support added, remaining gaps and the next smallest task.

Gridcraft's shared craftrules checkout was not available during setup. Its applicable safety,
clean-room, external-corpus and layering conventions are adapted here; no nonexistent sibling
path is required for standalone work. Editor/debugger UI must follow the documented theme,
accessibility, original-asset and font conventions; add UI snapshot/asset checks with the UI implementation.
