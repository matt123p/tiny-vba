# M3 audit and M4 host acceptance

Updated 2026-10-29. Current plan numbering; historical M3/M4 language reports are
separate. Excel implementations belong to the sibling GridCraft repository.

M3.1–M3.3 are implemented: lightweight references and bounded materialization,
Cells/Rows/Columns/Item/CountLarge, relative/named/qualified/multi-area ranges,
UsedRange/End/Find and native structural commands with undo/protection/recalc.
The old tiny-vba progress status was stale. The owning detailed contract is
GridCraft `docs/vba-m3-ranges-and-worksheets.md`; tests are its
`crates/vba/tests/ranges.rs` and `crates/engine/tests/structure.rs`.

M4.1–M4.3 code is implemented locally. GridCraft's
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

## Remaining publication gate

Automatic approval review rejected the feature-branch push because repository
export to the external Git remote was not explicitly authorized. Approval has
been requested. The safe local validation override is temporary; it is not a
shipping dependency pin. After approval, publish the prerequisite, update all
GridCraft tiny-vba revisions consistently and remove the override. Verify immutable
resolution before moving M4 out of the remaining plan. Main branches are unchanged.
