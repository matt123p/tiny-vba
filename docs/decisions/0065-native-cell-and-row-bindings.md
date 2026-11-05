# ADR 0065 — Native cell and row bindings

Status: shared implementation and validation passed; pinned fork integration in progress. M7A.4 remains open.

The Forms provider decodes ControlSource and RowSource from bounded native
OleSiteConcreteControl fields without resolving or executing them during import.
Original source/resources remain separate. Non-value/list families reject
inapplicable nonempty source metadata. Runtime source attachment happens during
normal form loading, before UserForm_Initialize, or through ordinary properties.

Bindings are ordinary VBA objects owned by controls. They retain the source
spelling and the existing host range handle; the control's existing value/rows
remain canonical. A pending attachment is committed only after ordinary control
coercion/list validation succeeds. Generated source uses a reserved alias of the
embedding's explicit Range import, preserving host identity, authorization,
command/undo/recalculation and existing worksheet-range resolution. It never
opens a workbook, invokes COM or introduces another writable data model. Hosts
without that capability expose empty sources and refuse nonempty assignments.

ControlSource accepts one contiguous cell. RowSource accepts one contiguous
range within the existing 1,024-row/ten-column packet; singleton cells become a
one-cell 2D array. The list model continues its established string normalization.
Bound rows reject AddItem/RemoveItem/Clear/indexed/bulk replacement outside the
binding refresh path. Clearing RowSource disconnects and clears rows. Clearing
ControlSource disconnects while retaining the current control value. Failed
attachment leaves the previous source/value/rows intact. Value, row-removal/clear and physical-text write failures are preflighted before
canonical mutation; bound callbacks cannot bypass the row write guard. Broader
property/event timing still requires further validation.

Value getters and list queries synchronize through ordinary VBA frames. Source
inspection and rendering read canonical fields without invoking getters or
loading forms. Unchanged control values do not overwrite a linked cell changed
by another actor; synchronization then reads that cell. Empty linked list cells
mean no selection. Programmatic control writes use the existing range setter,
with canonical value mutation after its successful result. Runtime source edits
are not native designer edits or an M7A.5 persistence implementation.

GridCraft queues refresh callbacks after document revisions or input-owner/form
availability change, behind its existing document-event delivery. Handles are
only bounded scheduling state, not another control/value store. Refresh requires
current trust and ordinary VM callback/input ownership; source changes, Stop,
Reset, close and stale handles retain existing continuation/lifetime rules. The
callback targets are a bounded snapshot of canonical controls. The optional
host-neutral `Host::object_revision` query validates the existing object and may
return a data epoch only when it covers every relevant value change. Bindings
remember that epoch before reading source data and invalidate it on failed apply
or write; absent epochs always read again. This avoids rebuilding unchanged large
lists without making the cache authoritative. GridCraft returns document revisions
only for bounded plain-cell ranges; formulas use the uncached path because
recalculated values need not change the content revision. Generic For Each
now accepts object arrays held in Variant through the existing EnumerateObjects
path, preserving project identity without a host dispatch.

On 2026-11-05 the owner explicitly approved increasing the static host-permission
allowlist cap from 1,024 to **4,096** entries. `HostBinding::MAX_STATIC_PERMISSIONS`
bounds both derived and supplied tables. Every host call still requires its own
site authorization; executed-call, work, object, location and memory limits are
unchanged. Boundary tests accept 4,096 and reject 4,097. No new IR version or
project-only dispatch path is introduced. Shared typed initializer helpers also
consolidate repeated constructor setup without invoking authored code.

The remaining M7A.4 lifecycle, keyboard/mouse/accelerator, platform and full
property/error/resource ledger is not closed by this increment. Full binding
validation/update timing, further shapes/types and background-form synchronization
still need evidence. Native edited saving/removal remain M7A.5/.6. Authored Rust
fixtures/tests are MIT OR Apache-2.0; fresh Office observations remain NOT_RUN in
M8.5.

Primary contracts:

- [ControlSource](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/controlsource-property)
- [RowSource](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/rowsource-property)
- [SitePropMask](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/896d3774-dd6e-46b5-bfa7-6651aba111a8)
