# ADR 0053 — native list array loading and Column access

Date: 2026-11-04. Status: adopted for the selected unbound native packet.
M7A.4 remains IN PROGRESS.

## Ownership and behavior

The IDE composes ordinary property procedures shared by native ListBox and
ComboBox. `List = array` loads two-dimensional rows/columns; `Column = array`
loads the transposed axes. Source lower bounds may differ from zero. Existing
indexed List access remains zero-based. `Column(col, row)` reads/writes a cell;
omitting row uses ListIndex, with Null returned when no row is selected.
Column writes reuse List cell conversion and selected-row refresh.

Loading builds a local, independent Variant array before changing canonical
VbaItems, VbaCount or VbaListIndex. All source cells must convert through the
existing VBA CStr intrinsic, matching this packet's unbound string cells. This
leaves the source array element type and bounds unchanged; loaded rows do not
alias that source. Arrays are not flattened through JSON, a host cursor or a
second mutable store.
An accepted load replaces rows, clears selection and refreshes Value/Text using
the existing control-specific Change/Click continuations. All fields commit
before authored callbacks. Stop/resume, reentrance, instruction/allocation budgets
and Reset retain ordinary VM behavior. Inspection/import remain inert; native
source/resources and the legacy codec are unchanged. No VM opcode, intrinsic,
new dependency, host API or toolkit state is introduced.

## Limits and evidence

The existing 1,024-row, ten-column and 16,384-UTF-16-unit/cell limits apply.
Oversized input raises 7 before mutation. Scalar input follows LBound error 13;
missing dimensions fail with 9 through bounds queries, and extra dimensions
fail with 9 when indexed within the selected size limits. Size-limit refusal
precedes cell indexing. Null cells follow CStr error 94. Indexed bounds failures
retain 381. No error-handler rank probes are used, so successful validation does not manufacture or swallow Err failures.
A failed load leaves existing rows, selection, Value and Text intact.

These are authored expectations based on public documentation and the existing
bounded string-cell profile, not fresh Office observations. Exact invalid-input
error parity, selection/event details and coercion still need M8.5 comparison.
Whole-array reads (including empty-list shape), one-dimensional/empty-array
loading, column-vector assignment, wider data, external bindings, multiple
selection and further list resources/properties remain unsupported. No-argument
array reads explicitly raise 381 in this selected packet; this is a capability
limit, not a claim about Office's error behavior. Full M7A.4 scope is unchanged.

Sources: [List](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/list-property-microsoft-forms),
[Column](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/column-property),
[Microsoft array-loading example](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/listbox-control-column-columncount-list-properties-example).
