# M1.2/M1.3 implementation log

Started 2026-10-28 on `feat/m1-defaults-and-collections`, following M1.1.
User request: complete the remaining M1.x work, preserving repository separation.

| Stage | State |
|---|---|
| M1.2 generic default invocation / value contexts | Complete; 316 native / 130 WASI suite results |
| M1.2 fork-owned default members / indexed Item | Pending |
| M1.3 bounded enumeration / For Each / debugger lifecycle | Complete; core CI / WASM / WASI gates pass |
| M1.3 fork-owned collection objects and unchanged source | Pending |
| Final native/WASM/UI/fork gates and documentation closure | Pending |

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
The authored `defaults_collections` suite has 12 native regressions; it is also
part of `cargo xtask wasm-run`. Final counts/gates will be recorded at closure.
GridCraft adapter work is on the matching feature branch; its final pin follows
tested core publication. Main branches remain untouched.

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
