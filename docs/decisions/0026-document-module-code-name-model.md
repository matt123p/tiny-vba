# ADR 0026: preserve document-module code names in the IDE

Status: adopted for implementation, 2026-10-29.

M5.1 needs a stable VBA code name distinct from a module's source/display name.
The toolkit-independent IDE owns that metadata and inert source validation;
GridCraft owns binding the name to its live workbook or worksheet object and
deciding whether execution is authorized. The compiler continues to use the
source `VB_Name` attribute as the project class identity.

`Module` therefore carries an optional `code_name`. `Ide::add_document_module`
accepts the source name and explicit code name, requires the latter to match the
parsed `VB_Name` attribute, and only adds source. Import never executes event
procedures. Existing generic document-module creation remains available for
editor workflows that have not yet assigned host identity; it derives the field
from `VB_Name` when present.

This adds no crate dependency, IR operand, host protocol operation, or automatic
event behavior. ADR 0027 defines the generic owner binding and callback path; the
GridCraft adapter resolves and supplies live workbook/worksheet objects. M5.1
handler signature/workflow acceptance and real workbook/worksheet evidence passed
for the declared profile; see the [M5 acceptance map](../m5-acceptance.md).
