# VBA attributes, exported source and the Excel code view

Investigated 2026-11-01. This is public-documentation research and local code
inspection, not a new Excel oracle observation. The fixes below are planned under
[M6.9–M6.12](implementation-plan.md); this change does not implement them.

## Findings for the two screenshots

Both issues involve the distinction between stored module text and the code view.
`Attribute` carries module/member metadata. It must survive import and source
export, but it must not become an executable statement or a step/breakpoint site.
Microsoft explicitly separates mechanically generated module headers from authored
module bodies in [MS-VBAL §4.2](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/4599fae2-3f41-4e70-968e-2398741f446b).

The first screenshot contains a member attribute immediately after a `Sub`
declaration:

```vb
Sub Macro2()
Attribute Macro2.VB_ProcData.VB_Invoke_Func = " \n14"
```

The VBE_Extras developer's [first-hand attribute reference](https://www.thevbahelp.com/post/vba-attributes)
documents hidden attributes and this Excel-specific encoding: the space means no
shortcut, the literal backslash-plus-`n` separates fields, and `14` is the function
category. It is not a runtime call. Microsoft independently documents category 14
as User Defined and exposes shortcut/category settings through
[Application.MacroOptions](https://learn.microsoft.com/en-us/office/vba/api/excel.application.macrooptions).
The category field does not turn this `Sub` into a worksheet function.

The second screenshot shows the raw header of a workbook document module.
Microsoft's [MS-OVBA §3.1.3 example](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/1075ecf1-679a-4a19-b68d-1fb97e859e9b)
contains the same eight attributes, including the same workbook `VB_Base` value.
An attribute-only module should therefore show an empty editable code pane.
Keep the `ThisWorkbook` document component in the explorer: it is the header that
should disappear from the normal code view, not the module.

## What the metadata controls

These are VBA/VBE metadata names, not Visual Basic .NET attributes. The
[reserved-identifier catalogue](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/7df907cb-ab6c-40d3-aa81-272742ce00c3)
includes implementation names beyond the ordinary statement keywords. Being listed
there does not establish support or complete semantics for a particular host.

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

## How users interact with these features

The [VBE_Extras author's account](https://www.thevbahelp.com/post/vba-attributes)
describes these interactions: attributes are hidden in the code pane; class
Instancing selects Private/PublicNotCreatable; Object Browser Properties edits
descriptions; export/edit/reimport accesses otherwise hidden metadata. Exported
document `.cls` files reimport as ordinary classes, so that workflow cannot replace
the workbook's existing document component. Its comments correct an earlier claim:
member descriptions can also be edited through Object Browser member Properties.

For the reported macro metadata, Excel exposes the Macro Options dialog;
[MacroOptions](https://learn.microsoft.com/en-us/office/vba/api/excel.application.macrooptions)
provides description, shortcut enablement/key, function category, help and argument
descriptions. The author's reference also reports recorder and Macro Options UI
generation of `VB_Invoke_Func`. The encoding explanation is Windows-focused
implementation experience, not a complete Microsoft serialization specification;
keep the original string and obtain reference-build evidence before normalizing it
or registering platform shortcuts.

Microsoft documents `(Name)` in the Properties window as the object's code name,
editable at design time and read-only through
[Workbook.CodeName](https://learn.microsoft.com/en-us/office/vba/api/excel.workbook.codename).
Consequently `(Name)` should show `ThisWorkbook`, not `ThisWorkbook.cls`.
The `.cls` suffix identifies an exported source file. Code name, export filename,
stream name and host display name must stay separate. Sheet tab renaming must not
rename its VBA code identity.

The normal workbook code pane supports authored declarations and event procedures.
Microsoft documents selecting events in the Procedure dropdown under
[Workbook object events](https://learn.microsoft.com/en-us/previous-versions/office/aa221604%28v%3Doffice.11%29)
and navigating general/event procedures in
[Find a procedure](https://learn.microsoft.com/en-us/office/vba/language/how-to/find-a-procedure).
The [Code window reference](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/code-window)
describes its Object and Procedures/Events boxes. Selecting a document module must
retain those capabilities even when it has no authored code. Event signatures and
host-object binding remain host-supplied; opening the module must stay inert.

## Related source constructs that must remain visible

Hiding every nonexecutable line would be wrong. Preserve these authored constructs
in the normal view, with their existing capability checks:

| Construct | User interaction and relevance |
|---|---|
| `Option Explicit` | Typed in declarations; requires declared variables. VBE's Require Variable Declaration setting can insert it into new modules. [Statement reference](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/option-explicit-statement), [Options dialog](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/options-dialog-box). |
| `Option Base 0/1` | Typed in declarations; controls default array lower bound. [Statement reference](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/option-base-statement). |
| `Option Compare Binary/Text/Database` | Module comparison policy; Database is Access-only. [Statement reference](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/option-compare-statement). |
| `Option Private Module` | Restricts public members to their project in supporting hosts. [Statement reference](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/option-private-statement). |
| `#Const`, `#If`, `#ElseIf`, `#Else`, `#End If` | Authored compile-time selection; preserve in code view. [Conditional compilation](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/understanding-conditional-compilation). |
| `Public`, `Private`, `Friend`, `Static`, `Property Get/Let/Set`, `Event`, `WithEvents`, `Implements` | Ordinary language declarations/modifiers, not hidden attribute records. Use the [VBA language reference](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/visual-basic-language-reference), not VB.NET syntax. |
| Export `VERSION … CLASS` and `BEGIN … END` property/designer headers | Implementation-generated export envelope; separate from executable code. MS-VBAL §4.2 explicitly excludes this envelope from the module body grammar. Do not hide executable `With`/`Select`/other blocks by textual heuristics. |

This is an adjacent compatibility inventory, not a claim that every listed
construct is implemented. Unknown language syntax must retain its diagnostics.

## Local implementation findings

Inspected in this checkout; unrelated ongoing code edits were left untouched:

- `crates/vba-project/src/text.rs::attributes` already extracts raw attribute
  names/values and UTF-8 spans throughout stored source and validates `VB_Name`.
  [The import contract](project-import.md) deliberately preserves hidden lines.
  Changing it to return stripped text would lose required evidence and metadata.
- `crates/vba-syntax/src/parser.rs::item` parses top-level `Attribute` paths and
  literals into `module.attributes`. `procedure` goes straight from its declaration
  into `block`, which uses statement parsing without that metadata branch.
  This explains why module headers parse while the screenshot's procedure
  attribute produces `VBA0001`. Signed member IDs need explicit coverage too:
  the current attribute-value check only accepts String/Number/Boolean AST kinds.
- `crates/vba-semantics/src/lib.rs` already consumes `VB_Name` and
  `VB_PredeclaredId`. Metadata cannot all be discarded as comments.
- `crates/vba-ide/src/code_aids.rs::editor_text` and
  `crates/vba-ui-egui/src/views.rs` provide source/editor conversions; audit and
  extend this mapping for omitted ranges, including the full-module editor in
  `crates/vba-ui-egui/src/lib.rs`. LF/CRLF/CR and UTF-8 preservation already matter.
- The Properties renderer uses `m.name`; the IDE separately carries `code_name`
  under [ADR 0026](decisions/0026-document-module-code-name-model.md).
  Existing IDE tests explicitly distinguish a `Sheet1.cls` source name from its
  code name. Fix the presentation/rename routing without conflating those fields.

## Proposed fixes and acceptance

These proposals extend existing owners without adding dependencies or changing the
VM/IR contract. Before implementation, record the source-projection and metadata
policy in the affected contracts/ADR; this research is not an adopted API change.

1. **M6.9 — metadata parsing.** Give module and member records explicit ownership,
   literal validation and raw spans. Accept the reported procedure position and
   other documented member positions, including property accessors/fields and
   signed IDs; validate target names and placement. Preserve unfamiliar metadata
   without claiming its semantics. Distinguish opaque presentation metadata from
   unsupported semantic attributes that require a capability diagnostic. Never
   emit executable sites or swallow malformed statements by a line-prefix filter.
2. **M6.10 — normal code projection.** Keep canonical raw source and a reversible,
   bounded mapping to the code view, omitting recognized metadata/export envelopes.
   Compose newline/UTF-8 conversion, omitted ranges and procedure views. Apply
   edits only to visible ranges; define insertion affinity at hidden records so
   edits cannot detach a member's attributes. Preserve raw spelling, order, unknown
   records and line endings through undo, redo and plain-source save/export.
   Offer a separate read-only raw-source/metadata inspector for investigation.
3. **M6.11 — document-component presentation and properties.** Show code names in
   explorer/Properties, preserving export/source names separately. An attribute-only
   workbook opens as blank code; event navigation remains available. Route document
   rename through host validation and consistent source/metadata updates, not a
   filename edit. Expose descriptions/Instancing only where supported; retain
   host-controlled fields as read-only. Excel Macro Options and shortcut dispatch
   belong to GridCraft; shared metadata UI belongs to tiny-vba.
4. **M6.12 — regressions and integrated evidence.** An independently authored
   minimal `Macro2` with the exact metadata parses/compiles/runs without a metadata
   checkpoint; unsupported `Sort` operations still report their own limits. Cover
   attributes at several member positions, invalid targets/values, default members,
   negative enumerator IDs and strings/comments containing `Attribute`. Prove blank
   workbook display, code-name presentation and raw-source retention. Check mapped
   diagnostics, completion, search/replace, selections, breakpoints, stepping, stack
   navigation and full/procedure views across CR/LF/CRLF, Unicode and hidden records
   between procedures. Include first insertion into an attribute-only module,
   procedure rename/delete, cross-boundary replacement and undo/redo/save revision
   checks. Run native/WASM/WASI and UI/native/browser gates plus real fork workflows.

M7 unchanged-document acceptance must additionally prove metadata retention through
load/edit/plain-source export, separately reporting package preservation and future
binary writeback. Do not run the user's workbook as a research step or claim that
fixing attributes makes its entire recorded macro executable. Reference-build
observations for platform encoding, rename behavior and dialogs remain NOT_RUN.
