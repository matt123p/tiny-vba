# Excel VBA Context, Name Resolution, and Object Discovery

**Purpose:** Implementation-oriented specification for a VBA runtime and Excel-compatible host object model, particularly a Rust interpreter integrating with a spreadsheet application.

**Scope:** Unqualified identifiers, `Range`, workbook/worksheet contexts, `Me`, `With`, host globals, object binding, default members, late binding, and error behavior.

> **Compatibility note:** Excel/VBA has both language-level name binding and host-specific conveniences. Do not implement all unqualified names as dynamic lookups on `ActiveWorkbook`. The active sheet is only one possible fallback, and compile-time binding takes precedence. Validate edge cases against desktop Excel with a compatibility test suite.

## 1. Core distinctions

Three separate questions must be answered for an expression such as `Range("A1")`:

1. **Name resolution:** What does the identifier `Range` denote in this lexical and module context?
2. **Receiver selection:** If it denotes a member, what object supplies that member?
3. **Execution:** How is the call dispatched and what object/value is returned?

`Range("A1")` in a standard Excel VBA module normally uses Excel's globally exposed `Range` member, which delegates to the active sheet. In a worksheet document module, an unqualified `Range` can bind to the worksheet's own member, equivalent in ordinary cases to `Me.Range`. These are *different binding paths*, even if they happen to return the same cell.

### Common object identities

| Expression | Meaning |
|---|---|
| `Application` | Current Excel application instance |
| `ActiveWorkbook` | Workbook active in the application; can be absent |
| `ThisWorkbook` | Workbook containing the executing VBA project |
| `ActiveSheet` | Active sheet in the active workbook; not necessarily a worksheet |
| `Me` | Current class/document-module instance; unavailable in standard modules |
| `Worksheets("Data")` | Worksheet collection lookup on the active workbook when unqualified in ordinary standard-module code |
| `ThisWorkbook.Worksheets("Data")` | Explicit lookup in the code-containing workbook |
| `Sheet1` | Worksheet **code name** identifier in the VBA project, distinct from its user-visible tab name |
| `Workbooks("Book2.xlsx")` | Lookup in the Excel application's open-workbook collection |
| `Selection` | Current selected object, which need not be a range |

`ActiveWorkbook` can change after activating another workbook, and may not be the workbook containing the macro. `ThisWorkbook` does not change during that macro's execution.

## 2. Resolution by module type

| Code location | Typical unqualified `Range("A1")` receiver | Important caveat |
|---|---|---|
| Standard module | Excel global `Range`, using `ActiveSheet` | Local/project names can shadow it |
| Worksheet document module | Current worksheet (`Me`) | Worksheet members participate in implicit-member lookup |
| `ThisWorkbook` document module | Excel global `Range`, using `ActiveSheet` | Workbook does not itself expose worksheet `Range` |
| UserForm module | Excel global `Range`, using `ActiveSheet` | Form members and declarations can shadow the name |
| Ordinary class module | Excel global `Range`, using `ActiveSheet` when Excel globals are available | Class members and declarations take precedence |

These describe the common, unshadowed cases, not a universal fixed precedence list for every possible collision.

### Standard module

```vba
Sub WriteActive()
    Range("A1").Value = 100
    ActiveSheet.Range("B1").Value = 200
End Sub
```

The two calls usually target the same worksheet, provided the active sheet is a worksheet and no intervening action changes it.

```vba
Sub WriteSpecific()
    ThisWorkbook.Worksheets("Data").Range("A1").Value = 100
End Sub
```

The second example is robust against a different workbook being active.

### Worksheet module

```vba
Private Sub Worksheet_Change(ByVal Target As Range)
    Me.Range("A1").Interior.Color = vbYellow
    Range("B1").Value = Target.Address
End Sub
```

`Me` explicitly identifies the worksheet containing this code. `Range("B1")` normally binds to the same worksheet's implicit member, even when another sheet becomes active.

### Workbook module

```vba
Private Sub Workbook_Open()
    ' Not automatically a cell in ThisWorkbook:
    Range("A1").Value = "Opened"
    ' Explicit, deterministic:
    Me.Worksheets("Data").Range("A1").Value = "Opened"
End Sub
```

`Me` in `ThisWorkbook` refers to the workbook object. It has a `Worksheets` collection, but not a worksheet-level `Range` member.

### UserForm and class modules

```vba
Private Sub CommandButton1_Click()
    ThisWorkbook.Worksheets("Data").Range("A1").Value = Me.TextBox1.Value
End Sub
```

Here `Me` denotes the form; a naked `Range` would not automatically use the form's associated workbook.

## 3. Lexical binding and shadowing

The runtime must not assume that a token spelled `Range` always names Excel's Range API.

```vba
Sub Shadowed()
    Dim Range As String
    Range = "not an Excel range"
    Debug.Print Range
End Sub
```

The local declaration wins for references in its scope. A call-like expression `Range("A1")` now requires VBA's normal type and call/indexing semantics; it must not silently fall back to Excel.

Other relevant name sources include parameters, procedure locals, module declarations, procedure names, properties, members of the current class/document module, project-visible declarations, referenced type libraries, and host-provided globals.

**Implementation rule:** Bind an identifier to a symbol or a deferred dispatch target first; only then evaluate arguments and invoke it. Name collision behavior, ambiguity, accessibility, and declaration order must be tested rather than guessed.

### A practical staged resolver

1. Parse expression shape: `Range(...)`, `.Range(...)`, `obj.Range(...)`, `!Name`, `obj(...)`.
2. Resolve local/parameter/module/project identifiers using VBA visibility rules.
3. Resolve eligible implicit members on `Me` for class and document modules.
4. Resolve imported/reference-library and Excel host-global identifiers.
5. If an object member is explicit, bind/dispatch that member on the selected receiver.
6. If a symbol's type or dispatch is dynamic, defer member lookup until runtime.
7. Apply argument passing, default-member, conversion, and error semantics.

This is an **engineering decomposition**, not a normative assertion that every VBA scope has exactly the numbered priority above. Maintain a tested precedence table for conflicting identifiers.

## 4. Excel host globals and active context

Excel exposes many convenient unqualified names through its global object surface, including `Range`, `Cells`, `Rows`, `Columns`, `Worksheets`, `Workbooks`, `ActiveSheet`, `ActiveWorkbook`, `Selection`, and `Application`.

Common unqualified forms in standard modules:

```vba
Range("A1")           ' Active-sheet-based Excel global
Cells(1, 1)           ' Active-sheet-based Excel global
Rows(1)               ' Active-sheet-based Excel global
Columns("B")          ' Active-sheet-based Excel global
Worksheets("Data")    ' Active-workbook-based collection
Workbooks("Book2.xlsx") ' Application's open workbooks
```

The precise binding in a document module can differ because the document object's members are also available implicitly.

**Important:** `ActiveSheet` may be a chart sheet, not a `Worksheet`; range operations can fail. There may also be no active workbook in some application states. Model errors rather than inventing a worksheet.

### Active context versus code context

```vba
Sub Demonstrate()
    Debug.Print ThisWorkbook.Name
    Debug.Print ActiveWorkbook.Name
    Workbooks("Other.xlsx").Activate
    Range("A1").Value = 42
End Sub
```

After activation, the final `Range` generally targets the active sheet in `Other.xlsx`, whereas `ThisWorkbook` still points to the code-containing workbook.

## 5. Qualified expressions and the `With` statement

```vba
With ThisWorkbook.Worksheets("Data")
    .Range("A1").Value = 1
    Range("B1").Value = 2
    .Cells(2, 1).Value = 3
End With
```

A leading dot selects the current `With` receiver. A bare `Range` does **not** inherit the `With` receiver: it follows ordinary name resolution. In a standard module it commonly targets the active sheet.

Nested `With` blocks use the innermost receiver for leading-dot expressions:

```vba
With ThisWorkbook
    With .Worksheets("Data")
        .Range("A1").Value = "inner worksheet"
    End With
End With
```

Implementation: maintain a lexical `With`-receiver stack. Do not mutate `ActiveSheet` when entering `With`.

### Avoid partial qualification

```vba
' Incorrect when the active sheet is not Data:
With ThisWorkbook.Worksheets("Data")
    .Range(Cells(1, 1), Cells(3, 3)).Value = 9
End With
```

`Cells` without a dot may resolve against a different sheet. Prefer:

```vba
With ThisWorkbook.Worksheets("Data")
    .Range(.Cells(1, 1), .Cells(3, 3)).Value = 9
End With
```

Similarly, `Worksheets("Data").Range(Cells(1,1), Cells(3,3))` can fail if the `Cells` arguments belong to another worksheet.

## 6. Object discovery and binding

**Object discovery** is not a single global search for an object by name. Distinguish:

- **Lexical symbols:** variables, procedures, properties, constants, module/class members.
- **Project document objects:** `ThisWorkbook`, worksheet code names such as `Sheet1`.
- **Host globals:** Excel's global entry points, backed by `Application` and current active state.
- **Collection lookup:** `Workbooks("...")`, `Worksheets("...")`, `Names("...")`; names are data, not VBA symbols.
- **Member lookup:** `obj.Range`, `obj.Cells`, `obj.Value`; uses the receiver's interface.
- **Late binding:** `Object` variables or COM automation may require runtime `IDispatch` lookup.
- **Default members:** `Range("A1")`, `Cells(1,1)`, `someCollection("key")` may combine callable member/indexing/default-member semantics.

### Early binding

```vba
Dim ws As Excel.Worksheet
Set ws = ThisWorkbook.Worksheets("Data")
ws.Range("A1").Value = 5
```

A typed `ws` can resolve member signatures against the Excel type library at binding time, while the object instance is supplied at runtime.

### Late binding

```vba
Dim obj As Object
Set obj = ThisWorkbook.Worksheets("Data")
obj.Range("A1").Value = 5
```

`obj.Range` requires runtime member discovery/dispatch, with compatible errors if the member is missing or invocation fails.

### `Set`, `Let`, and default members

```vba
Dim r As Range
Set r = ThisWorkbook.Worksheets("Data").Range("A1")
r.Value = 123
Debug.Print r          ' May coerce through the Range default member
```

Keep object identity separate from scalar value extraction. Do not automatically collapse all `Range` objects into their cell values.

### Bang syntax

```vba
Debug.Print someObject!FieldName
```

`!` is not interchangeable with `.`. It commonly performs named-item/default-collection access depending on the object's supported semantics. Implement it as a distinct expression form and validate host-specific cases.

## 7. Active workbook, worksheet, and event contexts

| Scenario | Safe interpretation |
|---|---|
| Macro launched from workbook A while workbook B is active | `ThisWorkbook` is A; active globals can refer to B |
| `Worksheet_Change` event | `Me` is the worksheet raising the event; `Target` is explicitly supplied |
| `Workbook_Open` event | `Me` is the workbook; naked `Range` is not implicitly scoped to it |
| Macro called by `Application.Run` | Preserve the executing project/module context; active workbook is independent |
| Macro in an add-in | `ThisWorkbook` is the add-in workbook; `ActiveWorkbook` often represents the user's document |
| `With ws` | Leading-dot members use `ws`; bare identifiers do not |
| A chart sheet is active | Active-sheet range operations may raise an Excel error |
| `ActiveWorkbook` changes mid-procedure | Later unqualified global lookups can use the new active context |
| Code in a UserForm | `Me` refers to the form, not the worksheet |
| Multiple Excel application instances | Each application has its own active state and workbook collection |

Do not introduce a universal implicit "current workbook" parameter to every VBA call; distinguish host active state from module provenance.

## 8. Excel range and object examples

### A. Reliable macro operating on its own workbook

```vba
Sub RecalculateData()
    Dim ws As Worksheet
    Set ws = ThisWorkbook.Worksheets("Data")
    ws.Range("A1").Value = Now
    ws.Range(ws.Cells(2, 1), ws.Cells(100, 3)).ClearContents
End Sub
```

### B. Operate on the user's current selection

```vba
Sub HighlightSelection()
    If TypeOf Selection Is Excel.Range Then
        Selection.Interior.Color = vbYellow
    End If
End Sub
```

`Selection` is not guaranteed to be a `Range`.

### C. Event handler using stable worksheet identity

```vba
Private Sub Worksheet_Change(ByVal Target As Range)
    If Not Intersect(Target, Me.Range("A:A")) Is Nothing Then
        Me.Range("B1").Value = "Column A changed"
    End If
End Sub
```

### D. Code-name versus sheet-tab name

```vba
Sheet1.Range("A1").Value = 1
ThisWorkbook.Worksheets("Sales").Range("A1").Value = 2
```

`Sheet1` is a VBA project code name; `"Sales"` is a worksheet collection key, usually the tab name. Renaming the tab need not change the code name.

### E. Dynamic workbook and sheet discovery

```vba
Dim wb As Workbook, ws As Worksheet
For Each wb In Application.Workbooks
    For Each ws In wb.Worksheets
        Debug.Print wb.Name, ws.Name
    Next ws
Next wb
```

This is runtime collection enumeration, not lexical symbol discovery.

## 9. Proposed Rust architecture

Separate binding from Excel host dispatch.

```text
VBA source
   |
Parser / AST
   |
Binder / symbol resolution
   |-- procedure and local scopes
   |-- module/project scopes
   |-- implicit Me members
   |-- referenced libraries
   `-- Excel global namespace
   |
Bound expression / deferred dynamic call
   |
Runtime evaluator
   |-- execution frame (Me, locals, With stack)
   |-- project context (ThisWorkbook, code-name objects)
   |-- Excel application state (ActiveWorkbook, ActiveSheet, Selection)
   `-- object handles / collections
   |
Host object model / dispatch
   |-- Workbook / Worksheet / Range
   |-- typed member invocation
   |-- late-bound dispatch
   `-- VBA-compatible errors and coercions
```

Suggested interfaces (conceptual):

```rust
struct ExecutionContext {
    project_id: ProjectId,
    module_id: ModuleId,
    me: Option<ObjectHandle>,
    with_stack: Vec<ObjectHandle>,
    application: ObjectHandle,
}

enum BoundTarget {
    Local(SymbolId),
    ModuleMember(SymbolId),
    ProjectMember(SymbolId),
    ImplicitMeMember(MemberId),
    HostGlobal(MemberId),
    ExplicitMember { receiver: ExprId, member: MemberId },
    LateBoundMember { receiver: ExprId, name: String },
}
```

The host should own live active-workbook and active-sheet state. A bound `HostGlobal(Range)` call must consult that state **when invoked**, not when the procedure starts. A bound implicit `Me.Range` call must retain its worksheet object receiver.

Recommended internal APIs:

- `resolve_identifier(name, lexical_context) -> BoundTarget`
- `resolve_member(receiver_type, name) -> MemberBinding`
- `invoke_member(receiver, member, args, invocation_kind) -> Variant`
- `excel_global_range(address, current_application_state) -> RangeHandle`
- `get_default_member(object, invocation_kind) -> Variant`
- `resolve_collection_item(collection, key) -> ObjectHandle`

Represent object handles distinctly from VBA `Variant` values and retain identity across `Set` assignments.

## 10. Compatibility tests

Create workbooks A and B with distinct cell values and a third worksheet; then test at least:

| Test | Expected behavior |
|---|---|
| Standard-module `Range("A1")` with B active | B's active worksheet |
| Worksheet A module `Range("A1")` with B active | Worksheet A |
| `ThisWorkbook` module `Range("A1")` with B active | B's active worksheet, in ordinary unshadowed case |
| `ThisWorkbook.Worksheets("Data").Range("A1")` | A/Data regardless of active workbook |
| `With A.Worksheets("Data"): .Range(...)` | A/Data |
| `With A.Worksheets("Data"): Range(...)` | Ordinary implicit resolution, not automatically A/Data |
| `.Range(.Cells(...), .Cells(...))` | All receivers agree |
| `.Range(Cells(...), Cells(...))` with different active sheet | Detect mismatched-sheet error or match observed Excel behavior |
| Local variable named `Range` | Local symbol shadows global |
| Worksheet code name `Sheet1` | Project document object, independent of tab caption |
| Late-bound `Object.Range` | Runtime member lookup and dispatch |
| Active chart sheet | Appropriate error on unsupported `Range` |
| `ThisWorkbook` in add-in | Add-in workbook, not user's active workbook |
| Active workbook changed between two calls | Each global call uses contemporaneous active state |
| `Selection` is shape/chart | Do not treat it automatically as a `Range` |
| Unqualified `Cells`, `Rows`, `Columns` in worksheet module | Check implicit worksheet member binding |
| Explicitly qualified `Application.Range` | Test actual Excel behavior and error handling |
| Name collisions with reference libraries and project modules | Match VBA precedence and ambiguity errors |

Automate a differential harness: run identical macros in Excel and the Rust implementation; capture cell changes, object identity, return types, and error numbers/descriptions. Record Excel version, module kind, workbook activation state, and source location for every case.

## 11. Implementation guidance and pitfalls

1. **Never** rewrite all `Range(...)` expressions to `ActiveWorkbook.ActiveSheet.Range(...)` at parse time.
2. Keep `ThisWorkbook`, `ActiveWorkbook`, `ActiveSheet`, `Me`, and `With` as distinct concepts.
3. Resolve unqualified names using the correct module and project environment.
4. Make active context dynamic: workbook and sheet activation can occur between statements.
5. Distinguish project code names from user-visible worksheet names.
6. Support runtime collection enumeration and item lookup separately from symbol lookup.
7. Preserve object handles, `Set` semantics, default members, and late binding.
8. Make errors observable and VBA-compatible; do not silently substitute a sheet.
9. Include chart sheets, add-ins, UserForms, events, and cross-workbook calls in regression tests.
10. Treat edge-case binding precedence as empirical compatibility work: verify against Excel before encoding assumptions.

## 12. Reference material

- Microsoft Learn — [Range object (Excel)](https://learn.microsoft.com/en-us/office/vba/api/excel.range(object))
- Microsoft Learn — [Application.Range property (Excel)](https://learn.microsoft.com/en-us/office/vba/api/excel.application.range)
- Microsoft Learn — [Worksheet.Range property (Excel)](https://learn.microsoft.com/en-us/office/vba/api/excel.worksheet.range)
- Microsoft Learn — [Application.ActiveWorkbook property](https://learn.microsoft.com/en-us/office/vba/api/excel.application.activeworkbook)
- Microsoft Learn — [Application.ThisWorkbook property](https://learn.microsoft.com/en-us/office/vba/api/excel.application.thisworkbook)
- Microsoft Learn — [Application.ActiveSheet property](https://learn.microsoft.com/en-us/office/vba/api/excel.application.activesheet)
- Microsoft Learn — [With statement](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/with-statement)
- Microsoft Learn — [Me keyword](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/me-keyword)
- Microsoft Learn — [Understanding scope and visibility](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/understanding-scope-and-visibility)
- Microsoft Learn — [TypeOf operator](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/typeof-operator)
- Microsoft Learn — [Set statement](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/set-statement)

These are reference starting points, not evidence that every implementation detail above has been experimentally verified. For high-fidelity VBA compatibility, the differential test suite is authoritative for behavior of the targeted Excel versions.
