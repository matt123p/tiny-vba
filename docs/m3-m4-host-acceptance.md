# M3 audit and M4 host acceptance

Updated 2026-10-29. Current plan numbering; historical M3/M4 language reports are
separate. Excel implementations belong to the sibling GridCraft repository.

M3.1–M3.3 are implemented: lightweight references and bounded materialization,
Cells/Rows/Columns/Item/CountLarge, relative/named/qualified/multi-area ranges,
UsedRange/End/Find and native structural commands with undo/protection/recalc.
The old tiny-vba progress status was stale. The owning detailed contract is
GridCraft `docs/vba-m3-ranges-and-worksheets.md`; tests are its
`crates/vba/tests/ranges.rs` and `crates/engine/tests/structure.rs`.

M4.1–M4.3 are complete for the declared profile. GridCraft's
`docs/vba-m4-identity-values-and-members.md` defines stable sheet tokens and
workbook-qualified handles, code-owner versus dynamic active roots, scoped consent
and undo, exact Value/Value2 Date/Currency paths, bounded arrays/error fill,
General/Text/formula input conversion, Address optional arguments and selected
live formatting/calculation. Its ten new independently authored source workflows
live in `crates/vba/tests/identity_values.rs`; native style/replay/atomicity tests
live in `crates/engine/tests/styles.rs`.

The shared prerequisite is [ADR 0025](decisions/0025-getter-alias-member-dispatch.md),
commit `22cc845e5ee74a1bcd15eec32cdb8a8f18d316f8` on
`feat/m4-getter-alias-dispatch`. Getter aliases resolve before chained members;
method aliases keep shortcut behavior. No Excel definitions, dependencies, IR
version or protocol changes enter the interpreter.

## Validation

- tiny-vba `cargo xtask ci`: PASS, 375 tests.
- tiny-vba `cargo xtask wasm`: PASS.
- tiny-vba `cargo xtask wasm-run`: PASS, 181 actual WASI tests.
- GridCraft native adapter suite: PASS, 58 tests, including ten M4 workflows.
- GridCraft actual JavaScript/WASM adapter suite: PASS, 58 tests (Node 22.23.4,
  wasm-bindgen-test-runner 0.2.129).
- GridCraft full `cargo xtask ci`: PASS, all ten gates, 2,593 test/doctest results.
- GridCraft native structural/style tests: PASS; strict optional-window Clippy and
  browser `--features vba` compilation: PASS.
- Corrected optional-window suite: PASS, 1,150 tests. Final full CI rerun: PASS,
  all ten gates against completed code. Optional font checkout `aced4137c022f1f587624d48e81ec77e1914a9cc`
  supplies required glyphs; it stays outside both repositories.

Public Microsoft documentation and independently authored expectations are
separate from Office observations. No fresh Office run, downloaded macro
execution, package writeback or universal Excel compatibility is claimed.
Unselected formatting/settings and M5 document events remain explicit refusals;
ScreenUpdating/EnableEvents are not success-returning placeholders. External
structural edits/undo invalidate saved range coordinates rather than retargeting.

## Publication and immutable dependency acceptance

The prerequisite feature branch `feat/m4-getter-alias-dispatch` is published.
GridCraft pins every shared dependency to
`22cc845e5ee74a1bcd15eec32cdb8a8f18d316f8`; all 13 lockfile packages resolve to
that Git revision. The temporary path override is removed, and unrelated locked
packages are unchanged. Final native/WASM/full-CI/window acceptance uses the
published Git source. M3/M4 are complete for their declared profiles and are
removed from the remaining plan; M5 is next. Main branches are unchanged.

Final owning GridCraft commit: `f320728dddefa8dd584b1f83c00f77d684ac708f` on
`feat/m3-ranges-and-worksheets`. Pinned acceptance passes 58 native and 58
JavaScript/WASM adapter tests, 1,150 optional-window tests and all ten CI gates
(2,593 test/doctest results). The commit includes code, regressions, dependency
pins and the owning architecture/compatibility/acceptance documentation.
