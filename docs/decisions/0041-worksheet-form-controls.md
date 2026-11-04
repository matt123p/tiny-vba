# ADR 0041 — Native worksheet Form Controls

Accepted 2026-11-04 for M7A.3 / FC1–FC2. Supersedes the metadata-only boundary
of ADR 0039; retains ADR 0038's Office compatibility policy.

GridCraft owns worksheet controls. All nine standard families live in its existing
Shape store with separate native Office IDs/names and process lifetime identities.
There is no second writable control registry, VM widget model, exchange format,
shared dependency change or new language feature. The model supplies pure binding
and grouping queries; engine commands own state, linked-cell mutations, protection,
undo and recalculation. The optional VBA adapter supplies Shape.ControlFormat and
standard Excel members; the UI uses native Rust/egui widgets and the existing
trusted execution bridge.

Import joins worksheet controls/ctrlProp/VML/DrawingML by native ID. Conflicting
identity, caption, geometry or assignment refuses the join; unknown/malformed
controls remain inert with original source parts retained. Legacy geometry is
cell-fraction based, the workbook model uses 96-dpi pixels, and Excel Shape
geometry uses Single points. Cell-relative endpoints survive hidden rows and
column/row resizing. Original package bytes remain separate from mutable state.

A click commits the control value and linked cell through one normal engine
transaction before its assigned no-argument public Sub starts. Trust, eligibility,
workbook qualification and owner identity are checked first. Import and property
writes never invoke assignments. Busy/paused/waiting/stale VMs refuse another
click; Reset cancels without replay. Application.Caller is the captured control
name and the owner sheet supplies implicit Range context throughout continuation.
Linked-cell writes recalculate but do not synthesize Worksheet_Change callbacks;
ordinary VBA range writes retain the existing event path.

Native worksheet edits patch the known OOXML/VML fields and keep unrelated XML,
relationships and original VBA project data. Untouched control parts stay byte
preserved. Linked references and persisted cells remain authoritative when a
control's cached list/value differs; reading/displaying does not dirty those
source caches. Saving is prepared on a workbook copy and can fail without changing
live state. Native UserForm resource encoding remains M7A.5, and ActiveX remains
outside this worksheet Form Control profile.

The exact supported properties, limits, refusals and authored evidence are in
[the M7A.3 contract](../m7a-worksheet-controls.md). No fresh Office observation or
blanket Excel compatibility claim follows; M8.5 comparisons remain NOT_RUN.
