# ADR 0039 — M7A native control inventory and first worksheet metadata slice

Date: 2026-11-04. Status: adopted implementation contract under ADR 0038.
Owner: GridCraft XLSX for worksheet formats; shared native UserForm codecs remain tiny-vba work.

## Audited boundaries

GridCraft's XLSX pass-through already retains worksheet `controls`, their control-
property relationships/parts and legacy VML. These bytes are not control model
objects, and the UI correctly warns that they are not shown. The ordinary DrawingML
shape reader captures nonempty `macro` but replaces source shape IDs with model IDs;
it cannot safely join a native worksheet control to a DrawingML object by that ID.
GridCraft's native cell checkboxes are a separate feature, not legacy Form Controls.

Tiny-vba's project importer explicitly refuses designer licence data and `BaseClass`
UserForm declarations. The shared IDE has a reusable three-control design/runtime,
but its current `.tvform` exchange is not a native resource decoder. Both forks'
source routes use the existing shared IDE; Word's package constructor attaches
only standard/class/ThisDocument components. Neither fork encodes edited native
UserForm resources. M7's preservation tests do not establish that ability.

## First implementation contract

Expose a bounded, headless **read-only metadata decoder** from GridCraft XLSX over
an existing sheet's preserved package graph. Do not add a second writable control
store, a user-facing format, UI command, host permission or VM dependency.
The graph remains authoritative for original-byte preservation. Returned metadata
is a snapshot, not a control instance or executable macro authorization.

The first slice joins worksheet control name/shape ID, internal `ctrlProp` relation,
`formControlPr` object type and matching legacy VML shape by native shape ID. It
extracts Button caption text, raw VML style/anchor and textual macro assignment
from VML `FmlaMacro` and worksheet `controlPr/@macro`. Empty assignments do not
mask populated sources; conflicting nonempty sources refuse. No assignment is
resolved, compiled or run. Keep unsupported native control types explicit.
Missing/duplicate/ambiguous identities, foreign namespaces, external control
relationships, wrong content types, malformed XML and resource caps refuse without
publishing a partial snapshot. Keep original package state unchanged on failure.

Bounds apply before XML trees/strings grow: cap aggregate input, tree node count,
depth, control/shape count and per-field text. Resolve XML namespaces rather than
assuming spelling of prefixes. Preserve inherited worksheet namespace declarations
when capturing fragments. Caption text follows source text order; this slice does
not interpret native rich-text formatting, geometry or enabled/visible defaults.

Do not infer DrawingML/VML precedence from this subset. Full FC1.1 reconciliation,
including source DrawingML identity and conflicting assignments, is a prerequisite
to the later rendering/activation slice. Geometry conversion, disabled/visible and
print behavior, macro target/signature/owner validation, explicit trust and exactly-
once debugger-aware activation are separate reviewed gates. Native metadata decode
must not be labelled display/input/run support.

## Native UserForms and retirement

Next shared work needs bounded `.frm`/`.frx` and MS-OFORMS resource contracts, inert
attachment and native save/edited package encoding under ADR 0037. Reuse model/
callback/debugger internals after checking actual Office semantics. Remove `.tvform`
only with native replacement workflows and equivalent regression coverage; this
slice neither expands the custom format nor pretends it has been removed.

## Evidence

Use independently authored package fixtures, never the user workbook or Office-
produced binaries. Assert native IDs, rich caption text, macro strings, unchanged
save/reopen, missing/conflicting metadata and parser resource limits. Existing
package and macro regressions remain required. No fresh Office observation is
claimed; Office comparisons remain M8.5.

Format references: [MS-XLSX control-properties part](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/3d054a6d-4f94-4082-837a-f939fd8d4a45),
[formControlPr](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/24baf70c-49ec-4d3c-81fa-133e8681490d),
[CT_FormControlPr](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/8577637a-66c3-485b-ab6e-f8c85689a7c4),
[worksheet controlPr](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.spreadsheet.controlproperties),
and [VML ClientData elements](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.linq.xvml.clientdata).
These establish format structure, not observed Office execution or precedence.
