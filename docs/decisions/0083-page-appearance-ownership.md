# ADR 0083: Page appearance belongs to MultiPage

Date: 2026-11-07. Status: shared acceptance passed; affected host acceptance in progress. M7A.4 remains open.

The published IPage interface has no Font, BackColor or ForeColor. ADRs 0080–0082
incorrectly exposed these through the generated Page class. This corrects that
surface rather than preserving a custom feature. Page keeps Controls, Parent,
ActiveControl, Index, Name and its existing independent container identity.

Remove the three public properties and their separate backing fields/objects.
A runtime-added child obtains independent Font defaults from Page.Parent.Font,
the owning MultiPage. Later additions see the owner's current attributes; existing
children retain their own copies. Native initializers resolve the corresponding
MultiPage from the inert hierarchy, before executable Parent links are connected.
They do not execute Parent getters during import or inspection.

The view reads Page's existing canonical parent key and projects the MultiPage's
current colours onto the page client surface. It never synthesizes Page colour
properties or calls a getter/owner event. Original native common container records
and resources remain retained separately. No second mutable appearance store,
VM change, new interface or increased execution/permission budget is added.

Authored model cases check absence through ordinary member errors, inert queries,
new child Font defaults, parent identities, Stop/Continue and Reset. The family
Font regression now admits twelve control families and refuses Page along with
Image, ScrollBar and SpinButton. An actual GPU case checks the page centre against
a MultiPage colour changed after page creation. Full remaining property, lifecycle,
binding and platform acceptance remains open; fresh Office comparisons are NOT_RUN.

Reference: [Microsoft Forms Page/IPage](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.page?view=office-pia).
