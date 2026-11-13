# Attributes and lossless code projection

Stored VBA source includes module/member metadata and export headers that do not
belong in the normal editable code view. The shared parser/importer and IDE retain
canonical source and expose a reversible bounded projection. This behavior follows
[ADR 0030](decisions/0030-attribute-metadata-and-code-projection.md); it is supported
by authored regressions, not a fresh Office comparison.

## Metadata ownership

Attributes retain owned module/member targets, literal validation and raw spans.
Recognized headers/metadata never become executable statements or breakpoint sites.
Unfamiliar presentation metadata remains preserved; unsupported semantic attributes
require an explicit capability diagnostic. Malformed statements must not disappear
through a line-prefix filter.

| Name | Meaning / treatment |
|---|---|
| `VB_Name` | Module identity; distinct from a file name. |
| `VB_PredeclaredId` | Default instance. |
| `VB_Exposed` | Cross-project visibility. |
| `VB_Creatable`, `VB_GlobalNameSpace` | MS-VBAL requires False in source projects; library projects differ. |
| `VB_Customizable` | Class-header Boolean; retain without guessing additional host behavior. |

The header grammar, defaults and source/library distinction above come from
[MS-VBAL §4.2](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/4599fae2-3f41-4e70-968e-2398741f446b).
That section also sets a 31-character module-name maximum and distinct names per
project. tiny-vba's importer identifier helper currently permits up to 128 bytes;
audit metadata-name limits separately from general identifiers rather than silently
changing accepted source in the display fix.

| Further metadata | Purpose / evidence |
|---|---|
| `VB_Description`, member `.VB_Description`, `.VB_VarDescription` | Object Browser descriptions. |
| Member `.VB_UserMemId = 0`, `.VB_VarUserMemId = 0` | Default member. |
| Member `.VB_UserMemId = -4` | Enumerator; Microsoft's [VBE glossary](https://learn.microsoft.com/en-us/office/vba/language/glossary/vbe-glossary#collection) documents the collection convention. |
| `.VB_ProcData.VB_Invoke_Func` | Excel shortcut/category metadata. |
| `VB_Base`, `VB_TemplateDerived` | Host metadata present in Microsoft's workbook example; preserve verbatim. |
| `VB_HelpID` | Help context, present alongside a description in the [Sheet1 example](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/ca44af28-96d5-4c6f-9ec6-61e2cfe81d64). |
| `VB_Ext_KEY`, `VB_Control`, `VB_MemberFlags`, `VB_VarMemberFlags`, other `VB_Invoke_*` / `VB_VarProcData` / help variants | Reserved implementation names; retain recognized metadata syntax without inventing execution semantics. |

Description/default-member meanings in this table are reported by the
[VBE_Extras author's reference](https://www.thevbahelp.com/post/vba-attributes).
Do not infer C++/VBA inheritance support from `VB_Base`, or assume that accepting
metadata implements COM activation, external creatability or cross-project calls.
Microsoft also documents [conflicting attributes](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/conflicting-attributes-were-found-in-itemthe-defaults-will-be-used),
including the requirement that a form's `VB_PredeclaredId` be True. Attribute
validity depends on component kind.


## Normal code view

Compose hidden ranges with newline/UTF-8 and procedure-view mappings. Visible edits
must preserve raw spelling, metadata order, unknown records and CR/LF/CRLF bytes.
Insertion affinity keeps a member's attributes attached during edits. Undo/redo,
rename/delete, scoped search/replace, diagnostics, completion, selections,
breakpoints and stack navigation use the same source/revision mapping.
A separate read-only inspector exposes raw source/metadata.

An attribute-only document module stays visible in Project Explorer and opens as
a blank code pane. Document code names remain separate from display/export names;
host-controlled identities require host validation. Descriptions/Instancing are
editable only where supported. Excel Macro Options/shortcut policy belongs to
GridCraft rather than the shared metadata layer.

## Preservation and evidence

Plain-source save/export retains the canonical metadata. Original package/project
bytes are separately preserved; native forms and explicit edited package copies
follow [forms](forms.md). Fixing metadata presentation does not establish support
for unrelated operations in a macro.

Shared parser/IDE/UI and real-fork regressions cover member placement, invalid
metadata, hidden-header projection, attribute-only insertion, Unicode/line endings,
revision checks and inert attachment. See [testing](testing.md). Fresh Office
rename/dialog/platform observations remain NOT_RUN.

Public format references: [MS-VBAL module structure](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/4599fae2-3f41-4e70-968e-2398741f446b)
and [MS-OVBA module example](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/1075ecf1-679a-4a19-b68d-1fb97e859e9b).
