# Deferred implementation plan — Office forms and controls

Created 2026-11-02. Status: **DEFERRED / NOT STARTED** by owner direction.
This is a future implementation backlog, outside the current M7–M8 delivery
profile. It has no milestone number or scheduled start and does not block M7.
Activate selected stages explicitly before implementing them; activation must
assign owners, capability versions, prerequisites and acceptance gates in the
[main plan](implementation-plan.md). The FC task IDs below remain stable.

## Problem and current support

Three distinct capabilities need separate treatment:

| Surface | Current support | Future work |
|---|---|---|
| Excel worksheet Form Controls | GridCraft preserves unsupported control package content for saving; does not display or operate it | Import, render, interact and retain macro/link metadata |
| tiny-vba `.tvform` forms | Shared Label/TextBox/CommandButton designer/runtime and exchange; GridCraft can import a separate `.tvform` | Extend controls and integrate native Office imports |
| Native VBA UserForms | `vba-project` explicitly refuses UserForms/designers; no `.frm`/`.frx` or binary designer import | Inert source/resource decoding, shared design mapping and runtime integration |

The local `examples/Food.xlsm` illustrates the first gap: sheet1 contains
`Button 10`, shape ID 1034, assigned to `[0]!CopyIngredients`. It has worksheet
`controls` metadata, a control-properties part and a legacy VML drawing. Its
DrawingML shape has an empty `macro` attribute, so reading only DrawingML macro
associations cannot recover its assignment. This is package inspection evidence;
the workbook was not executed and is not a distributable test fixture.

Current contracts remain [M2 acceptance](m2-acceptance.md),
[ADR 0024](decisions/0024-standard-form-execution-and-exchange.md) and
[read-only project import](project-import.md). Unsupported content remains
explicitly unsupported; preserving bytes does not establish rendering, execution
or edited writeback support.

## Ownership and implementation constraints

| Layer | Owner and responsibility |
|---|---|
| Workbook packaging and worksheet controls | GridCraft XLSX/engine/UI: OOXML relationships, VML, control properties, sheet anchors, document state, linked cells, undo and rendering |
| Excel objects and macro invocation | GridCraft VBA bridge: worksheet owner, Shapes/Form Controls object profiles, macro target resolution, trust, events and recalculation |
| VBA project/form resource extraction | tiny-vba `vba-project`: bounded, read-only decoding of VBA project components and designer resources; no macro execution or control-library activation |
| Shared form design and execution | tiny-vba `vba-ide` and reusable UI: validated form model, source attachment, design/runtime presentation, callbacks and lifecycle |
| Generic language/runtime changes | tiny-vba semantics/compiler/VM/debugger only where the existing class/event model needs a reviewed extension |
| Save, signatures and application workflows | Each fork owns its package policy; coordinate with M7's separately reviewed writeback contract |

Keep Office document objects and packaging out of the VM and generic foreign
protocol. Reuse ordinary typed values, class identities, Event/WithEvents,
bounded callback queues, debugger frames and host continuations. Generated form
support must remain separate from exact authored source. Do not add proposed
crates, dependencies or bytecode capabilities until a concrete change needs them.

## FC0 — Inventory and contracts

- **FC0.1:** Inventory real import/render/save paths and independently authored
  fixtures. Inspect GridCraft `crates/xlsx/src/passthrough/read.rs`,
  `crates/xlsx/src/drawing.rs`, `crates/vba/src/lib.rs` and
  `crates/ui-egui/src/vba.rs`; inspect tiny-vba `crates/vba-project/src/lib.rs`,
  `crates/vba-ide/src/forms.rs` and shared form orchestration before changing APIs.
- **FC0.2:** Publish a per-control/property/event matrix distinguishing worksheet
  Form Controls, UserForm controls and worksheet ActiveX controls. Record decode,
  display, edit, execute and save independently. Verify format details against
  public specifications when this work is activated; proposed behavior is not
  fresh Office evidence.
- **FC0.3:** Adopt an ADR for identities, resource limits, unsupported-content
  handling, import atomicity, source/resource attachment and persistence. Define
  stable identities across sheet rename/reorder and distinct control names,
  captions, shape IDs, project module names and VBA code names.

Exit: reviewed contracts and small lawful fixtures; no runtime/library activation
on import. Every later stage has an explicit supported subset.

## FC1 — Excel worksheet buttons first

Owner: GridCraft. Depends on FC0; does not require native UserForm decoding.

- **FC1.1:** Read worksheet `controls`, relationships, control properties and VML
  alongside DrawingML. Reconcile representations by their documented identities
  into one control; retain source parts and unsupported metadata without creating
  duplicate visuals or overwriting conflicting identities silently.
- **FC1.2:** Render a worksheet Button with caption, anchor, size, visibility and
  enabled state. Define geometry conversion, row/column resizing, hidden rows,
  zoom and DPI behavior. Use original assets and accessible focus/keyboard input.
- **FC1.3:** Resolve macro assignments through the existing trusted VBA launch
  path. Define workbook qualification, local `[0]!` syntax, eligible procedure
  signatures, disabled/missing targets and cross-workbook refusal rules. Capture
  the button's owning workbook/sheet independently of active UI state.
- **FC1.4:** Connect click/keyboard activation to exactly one invocation; retain
  debug pause/resume, error handling, cancellation and undo behavior. Design mode
  selects/edits the button without running it. Opening a workbook stays inert.
- **FC1.5:** Preserve untouched controls and relationships on save. Define edited
  caption/anchor/assignment persistence separately; refuse unsupported edits
  explicitly. No binary VBA project rewrite is needed merely to retain an
  existing control's macro assignment.

Exit: an independently authored Food-shaped package displays one button, invokes
an authored no-argument Sub once after explicit trust, and retains its control
parts/assignment on save and reopen. Food.xlsm itself remains an optional local
diagnostic specimen; its unrelated selection/clipboard/sort gaps cannot count as
button failures or be silently treated as implemented.

## FC2 — Remaining worksheet Form Controls

Owner: GridCraft. Depends on FC1's identity and lifecycle contracts.

- **FC2.1:** Add CheckBox and OptionButton with reviewed typed state,
  indeterminate-state support/refusals, grouping and macro ordering.
- **FC2.2:** Add ListBox, DropDown/ComboBox, ScrollBar and Spinner with bounded
  item/range data, selection, minimum/maximum/increment and error rules.
- **FC2.3:** Add Label and GroupBox presentation and grouping. Declare remaining
  types unsupported until individually implemented.
- **FC2.4:** Define `LinkedCell`/`ListFillRange` identity, address resolution,
  data updates, coercion, calculation and event ordering, including changes made
  by VBA while execution is paused. Route mutations through normal engine/undo
  commands and enforce recursion/work limits.
- **FC2.5:** Add the selected Shapes/ControlFormat collection, property and macro
  assignment surface to GridCraft's reference catalogue and host adapter.
  Publish exact names/signatures and unsupported members; no blanket object-model
  compatibility claim follows from rendering controls.

Exit: each supported type has import/render/input/linked-data/host-object/save
evidence plus negative cases and explicit unsupported property diagnostics.

## FC3 — Native VBA UserForm import

Owners: tiny-vba read-only decoder/shared IDE; forks attach package components.
Depends on FC0; can proceed independently of FC2 after activation.

- **FC3.1:** Design a bounded component/resource result for native designer
  modules instead of coercing them into ordinary classes. Preserve module IDs,
  names, source attributes, form identity and resource provenance. Retain existing
  protection refusals and never execute P-code or activate referenced libraries.
- **FC3.2:** Support the reviewed text `.frm` export subset and associated `.frx`
  resources, then embedded project designer storage. Treat these as distinct
  formats with distinct acceptance; text export support does not prove embedded
  binary form support. Resolve companion resources only through caller-supplied
  data, with no ambient path traversal or automatic filesystem access.
- **FC3.3:** Decode geometry, captions, names, supported properties and control
  hierarchies into validated shared design data. Start with the existing
  Label/TextBox/CommandButton subset; preserve exact authored handlers. Unknown
  controls/resources fail before partial project mutation or are explicitly
  quarantined as inert unsupported components under the reviewed import policy.
- **FC3.4:** Present imported forms in Project Explorer with View Object/View Code
  and Properties, attach source to the right instance and handlers, and run
  through the shared form runtime in standalone and embedded environments.
- **FC3.5:** Cover default instances, Load/Show/Hide/Unload, modal/modeless input,
  initialization/activation/termination and event order for the declared subset.
  Inspect current M2 behavior first; implement only reviewed gaps and document
  differences from observed Office behavior.

Exit: the same independently authored form imports from supported export and
embedded representations, opens its designer/code and executes supported handlers
unchanged. Import alone never fires initialization or other handlers. Unsupported
designer metadata remains visible in diagnostics rather than being discarded.

## FC4 — Broader shared controls and standard ActiveX compatibility

- **FC4.1:** Extend the shared form model/runtime in small packets for CheckBox,
  OptionButton, ComboBox, ListBox, Frame, MultiPage/TabStrip, ScrollBar, SpinButton,
  ToggleButton and Image, subject to FC0's reviewed inventory. Version exchange
  data as needed and preserve explicit refusal by older readers.
- **FC4.2:** Specify focus/tab order, default/cancel buttons, accelerators,
  validation/change/click events, grouping, item/value semantics and property
  defaults per packet. Extend native and browser presentation together.
- **FC4.3:** Evaluate Office-supplied worksheet ActiveX controls as a separate
  compatibility profile using independent Rust implementations and inert binary
  decoding. Activation requires a separate design review; no COM/OCX loading or
  native code execution follows from this backlog.

Third-party ActiveX/OLE controls, arbitrary COM libraries, proprietary control
loading, CommandBars and custom Ribbon support are outside this plan. Unknown
controls remain preserved/inert where safe and explicitly unsupported at runtime.

## FC5 — Persistence and acceptance

- **FC5.1:** Specify original-byte preservation, edited worksheet-control
  serialization, `.tvform` export and edited native UserForm writeback as separate
  capabilities. Native form edits affecting `vbaProject.bin` require the adopted
  M7 writeback/signature contract; separate export remains available until then.
- **FC5.2:** Verify relationship/content-type integrity, source and resource
  identity, unknown-part retention and save/reopen behavior. Never report edited
  forms saved when only the original package was retained. Use revision-checked
  save acknowledgments for source and design changes.
- **FC5.3:** Cover trust loss, Reset, close/unload, missing/deleted owners,
  stale handles, callback failures, modal waits, queue exhaustion, cancellation,
  event ordering and undo/recalculation against real fork state. Bound nesting,
  control/item counts, image/resource allocation, decompression and callback work.
- **FC5.4:** Run relevant importer/runtime/native/WASM/WASI and UI/browser gates,
  plus actual GridCraft workflows. Record independent authored expectations,
  public-specification expectations and fresh permitted Office observations
  separately. Fresh comparisons require named build/OS/locale and raw results;
  unavailable comparisons remain NOT_RUN.

On activation, create a ledger per task/control with owner, capability version,
fixture rights/provenance, dependencies, implemented subset, checks and unresolved
gaps. Report load/compile/display/input/run/save independently using PASS,
MISMATCH, UNSUPPORTED and NOT_RUN. Promotion to the main plan requires an explicit
scope decision; this document alone authorizes no implementation.

First task when resumed: **FC0.1**, followed by the reviewed FC1 worksheet Button
slice. No code, capability, compatibility or acceptance result changes with this plan.
