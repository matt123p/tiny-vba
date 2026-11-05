# ADR 0060 — Native RefEdit and host range selection

Status: adopted for the selected native RefEdit contract; integration validation in progress. Full M7A.4 remains open.

The user requires Office-native forms and Office behavior. RefEdit belongs to
Excel's host integration; it is not an arbitrary external control or a new widget.

`vba-project` reads bounded MS-OFORMS SiteClassInfo records and resolves the native
RefEdit CLSID `{00024512-0000-0000-C000-000000000046}`. Unknown used classes and
invalid indices refuse explicitly. Type information stays inert: it grants no
COM, registry, DLL, filesystem or host authority. RefEdit uses TextBox MorphData
with ShowDropButtonWhen=2 (Always) and DropButtonStyle=3 (Reduce). Existing source
and original resource bytes remain independent of runtime state.

`vba-ide` provides ordinary RefEdit classes, separate RefEdit nominal metadata,
canonical UTF-16 text/selection and drop state, normal focus/validation and
Change/DropButtonClick callbacks. An inert query derives the active range picker
from live modal ownership and canonical fields, including parked callbacks.
No second mutable control store is introduced. Opening selection is an executable
form command and uses the existing authorization/debugger/continuation path.

`vba-ui-egui` receives explicit range-selection availability from the embedding.
The reduce button collapses a modal form to a small text/restore window and
returns worksheet input to the embedding. Standalone/Word hosts do not advertise
an Excel range-selection service. GridCraft owns native sheet selection, reference
formatting and command/trust/document policy; it feeds selection through the same
ordinary form text-input API. UI tracking of the last observed sheet selection is
a derivative input cursor, never canonical RefEdit text or a persisted resource.
Reset, close, source invalidation and trust loss invalidate picker ownership.

This increment does not implement modeless/nested-Frame RefEdit platform parity,
cross-workbook selection, bindings, every inherited property/event, native edited
saving or `.tvform` removal. These remain in the existing M7A.4/.5/.6 ledger.
Authored/specification tests do not establish fresh Office comparison; M8.5 stays
NOT_RUN until measured.

Primary contracts:

- [MS-OSHARED 2.3.5 RefEdit](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oshared/45556c0e-0f42-4c79-864e-503cd04e38ba)
- [MS-OFORMS SiteClassInfo](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/49750bfd-3787-4461-a997-461a41233cbf)
- [ClassInfoPropMask](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/23e7d1b6-a8c3-40c8-984b-cba659aea614)
- [ClsidCacheIndex](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/60fcfe48-aece-4114-b719-c7563195837f)
- [Microsoft MS-OFORMS specification, example 3.4](https://officeprotocoldoc.z19.web.core.windows.net/files/MS-OFORMS/%5BMS-OFORMS%5D.pdf)
