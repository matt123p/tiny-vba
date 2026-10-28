# Craft* VBA Editor and Debugger — Public-Source UI Compatibility Specification

Scope update, 2026-10-28: the [remaining plan](../implementation-plan.md) takes
precedence over this earlier proposal. Native extensions/translation and AI
assistance are removed from scheduled scope. Third-party ActiveX/OLE controls
not supplied with Office are excluded; standard Office controls remain in scope.
The P0–P8 phases below are reference groupings, not the current task sequence.

**Document version:** 1.0<br>
**Research date:** 27 October 2026<br>
**Primary target:** The classic Microsoft Visual Basic Editor (VBE) integrated into **Windows desktop Excel**, as the familiar user-facing model for **GridCraft**.<br>
**Secondary target:** The same shared editor/debugger UI integrated with **WordCraft** and future Craft* applications.<br>
**Implementation stack assumed:** Rust VBA language runtime and IR, Rust host object-model adapter, and a native Craft* UI.<br>
**Status:** Research-based product/engineering specification, not a Microsoft document and not a claim of pixel-perfect verification against every Office release.

> **Compatibility goal:** An experienced Excel user should immediately recognise where to open a VBA project, add a module, write a procedure, use IntelliSense, run a macro, set a breakpoint, inspect variables, and return to the workbook — without learning a different IDE.
>
>
> **Clean-source goal:** Implement UI *behaviour and functional conventions* using public documentation and independently written code and artwork. Do not import proprietary VBE source, decompiled implementations, extracted icons/resources, leaked material, or undocumented internals. Publicly viewable screenshots may inform *functional observations* but should not be copied into distributed UI assets.

---

## Contents

1. [Scope, provenance, and interpretation](#1-scope-provenance-and-interpretation)
2. [Reference UI and window arrangement](#2-reference-ui-and-window-arrangement)
3. [Detailed pane specifications](#3-detailed-pane-specifications)
4. [Full menu specification](#4-full-menu-specification)
5. [Toolbar and button inventory](#5-toolbar-and-button-inventory)
6. [Code editor behaviour](#6-code-editor-behaviour)
7. [Completion, help, navigation, and other editor aids](#7-completion-help-navigation-and-other-editor-aids)
8. [Debugger user experience and semantics](#8-debugger-user-experience-and-semantics)
9. [Optional Copilot-style improvements](#9-optional-copilot-style-improvements)
10. [GridCraft/WordCraft integration](#10-gridcraftwordcraft-integration)
11. [Rust-facing UI contracts](#11-rust-facing-ui-contracts)
12. [Accessibility, configuration, and polish](#12-accessibility-configuration-and-polish)
13. [Development roadmap and acceptance tests](#13-development-roadmap-and-acceptance-tests)
14. [Public sources and traceability](#14-public-sources-and-traceability)
15. [Open verification items and release gate](#15-open-verification-items-and-release-gate)
16. [M8 code assistance and syntax feedback examples](#16-m8-code-assistance-and-syntax-feedback-acceptance-examples)

---

## 1. Scope, provenance, and interpretation

### 1.1 The reference product

The reference is the **classic Visual Basic Editor**, not Visual Studio, VS Code, the Office Ribbon, or Microsoft's modern Excel Copilot interface. From the host application, users enter VBE with **Alt+F11** or **Developer → Visual Basic**; VBE traditionally occupies a separate application-level window with its own menu bar, dockable toolbars, code/document windows, project explorer, and debugging panes. This arrangement is illustrated in the publicly distributed Microsoft Press sample *Unleashing the power of Excel with VBA* [S02].

The baseline shall be **Excel-on-Windows VBE conventions**, which are more specific than generic VB6/VBA Help pages. Menus, shortcuts, and availability can differ across hosts, releases, current context, and operating systems. We must therefore distinguish:

| Mark | Meaning | How to implement |
|---|---|---|
| **C0 — Core compatibility** | Documented conventional VBE feature required for familiar Excel workflows. | Match label, position, behaviour, command and shortcut where reliably evidenced. |
| **C1 — Extended compatibility** | A documented VBE feature less common in first-use workflows, including the UserForm designer and advanced debugger operations. | Match when implemented; disable honestly until supported. |
| **X — Craft* extension** | Deliberately new affordance, accessibility enhancement, diagnostics, or AI assistance. | Keep out of **Classic/VBE compatibility mode**, or place in an obviously separate optional surface. |
| **V — Verify** | Public sources differ or do not establish exact release-specific behaviour. | Check an authorised, legitimately acquired running copy or additional public documentation; record the outcome before asserting exact equivalence. |

**Important:** The Microsoft Learn UI reference documents the **existence and function** of controls more reliably than it documents their exact pixels, icon drawings, default colours, or drop-down item ordering. This specification captures the documented functional surface; any approximate measurements below are **Craft* design proposals**, *not* values asserted to have been measured from VBE.

### 1.2 What may and may not be reused

**Permitted inputs for a clean implementation process:**

- Public Microsoft Learn descriptions of menus, dialogs, toolbars, keyboard shortcuts and objects [S01, S03–S28].
- Microsoft Open Specifications, including **[MS-VBAL]** for language semantics and **[MS-OVBA]** for file format structure, subject to their own stated IP terms [S29, S30].
- Lawfully accessed public tutorials as secondary corroboration of user workflows and visual arrangement, with author, URL, retrieval date and provenance recorded [S02].
- Black-box interaction tests carried out on a legitimately licensed copy where the test is permitted; write down *observable behaviour*, not implementation details.
- Original Craft* icon drawings, fonts used under appropriate licences, and Rust/UI code written from this specification.

**Do not use:**

- Leaked Microsoft code, non-public VBE design documentation, employee-only specifications, debugger symbols, proprietary UI resources, or undocumented binary assets.
- Decompiled/reconstructed VBE code, extracted DLL assets, resource bitmaps, copied sprite sheets, or Microsoft trademarked imagery as distributable assets.
- Copied HTML, images, diagrams or article text from tutorials as part of Craft* code/assets; link to sources instead.
- Existing third-party VBE clones whose provenance/licence has not been reviewed. In particular, this design assumes **independent implementation**, not reuse of the `craft-vba` implementation previously excluded from this project.

**Operational procedure:** Maintain `docs/provenance/vbe_sources.yaml` recording a source ID, public URL, publisher, date consulted, what was learned, applicable licence/IP notices, and linked requirements. Contributors attest that the relevant implementation was independently authored. Review artwork independently from UI behaviour. Public documentation does not automatically grant a right to reproduce every visual element or third-party image; consult appropriate legal advice if shipping highly imitative commercial UI or importing Microsoft's documentation into training/retrieval assets. This is a risk-control plan, **not a legal guarantee**.

### 1.3 Fidelity principles

1. **Muscle memory first.** Preserve **Alt+F11**, **Ctrl+R**, **Ctrl+G**, **F2**, **F4**, **F5**, **F7**, **F8**, **F9**, **Shift+F8**, **Shift+F9**, and **Ctrl+Space** in the corresponding context.
2. **Traditional UI names.** Users should find `Project Explorer`, `Properties Window`, `Immediate Window`, `Watch Window`, `Object Browser`, `Compile`, `Run Sub/UserForm`, `Step Into`, and `Toggle Breakpoint` under expected menus.
3. **Context matters.** When a form designer has focus, the Format menu and Toolbox are useful; when a code window has focus, editing and debugging commands take priority.
4. **Same host semantics.** A breakpoint in `Worksheet_Change` must stop code triggered by an actual worksheet change in GridCraft, not a simulated detached script.
5. **No fake affordances.** A visible button must work, be visibly disabled with an explanation, or be marked as a Craft* extension/preview. Do not simulate running or debugging if the execution engine cannot support it.
6. **Fidelity without copying assets.** Functional affordances, labels, shortcuts, geometry, and independent iconography matter far more than verbatim toolbar bitmap replication.

---

## 2. Reference UI and window arrangement

### 2.1 Window hierarchy

**C0 main window structure (conceptual, not a screenshot):**

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Microsoft-style VBA Editor title area (Craft* title/branding)                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│ File  Edit  View  Insert  Format  Debug  Run  Tools  Add-Ins  Window  Help        │
├─────────────────────────────────────────────────────────────────────────────────┤
│ [Host] [Insert▼] [Save] [Cut][Copy][Paste][Find][Undo][Redo] ... [Run][Break]...│
│ (optional Editor, Debug, UserForm toolbars when shown or enabled)                │
├─────────────────────────┬───────────────────────────────────────────────────────┤
│ Project Explorer        │ ┌───────────────────────────────────────────────────┐ │
│ [View Code][View Object]│ │ [Object ▼]                  [Procedure/Event ▼]     │ │
│ [Toggle Folders]        │ ├───────────────────────────────────────────────────┤ │
│ VBAProject (Book1.xlsm) │ │                                                   │ │
│  ▾ Microsoft Excel Obj. │ │                  Code window                      │ │
│    Sheet1 (Sheet1)      │ │     [breakpoint gutter] Sub Demo() ...            │ │
│    ThisWorkbook         │ │                                                   │ │
│  ▾ Modules              │ │                                                   │ │
│    Module1              │ └───────────────────────────────────────────────────┘ │
├─────────────────────────┤                                                       │
│ Properties - Module1    │ ┌───────────────────────────────────────────────────┐ │
│ [Alphabetic][Categor.] │ │ Immediate (Ctrl+G), Locals, or Watch (dockable)    │ │
│ (Name)     Module1      │ │ ? Application.ActiveWorkbook.Name                │ │
│                         │ └───────────────────────────────────────────────────┘ │
└─────────────────────────┴───────────────────────────────────────────────────────┘
```

This diagram captures the widely recognised arrangement: menu and toolbars across the top, Project Explorer on the left, Properties usually beneath it, code/document windows in the larger central workspace, and debugger utility windows typically along the lower area [S02, S03–S09]. It is **not** evidence of mandatory pane sizes. Users may float, close, resize, or dock the applicable windows [S08, S10].

**Default Craft* classic layout proposal** (design input, must be user-adjustable): left column about 250–310 logical pixels or 23% of width, Properties below Project Explorer, central editor taking the remainder, Immediate hidden until invoked or last-used state restored. Initial code window occupies the full remaining central height when no debugging pane is visible. Save layout per user and host, not in the workbook by default.

### 2.2 Window and docking rules

- The application chrome carries the familiar **menu bar** and movable **Standard, Editor, Debug, UserForm** toolbars. `View → Toolbars` exposes these groups and a `Customize` entry [S01, S11].
- Multiple code and form windows can be opened; provide close, activate, resize, and switch-between-window behaviours. `Window → Cascade`, `Tile Horizontally`, `Tile Vertically`, `Arrange Icons`, `Split`, plus open-window list implement classic operations where applicable [S12].
- Each dockable utility window has a title, close affordance, keyboard focus and an appropriate context menu; allow dragging/re-docking and turning dockability off through `Tools → Options → Docking` [S10].
- Prefer native **MDI-like** interactions or a well-behaved emulation in classic mode. Document tabs may be offered in **X mode**, but never replace the visible Window menu or its keyboard navigation in Classic mode.
- Minimise/restore, multiple monitors, HiDPI scaling and window-state persistence shall be supported; no proprietary VBE window-manager implementation is required.
- A small status area may show mode, cursor line/column and activity in Craft*; exact historical status-bar pixel/label fidelity is **V** pending target-release verification.

### 2.3 Title, focus, and mode

The title must identify **Craft* VBA Editor**, workbook/document, project, and (where relevant) that execution is paused. **Never call the application “Microsoft Visual Basic” or otherwise suggest Microsoft authorship.** The familiar command labels and layout are for interoperability and transition. VBE distinguishes designing/editing, running, and **break mode**; command enabled states, highlighted statement and debugger panes respond to the runtime state [S13, S14].

**Mode transitions:**

```text
  EDIT/DESIGN ── Run (F5) ──> RUNNING ── Breakpoint/error/Break ──> BREAK
       ▲                          │                                 │
       │                          └──── normal completion ─────────┤
       │                                                            │
       └──── Reset ◄────────────── Continue (F5) ────────────────> RUNNING
```

Here `Reset` returns to a non-running state, discarding VBA execution stack and module-level runtime variables; it does **not** undo changes already made to the workbook or external world. `Design Mode` is a separate project/host event-handling control, not an alias for resetting an application [S13].

---

## 3. Detailed pane specifications

### 3.1 Project Explorer — C0

**Purpose:** A hierarchical view of loaded VBA projects and their components [S03].

**Header/buttons:** `View Code`, `View Object`, `Toggle Folders`; title and close. **Keyboard:** `Ctrl+R` shows the pane; `F7` or `Shift+Enter` opens the selected code window; `Enter` opens/expands selection; `Shift+F10` context menu. Navigation keys include arrow/Home/End [S03].

**Excel project tree expectations:**

```text
VBAProject (Budget.xlsm)
├─ Microsoft Excel Objects
│  ├─ Sheet1 (Summary)
│  ├─ Sheet2 (Data)
│  └─ ThisWorkbook
├─ Forms / UserForms         [if present]
│  └─ UserForm1
├─ Modules                   [if present]
│  └─ Module1
└─ Class Modules             [if present]
   └─ Class1
```

The **exact names and group ordering are V** (specific Excel/VBE release, context and user settings), while project/module types, component relationships, and buttons are documented. A sheet code name may differ from its worksheet tab caption: show both clearly, and allow rename only where host/project semantics permit. Referenced libraries must be accessible through `Tools → References` and Object Browser even if not represented as an always-visible node.

**Interactions:** single selection feeds Properties; double-click opens code/form designer; right-click exposes context actions (View Code/Object, Insert, Rename/Properties as relevant, Import/Export/Remove and project-specific options). `Remove <Item>` must offer an export-first confirmation; deletion is not silently undone [S15]. Projects may be protected/unavailable; display a non-deceptive locked state rather than discarding content.

### 3.2 Properties Window — C0

The pane lists design-time properties of the selected project item, form or control. Provide an object selector/heading, **Alphabetic** and **Categorized** tabs, two-column `Property | Value` grid, editable text cells, drop-down choices where applicable, scrolling, keyboard navigation, and property-specific validation. For multiple selected form controls show common properties [S04].

- **C0:** Module name and relevant accessible metadata; selecting a sheet updates properties of that item.
- **C1:** UserForm/control dimensions, caption, name, enabled/visible, colours, tab order and control-specific properties.
- **F4** is the standard well-established Excel VBE shortcut [S16]; some generic Microsoft pages also list **F6** for certain contexts [S04]. Implement F4; consider F6 as an alias after verification, not as the sole binding.
- When a property cannot be edited, explain why (`Read-only`, `Not supported by this host`, `Protected project`, etc.). Changing `(Name)` must update the symbol model and event association according to VBA semantics, without rewriting unrelated code unexpectedly.

### 3.3 Code Window — C0

**Top row:** two dropdowns: **Object** selector (usually `(General)`, or worksheet/workbook/form/control) and **Procedure/Event** selector. Selecting a procedure navigates to it; selecting an event may create/select its correctly named handler [S05, S17].

**Body:** monospaced editable source; vertical and horizontal scrolling; indicator margin; one line of source can map to several executable statements; optional horizontal split into independently scrolling panes; toolbar or lower-left toggles for **Procedure View** vs **Full Module View**; optional procedure separators. The default is full-module scrolling [S05, S10]. Do not replace these with a different source navigation model in classic mode.

**Code windows:** users can have multiple windows and copy/move text between them, the Immediate window and the Watch window. Active module, caret, selection, undo history and scroll offset remain independent [S05].

### 3.4 Immediate Window — C0

Invoked with `Ctrl+G` or `View → Immediate Window`, dockable and editable as a command/output console [S06].

- Accept a single VBA statement or expression and execute on Enter, subject to host policy and runtime state.
- Support `? expr` as the familiar immediate-print idiom, `Print`/`Debug.Print`, variable inspection, assignment to a writable value while paused, and direct calls to procedures.
- While paused, evaluate in the **selected stack-frame context**, not an unrelated global context; scope/host bindings must match the debugger.
- Output is transient session/debugger data, **not code saved into the workbook**.
- Handle errors as VBA errors with useful text; a failed expression must not corrupt the current breakpoint context.
- If the host cannot safely evaluate a function or object property from the Immediate Window, disable it with an explicit diagnostic until supported; do not return fabricated results.

### 3.5 Locals Window — C0

Shows current stack-frame variables as an expandable grid: **Expression**, **Value**, **Type**; includes a **Calls / Call Stack** control. In a class context show `Me`; where applicable include a synthetic grouping for module variables [S07].

- Refresh upon stepping, entering break mode, and changing selected stack frame.
- Expand object/array members lazily, show strings and numerical types appropriately, handle `Nothing`, `Empty`, `Null`, and errors according to the interpreter's actual type system.
- Users may change permitted **Value** cells in break mode. Reject illegal assignments with a validation error, preserving the old value. Identifiers and types are not directly editable.
- Do not suggest that globals from arbitrary projects are automatically visible: the reference pane has specific scope limitations [S07].

### 3.6 Watch Window and Watch dialogs — C0/C1

Grid columns: **Expression**, **Value**, **Type**, **Context**. Expand compound values, edit appropriate values, and update displayed expressions on pause/step. A watch with an out-of-scope expression indicates that condition instead of showing a made-up value [S08].

`Debug → Add Watch` dialog fields [S18]:

| Field | Required behaviour |
|---|---|
| Expression | Text entry; prefill from code selection where possible. |
| Context: Procedure | Selected or all procedures; scope determines evaluation. |
| Context: Module | Selected or all modules. |
| Context: Project | Current VBA project. |
| Watch Type: Watch Expression | Report current value at supported pauses. |
| Watch Type: Break When Value Is True | Pause when value becomes true/nonzero. |
| Watch Type: Break When Value Changes | Pause when observed value changes. |

`Edit Watch` updates the definition; `Delete Watch` removes it. **Conditional watch expressions may be evaluated frequently**: warn that wide scope, property getter side effects, and complex function calls can slow or alter execution. Preserve the single-threaded determinism of expression evaluation where VBA expects it. Additional editor-only watch features, e.g. pinning and history graphs, are **X**.

### 3.7 Object Browser — C0/C1

Invoked with **F2** or `View → Object Browser` [S09]. Reproduce the classic arrangement:

- **Project/Library** selector including `<All Libraries>`.
- **Search Text** box, Search button, optional Search Results pane with show/hide control.
- **Classes** list and **Members Of** list.
- **Details** pane displaying signatures/definitions, with navigable type/member links.
- **Back**, **Forward**, **Copy to Clipboard**, **View Definition**, **Help** controls, plus resizable dividers.

Populate using the **VBA symbol database**, built-in language/library metadata, GridCraft Excel object-model metadata, loaded projects, and available references. Member type, parameters, optional/default arguments, inherited members, methods, properties, events and constants must derive from the same metadata used by completions. Mark unsupported host members explicitly rather than pretending they are callable. References should be the **publicly modelled APIs**, not an undocumented reflection into VBE internals.

### 3.8 UserForm designer and Toolbox — C1

**Current allocation: new M2.** Form/class runtime and standard Office control
designer work belong to tiny-vba; host persistence/integration belongs to new
M6/M7. Third-party ActiveX/OLE controls are excluded. See [the scope index](post-m8-roadmap.md).

The UserForm window is a visual design canvas with a grid, drag/resize handles, selection, form properties and event handling [S19]. `View → Toolbox` (or Standard toolbar Toolbox) displays the control palette when a form is active; controls are placed by drawing on the form [S20].

Documented standard control inventory [S20]: **Select Objects**, **Label**, **TextBox**, **ComboBox**, **ListBox**, **CheckBox**, **OptionButton**, **ToggleButton**, **Frame**, **CommandButton**, **TabStrip**, **MultiPage**, **ScrollBar**, **SpinButton**, **Image**.

- `Format` menu performs control alignment, equal spacing, resize, centring, z-order and grouping; **UserForm toolbar** provides convenient equivalents [S21, S22].
- Selecting a control updates the Properties grid. Double-clicking a control opens or creates its default event procedure; the Procedure dropdown shows available events. A command button named `CommandButton1` would normally have a handler such as `CommandButton1_Click` [S17].
- `View → Tab Order` and tab navigation must respect form control order; support grid size/snap settings and designer zoom. The UserForm toolbar documents a **10–400%** zoom range [S21].
- Third-party ActiveX/COM designer support is a **separate compatibility tier**, especially outside Windows. Show unsupported-control placeholders while preserving serialised data; never silently strip the control or associated code.

---

## 4. Full menu specification

**Menu bar: `File | Edit | View | Insert | Format | Debug | Run | Tools | Add-Ins | Window | Help`** [S01].

The following is a **feature-complete command inventory across Microsoft Learn VBE references**, *not a verified byte-for-byte menu resource dump or guaranteed precise visual ordering*. Exact accelerator letters, separators, disabled-item order, host-dependent wording and some shortcuts are subject to **V**. This deliberate distinction avoids inventing or relying on extracted Microsoft UI resources.

### 4.1 File — C0 core, C1 otherwise [S15]

| Command | Behaviour | Context / notes |
|---|---|---|
| `Save <host document>` | Persist host document and contained VBA project. **Ctrl+S**. | C0; must use host save pipeline. |
| `Import File...` | Load a standard/class module or form as a project component. **Ctrl+M** is listed for import in the File reference; see shortcut conflict below. | C0 modules; C1 forms. |
| `Export File...` | Export selected project component. **Ctrl+E**. | C0; selected item required. |
| `Remove <Item>...` | Ask whether to export, then remove item; irreversible by traditional semantics. | C0; project selection required. |
| `Print...` | Print selected source or form. **Ctrl+P**. | C1; printer/platform-dependent. |
| `Close & Return to <host application>` | Hide editor and switch to workbook/document UI. | C0; should **not** unload current workbook. |
| `Close Project` | Close current project with save prompt. | V; generic reference describes it but it is not necessarily exposed in Excel's hosted VBE. |
| `New Project`, `Open Project`, `Make <Project>` | Standalone-project operations. | **Not part of the normal Excel baseline**; only expose if future host supports them. |

### 4.2 Edit — C0 [S23]

| Group | Commands/expected behaviour |
|---|---|
| Undo/redo | `Undo`, `Redo`; Ctrl+Z for undo. Allow a more capable internal undo stack as X, while preserving standard commands. |
| Clipboard/selection | `Cut` Ctrl+X, `Copy` Ctrl+C, `Paste` Ctrl+V, `Delete/Clear`, `Select All`. |
| Find/replace | `Find...` Ctrl+F; `Find Next` and backwards navigation; `Replace...` Ctrl+H. Search in active module or project, as specified by dialog. |
| Indentation | `Indent`, `Outdent`, act on whole selected lines, preserving relative indentation. Reference lists Ctrl+M/Ctrl+Shift+M for these in code context. |
| Code comments | `Comment Block`, `Uncomment Block`; apply apostrophe prefix/removal to selected lines. |
| IntelliSense | `List Properties/Methods` Ctrl+J; `List Constants` Ctrl+Shift+J; `Quick Info` Ctrl+I; `Parameter Info` Ctrl+Shift+I; `Complete Word` Ctrl+Space. |
| Bookmarks | `Bookmarks → Toggle Bookmark`, `Next Bookmark`, `Previous Bookmark`, `Clear All Bookmarks`. |

**Shortcut conflict, V:** Microsoft's generic File menu documents **Ctrl+M for Import File**, while its Edit menu documents **Ctrl+M for Indent** [S15, S23]. These are not an implementation permission to assign both unconditionally. The shortcut resolver must be **focus-sensitive**, and a final Windows Excel baseline check must establish which command wins in each context. Likewise, Microsoft documentation names both **F3** and **Shift+F4** for some Find Next behaviours [S05, S16, S23]. Support well-established aliases without breaking normal navigation; record precise defaults after validation.

### 4.3 View — C0 core [S11]

| Command | Behaviour | Shortcut / context |
|---|---|---|
| `Code` | Open/activate current code module. | **F7** |
| `Object` | Open/activate designer/selected object. | **Shift+F7** |
| `Definition` | Navigate to symbol definition, or Object Browser for referenced symbol. | **Shift+F2** |
| `Last Position` | Navigate back to prior code position. | **Ctrl+Shift+F2**; traditional documented history is limited. |
| `Object Browser` | Library/type/member explorer. | **F2** |
| `Immediate Window` | Show Immediate pane. | **Ctrl+G** |
| `Locals Window` | Show Locals pane. | No universal shortcut asserted. |
| `Watch Window` | Show Watch pane. | No universal shortcut asserted. |
| `Call Stack...` | Show active stack during break mode. | **Ctrl+L** |
| `Project Explorer` | Show project tree. | **Ctrl+R** |
| `Properties Window` | Show Properties pane. | **F4** typical Excel; see F6 discrepancy. |
| `Toolbox` | Show control Toolbox. | Only useful on active UserForm. |
| `Tab Order...` | Open form control tab-order editor. | UserForm context. |
| `Toolbars → Standard / Editor / Debug / UserForm / Customize` | Show/hide or edit toolbars. | Availability contextual. |
| `<Host application>` | Switch to GridCraft/WordCraft window. | **Alt+F11** |

### 4.4 Insert — C0 modules, C1 forms [S24]

`Procedure...`, `UserForm`, `Module`, `Class Module`, `File...` (insert text into active code module), and `Components...` (designer component management, where supported). Contextual `Procedure...` must allow at least `Sub`, `Function`, `Property` selection with relevant visibility options; avoid scaffolding invalid VBA declarations. A newly added module becomes visible in Project Explorer and opens in the editor.

### 4.5 Format — C1, form designer only [S22]

| Submenu | Commands |
|---|---|
| `Align` | `Lefts`, `Centers`, `Rights`, `Tops`, `Middles`, `Bottoms`, `To Grid`. |
| `Center in Form` | `Horizontally`, `Vertically`. |
| `Horizontal Spacing` | `Make Equal`, `Increase`, `Decrease`, `Remove`. |
| `Vertical Spacing` | `Make Equal`, `Increase`, `Decrease`, `Remove`. |
| `Make Same Size` | `Width`, `Height`, `Both`. |
| `Size to Fit`, `Size to Grid` | Resize one or more selected controls. |
| `Order` | `Bring to Front` **Ctrl+J in form designer**, `Send to Back` **Ctrl+K**, `Bring Forward`, `Send Backward`. |
| `Group`, `Ungroup` | Group or ungroup selected form controls. |
| `Arrange Buttons` | `Bottom`, `Right`. |

**Context collision:** `Ctrl+J` is *List Properties/Methods* in the code window but *Bring to Front* in UserForm context. The action-dispatch system must resolve shortcuts by focus/mode.

### 4.6 Debug — C0/C1 [S25]

| Command | Keyboard | Requirement |
|---|---|---|
| `Compile <Project>` | — | Parse/type-check/code-generate, report first or navigable errors. C0. |
| `Step Into` | **F8** | Advance to next statement; enter called VBA procedure. C0. |
| `Step Over` | **Shift+F8** | Execute call without showing intermediate callee statements, then pause at caller's next statement. C0. |
| `Step Out` | **Ctrl+Shift+F8** | Continue until caller's next statement. C0. |
| `Run to Cursor` | **Ctrl+F8** | Run to selected executable statement; scoped transient stop. C1. |
| `Set Next Statement` | **Ctrl+F9** | Change next execution point inside the **same procedure**; must enforce runtime restrictions. C1. |
| `Show Next Statement` | — | Scroll/select currently stopped statement. C0. |
| `Toggle Breakpoint` | **F9** | Set/remove valid statement breakpoint. C0. |
| `Clear All Breakpoints` | **Ctrl+Shift+F9** | Remove project breakpoints. C0. |
| `Add Watch...` | — | Create expression/conditional/change watch. C0 expression, C1 conditions. |
| `Edit Watch...` | **Ctrl+W** | Change selected watch definition. C1. |

Breakpoints are *not* valid on blank lines, comments, or declarations in the reference [S25]. Disable with a reason, rather than silently attaching a breakpoint somewhere else.

### 4.7 Run — C0 [S13]

- `Run Sub/UserForm` / `Run Macro` / `Continue` — **F5**; label and action depend on current context and whether paused.
- `Break` — **Ctrl+Break**; request suspension of running VBA at a safe sequence point.
- `Reset <Project>` — clear execution stack and module-level variables; do not roll back workbook effects.
- `Design Mode` / `Exit Design Mode` — switch project event execution/design behaviour as documented; separate from debugger running/paused state.
- `Run Project` / `Stop Project` — documented only for some stand-alone project environments; **omit from initial Excel host UI**.

### 4.8 Tools — C0/C1 [S26]

`References...` (add/remove typed libraries and project references); `Macros...` (macro chooser); `Options...` (Editor, Editor Format, General, Docking); `<Project Name> Properties...` (name, description, protection where supported); `Digital Signature...` (only when genuinely implemented); `Additional Controls...` (form/ActiveX context); `Quick Watch...` **Shift+F9** (evaluate current selection and optionally add it as a watch).

**MacOS/non-Windows caveat:** Arbitrary COM/ActiveX libraries cannot be presumed portable; clearly label unsupported references and preserve them in project data. Do not display a working “Digital Signature” command unless it can actually sign projects to the requisite format and trust policy.

### 4.9 Add-Ins — C1 [S27]

`Add-In Manager...` to load/unload VBE IDE extensions is a documented menu action. In Craft*, support a **separate, native Craft* extension interface** when available. Do **not** claim that binary VBE add-ins (`VBIDE`/COM plugins) run unmodified on Rust or non-Windows hosts. Show a compatibility note if legacy add-ins are listed in a project but cannot run.

### 4.10 Window — C1 [S12]

`Split`, `Tile Horizontally`, `Tile Vertically`, `Cascade`, `Arrange Icons`, and dynamically populated **open window list**. Split applies to active code view, while tiled/cascaded behaviour applies to opened code/form document windows. Do not use this menu for project tree/utility-pane management (that is `View`).

### 4.11 Help — C0 [S28]

`VBA Help` **F1**: open context-sensitive public/reference documentation for the current symbol; `About Craft* VBA Editor` must show **Craft* identity/version**, licence notices and diagnostic details, not falsely reproduce “About Microsoft Visual Basic for Applications.” F1 should prefer a locally bundled/help-approved index if offline; reference URLs may be presented as links.

### 4.12 Context menus — C0/C1 [S31]

Context menus appear on right click or **Shift+F10**. Make them specific to **code window, project tree, form designer, Properties grid, toolbar, Locals/Watch grid, Object Browser**. Important items include `Definition`, `Close`, `Dockable`, `Delete Watch`, `Collapse Parent`, `Add/Remove`, `View Code/Object`, `Export/Import` and toolbar `Customize`, shown only where applicable. Do not place all global commands in every context menu.

---

## 5. Toolbar and button inventory

The public VBE Help identifies four toolbar families; in Craft* Classic they shall be user-showable/hideable, movable and accessible via ToolTips [S32–S35]. **Icons are independently drawn**, using recognisable functional metaphors (triangle for Run, pause/break, stop/reset, breakpoint dot), never extracted Microsoft artwork.

### 5.1 Standard toolbar — C0

| Buttons/controls in documented inventory | Functional requirement |
|---|---|
| View host application | Switch to workbook/document UI. |
| Insert dropdown | Choose UserForm, Module, Class Module, Procedure; icon may represent last item. |
| Save host document | Save VBA changes with host. |
| Cut, Copy, Paste | Standard focused-editor/designer operations. |
| Find | Search modal/pane. |
| Undo, Redo | Focused editor/designer history. |
| Run Sub/UserForm / Run Macro | **F5** behaviour; button/label context-aware. |
| Break | Pause running program. |
| Reset project | Stop and clear execution state. |
| Design Mode | Toggle project design mode where relevant. |
| Project Explorer | Show project tree. |
| Properties Window | Show property grid. |
| Object Browser | Open browser. |
| Toolbox | Show Toolbox if a form is active. |
| VBA Help | Context help. |

### 5.2 Editor toolbar — C0 [S33]

`List Properties/Methods`, `List Constants`, `Quick Info`, `Parameter Info`, `Complete Word`, `Indent`, `Outdent`, `Toggle Breakpoint`, `Comment Block`, `Uncomment Block`, `Toggle Bookmark`, `Next Bookmark`, `Previous Bookmark`, `Clear All Bookmarks`.

Use the **exact command names** as tooltips/accessibility labels. Disabled buttons should remain aligned to their familiar positions where user preference is to show a full toolbar.

### 5.3 Debug toolbar — C0 [S34]

`Design Mode`, `Run Sub/UserForm or Run Macro`, `Break`, `Reset`, `Toggle Breakpoint`, `Step Into`, `Step Over`, `Step Out`, `Locals Window`, `Immediate Window`, `Watch Window`, `Quick Watch`, `Call Stack`.

Only show `Step Over`/`Step Out` as active when in valid break context. A `Call Stack` button shall display the familiar dialog or its exact functional equivalent; do not just show a passive list with no navigation.

### 5.4 UserForm toolbar — C1 [S35]

`Bring To Front`, `Send To Back`, `Group`, `Ungroup`, `Align` variants, `Center` horizontally/vertically, `Make Same Size` width/height/both, and `Zoom` (documented 10–400%). The full `Format` menu remains available as the command-discovery fallback.

### 5.5 Toolbar customisation — C1

`View → Toolbars → Customize...` should permit show/hide, reordering and creating custom toolbar configurations if fully implemented. **X** may use modern customisation controls, but retain VBE's navigation pathway. Persist toolbar arrangement per user. Avoid duplicate keyboard shortcuts created by plugins without warning.

---

## 6. Code editor behaviour

### 6.1 Source text and VBA lexical rules — C0

The authoritative language baseline is **MS-VBAL** [S29]. Implementation requirements:

- Treat VBA identifiers and keywords according to the language's case-insensitive resolution; preserve user-entered source spelling unless an intentional canonicalisation option applies.
- Recognise keywords, identifiers, members, line labels, numeric/string/date literals, comments (`'` and valid `Rem` contexts), continuation underscores, preprocessor directives, and VBA declaration attributes.
- Support procedure declarations (`Sub`, `Function`, `Property Get/Let/Set`), `Public/Private/Friend`, events, enums, types, constants, `Option Explicit`, `Option Compare`, `Option Base` as supported by the shared parser.
- Render token colours and syntax feedback. The VBE reference supports customising **foreground/background/indicator colours, font and size, and the margin indicator bar** via `Tools → Options → Editor Format` [S10]. Provide a recognisable Classic scheme, independently chosen; exact RGB default colours remain **V**.
- **Auto Syntax Check** validates a completed line and may show a prompt (where enabled), while an unchecked option should still permit unobtrusive error indication. Errors must come from the parser/compiler and identify exact source locations.
- Tab indentation, tab width **1–32**, default **4**, optional **Auto Indent**, optional **Require Variable Declaration** for newly created modules, and `Option Explicit` insertion are configurable [S10].
- Support both **Procedure View** and **Full Module View**, optional separators, code-window horizontal split, current-line/gutter indicators and bookmarks [S05, S10].
- Native Ctrl+Home/End, Ctrl+arrow, Home/End, Ctrl+Up/Down procedure navigation and Find/Replace keys must remain natural [S05, S16].
- Text edits must reliably update compiler diagnostics, navigation, completions and debugger source locations without clobbering the user's cursor position or other modules.

**Source-of-truth invariant:** UI coloration is a *presentation of the same parser/tokenisation* used by compiler/IR and object-browser metadata, not a separate incompatible “JavaScript-like syntax highlighter.” Incremental parsing can drive fast live feedback without executing code.

### 6.2 Editing semantics — C0

- Edit source modules in UTF-compatible internal representation but honour recorded workbook/project code-page and persisted VBA format requirements when reading/writing [S30].
- Standard selection, multi-line indent/outdent, Cut/Copy/Paste, undo/redo, find/repeat-find, replace, drag/drop, and comment/uncomment.
- Undo groups should be predictable: typing and programmatic inserts are separate transactions. An AI suggestion, rename or quick fix must be **one undoable unit**.
- A source edit to a paused procedure may invalidate runtime frames/sequence points; the debugger contract (section 8) decides whether it is safe to continue or requires Reset. **Never** silently resume stale machine code.
- Auto-insertion of syntactic closing forms and auto-formatting that diverge from old VBE should be **X**, switched off in strict Classic mode unless validated as familiar behaviour.
- For `File → Save`, edits are saved **inside the host workbook/document**, not only to an independent `.bas` cache. Autosave and user-visible unsaved state must be reconciled with GridCraft/WordCraft save semantics.

### 6.3 Excel-specific source navigation — C0

- Standard modules: `(General)` at left, named procedures at right.
- Worksheets: `(General)` and `Worksheet` / documented host object selector; event choices include `Change`, `SelectionChange`, `Calculate`, `Activate`, `BeforeDoubleClick`, etc. [S36].
- `ThisWorkbook`: choose the workbook event set based on available object-model metadata; handler declarations must carry accurate signatures.
- UserForms and controls (C1): event choices, e.g. `Click` and `Initialize`, with correctly created handlers [S17].
- `View → Definition` and Object Browser navigation must work on **project symbols and host object-model symbols**, not merely text-match occurrences.
- Repeatedly selecting an existing event **must not insert duplicate procedures**.

### 6.4 Search and bookmarks — C0

Support Find/Replace dialog scope to current procedure, current module or current project where exposed, case/whole-word and direction as implemented. Result selection must preserve module identity and navigate to the right source location. Bookmarks attach to stable source locations through edits and module reopen, and are not persisted as executable VBA code.

---

## 7. Completion, help, navigation, and other editor aids

### 7.1 Classic VBE IntelliSense — **mandatory C0**

This is the native VBE experience, **not Copilot or AI generation** [S23, S33, S10]:

| Feature | Trigger | Behaviour | Backend information |
|---|---|---|---|
| **Auto List Members** | After `object.` (if option enabled), or **Ctrl+J**. | Popup with applicable properties/methods; type to narrow/highlight, arrows to navigate; **Tab** or **Enter** inserts selected item; Esc dismisses. | Semantic type of receiver, visibility, host object-model metadata. |
| **List Constants** | Property assignment/enum argument or **Ctrl+Shift+J**. | Suggest valid constant values and insert selected constant. | Enum and argument signatures, property value domains. |
| **Complete Word** | **Ctrl+Space**. | Complete uniquely identifiable symbol/keyword/name; fall back to a list on ambiguity. | Scope and symbol index. |
| **Auto Quick Info** | After typing a function/procedure/statement or **Ctrl+I**. | Syntax/signature tooltip with relevant parameter emphasised. | Signature table and parsed call expression. |
| **Parameter Info** | On argument entry or **Ctrl+Shift+I**. | Inline tooltip; current parameter highlighted; Esc dismisses. | Nested call/argument position analysis. |
| **Auto Data Tips** | Hover identifier in **break mode**. | Show real current runtime value; never show guessed value when unavailable. | Selected debugger stack frame/safe evaluator. |
| **Definition** | **Shift+F2**. | Jump to user symbol or selected referenced-library definition in Object Browser. | Typed symbol reference, declaration spans. |
| **Context-sensitive Help** | **F1**. | Open help for token/member/event. | Public documentation lookup, offline cache. |

**Matching the older behaviour matters:** VBE documentation explicitly notes that a late-bound `Variant` object does not necessarily offer a member list after a dot [S23]. In Craft* **Classic mode**, if the type is unknown, the editor must not misleadingly claim guaranteed members. An **X smart-completion mode** may offer speculative late-bound suggestions, marked clearly as such.

### 7.2 Completion popup interaction contract

1. Show close to caret, reposition to remain visible, keep code editor keyboard focus and screen-reader announcements coherent.
2. Prioritise exact/prefix matches, then public/visible symbols; preserve VBA case-insensitive matching.
3. Show names and meaningful independently designed icons distinguishing property/method/event/constant. Do **not** use unlicensed Microsoft VBE glyph assets.
4. Never suggest a member inaccessible because of VBA visibility, type or current context; flag a known-but-unimplemented GridCraft API instead of inventing success.
5. `Tab` inserts without forced newline; `Enter` matches documented popup insertion behaviour (in some classic descriptions it moves to the next line). Make the exact effect a **V** target-specific behaviour verified with hands-on testing [S23].
6. Escape always closes, punctuation boundaries select/cancel predictably, and accepting a completion is an undoable edit.
7. **Latency design target (Craft* proposal):** locally indexed common completions under 50 ms at p95, foreground semantic requests under 150 ms at p95 on a typical desktop. These are engineering targets, **not documented VBE metrics**.

### 7.3 Options dialog — required fields C0 [S10]

**Tools → Options → Editor:**

- `Auto Syntax Check`
- `Require Variable Declaration`
- `Auto List Members`
- `Auto Quick Info`
- `Auto Data Tips`
- `Auto Indent`
- `Tab Width` (1–32; 4 default)
- `Drag-and-Drop Text Editing`
- `Default to Full Module View`
- `Procedure Separator`

**Tools → Options → Editor Format:** category colours foreground/background/indicator; font and size; show/hide margin indicator. Ensure user-chosen break/current-line colours retain contrast.

**Tools → Options → General:** UserForm grid visibility and cell sizes; align-to-grid; tooltips; collapse-project-hides-windows; warn before state loss; error trapping options; compile options.

**Tools → Options → Docking:** per-pane docking toggles; persist panel state.

**General-tab debugger modes:** `Break on All Errors`, `Break in Class Module`, `Break on Unhandled Errors`. `Compile On Demand` and `Background Compile` are documented reference options. If Craft* compiles ahead-of-time or incrementally rather than exactly matching an older VBE compiler, expose equivalent behaviour only when the observable semantics are supported; otherwise mark legacy choices as unavailable instead of promising invisible parity.

### 7.4 Diagnostics and editor hints

Errors include missing symbols, wrong event signatures, wrong argument types, invalid syntax, duplicate declarations, missing `End Sub`, etc. An error panel, inline underlines, code actions and a problem count are **X**; in Classic mode, the first-error navigation/compile message and colour conventions are primary. Diagnostic rules must come from language semantics and current host metadata, not an AI guess.

---

## 8. Debugger user experience and semantics

### 8.1 Launching and stopping — C0

**Example:** A user opens `Budget.xlsm`, presses **Alt+F11**, selects `Module1`, adds a breakpoint on an executable line, presses **F5**, and reaches break mode. The relevant module opens/activates automatically, the current statement is indicated in the margin, and the user can inspect the Immediate, Locals and Watch windows. `F8` advances by statement; `Shift+F8` executes a called procedure without entering it; `F5` continues [S13, S25, S34].

**Execution events to support:** running `Sub`s and `Function`s, workbook and worksheet events, UserForm events (C1), `Debug.Print`, runtime/compile errors, and macro invocation from the host UI. Breakpoints set in a worksheet handler must work **when GridCraft actually raises the event**.

### 8.2 Breakpoint visuals and valid locations — C0

- Clicking indicator margin or pressing **F9** sets/removes a breakpoint on an executable statement. The reference states the source line's colours come from the Editor Format settings [S25].
- Render a visually distinct **breakpoint marker** and a **current/next executable statement marker**; preserve visibility for colour-blind users via shape/icon and accessible labels as well as colour.
- Support several logical statements on a physical line, continuation lines, and procedure declarations by mapping runtime sequence points to source spans; users reason about **VBA statements**, not IR basic blocks.
- Clear all project breakpoints with **Ctrl+Shift+F9**, without resetting user bookmarks.
- Track breakpoints through edits using syntax/node identity where possible; otherwise invalidate with a visible notification.
- **X** conditional-on-expression *line* breakpoints may be added as a convenient shortcut, but the classic conditional-stop mechanism is **Add Watch → Break When...**.

### 8.3 Debugging commands and stepping — C0/C1

| Action | Runtime requirement | UI result |
|---|---|---|
| Step Into | Execute to next statement, descending into call when possible. | Highlight callee statement; stack grows. |
| Step Over | Run current statement, including called procedures, until caller next statement or fault/breakpoint. | Highlight caller successor. |
| Step Out | Run until current frame returns. | Highlight caller successor. |
| Break | Cooperatively suspend current execution at an interruptible point. | Show break mode; focused code line. |
| Continue | Resume from current point until completion/stop. | Resume host and code. |
| Reset | Destroy execution frames and module-level runtime variables; preserve host side effects. | Editor returns to design/editing mode. |
| Run to Cursor | Set temporary stop at selected executable location. | Pauses at selected statement if reached. |
| Set Next Statement | Modify instruction pointer **only within current procedure** and only where valid. | Update next-statement indicator without executing skipped code. |
| Show Next Statement | Navigate to active stop. | Scroll and select correct source span. |
| Compile Project | Compile and validate before execution. | Report source-associated errors. |

**Set Next Statement** is deceptively difficult: moving backwards must not re-run declarations or cause unsound stack state; moving across lexical scopes/error-handling regions can be invalid. The public UI documentation allows moving forward/backward in the **same procedure** [S25]; Craft* must enforce *additional runtime-safe constraints where necessary*, displaying why a requested jump is disallowed. Do not claim complete equivalence before compiler/runtime can guarantee it.

### 8.4 Call Stack — C0

`View → Call Stack...` **Ctrl+L** shows active procedure calls, including those initiated by Immediate when relevant [S11, S37]. A listed item identifies project/module/procedure; `Show` navigates to the call site and highlights a call-stack indicator. Stack-frame selection must change Locals, Immediate evaluation context and hover Data Tips consistently. Ignore internal Rust/IR frames in the user-visible stack unless a Craft* developer diagnostics mode is enabled.

### 8.5 Variable inspection, modification and watches — C0/C1

- Locals refresh on pause/step; types and values are accurate VBA runtime values, not just Rust debug-format strings.
- Direct editing of writable variables in Locals/Watch/Immediate changes the live context, validating VBA type/coercion semantics.
- Quick Watch (**Shift+F9**) evaluates selected expression in the current context and offers **Add** to create a persistent watch [S26].
- Conditional `Break When Value Is True` and `Break When Value Changes` require runtime observation of expressions after relevant execution steps [S18]. No watch predicate may run in a separate context and report a value that has never existed in the user's thread.
- To avoid side effects and lockups, document a **safe evaluation policy**: simple variables/pure member inspection first; potentially side-effecting function calls require user action/consent or explicit policy, especially if watch conditions are evaluated repeatedly. This is an implementation safeguard; precise historic VBE expression evaluation is **V**.
- Arrays, `Variant`, objects, Nothing, ByRef aliases, default members and scope transitions must match the VBA type system and host object model.

### 8.6 Errors and break-mode policy — C0

The **General** options determine whether errors stop at every occurrence, stop at class-module errors, or stop only when unhandled [S10]. Respect `On Error GoTo`, `On Error Resume Next`, `Resume`, `Err` state, caller unwinding and appropriate handler transitions as the runtime supports them.

- A runtime error dialog should provide an understandable VBA error number/message, **Debug**, **End/Reset**, and **Help** actions as appropriate to the reference behaviour; exact dialog button permutations vary by event/host and are **V**.
- Debug jumps to the statement responsible when permitted; choosing End discards VBA execution state but **must not revert workbook edits**.
- Break during a host event preserves enough state to continue safely; prevent accidental reentrancy if the user edits the workbook while paused.
- If a project is reset or modified so continuing is unsafe, warn about state loss where the `Notify Before State Loss` setting demands it.

### 8.7 Source mappings from Rust IR — core architecture dependency

The editor/debugger must attach to a **debuggable VBA execution engine**, rather than interpreting a list of GUI commands. Provide:

```text
Workbook/xlsm ↔ project loader ↔ editable VBA source + component model
                                  │
                             parser/symbols
                                  │
                           high-level VBA IR
                                  │   source spans + sequence points
                            executable IR / VM
                                  │
                 ┌────────────────┴───────────────────┐
                 │ Debugger API                       │ Host API adapter
                 │ frames, step, breakpoint, eval    │ GridCraft/WordCraft
                 └────────────────┬───────────────────┘
                                  │
                      Craft* VBE-compatible UI
```

Each executable IR statement needs: `project_id`, `module_id`, stable `source_span`, `procedure_id`, `statement_id`, `scope_id`, optional `event_source`, and an execution **sequence-point** mapping. Breakpoints attach to sequence points, not arbitrary text offsets. Make semantic symbol/reference information reusable across editor completions, Object Browser, compilation and debugger evaluation. This also helps the project's longer-term goal of turning well-preserved high-level IR into readable **Python**: keep source metadata and original VBA constructs intact instead of discarding them during lowering.

### 8.8 Runtime/host state safety

- Macro runtime must coordinate synchronous host method calls and worksheet/workbook event dispatch with the UI event loop without freezing or corrupting projects.
- Stepping calls that change a cell should expose those changes in GridCraft while preserving debuggable call stack and event context.
- Handle recursive events, disabled event flags, user cancellation and Reset explicitly. Avoid executing an untrusted workbook macro just because the user opened its project for inspection.
- External effects (files, network, COM/ActiveX, clipboard) require a permission model; consider a **restricted macro profile** for the initial Craft* release. Explain deviations from Excel macro execution; never misrepresent a restricted runtime as full Excel equivalence.

---

## 9. Optional Copilot-style improvements — X, not historic VBE

**Removed from current scope.** The following AI/extension proposal is retained
as historical reference only; it is not scheduled work. See [the scope index](post-m8-roadmap.md).

**Key distinction:** Traditional VBE documented **IntelliSense** is deterministic local completion/parameter help [S10, S23, S33]. Copilot-style *multi-token prediction, natural-language code authoring and chat* are **new Craft* features**; they should not be portrayed as controls that exist in the legacy VBA editor. The Microsoft Learn UI inventory does not establish a built-in Copilot interface for classic VBE.

### 9.1 Operating modes

| Mode | Default for transition users? | Behaviour |
|---|---|---|
| **Classic / VBE Compatibility** | **Yes** | Familiar menus/toolbar/shortcuts and editor aids; no ghost-text AI or intrusive side pane by default. |
| **Enhanced Editor** | Optional | Same VBA commands; improved search, diagnostics, refactoring, accessible layout and modern indexing. |
| **AI-assisted** | Explicit opt-in | Inline predictions, ask/explain/fix/test assistant, controlled use of locally available code and approved docs. |

Mode switching must not alter the workbook's VBA source semantics or hide existing actions.

### 9.2 AI inline suggestions — X

**Optional interaction design:** a muted “ghost text” completion beyond the caret; **Tab** accepts if *no classic completion menu* is open, **Esc** dismisses, further typing cancels/stales the suggestion, and a deliberate shortcut opens detailed alternatives. Show clear `AI suggestion` labelling; use a distinct presentation from local IntelliSense. Disable suggestions in comments/strings when context inappropriate. A generated multi-line block previews exactly what will be inserted; accepting becomes one undo action.

**Priority order:** Classic completion popup wins over ghost text; menu shortcuts and editor keystrokes remain authoritative. Never steal **F5/F8/F9**, Ctrl+J, Ctrl+G or existing VBE keyboard semantics for AI.

### 9.3 AI side pane — X

An optional, closeable pane might offer **Explain this procedure**, **Find references**, **Suggest VBA fix**, **Write macro from description**, **Generate unit-test cases**, and **Convert to Python (preview)**. Suggestions must be shown as a **diff** before being applied. Generated code is never silently executed. A `Why?` affordance should cite the user-supplied code and *approved public documentation* for claimed API usage.

### 9.4 Privacy, licences and tainted-source controls

- **Off by default.** Distinguish local inference from remote API use, clearly indicating what source snippets/workbook metadata will leave the user's machine.
- Never upload private workbook data, credentials or customer information without specific permission; allow workspace policy to prohibit external requests.
- Treat **source comments, worksheet cell values, document text, tooltips and filenames as untrusted content**: they can carry prompt-injection instructions. An AI tool may read them as data, but must not execute instructions found inside them.
- Approve and pin the retrieval corpus (public MS-VBAL, MS-OVBA and public host API references plus code the user authorised). Maintain a **provenance/audit record** for generated suggestions. Exclude leaked or licence-questionable repos and scraped copyrighted tutorials.
- Do not promise that any general-purpose model's training history is provably free of copyrighted or non-public code. Use deterministic clean-room authored requirements and human review for compatibility-critical implementation; enforce source restrictions on all *retrieved/tool-supplied* material.
- Prevent AI from invoking debugger execution, file write, macro run, network access, or conversion/export without an explicit user action and valid policy. The assistant is an editing aid, not an implicit macro execution authority.

---

## 10. GridCraft/WordCraft integration

### 10.1 Entry and exit points — C0

**GridCraft:** Add `Developer → Visual Basic` in the host UI (with a route for users who haven't enabled the Developer tab), and global **Alt+F11** toggle between host and VBE-like editor. `Developer → Macros` should open a macro chooser supporting Run/Edit where implemented. `File → Save` from the VBE-like UI saves the **workbook and its macro project** [S02, S15].

**WordCraft:** Same editor/debugger shell, with Word document object/host models replacing Excel sheets/workbooks and Word-specific events and module hierarchies. Avoid duplicating the editor codebase by host. The VBE title/host selector and exposed host object library change, not the core editor.

### 10.2 Loading macro-enabled documents

- Recognise workbook/document formats that can contain VBA project storage (e.g. Excel `.xlsm`/`.xlsb`, template cases when supported); use the documented **MS-OVBA** structure and host file-format specifications [S30].
- Read project metadata, standard/class modules, object modules, references, attributes, form designers, protection/signature metadata where available.
- Preserve **unrecognised or unsupported payloads** through round-trip where technically and legally possible; explicit warning before any lossy save. A “can open” claim is not the same as safe macro execution compatibility.
- If editing invalidates a digital signature, warn clearly and avoid suggesting that a previous signature remains valid. Do not invent signatures on save.
- Distinguish **view/edit macro source** from **enable/execute macro**. The user must be aware of macro trust status and project protection. Do not auto-run `Workbook_Open` as a side effect of inspecting the document unless that execution has been authorised under the host's documented security policy.
- Never bypass a protected project or recommend bypassing protection. If source is unavailable, show a protected-project state and preserve document content safely.

### 10.3 Host object-model and editor features

Create a shared **symbol/type catalogue** (`Workbook`, `Worksheet`, `Range`, `Application`, etc.) from publicly documented Excel APIs and the Craft* host adapter. This must power *all* of the following: editor type-ahead, parameter info, Object Browser, compiler binding, runtime dispatch, debugger property inspection, Help links, and AI validation. One catalogue prevents the common failure mode where an API autocompletes but cannot execute.

**Event flow example:**

```text
GridCraft cell edit
  → workbook host event router
  → Worksheet_Change handler in VBA VM
  → debugger sequence point / breakpoint
  → VBE Code window highlights current source
  → Locals / Watch / Immediate inspect same live frame
  → Continue returns to host with correct event lifecycle
```

### 10.4 Host divergence and unsupported features

Do not include standalone `Run Project` or arbitrary COM designer commands simply because they appear in generic VB reference pages. Do not imply that ActiveX, binary add-ins or Windows DLLs run on Linux/macOS. Unavailable features should have consistent notices, safe retention of unknown project data, and a planned implementation tier. For a portability-first Rust codebase, **C0 editor/debugger fidelity can precede C1 UserForm/COM fidelity**.

---

## 11. Rust-facing UI contracts

This is a **proposal** for integration, not a Microsoft API. Use Rust trait interfaces and structured internal events in-process where GridCraft architecture allows it; avoid binding the UI directly to VM internals or forcing a JSON-RPC transport for the primary embedded editor.

```rust
pub trait VbaWorkspace {
    fn projects(&self) -> Vec<ProjectSummary>;
    fn components(&self, project: ProjectId) -> Vec<ComponentSummary>;
    fn source(&self, component: ComponentId) -> SourceSnapshot;
    fn apply_edits(&mut self, component: ComponentId, edits: Vec<TextEdit>)
        -> Result<SourceRevision, WorkspaceError>;
    fn save_host_document(&mut self) -> Result<(), WorkspaceError>;
}

pub trait VbaLanguageService {
    fn parse(&self, source: SourceSnapshot) -> ParseResult;
    fn complete(&self, location: SourceLocation) -> Vec<CompletionItem>;
    fn signature_help(&self, location: SourceLocation) -> Option<SignatureHelp>;
    fn definition(&self, location: SourceLocation) -> Option<SymbolLocation>;
    fn diagnostics(&self, project: ProjectId) -> Vec<Diagnostic>;
    fn symbols(&self, scope: SymbolScope) -> Vec<SymbolInfo>;
}

pub trait VbaDebugSession {
    fn state(&self) -> DebugState;
    fn set_breakpoint(&mut self, site: SequencePointId) -> Result<(), DebugError>;
    fn step(&mut self, kind: StepKind) -> Result<(), DebugError>;
    fn resume(&mut self) -> Result<(), DebugError>;
    fn request_break(&mut self) -> Result<(), DebugError>;
    fn reset(&mut self) -> Result<(), DebugError>;
    fn stack_frames(&self) -> Vec<FrameInfo>;
    fn locals(&self, frame: FrameId) -> Vec<VariableInfo>;
    fn evaluate(&mut self, frame: FrameId, expression: &str, policy: EvalPolicy)
        -> Result<VbaValue, EvalError>;
    fn assign(&mut self, target: VariableRef, value: VbaValue)
        -> Result<(), DebugError>;
}
```

These signatures omit asynchronous dispatch/lifetimes/IDs for readability; the real crate contract should address thread affinity, source revisions, cancellation, debugger event streaming and GUI reentrancy. Suggested crates/components (names illustrative):

```text
craft-vba-ui/                 # host-agnostic VBE-compatible window/panes/commands
craft-vba-editor/             # text buffer, layout, completion popup, diagnostics
craft-vba-project/            # project and component model; import/export, file format
craft-vba-language-service/   # parser and symbol/type queries
craft-vba-debug-protocol/     # runtime-neutral frames/sequence points/step requests
craft-vba-runtime/            # VM/interpreter/IR integration (separate)
craft-host-gridcraft/         # Excel API + macro events + save bridge
craft-host-wordcraft/         # Word API + macro events + save bridge
craft-vba-ai/                 # OPTIONAL separate feature; off by default
```

All commands flow through a **single command registry**, allowing menu, toolbar, shortcut and context menu to share enable/disable logic. For example:

```text
command_id: debug.step_over
label: Step Over
shortcut: Shift+F8
visible_when: code_editor_active
enabled_when: debugger_state == Break && valid_execution_frame
handler: VbaDebugSession.step(Over)
source: S25
```

Store UI descriptions declaratively with **provenance/source IDs**, priority tier and host-context predicates. This is especially helpful because `Ctrl+J`, `Ctrl+M` and properties-panel shortcuts are context-dependent or conflict between generic references. A formal command registry prevents accidental global shortcut collisions.

---

## 12. Accessibility, configuration, and polish

### 12.1 Native keyboard interaction

- Every menu and pane is operable by keyboard; focus order, menu navigation and accelerator mnemonics must be testable.
- Keyboard layout and host OS may reserve shortcuts (especially on macOS). Windows Classic keymap is the canonical baseline; alternate platform map may be configured with an on-screen explanation.
- Provide accessible names for toolbar buttons, selected tree items, debugger stops and error banners. Reading order must make Project Explorer and open code discoverable.
- Do not use colour alone to communicate breakpoint, current statement, error or unsupported API.

### 12.2 Persisted user preferences

Per user/host: pane placement/visibility, window split, editor colours/font, tab width, code view, auto-list/help toggles, search history as appropriate, toolbar visibility/order, debugger watches/bookmarks as safely supported, AI opt-in and permissions. Project source modifications and bookmarks need separate persistence decisions. Avoid embedding UI state into `vbaProject.bin` unless clearly required by the documented format.

### 12.3 Performance and robustness targets (Craft* proposals)

- Open editor and restore panes without blockingly compiling every library.
- Incremental editor updates must remain responsive on modules containing thousands of lines.
- A slow host object-model call or AI suggestion must not block editor caret or UI input.
- Evaluation of a broken `Watch` expression must not crash runtime/editor.
- Source- and host-model integrity matters more than pixel-perfect likeness or animation polish.

---

## 13. Development roadmap and acceptance tests

### Repository milestone allocation (2026-10-27)

The completed M8 repository gate covers the baseline in [its acceptance record](m8-acceptance.md).
Current remaining allocations are in the [scope index](post-m8-roadmap.md) and
[implementation plan](../implementation-plan.md). Their new M1–M8 numbering
supersedes the original sequence. The P0–P8 phases below are historical
product-requirement groupings; removed extensions/AI and third-party controls
must not be treated as outstanding acceptance requirements.

### 13.1 Ordered delivery plan

| Phase | Deliverable | Completion gate |
|---|---|---|
| **P0 — Baseline capture/provenance** | Record target Excel/VBE build, legal acquisition/usage, public-source matrix, command/shortcut decisions and independently designed icon set. | Source review passes; V items prioritised; no unlicensed assets. |
| **P1 — Shell** | Menu bar, Standard toolbar, Project Explorer, Properties, code windows, window switching and Alt+F11 host toggle. | Excel user can find/open/add/view modules without tutorial. |
| **P2 — Editing** | Text buffer, code navigation, split and procedure views, syntax colour, find/replace, imports/exports, saves. | Modules reliably round-trip; keyboard tests pass. |
| **P3 — Classic code aids** | Scope-aware List Members/Constants, Complete Word, Quick/Parameter Info, definition, Object Browser, editor options. | Completion matches real type/metadata; no invented APIs. |
| **P4 — Debugger C0** | Compiler diagnostics, run/break/reset, F8/Shift+F8, F9, execution marker, Locals, Immediate, Quick Watch, stack navigation. | Real GridCraft macro and worksheet event can be stepped. |
| **P5 — Advanced debugging** | Step Out, Run to Cursor, Set/Show Next Statement, conditional watch expressions, live assignment, error-mode options. | Verified handling of scope, side effects and state loss. |
| **P6 — UserForms and extended VBE** | Toolbox, designer, Format, property grid, events, form import/export and Win-specific control warning. | Form can be designed, saved, reopened and its events debugged. |
| **P7 — WordCraft/generalisation** | Host-independent editor/VM debugger, Word object library and event wiring. | Same shell works with second host without copying UI implementation. |
| **P8 — Optional enhancements** | Improved diagnostics and AI inline/chat, Python translation view and privacy policy controls. | No regression in Classic mode; security tests pass. |

### 13.2 Mandatory user acceptance scenarios

**Editor workflows (C0):**

1. Open `Budget.xlsm` in GridCraft; press **Alt+F11**. Project Explorer lists correct workbook, sheets, ThisWorkbook and modules.
2. Press **Ctrl+R**; select a module; press **F7**; edit and save using **Ctrl+S**; reopen without losing macro source.
3. `Insert → Module` creates a named component. `File → Export File` produces a valid module. Import restores expected source.
4. Open a worksheet object; select a `Worksheet` event from the code dropdown; correctly signed event handler is created once and is navigable via right selector.
5. Enter `Application.` or a properly typed `Worksheet.` expression; member completion lists only type-appropriate entries; **Ctrl+J** invokes list, **Ctrl+Space** completes keyword/name.
6. Function call Quick Info and Parameter Info show the correct parameter and switch highlighted parameter as commas are entered.
7. **F2** opens Object Browser with searchable public host API types; selecting a member shows a real signature, definition/help navigation works.
8. Toggle Editor Options; auto list/quick info/auto syntax options visibly change behaviour. F4 shows Properties and does not trigger an unrelated action.

**Debugger workflows (C0):**

9. On an executable VBA line press **F9**, then **F5**; pause at correct line. F9 on a comment is refused or ignored with clear feedback, not relocated invisibly.
10. **F8** enters a called Sub; **Shift+F8** steps over a call; **Ctrl+Shift+F8** returns to caller; verify stack frame and next-source selection.
11. In break mode **Ctrl+G** opens Immediate; `? counter` outputs current value; `counter = counter + 1` changes a writable local and is reflected by Locals.
12. Add a Watch expression; its value refreshes on steps and becomes visibly out-of-scope when appropriate. Test `Break When Value Changes` and `Break When Value Is True`.
13. **Shift+F9** Quick Watch shows a real selected variable; **Ctrl+L** shows stack and Show action navigates correctly.
14. **Ctrl+F8** Run to Cursor pauses at selected statement; **Ctrl+F9** Set Next Statement modifies instruction pointer only when safe; invalid cross-procedure jumps are rejected.
15. Raise a handled and unhandled runtime error under all three General error trapping choices; break and resume semantics align with the VM/host model.
16. After macro mutates a worksheet, Reset stops execution but **does not undo** the worksheet change.
17. Change a cell in GridCraft to trigger `Worksheet_Change`; breakpoint in its VBA handler stops in the editor; Resume completes event without double invocation.

**Host/storage/security (C0):**

18. A project that contains unknown form/ActiveX content is preserved or the user receives an explicit loss warning before save.
19. Opening a file with macros does not run code solely because the editor examines source; protected projects remain protected.
20. Editing signed VBA warns if changes invalidate signature; Craft* never silently advertises a signature as valid.
21. Offline and without AI, all Classic IDE keyboard commands, completion and debugger features continue to function.
22. Copy an Excel-trained user's workflow: **Developer → Visual Basic → Project Explorer → double click Module1 → F9 → F5 → Ctrl+G → F8 → Alt+F11**. It should be recognisable throughout.

**Extended/AI (C1/X):**

23. Design a UserForm with Label, TextBox and CommandButton, set properties, use Format Align and tab order, generate Click event, run/debug and round-trip.
24. AI suggestion is off initially; when enabled, a classic Ctrl+J popup takes precedence over a proposed ghost-text completion.
25. Paste malicious “AI instructions” into a workbook cell/comment; AI treats them as untrusted data, does not exfiltrate source, and cannot execute a macro without authorisation.
26. Examine provenance artefacts for code/icons/reference docs. No proprietary resource bitmaps, decompiled code or unapproved sources are included in distributable outputs.

### 13.3 Test fixtures and automated checks

Create **independently authored** `.xlsm`/macro fixtures covering module types, sheet/workbook events, nested calls, ByRef/Variant/object variables, watch conditions, `On Error` paths, breakpoints near multi-line statements and save/restore. Use Excel as a **black-box behavioural oracle** only with lawful access. Tests should assert observable commands, state transitions and resulting values. Keep screenshot-based regression tests on **Craft* original renderings**, not stored proprietary Windows/VBE artwork. Accessibility keyboard automation and GUI smoke tests should run on every release.

---

## 14. Public sources and traceability

**Method:** Prioritise **Microsoft's publicly published VBE UI/reference material** over hobby screenshots or unsourced summaries. The Microsoft Press sample [S02] is secondary corroboration of the typical arrangement and entry workflow, not a source of redistributed image assets. Some Microsoft UI pages describe older Visual Basic editor versions or stand-alone hosts: feature applicability is explicitly constrained above. URLs consulted 27 October 2026. Microsoft Learn pages may show authorisation banners yet still render substantive public text; maintain URL/date snapshots or manually verify accessibility before publication.

| ID | Public source (publisher) | Main facts derived |
|---|---|---|
| S01 | [Menus and commands](https://learn.microsoft.com/en-us/office/vba/language/reference/menus-commands) — Microsoft Learn | Eleven VBE menus, menu/context-menu model. |
| S02 | [Understanding the VB Editor](https://www.microsoftpressstore.com/articles/article.aspx?p=2952388&seqNum=7) — Microsoft Press Store/Pearson | Excel entry via Alt+F11/Developer and conventional tree/properties/code screen arrangement. |
| S03 | [Project Explorer](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/project-explorer) — Microsoft Learn | Project tree, 3 buttons, keyboard navigation. |
| S04 | [Use the Properties window](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/use-the-properties-window) — Microsoft Learn | Two property modes, editing, F4; some docs differ on shortcut. |
| S05 | [Code window](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/code-window) — Microsoft Learn | Object/Procedure selectors, split bar, gutter, procedure/full view, navigation keys. |
| S06 | [Immediate window](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/immediate-window) — Microsoft Learn | Interactive statement execution and paused-scope evaluation. |
| S07 | [Locals window](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/locals-window) — Microsoft Learn | Expression/Value/Type, editing and expansion. |
| S08 | [Watch window](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/watch-window) — Microsoft Learn | Expression/Value/Type/Context and scope handling. |
| S09 | [Object Browser](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/object-browser) — Microsoft Learn | Browser structure, search, classes, members, definitions. |
| S10 | [Set Visual Basic environment options](https://learn.microsoft.com/en-us/office/vba/language/how-to/set-visual-basic-environment-options) — Microsoft Learn | Editor/Format/General/Docking settings, error trapping. |
| S11 | [View menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/view-menu) — Microsoft Learn | Pane command catalogue, shortcuts and toolbar groups. |
| S12 | [Window menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/window-menu-commands) — Microsoft Learn | Split, Cascade, Tile, Arrange Icons. |
| S13 | [Run menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/run-menu) — Microsoft Learn | Run/Continue, Break, Reset, Design Mode, standalone host exclusions. |
| S14 | [Debug toolbar](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/debug-toolbar) — Microsoft Learn | Debug actions and utility-window toolbar buttons. |
| S15 | [File menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/file-menu) — Microsoft Learn | Save, Import, Export, Remove, close/print, standalone exceptions. |
| S16 | [Code window menu shortcut keys](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/menu-shortcut-keys-available-in-the-code-window) — Microsoft Learn | Common VBE shortcuts; note conflicting context documentation. |
| S17 | [Control and dialog box events](https://learn.microsoft.com/en-us/office/vba/excel/concepts/events-worksheetfunctions-shapes/control-and-dialog-box-events) — Microsoft Learn | Handler naming and event selector operations. |
| S18 | [Add Watch dialog box](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/add-watch-dialog-box) — Microsoft Learn | Watch context and three watch types. |
| S19 | [UserForm window](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/userform-window) — Microsoft Learn | Canvas behaviour and shortcuts. |
| S20 | [Toolbox](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/toolbox) — Microsoft Learn | Standard form control list. |
| S21 | [UserForm toolbar](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/userform-command-bar) — Microsoft Learn | Alignment, grouping, zoom. |
| S22 | [Format menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/format-menu) — Microsoft Learn | Designer control arrangements. |
| S23 | [Edit menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/edit-menu) — Microsoft Learn | Editing actions, bookmarks and native IntelliSense. |
| S24 | [Insert menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/insert-menu) — Microsoft Learn | Module, class, UserForm, procedure and file insertion. |
| S25 | [Debug menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/debug-menu) — Microsoft Learn | Breakpoints and step/next commands. |
| S26 | [Tools menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/tools-menu) — Microsoft Learn | References, Options, Quick Watch, signatures. |
| S27 | [Add-Ins menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/add-ins-menu) — Microsoft Learn | Add-In Manager. |
| S28 | [Help menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/help-menu) — Microsoft Learn | F1/Help and About. |
| S29 | [[MS-VBAL] VBA Language Specification](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/d5418146-0bd2-45eb-9c7a-fd9502722c74) — Microsoft Open Specifications | Public source for parser/type/runtime compatibility. |
| S30 | [[MS-OVBA] Office VBA File Format Structure](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/575462ba-bf67-4190-9fac-c275523c75fc) — Microsoft Open Specifications | Public structure for embedded VBA projects. |
| S31 | [Shortcut menu](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/shortcut-menu) — Microsoft Learn | Context actions, Dockable, Definition and watch actions. |
| S32 | [Standard toolbar](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/standard-toolbar) — Microsoft Learn | Standard buttons. |
| S33 | [Editor toolbar](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/editor-toolbar) — Microsoft Learn | Editing, IntelliSense and bookmarks toolbar. |
| S34 | [Debug toolbar](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/debug-toolbar) — Microsoft Learn | Debug toolbar and panes. |
| S35 | [UserForm toolbar](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/userform-command-bar) — Microsoft Learn | Designer toolbar. |
| S36 | [Worksheet object events](https://learn.microsoft.com/en-us/office/vba/excel/concepts/events-worksheetfunctions-shapes/worksheet-object-events) — Microsoft Learn | Excel event selector options and handler examples. |
| S37 | [Call Stack dialog box](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/call-stack-dialog-box) — Microsoft Learn | Call hierarchy and Show/navigation button. |
| S38 | [Language reference for VBA](https://learn.microsoft.com/en-us/office/vba/api/overview/language-reference) — Microsoft Learn | Link hub for public language, forms, IDE model and UI documentation. |

**Revision caution:** Microsoft Learn has older UI entries last revised in ~2021–2022; the open file-format specification index has newer revisions (e.g. an MS-OVBA entry dated May 2026). These are not evidence that the classic UI has been redesigned in 2026. The deliverable is a *documented functional baseline*, and exact current-versus-legacy discrepancies require direct lawful verification.

---

## 15. Open verification items and release gate

The following should be resolved against a declared licensed **reference build** (for example, a specific desktop Excel Microsoft 365 version on Windows, with build number recorded). Do this by observing public behaviour, not inspecting binaries or resource code:

1. **Precise menu item ordering, separators, accelerators and disabled-state placement** within the 11 menus; generic Learn tables frequently sort/group documentation differently from live UI.
2. **Shortcut discrepancies**, especially F4/F6 for Properties, `Ctrl+M` for Import versus Indent, and F3/Shift+F4 for Find Next.
3. **UI geometry, pane default visibility, toolbar default groups, status text and colour** at the chosen zoom/DPI/theme. Record measurable constraints from authorised observation; draw Craft* originals.
4. **Excel-specific Project Explorer folder labels/order** and how References or protected/locked projects appear.
5. **Exact statement markers, tooltip styling, runtime error dialog button variations, tooltip formatting and completion acceptance** in the chosen target release.
6. **Host behaviour** when editing a running procedure, changing the next statement, resetting from break mode, and stepping across a worksheet event.
7. **UserForm file round-trip**, external binary references/ActiveX support, macro project signatures and unsupported extension behaviour by platform.
8. **Public spec licence review**, product naming, trade dress, branding, and any redistributed documentation: seek legal review where needed.

### Definition of done

Craft* may claim **“familiar VBA editor UI”** after the C0 flows pass. It may claim **“VBE-compatible editor and debugger workflows”** once P1–P5 scenarios pass on real `.xlsm` workbooks with no fabricated runtime actions. It should **not claim pixel-perfect Microsoft VBE replication**, full Excel VBA compatibility, or native binary add-in compatibility until those claims have separately been established and tested. Any incomplete feature must be visibly recorded in a compatibility matrix.

**Product recommendation:** Make the **Classic VBE shell the unchanging compatibility surface**, reuse one host-agnostic editor/debugger implementation across GridCraft and WordCraft, and put AI/Python-translation features in optional extensions. This preserves the value of Excel users' muscle memory while allowing significant modernisation without contaminating the clean public-source implementation.

## 16. M8 code-assistance and syntax-feedback acceptance examples

Added 2026-10-27 following the reported broken IntelliSense and syntax-error highlighting.
This section makes functional word/member completion and live syntax feedback part of M8.
Historical M10 covers classic code aids; remaining host type binding is new M1
and classes/standard Office forms are new M2. Extensions and AI are removed from
the [current scope](post-m8-roadmap.md). These examples are independently authored tests of public behavior,
not copied Microsoft sample programs or observations from an installed Office build.

### Public behavior references

Microsoft documents insertion-point member suggestions, function argument information and checking
syntax after entering a line under [VBA environment options](https://learn.microsoft.com/en-us/office/vba/language/how-to/set-visual-basic-environment-options).
The [Editor toolbar](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/editor-toolbar)
describes member lists following a period and parameter information inside the Code window.
[Code window keys](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/code-window-general-use-keys)
documents Ctrl+J and Ctrl+Space. [Expected End Sub](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/expected-end-sub)
identifies a mismatched procedure terminator. These are behavior references; no wording, artwork,
HTML, fonts or implementation code from those pages is incorporated into the product.
Cached reference HTML is ignored under `.cache/evidence/vbe/`; URL, retrieval date, SHA-256 and
observations are recorded in [provenance](../provenance/vbe_sources.yaml). Never commit that cache.

### IntelliSense scenarios (authored fixtures)

```vb
Sub Main(ByVal limit As Long)
    Dim counter As Long
    counter = limit
    Debug.Print counter
End Sub
```

At a statement position in Main, `cou` plus Ctrl+Space offers `counter`; `li` offers `limit`.
Names are matched without case sensitivity. A variable declared only inside another procedure
must not appear here. Private procedures in another module must not be suggested. Suggestions
work in design mode and use source declarations, without requiring an earlier debugger pause.
Typing `Debug.` automatically opens a list containing `Print`; accepting it keeps the receiver
and inserts exactly `Debug.Print`. Typing `Debug.pr` filters the same list. Up/Down selects,
Enter/Tab accepts, Escape cancels, and pointer selection also works. Completion stays attached to
the source caret and typing continues there after acceptance. Comments and strings do not trigger
suggestions. Unknown receivers display no invented API members.

For a generic injected catalogue with supported `Document.Title` metadata, an independently authored
`Dim document As Document` declaration followed by `document.` offers `Title`. Unsupported catalogue
entries stay absent. This is metadata assistance, not a claim that tiny-vba owns Word's object model
or executes arbitrary object types. The host still owns capabilities and library metadata.
A project call such as `total = SumTo(` shows the parsed procedure signature while entering arguments;
assistance never calls VBA, getters or the host to discover values.

### Syntax feedback scenarios (authored fixtures)

```vb
Sub Main()
    Dim counter As Long
    counter =
End Sub
```

The incomplete assignment must produce an accessible parser message and mark the affected physical
line in red with an underline before Compile/Run. Replacing it with `counter = 3` clears that feedback
without compiling and leaves the insertion point where the user is typing. Multiple parser failures
are represented, including lexical failures, and empty/EOF spans have a visible line marker.

```vb
Sub Main()
    Debug.Print 3
End Function
```

The mismatched terminator must be rejected and identified in the source. Changing it to `End Sub`
clears the syntax failure. Compile/binding/runtime failures remain distinguishable from live parser
feedback: a valid expression with an unknown name is a binding failure, not necessarily a syntax
error. Yellow execution highlighting remains tied to the current compiled statement and must not
be confused with red syntax feedback. Editing paused source makes that execution stale.

The M8 implementation checks on each source revision (earlier than the documented line-commit
check) and uses nonmodal messages plus red/underlined lines instead of interrupting typing with a
modal dialog. This is an intentional, testable UI policy, not an assertion of exact VBE appearance.
LF, CRLF, lone CR and UTF-8 text must share compiler/debugger coordinates. Display normalization
must preserve untouched source bytes and map editor offsets back to the original source.
EOF errors may mark the preceding nonempty line so an omitted terminator remains visible.

### Verification evidence

Owning-crate model and egui interaction tests exercise these scenarios, including keyboard/pointer
acceptance and correction without Compile. `cargo xtask ui` renders original completion and syntax
feedback snapshots in light/dark themes as well as the existing paused/narrow/scaled/host variants.
The standalone native and Chromium harnesses exercise the same reusable Rust panels. Screenshots
remain ignored local reports; exact Office geometry and fork integration require later evidence.

Ordinary editor commands and their exact boundaries are recorded in the
[embedding guide's editor matrix](embedding.md#ordinary-editor-commands): M8 verifies keyboard/menu
Undo/Redo, selection-aware Cut/Copy/Paste and deletion. Browser Paste uses the browser keyboard
route. Automatic indentation, whole-block indentation/comments and richer editing workflows remain
M10. This avoids treating every conventional editor feature as implicitly implemented.
