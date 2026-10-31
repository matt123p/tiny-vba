# VBA Module Types and Execution Contexts

**Design specification for a Rust VBA interpreter and Excel-compatible host integration**

## 1. Terminology

VBA does not have a switch called “standard mode”. The relevant distinction is **module kind**: a procedure executes in a standard module, a document module (worksheet or workbook), a class module, or a UserForm module. The module kind controls the available implicit instance (`Me`), accessible members, lifecycle, and event handling. It does **not** replace the independent Excel application state (`ActiveWorkbook`, `ActiveSheet`, `Selection`).

This document concentrates on Excel VBA but separates language rules from Excel host-specific behaviour.

## 2. At-a-glance matrix

| Module kind | Has `Me`? | Instance | Unqualified `Range("A1")`, absent shadowing | Typical use |
|---|---|---|---|---|
| Standard (`.bas`) | No | Module-level namespace, not an object instance | Excel global `Range`, normally active worksheet | Public macros, utility functions |
| Worksheet document module | Yes | That worksheet | Worksheet's implicit `Me.Range` | Worksheet events and sheet-local logic |
| Workbook document module (`ThisWorkbook`) | Yes | Workbook containing the code | Excel global `Range`, normally active worksheet | Workbook events, startup/closing |
| UserForm (`.frm`) | Yes | Specific form instance | Excel global `Range`, normally active worksheet | Dialogs and form events |
| Ordinary class (`.cls`) | Yes | Specific class instance | Excel global `Range`, normally active worksheet | Domain objects and encapsulation |
| Chart-sheet document module | Yes | Chart sheet | No worksheet `Me.Range`; normally Excel global `Range` | Chart events |

**Important:** This is the ordinary, unshadowed behaviour. A local, parameter, module declaration, or accessible member may change binding. A chart sheet can be the active sheet, in which case an active-sheet-based range call may fail.

## 3. Standard modules

### Nature and lifetime

A standard module is a container for procedures, functions, constants, types, and module-level variables. It has no `Me` instance. Public procedures may be exposed as macros; `Private` members are module-scoped. `Option Private Module` restricts external visibility but does not convert the module into a class.

```vba
' Module: modReports
Option Explicit

Private reportCount As Long

Public Sub BuildReport()
    reportCount = reportCount + 1
    Range("A1").Value = reportCount  ' Usually the active worksheet
End Sub
```

The module's state is project-level VBA state, not per workbook object or per call. Resetting the VBA project can reset module-level variables.

### Recommended practice

Pass workbook/worksheet objects explicitly when correctness matters:

```vba
Public Sub BuildReportFor(ByVal ws As Excel.Worksheet)
    ws.Range("A1").Value = "Report"
End Sub
```

## 4. Worksheet document modules

Each worksheet has an associated document module. Its instance is supplied by Excel; it is not created by calling `New`. `Me` refers to that particular worksheet, not necessarily the active sheet. Worksheet members can be used implicitly.

```vba
' In worksheet code module for the sheet whose code name is Sheet1
Private Sub Worksheet_Change(ByVal Target As Range)
    If Not Application.Intersect(Target, Me.Range("A:A")) Is Nothing Then
        Me.Range("B1").Value = "Changed"
    End If
End Sub
```

In this module, `Range("A1")` ordinarily means `Me.Range("A1")`. The worksheet code name (`Sheet1`) is a project-level identifier distinct from the tab caption (`Name`), which users can rename.

```vba
Sheet1.Range("A1").Value = 1                 ' Code-name object
ThisWorkbook.Worksheets("Sales").Range("A1").Value = 2  ' Tab-name lookup
```

A worksheet module may also expose public methods and properties callable via the worksheet's code-name instance. Such methods are not automatically global macros.

## 5. Workbook document module (`ThisWorkbook`)

This module is the workbook object's document module. `Me` refers to the workbook that contains the project. Its events include `Workbook_Open`, `Workbook_BeforeClose`, and `Workbook_SheetChange`.

```vba
' In ThisWorkbook
Private Sub Workbook_Open()
    Me.Worksheets("Dashboard").Range("A1").Value = Now
End Sub
```

A workbook has no worksheet-level `Range` property. Therefore bare `Range("A1")` does not become `Me.Range("A1")`; it ordinarily resolves through Excel's global object surface, based on the active worksheet.

`ThisWorkbook` and `ActiveWorkbook` are different identities. This is particularly important for add-ins and cross-workbook automation.

## 6. UserForm modules

A UserForm is a class-like module associated with a visual form and controls. `Me` denotes the current form instance; controls are exposed as members. A UserForm can have a default instance, and code can also create explicit instances.

```vba
' In frmEntry
Private Sub cmdSave_Click()
    ThisWorkbook.Worksheets("Data").Range("A1").Value = Me.txtName.Value
    Me.Hide
End Sub
```

```vba
Dim dialog As frmEntry
Set dialog = New frmEntry
dialog.Show
Unload dialog
```

**Implementation considerations:** default-instance semantics, initialization/termination, modal versus modeless display, form control member resolution, event callbacks, `Hide` versus `Unload`, and form state persistence.

Unqualified Excel calls such as `Range` do not implicitly acquire a workbook from the form.

## 7. Ordinary class modules

A class module defines a type; each instance has its own instance fields and `Me` reference. Members can be `Public`, `Private`, or `Friend` as supported by VBA. `Class_Initialize` and `Class_Terminate` are lifecycle hooks.

```vba
' Class: CReport
Option Explicit
Private mSheet As Excel.Worksheet

Public Sub Initialize(ByVal ws As Excel.Worksheet)
    Set mSheet = ws
End Sub

Public Sub WriteTitle(ByVal title As String)
    mSheet.Range("A1").Value = title
End Sub
```

```vba
Dim report As CReport
Set report = New CReport
report.Initialize ThisWorkbook.Worksheets("Data")
report.WriteTitle "Quarterly results"
```

Class members can shadow Excel global identifiers. Avoid implementing a class method as if it always ran in a standard module.

### `WithEvents`

VBA can declare an object variable `WithEvents` in a class-like module and bind event procedures to it:

```vba
' In class CApplicationEvents
Private WithEvents App As Excel.Application

Public Sub Attach(ByVal excelApp As Excel.Application)
    Set App = excelApp
End Sub

Private Sub App_WorkbookOpen(ByVal Wb As Excel.Workbook)
    Debug.Print Wb.Name
End Sub
```

The instance holding the event sink must remain alive for callbacks to continue.

## 8. Chart-sheet and other host document modules

Excel supports chart sheets as distinct sheet objects. Their document modules can have `Me` and host-specific events, but a chart is not a worksheet and does not expose the same member set. Do not assume all sheet document modules support `Range`, `Cells`, or worksheet events. Some other Office hosts supply their own document-module types, which should be modelled as host extensions rather than built into the core VBA language.

## 9. How module kinds affect name resolution

For every unqualified identifier, first consider the VBA language's declarations and scopes, then eligible implicit members and host-provided names. Exact shadowing and ambiguity behaviour should be tested against Excel rather than assumed from a simplistic global search order.

```vba
' Standard module
Sub Demo()
    Dim Range As String
    Range = "local variable"  ' Not Excel.Range
End Sub
```

```vba
' Worksheet module
Sub Demo()
    Me.Range("A1").Value = 1
    Range("B1").Value = 2   ' Normally implicit worksheet member
End Sub
```

```vba
' Any applicable module
With ThisWorkbook.Worksheets("Data")
    .Range("A1").Value = 1   ' Leading dot uses With receiver
    Range("B1").Value = 2    ' Bare identifier uses normal module binding
End With
```

A `With` statement does not change `Me`, `ActiveSheet`, or the active workbook. It only supplies a receiver for dot-prefixed expressions.

## 10. Call entry points and context preservation

A procedure can be entered via the macro dialog, a button, `Application.Run`, another VBA procedure, a workbook/worksheet event, a form control event, or a `WithEvents` callback. These entry points affect parameters and event provenance but **do not change the procedure's declared module kind**.

| Entry path | What must be retained |
|---|---|
| Standard macro | Project/module identity, active Excel application state |
| Worksheet event | Specific worksheet object as `Me`, event arguments (`Target`, etc.) |
| Workbook event | Specific workbook as `Me`, event arguments |
| UserForm event | Form instance and its control tree |
| Class method | Instance fields and object identity |
| `Application.Run` | Correct target project/procedure and runtime application state |
| Cross-module call | Callee's module binding rules, not caller's implicit `Me` |

Example: a worksheet event calling a standard-module helper does **not** make that helper's bare `Range` implicitly refer to the event worksheet.

```vba
' Worksheet module
Private Sub Worksheet_Change(ByVal Target As Range)
    UpdateFlag Me
End Sub

' Standard module
Public Sub UpdateFlag(ByVal ws As Worksheet)
    ws.Range("B1").Value = "Updated"
End Sub
```

## 11. Instance identity and lifecycle

The interpreter should distinguish:

- **Project identity:** workbook or add-in containing the VBA project.
- **Module identity:** the lexical home of a procedure.
- **Instance identity:** worksheet, workbook, UserForm, or class object bound to `Me`.
- **Application state:** active workbook, active sheet, and selection, which can change independently.
- **Event subscription:** which instance is subscribed and how long it remains alive.

A worksheet's code-name object should map to the corresponding host object handle. Renaming a tab should not break its code-name binding. Replacing or closing a workbook should invalidate or retire associated host handles according to tested Excel semantics.

## 12. Proposed Rust model

```rust
enum ModuleKind {
    Standard,
    WorksheetDocument,
    WorkbookDocument,
    ChartDocument,
    UserForm,
    Class,
}

struct ModuleDescriptor {
    kind: ModuleKind,
    project_id: ProjectId,
    module_id: ModuleId,
    // Compiled declarations, host type, visibility, event metadata...
}

struct CallFrame {
    module_id: ModuleId,
    me: Option<ObjectHandle>,
    with_receivers: Vec<ObjectHandle>,
    // Local variables, arguments, instruction pointer...
}

struct ExcelHostContext {
    application: ObjectHandle,
    // ActiveWorkbook, ActiveSheet and Selection are queried live.
}
```

The **binder** handles module-kind-specific implicit members and lexical symbols. The **runtime** binds `Me` to a concrete instance. The **Excel host layer** supplies workbook/sheet objects, active application state, and events. Avoid injecting an implicit workbook into every call frame.

### Suggested implementation sequence

1. Standard modules, project declarations, and Excel host globals.
2. Workbook and worksheet document modules with code-name objects and `Me`.
3. Cross-module calls, explicit qualification, `With`, and shadowing.
4. Class instances, constructors/lifecycle, properties, and object identity.
5. UserForms and default instances, then control events.
6. Excel document events and `WithEvents` subscriptions.
7. Chart modules, add-in cases, and detailed compatibility error handling.

## 13. Differential compatibility test matrix

Create two workbooks, each with multiple worksheets, and execute tests with a different workbook active from the one containing the code.

| Test | Expected result |
|---|---|
| Standard module `Range("A1")` | Active worksheet, absent shadowing |
| Worksheet module `Range("A1")` | Worksheet bound to that module |
| Worksheet module `Me.Range("A1")` | Always module's worksheet |
| `ThisWorkbook` module bare `Range` | Active worksheet, absent shadowing |
| Workbook module `Me.Worksheets("X")` | Workbook bound to module |
| UserForm `Me` | Form instance |
| Class `Me` | Class instance |
| Worksheet event calls standard helper with bare `Range` | Helper uses its own standard-module binding rules |
| Worksheet code name after tab rename | Same worksheet object |
| `With ws: .Range(...)` | `ws` |
| `With ws: Range(...)` | Ordinary unqualified resolution |
| Add-in `ThisWorkbook` | Add-in workbook |
| `WithEvents` handler | Fires only while subscribed object and sink are alive |
| Chart sheet active, standard bare `Range` | Match Excel's error behaviour |
| Local identifier shadows `Range` | Local binding, not Excel global |

Capture not just resulting cell values but error number, error description, object identity, module name, active workbook/sheet, and Excel version.

## 14. Design decisions

**Recommended:** represent module kind as immutable compilation metadata, not as a runtime mode flag. Use instance-bound execution frames for document, class, and form modules. Resolve host globals independently from `Me`. This keeps the interpreter reusable across GridCraft, WordCraft, and other Craft hosts: each host supplies its document module types, global names, and event adapters without changing core VBA semantics.

## 15. References

- [Microsoft Learn — VBA language reference](https://learn.microsoft.com/en-us/office/vba/language/reference/)
- [Microsoft Learn — Understanding scope and visibility](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/understanding-scope-and-visibility)
- [Microsoft Learn — Me keyword](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/me-keyword)
- [Microsoft Learn — With statement](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/with-statement)
- [Microsoft Learn — Worksheet object (Excel)](https://learn.microsoft.com/en-us/office/vba/api/excel.worksheet)
- [Microsoft Learn — Workbook object (Excel)](https://learn.microsoft.com/en-us/office/vba/api/excel.workbook)
- [Microsoft Learn — UserForm object](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/userform-object)
- [Microsoft Learn — Class module](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/class-module)
- [Microsoft Learn — WithEvents statement](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/withevents)

*This is an implementation guide, not a claim that all edge cases have been experimentally validated. Verify ambiguous precedence, host event ordering, and lifecycle behaviour against target Excel versions.*
