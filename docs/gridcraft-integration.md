# GridCraft integration work log

Started 2026-10-27. This is the interruption/resumption record for the user-requested
integration. Interpreter and shared IDE remain in tiny-vba; Excel objects and the
adapter belong in GridCraft. WordCraft can consume the same tiny-vba revision.

Branches: tiny-vba `feat/gridcraft-integration`; GridCraft `feat/tiny-vba-integration`.
Both working trees were clean before branch creation. Starting tiny-vba revision:
`f4629598b7ce475f5fef4b478c5b6d93338877b6`.

## Stages and acceptance

1. Inspect M5/M7/M8 contracts and actual GridCraft seams; record design before code.
2. Implement a fork-owned, toolkit-free `gridcraft-vba` adapter above engine. Native
   `vba-foreign` protocol; bounded object handles tied to existing document/sheet
   identities; explicit unsupported-member/type errors. No Office code in tiny-vba.
3. Drive the existing reusable IDE through the adapter. All workbook writes use
   GridCraft commands, so protection, recalculation, recording and undo remain owned
   by the engine. Group successful writes into one undo entry, also on failure/stop;
   do not roll back successful macro effects.
4. Mount shared editor behind an opt-in GridCraft feature using host theme tokens.
   Explicit launch/trust, bounded frame ticks, cleanup on stop/close/document loss.
   Preserve original VBA package bytes; source import never executes code.
5. Test authored workbook macros, types/arrays/errors/negative paths, trust,
   identity/lifecycle, break/edit/resume, undo/recalc, package round trips and UI.
   Run both repositories' CI/WASM gates and inspect rendered integration UI.
6. Record exact results, limits, commands and the next task here. No unrun check is
   a pass, and this bounded profile is not full Excel object-model compatibility.

## Design decisions

- Use a separate optional adapter crate above `gridcraft-engine`; the engine does
  not depend on VBA. Small shell hooks consume adapter/shared UI APIs.
- Reuse M5 imports/HostBinding and M8 Ide, not a second compiler/editor/VM driver.
- Root dependency revisions must be pinned; tiny-vba stays a separate repository.
- Shared craftrules and GridCraft's ignored plan/ files are absent from this
  checkout. Follow tracked AGENTS.md and docs/architecture.md; no assumptions about
  unavailable local planning files.

## Evidence and current checkpoint

- Shared compiler fix `16f2a492f68bd361bc2c4e06fa9df1c3a1d47d5e` is committed
  and pushed to the tiny-vba feature branch with explicit user approval.
- A fresh Cargo Git fetch also fetched unrelated reference/corpus submodules.
  Set their `.gitmodules` update strategy to `none`; corpus CI/docs explicitly
  opt in with `git submodule update --init --checkout --depth 1`. This keeps
  host dependency builds independent of corpus downloads, as required by scope.
  Cargo behavior is documented at https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#git-submodules.
- GridCraft's complete Windows `cargo +1.99.1 xtask ci` passes all ten steps,
  2,534 tests at that checkpoint, with the existing pinned craft-fonts input
  `b076417b5880bd4154bd22f00695b1d28c6bea83` in ignored `target/craft-fonts`.
  Without it, two existing multilingual UI tests fail on missing fallback glyphs.
- All seven optional editor/control/source save-discard/accessibility tests pass;
  strict all-target optional-feature Clippy passes. Original paused-state light,
  dark and narrow/scaled PNGs in GridCraft `target/vba-ui` were rendered and inspected.
- GridCraft's actual `gridcraft-web/vba` browser application and toolkit-free
  adapter compile for wasm32-unknown-unknown. An additional WASI adapter run is
  not a pass: GridCraft's formula clock uses browser JS imports (chrono/js-sys),
  unavailable in WASI. JavaScript-hosted WASM acceptance is being added instead.

- Read both AGENTS.md files and tiny-vba development, host, architecture and editor
  skills, M5/M8 contracts, and actual GridCraft command/undo/trust APIs.
- Matching egui fork revisions confirmed in both workspace roots.
- Windows has Rust 1.99.1 installed; GridCraft's unqualified cargo uses 1.92, so
  use `cargo +1.99.1` there.
- Baseline `cargo +1.99.1 test -p gridcraft-engine --lib --offline
  --no-default-features` first failed because the sandbox denied linker temporary
  files. Retried with approved compiler access; result pending.
- Stages 1–3 implemented in the fork; stage 4 shell hooks implemented. Acceptance
  and review continue. Ten authored workbook tests pass against the local compiler
  fix: FillCells, recalc/undo/redo, typed values/literal formula-like text/CVErr,
  arbitrary-bound rectangular arrays, atomic conversion failure, named arguments,
  VBA handlers, exact errors, trust/revocation, break/edit/resume without replay,
  stop/error partial effects, document loss/topology and XLSM byte preservation.
- Those tests exposed root-object member calls incorrectly taking the fixed-alias
  binder shortcut. ADR 0017 and a generic exact-dispatch VM regression fix this
  here. The fork currently uses ignored `target/vba-local.toml` overrides to test
  that change; replace its Git pin with the tested tiny-vba commit before delivery.
- `cargo xtask ci` passes on Windows with the bundled Python runtime on PATH:
  fmt, strict Clippy, workspace tests including the new regression, layer graph,
  bundled evidence/provenance checks. External corpus checkouts are absent:
  seven pins checked, 0/646 upstream candidate hashes checked (not a corpus pass).
- `cargo xtask wasm` and `cargo xtask wasm-run` pass on Windows after installing
  wasm32-unknown-unknown and wasm32-wasip1: native/WASI smoke, 14 import tests,
  17 callback tests, 25 headless IDE tests. Embedded GridCraft UI tests pending.
- Windows Python discovery fix: xtask uses `python` on Windows, `python3` elsewhere.
  Available Python directory for this machine:
  `C:/Users/mattp/.cache/codex-runtimes/codex-primary-runtime/dependencies/python`.
