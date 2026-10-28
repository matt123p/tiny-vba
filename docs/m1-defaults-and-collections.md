# M1.2/M1.3 implementation log

Started 2026-10-28 on `feat/m1-defaults-and-collections`, following M1.1.
User request: complete the remaining M1.x work, preserving repository separation.

| Stage | State |
|---|---|
| M1.2 generic default invocation / value contexts | Complete; 316 native / 130 WASI suite results |
| M1.2 fork-owned default members / indexed Item | Complete; native / JS-WASM acceptance |
| M1.3 bounded enumeration / For Each / debugger lifecycle | Complete; core CI / WASM / WASI gates pass |
| M1.3 fork-owned collection objects and unchanged source | Complete; 40 native / 40 JS-WASM adapter tests |
| Final native/WASM/UI/fork gates and documentation closure | Complete; full CI and affected desktop/browser gates |

Core owns explicit generic invocation kinds and verified execution. No Excel
member names or object definitions enter tiny-vba. GridCraft owns default-member
selection, collection identity, Count/Item, ordering and actual engine effects.
Native host arguments retain names, Missing, ByRef obligations and Let/Set intent.
No library loading, COM activation, implicit macro import/execution or JSON dispatch.

Default value resolution must be separate from object-reference contexts. A value
conversion may call a default Get only with ordinary host permissions/budgets;
safe watches stay side-effect-free. Paused/resumed host operations must not replay.

Enumeration must be bounded and inert until an explicit Run. Host enumeration
returns owned bounded values with existing wait/cancel rules, not a retained host
or document borrow. Native arrays retain live element reads and resize protection.
Loop scope cleanup, type checks, breaks/steps, errors and cancellation need tests.

Public behavior references: Microsoft VBA For Each...Next and array/control-variable
rules, Excel Range default member forwarding to Value/Item, Worksheets Item/Count.
All code and fixtures are original. Documentation is not a fresh Office oracle.

Implementation checkpoint: generic IR 6/default contexts, dynamic array/object
applications, canonical ByRef versus property temporaries, For Each lowering,
array locks, source-region/error cleanup and paused element editing are in place.
The authored `defaults_collections` suite has 12 regressions executed natively
and as part of `cargo xtask wasm-run`. Core validation: 316 workspace tests,
strict full CI, headless WASM compilation and 130 actual WASI tests pass.

GridCraft's matching feature branch passes 40 native and 40 JavaScript/WASM
adapter tests, including unchanged source, real commands/undo, trust revocation,
collection errors/order/resource limits and spreadsheet UDF recalculation.
Full fork CI passes all 10 gates with 2,568 test/doctest results. Strict optional
window Clippy, 1,150 desktop window tests and browser VBA compilation pass.
Existing browser dead-code warnings remain. Node 24.14.0 crashed in runner
shutdown; the complete suite passes with successful exit on existing Node
24.22.1. No failed runner attempt is counted as success.

Publication: [tested tiny-vba code](https://github.com/matt123p/tiny-vba/commit/d80e0393b408e0421a8df7e16667dbbbc970783b),
[GridCraft adapter and immutable pin](https://github.com/matt123p/gridcraft/commit/55d3b1bb825499086581afdbd2e07eec87bc3c29).
Both branches are `feat/m1-defaults-and-collections`; main branches are untouched.
GridCraft has no embedded tiny-vba copy, submodule or sibling path override.
The core exposes no Excel object definitions; the fork's existing shared object
store owns the six Excel types and default/collection selection.

Logs remain ignored under tiny-vba `target/m1-ci.log`, `m1-wasm.log`,
`m1-wasm-run.log` and GridCraft `target/m1-collections-native.log`,
`m1-collections-wasm-node24.log`, `m1-grid-ci.log`, `m1-window-clippy.log`,
`m1-window-tests.log`, `m1-web-vba.log`. M1.x is complete for this declared
profile. Next is M2.1; larger range/identity/event/package acceptance remains
in M2–M8, without renumbering established follow-up task IDs.

Public references used to define behavior (documentation, not fresh Office results):

- [For Each...Next](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/for-eachnext-statement)
- [Array controls require Variant](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/for-each-control-variable-on-arrays-must-be-variant)
- [Range default Value/Item forwarding](https://learn.microsoft.com/office/vba/api/excel.range(object))
- [Range.Item](https://learn.microsoft.com/office/vba/api/excel.range.item)
- [Range.Cells](https://learn.microsoft.com/office/vba/api/excel.range.cells)
- [Worksheets](https://learn.microsoft.com/office/vba/api/excel.worksheets)

No Office-generated files, vendor assets, third-party implementation code or
unlicensed samples were used. Existing M3–M7 range/identity/event/package work
remains outside this milestone; support for collection iteration does not assert
complete unchanged-workbook compatibility.
