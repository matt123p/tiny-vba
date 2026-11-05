# ADR 0057 — native TabStrip and tab identity

Date: 2026-11-05. Status: adopted for the selected TabStrip packet.
M7A.4 remains IN PROGRESS with its full remaining scope.

vba-project decodes cached class 18 independently of Frame. A TabStrip is a
leaf control owning tabs, not a parent for nearby controls. Its native record
has ordered character-counted ArrayStrings, mandatory TextProps and optional
per-tab flags. Preserve names, UTF-16 captions, tags, tips, accelerators,
selection, orientation/style, row/tooltip flags and fixed tab dimensions.
Check uncompressed character counts before combining them with the compression
flag; oversized counts cannot alias that flag and masquerade as empty strings.
Validate parallel array lengths, unique persisted identifiers, selection and
flag bounds; refuse pictures/custom pointers, unknown masks and missing tab
names rather than inventing client-generated names. Source and resources remain
inert and independent. Arrays and the total tabs per attached form are capped at
128, with existing string/resource/VM execution budgets unchanged.

The IDE generates ordinary TabStrip, Tab and Tabs classes. Tabs use a separate
nominal MSForms.Tabs interface and contain ordinary MSForms.Tab objects, not
MSForms.Control objects. The shared bounded enumerator snapshots their canonical
entries through the existing transient project view. No VM names, new IR, COM,
mutable host registry or object back-reference cycle is introduced. Root
Controls contains the TabStrip and keeps its tabs separate.

Value selects an index or -1; SelectedItem returns the canonical tab or Nothing
and is the TabStrip default property. A Tab's default Caption uses ordinary
property frames. Indexed/named Item and initial named-tab aliases resolve the
same objects. Shared aliases across component instances refuse missing members
with 438. Metadata and captions remain ordinary fields; read-only UI queries
inspect backing fields without executing getters or creating objects. Invalid
selection, orientation/style and fixed dimensions fail before mutation under the
selected authored 380 contract. Physical selection refuses disabled/hidden tabs,
uses existing focus/validation and commits before ordinary Change/Click callbacks.
Debugger Stop/resume does not replay selection or preceding handlers. Commands
are classified as executable before application policy dispatch.

The shared renderer paints tab rows/columns using injected theme tokens and
host fonts, clips to the control/ancestor surface and exposes TabList/Tab
accessibility ancestry. Pointer and arrow input submit commands rather than
mutating state. Arrows skip unavailable tabs. Tips are inert and use the injected
theme and toolkit hover delay; missing glyphs retain the caller's font policy
and never replace canonical UTF-16 content.

This packet does not implement MultiPage/Page storage, Tabs Add/Remove/Clear,
writable Index/Name and structural-mutation behavior, all native presentation
or the complete key/mouse event matrix. Those remain M7A.4 requirements alongside
Image/RefEdit, bindings, platform controls and the existing lifecycle/property
ledger. Native edited persistence and .tvform removal remain separate M7A.5/.6
requirements; the legacy codec is not expanded. Fresh Office observations remain
NOT_RUN under M8.5. Tests are authored assertions against public format/API
contracts, not measured Office event/error traces.

Sources: [MS-OFORMS](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/),
[cached classes](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/5f649a6c-90df-4226-b8c7-2a7d98cd9647),
[TabStrip](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/tabstrip-control),
[Tab object](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/tab-object),
[SelectedItem](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/selecteditem-property),
[Index](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/index-property-microsoft-forms).
