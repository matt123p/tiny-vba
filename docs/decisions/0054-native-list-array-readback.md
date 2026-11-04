# ADR 0054 — native list array readback

Date: 2026-11-04. Status: adopted for the existing unbound ten-column packet.
Extends [ADR 0053](0053-native-list-array-loading.md); full M7A remains IN PROGRESS.

The IDE generates ordinary property/function procedures for both native ListBox
and ComboBox. No-argument List returns a detached two-dimensional Variant array
with zero-based row/column bounds; Column returns its transpose. This packet
exposes the existing ten canonical unbound columns, independent of ColumnCount
and selection. An empty control returns Null; a singleton retains two dimensions.
Each nonempty read allocates a bounded local array and copies canonical cells.
Changing a snapshot, its source array or the control cannot change another copy.
Snapshot lifetime is independent of Clear, Stop/resume and native resources.

Readback never refreshes selection, raises events, probes error handlers or
loads another instance. Existing Err state remains intact. Canonical row counts
are checked before allocation; normal VM array/payload/instruction budgets apply.
There is no new opcode, intrinsic, dependency, mutable host store or UI model.
List and Column now use the documented pvargIndex/pvargColumn argument names,
including reordered named reads/writes through the existing language binder.
Partially omitted index forms still raise 381 rather than being mistaken for
whole-array reads. Original authored source/resources remain unchanged.

The shape/empty/string-cell assertions are authored expectations for this
bounded packet, inferred from the existing unbound storage and published Forms
array APIs. Fresh Office build/OS/locale traces remain NOT_RUN under M8.5;
source-width/unused-cell-type, empty-array loading, one-dimensional loading,
column-vector assignment, wider data, multiple selection and binding parity
remain explicit work. This does not close M7A.4 or alter M7A.5/M7A.6 requirements.

Sources: [List](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/list-property-microsoft-forms),
[Column](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/column-property),
[List parameters](https://learn.microsoft.com/en-us/office/vba/api/outlook.listbox.list),
[Column parameters](https://learn.microsoft.com/en-us/office/vba/api/outlook.listbox.column).
