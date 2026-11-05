# ADR 0058 — native MultiPage and Page containers

Date: 2026-11-05. Status: adopted for the selected native MultiPage profile.
M7A.4 remains IN PROGRESS; this decision does not narrow its requirements.

vba-project owns bounded cached MultiPage (57) and Page/Form (7) decoding.
MultiPage is a native parent storage containing Form `f`, internal TabStrip `o`,
Page child storages and `x`. Require one internal TabStrip, reconcile its ordered
names with PageIDs and Page storages, and require PageCount agreement. Consume
one ignored PageProperties followed by one per page and MultiPageProperties.
Reject missing, duplicate, unknown or unclaimed page records/resources and
conflicting nonempty metadata. The selected profile requires cached immediate
sites, persisted tab names, default page transitions and the existing supported
parent properties. Unsupported variants refuse explicitly. Existing depth,
control, text, CFB and resource bounds apply. Original resource/source bytes
remain separate and inert.

The IDE generates ordinary MultiPage, Page and Pages classes. Public form
Controls contains the MultiPage; Pages contains the actual Page objects; each
Page.Controls contains its immediate children. The internal TabStrip does not
escape into any public collection. Page/Pages have distinct nominal interfaces.
Parent references use the existing event bridge, without owning back-references.
Pages enumeration uses the existing bounded canonical-object snapshot provider.
Value is the MultiPage default property; SelectedItem and named page aliases
resolve those same Page objects. Runtime selection, captions, metadata and page
children remain canonical ordinary class state, without a second mutable store,
new IR, COM, registry or VM-specific Office names.

Input to unselected Page descendants refuses before execution. Canonical page
selection also gates focus and tab traversal. Physical page changes pass through
existing dirty-field validation and container Exit/Enter handling before
selection; cancellation or Stop preserves the preceding selection until the
ordinary continuation commits. Internal navigation scans take bounded local
snapshots of canonical object references, avoiding repeated public Item getter
frames and preserving the existing location budget. No budget is raised.

Correct ADR 0057's Click contract: native TabStrip and MultiPage Click carry a
Long page/tab index. Generated event bridges preserve that argument, and event
insertion supplies a typed parameter. Re-selecting the current page/tab raises
neither Change nor Click. Change commits before an authored Stop; Click resumes
once with the selected input index. Zero-argument Click handlers now refuse
instead of accepting a non-Office signature. This correction follows Microsoft's
published Forms event contract, not fresh measured Office traces.

The shared renderer reuses themed tab headers, paints only the selected Page and
its descendants, retains live relative geometry/ancestor clipping, and exposes
Page container accessibility ancestry. Pointer/arrow selection is an executable
command and retains existing application authorization and modal ownership.
Read-only inspection and painting never construct forms or invoke VBA getters.

Remaining M7A.4 includes dynamic Pages/Tabs mutation and writable Name/Index,
empty/default persistence variants, additional page transitions/property/resource
and presentation cases, programmatic selection/focus and lifecycle edge cases,
the rest of the input/event matrix, Image/RefEdit, list/binding/platform controls
and the existing ledger. Native edited saving and .tvform removal remain
M7A.5/.6. Authored fixtures/tests are independent Rust under MIT OR Apache-2.0;
fresh Office build/OS/locale comparisons remain NOT_RUN under M8.5.

Sources: [MS-OFORMS MultiPage storage](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/aa2a2c6b-05e2-461f-a366-c10f15ef7731),
[MultiPageProperties](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/7e92e729-7296-48ee-a7d8-47d31f10d304),
[PageProperties](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/6588bbac-e5df-4c7a-9372-5246f2ea8e32),
[MultiPage default property](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/multipage-control),
[Page access and selection](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/page-object-pages-collection-multipage-control-selecteditem-property-example),
[indexed Click and repeated selection](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/click-event).
