# M7A.4 native runtime — current implementation and remaining work

Updated 2026-11-06. **IN PROGRESS. M7A.4 is not complete.** This record extends
[the first instance increment](m7a4-userform-instances.md) under
[ADR 0043](decisions/0043-native-controls-and-project-inspection.md).

Implemented in shared tiny-vba:

- [ADR 0070](decisions/0070-native-keyboard-events.md): selected typed keyboard
  callbacks, mutable key arguments and Shift masks, static/dynamic control event
  sinks, canonical completion receipts and the bounded pre-widget input barrier.
  Stop/Continue/cancellation/replacement/Reset and repeat ordering have authored
  tests; full keyboard/platform/focus/IME behavior remains partial.

- [ADR 0069](decisions/0069-userforms-without-form-components.md): minimal built-in
  UserForms support without a form component; Count, Item, Add errors, aliases,
  empty enumeration, shadowing, Reset/Stop and document callbacks.

- [ADR 0068](decisions/0068-lazy-project-class-array-elements.md): lazy fixed/dynamic project-class/native-form arrays,
  independent elements, Preserve/Erase, static/private storage, canonical ByRef,
  constructor and For Each continuations, inert inspection and unchanged budgets.

- [VarType default values](decisions/0067-vartype-default-value-inspection.md):
  one compatible public default getter/function through normal frames or authorized
  host calls, preserving getter errors, Variant/array tags and Stop/wait continuations.
  TypeName/IsObject and safe watches retain inert reference inspection. Additional
  default/arity comparisons remain M8.5 NOT_RUN.

- [Native cell/row bindings](decisions/0065-native-cell-and-row-bindings.md): inert
  ControlSource/RowSource site metadata, ordinary single-cell value links and
  bounded contiguous list sources through an explicit existing host Range import.
  Bound rows remain read-only during authored callbacks; protected writes are
  preflighted. Inert rendering/inspection and trusted revision-driven callbacks
  retain canonical state and original resources. Further binding timing/types
  remain open.

- [Native array readback](decisions/0054-native-list-array-readback.md): detached
  zero-based two-dimensional List/Column snapshots for the current ten-column
  unbound packet, singleton/empty cases, named Forms parameters and independent
  lifetime across Clear/Stop. Further shape/loading and binding timing/type parity remains open.
- [Native metadata and tips](decisions/0055-native-form-metadata-and-control-tips.md):
  inert Tag/ControlTipText/HelpContextID decoding, ordinary per-instance state and
  theme-injected runtime tips without focus/code execution. Actual Help/F1 routing,
  native edited persistence and the broader property/platform packet remain open.

- [Native list array loading](decisions/0053-native-list-array-loading.md): shared
  two-dimensional List and transposed Column assignment, independent source
  arrays/lower bounds and indexed/current-row Column access. Validation completes
  before row/selection replacement and ordinary Change continuations. Additional array/list/binding behavior remains open.

- [Native ComboBox](decisions/0052-native-combobox.md): selected editable/drop-list
  records, shared ordinary list rows, Value/Text/selection/matching and VM-owned
  drop state. Authorized text/row/drop/navigation input preserves validation and
  debugger continuations; contextual F4 opens the list. Locale/case service limits
  and further list/property/binding/resource/drop-lifecycle work remain open.

- [Native unbound ListBox](decisions/0051-native-unbound-listbox.md): cached native
  records, indexed ten-column row data, AddItem/RemoveItem/Clear, selection,
  Value/Text/column properties, ordinary Change/Click and authorized pointer/key
  input. Focus/validation, original resources and debugger continuations remain
  intact. ADR 0063 adds multiple selection; ADR 0065 adds selected bindings. Further list properties remain open.

- [Native SpinButton/ScrollBar](decisions/0050-native-spinbutton-and-scrollbar.md):
  distinct native resource packets, Long bounds/value/increments, orientation and
  delay, canonical Change/SpinUp/SpinDown/drag-only Scroll, reversed ranges and
  ordinary focus/debugger continuations. Arrows/track/thumb/keyboard input use
  authorized callbacks and the shared renderer; resources stay unchanged.

- [Native TextBox selection/input](decisions/0049-native-textbox-selection-and-input.md):
  UTF-16 SelStart/SelLength/SelText and atomic input/cursor commits, native multiline,
  word-wrap, Enter/Tab behavior, SelectAll/RecallSelection and custom password
  characters. Programmatic state and actual Unicode keyboard input share ordinary
  frames and existing Change/validation. Decoder flags and original resources stay
  separate; the renderer preserves host fonts and canonical text on missing glyphs.

- [Native container focus/validation](decisions/0048-native-container-focus-and-validation.md):
  read-only ActiveControl, SetFocus, first eligible tab selection, Enter/Exit and
  TextBox BeforeUpdate/AfterUpdate with mutable ByVal MSForms.ReturnBoolean cancellation.
  Physical input follows validation in ordinary frames. Tab/Shift+Tab and pointer
  requests preserve canonical focus across cancellation, Stop/resume and Reset.
  Leaf WithEvents interfaces keep their distinct event exposure.

- [MSForms nominal types and qualified library access](decisions/0047-native-forms-nominal-types-and-library-access.md):
  generic IR 11 class interfaces bind native UserForm/Control/Controls and ten leaf
  types without a host object registry or COM. Specific leaf WithEvents preserves
  canonical objects while event allowlists hide bridge events; non-source Control
  connections refuse with error 459. Mutable ReturnBoolean/ReturnInteger holders
  preserve ByVal object identity for typed event delivery. VBA.UserForms supports
  Count/Item/index/For Each even when a project function shadows bare UserForms.

- [Lazy As New](decisions/0046-lazy-project-class-variables.md) for scalar project
  classes/native forms, module/static and per-instance variables, canonical ByRef,
  Nothing re-creation, constructor Stop/failure/retry and inert debugger inspection
  under IR 10. Non-predeclared forms can also launch from the designer.

- [Window/code lifetime separation](decisions/0044-userform-window-and-code-lifetime.md):
  unloading drops display/default ownership while aliases retain authored code
  fields; named control getters reload initial controls. Fresh default references
  are independent of retained aliases. Terminate follows actual finalization;
  cancelled close preserves identity/values, and paused Reset performs no cleanup.
  This uses a generic validated root-release API and bounded internal continuation,
  without a new IR or user-visible waiting/prompt feature.

- Flat native Label, TextBox, CommandButton, CheckBox, OptionButton and ToggleButton
  import/runtime. New boolean families decode native cached class/DisplayStyle,
  Caption/Value/GroupName and Locked. Unknown masks, containers, list/binding and
  unsupported resource variants still refuse atomically. Exact original source
  and resources remain separate from runtime state.
- Boolean/Null Value, default Value access, Change/Click rules, user TripleState,
  Locked and root-container option groups, all using ordinary classes/WithEvents.
  A new value is committed before callbacks; Null does not emit Click. A deselected
  option emits Change but not Click. Reassigning the same value stays inert.
- Integer TabIndex with neighbor reordering, nonnegative validation and maximum
  clamping. Button Default/Cancel exclusivity, eligible Enter/Escape dispatch in
  the input-owning form, and CommandButton Value=True click delivery.
- Native persisted button roles and TextBox MaxLength; MaxLength affects input,
  while programmatic text remains writable. Locked blocks input without disabling
  programmatic writes. Common control Name is read-only; TextBox no longer
  advertises the nonexistent Caption property. Form Name and bounded point sizes
  are available through generated ordinary properties.
- Controls/UserForms For Each enumeration uses the IR 9 class-enumerator contract
  in [ADR 0045](decisions/0045-class-and-native-collection-enumerators.md). Nested
  and empty loops retain canonical objects through unload, Stop and resume;
  collection queries remain inert and never load defaults. Native loop snapshots
  now retain object lifetimes in active and parked frames.
- Controls.Count and zero-based/name Item/default lookup preserve child identity.
  Names use the admitted ASCII identifier profile without requiring a locale
  service for internal lookup. UserForms.Count/Item/default lookup inspect live
  loaded instances in creation order; UserForms.Add(name) creates an explicit
  instance. Queries never load a default form. Load/Unload through Object variables
  use the ordinary receiver/member path. Non-predeclared native components can
  be constructed explicitly.
- A generic transient, read-only host view over canonical project objects, with
  stale/private/getter refusals and work/payload budgets. FormHost forwards host
  intrinsic services and their waits. No second mutable object registry or
  external dependency or custom form-format extension is introduced.

The design cap remains 128 controls, existing native resource limits remain in
force, VM instances/locations/callback and instruction budgets remain authoritative.
The legacy encoder refuses the native-only added control families. Native edited
export/save remains unimplemented until M7A.5; there is no conversion feature.

Original Rust fixtures under MIT OR Apache-2.0 supply the evidence. No Microsoft-
produced form/document, vendor asset, downloaded macro or other interpreter code
is included or executed:

- [IDE native instance/runtime tests](../crates/vba-ide/tests/form_instances.rs)
- [Actual egui input tests](../crates/vba-ui-egui/tests/form_instances.rs)
- [Native decoder tests](../crates/vba-project/tests/native_forms.rs)
- [Generic VM view regression](../crates/vba-vm/tests/classes.rs)
- [Contextual property parser regression](../crates/vba-syntax/tests/property_keywords.rs)
- [Chromium boolean-control harness](../tools/ui-browser/native-boolean-forms.cjs)
- [Chromium native text harness](../tools/ui-browser/native-text-forms.cjs)
- [Chromium native numeric harness](../tools/ui-browser/native-numeric-forms.cjs)
- [Actual multi-instance render builder](../crates/vba-ui-egui/examples/native_instances_snapshots.rs)

Validation and consumer integration status are recorded in
[implementation progress](implementation-progress.md).

Remaining M7A.4 requirements, without reducing the milestone's scope, are made
concrete in [the completion checklist](m7a4-completion-checklist.md):

- Additional default/control-reference alias and reload edge cases against Office;
  full Activate/Deactivate ordering and modeless focus ownership. The selected
  replacement/code-lifetime cases are implemented under ADR 0044.
- Additional implicit/alias behavior beyond ADR 0068 array instancing, additional dynamic
  collection cases beyond ADR 0069 empty-project support, direct enumerator cursor interfaces and structural-mutation
  parity. External class activation remains outside the project-only path.
  For Each uses the existing bounded snapshot profile.
- Further focus/validation edge cases across recursive redirection, window/container
  activation, boolean update timing and data binding; typed keyboard/mouse/accelerator ordering, further text/selection edge cases
  and remaining property/default/error/coercion contracts,
  beyond the compatible default-value inspection added by ADR 0067. ADR 0066 adds
  foreign-object dynamic TypeName through existing host metadata. ADR 0048 implements the selected flat native
  focus/typed cancellation path, without claiming the full event/property matrix.
- ADR 0060 adds selected native RefEdit text/drop semantics and host picker integration; broader RefEdit/platform behavior remains open. ADR 0059 adds selected Image raster records,
  picture identity/assignment, layout and Click; further picture formats/metadata,
  loading/platform services and the complete event/presentation matrix stay open. ADR 0058 adds selected native MultiPage/Page;
  ADR 0061 adds selected dynamic Pages/Index/Name; further transitions/properties/resources, programmatic selection/
  focus and lifecycle edge cases remain open.
  ADR 0057 implements selected native TabStrip records and ordinary Tab/Tabs;
  ADR 0061 adds tab structural mutation and writable Index/runtime Name; further
  mutation/error/locale and full event/presentation parity remain open. ADR 0056 implements selected native Frame parents; additional storage variants,
  properties, resource/presentation and full event parity remain open.
  ComboBox and ListBox now have selected unbound packets; further array-shape/loading cases, binding timing/types and further presentation remain open; ADR 0063 implements native multiple selection. The selected numeric packet does not close all
  ScrollBar/SpinButton property/resource/presentation cases.
- Applicable Office-supplied worksheet ActiveX/platform packets; third-party
  controls and COM/OCX loading remain outside scope.
- Fresh Office build/OS/locale observations remain NOT_RUN under M8.5. The two
  forks now pin the accepted shared runtime increment and have native/WASM/
  package/input-authorization regression evidence; broader milestone acceptance
  still requires the remaining implementations above.

Source basis: [MS-OFORMS cached classes](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/5f649a6c-90df-4226-b8c7-2a7d98cd9647),
[MorphData masks](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/19014e19-67fa-4060-8c81-d1463809f117),
[persisted Value](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/0992cd3a-9a63-4e95-91ae-43c025082847),
[UserForm/collections](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/userform-object),
[TripleState](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/triplestate-property),
[TabIndex](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/tabindex-property),
[Click](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/click-event),
[MaxLength](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/maxlength-property).
These references do not establish fresh measured Office event traces or full parity.

Native Frame containers follow [ADR 0056](decisions/0056-native-frame-containers.md):
bounded native parent storages, immediate child Controls/Parent identity, scoped
TabIndex/option groups and nested ActiveControl/focus cancellation. Canonical state
remains ordinary classes; effective ancestor flags, live relative geometry,
clipping and accessible groups use the shared renderer. Original source/resources
stay inert and unchanged. Further container/page/image/binding/platform semantics,
native edited persistence and .tvform removal remain in M7A; fresh Office traces
remain NOT_RUN under M8.5.

[ADR 0057](decisions/0057-native-tabstrip.md) adds selected native TabStrip
records and separate ordinary Tab/Tabs identity, indexed/named lookup,
enumeration/default selection and authorized widget input. Canonical state stays
in ordinary classes; inert queries/rendering do not instantiate or execute VBA.
Dynamic page/tab mutation/index/name semantics and the remaining M7A.4
ledger stay open. Native edited persistence and .tvform removal remain separate.

[ADR 0058](decisions/0058-native-multipage.md) adds selected native MultiPage
f/o/x reconciliation, distinct Page/Pages identity, immediate child collections,
selection-gated input/focus and actual page rendering. Internal tabs stay private;
canonical state remains ordinary classes and source/resources stay inert. Indexed
Click and repeated-selection behavior also correct the prior TabStrip contract.
Dynamic page/tab mutation, further persistence/property/event/lifecycle cases and
the full remaining M7A.4 ledger remain open. Native edited saving/removal stay
M7A.5/.6; fresh Office traces remain NOT_RUN under M8.5.

[ADR 0059](decisions/0059-native-image-pictures.md) adds selected native Image,
bounded inert BMP/GIF/JPEG/ICO decoding, canonical stdole picture identity and
assignment, display properties and nonfocusable Click. Immutable raster assets
and capped derivative texture caches preserve source/resources and existing VM
budgets. Further picture metadata/platform/format and event/presentation cases,
RefEdit, bindings, dynamic collections and remaining M7A.4 semantics stay open.
Native edited saving/.tvform removal remain M7A.5/.6; Office traces remain NOT_RUN
under M8.5.

[ADR 0060](decisions/0060-native-refedit.md) adds bounded inert RefEdit class-table
resolution, ordinary typed RefEdit text/drop state and selected focus/validation/
Change/DropButtonClick behavior. The shared renderer accepts host range-selection
availability and collapses/restores modal RefEdit; GridCraft owns worksheet
selection and authorization. Original source/resources stay separate. Broader
RefEdit/platform/property/event/binding semantics and the unchanged M7A.4 ledger
remain open. Native edited saving/removal stay M7A.5/.6; fresh Office traces stay
NOT_RUN under M8.5.

## Native Tabs/Pages mutation

ADR 0061 implements Add/Remove/Clear, zero-based Integer Index and runtime Name,
including named/omitted Add arguments, canonical enumeration, identity-preserving
reordering, first/last/empty selection, detached aliases and ordinary event
continuations. Inert queries validate scalar position cells against actual member
slots. The existing bounded For Each snapshot profile is unchanged. Programmatic
mutations do not change native source/design/resource bytes or synthesize Click.
Reordered native Page descendants remain attached to the same Page and removed
Page descendants cannot receive widget input. Added Pages are empty; dynamic
Controls.Add and complete focus/lifecycle/detachment parity remain open alongside
the unchanged broader ledger above. Detailed selection/error behavior is authored
inference pending M8.5 Office traces, not fresh comparison evidence.

[ADR 0062](decisions/0062-native-dynamic-controls.md) implements selected native
Controls.Add/Remove/Clear, runtime names, scoped collections/notifications and
WithEvents focus/validation. An inert runtime projection drives widgets and
input from ordinary canonical objects; imported source/design/resources stay
unchanged. Weak context requests avoid ownership cycles. Shared typed helpers,
indexed verifier metadata and generic dispatch optimizations preserve all existing
VM limits. Full M7A.4, native edited persistence/removal and M8.5 comparisons remain
separate requirements; CI/WASM/WASI validation of ADR 0062 passed.

[ADR 0063](decisions/0063-native-listbox-multiple-selection.md) adds native ListBox
multiple selection, independent focus/selection, row mutation and authorized
pointer/keyboard input. [ADR 0064](decisions/0064-native-runtime-type-names.md) adds
IR 12 inert runtime class names and native TypeName through canonical aliases.
State stays in ordinary classes, rendering stays inert, and budgets are unchanged.
Bindings and the remaining M7A.4 ledger remain open; M7A.5/.6 and M8.5 are separate.

[ADR 0065](decisions/0065-native-cell-and-row-bindings.md) defines native
ControlSource/RowSource through ordinary classes and the existing explicit host
range import. Import and rendering remain inert; GridCraft owns cell commands,
trust and revision-driven callbacks. The owner approved 4,096 static permission
entries on 2026-11-05; per-site authorization and executed-call/work/memory limits
remain in force. Further binding timing/types and the broader M7A.4 ledger remain open.


[ADR 0066](decisions/0066-foreign-runtime-type-names.md) extends existing read-only
host type resolution with bounded native TypeName metadata. Identity/lifetime,
no-getter/no-activation rules, lookup/allocation budgets and generic Object fallback
are retained. TypeName/VarType/IsArray named varname binding matches their documented
signatures. Shared and pinned fork validation passed; the remaining M7A.4 ledger stays open.


[ADR 0067](decisions/0067-vartype-default-value-inspection.md) adds IR 13
inspection-defaults-v13: VarType resolves one compatible public default value
through normal project frames or authorized host calls, preserving errors and
Stop/wait continuations. Public default functions and New value contexts use
ordinary binding. Inert watches, host ownership and existing budgets are retained.
Shared CI, WASM and actual WASI validation pass; the full M7A.4 ledger stays open.

Native typed keyboard callbacks and the raw-input/default-processing barrier are
implemented under [ADR 0070](decisions/0070-native-keyboard-events.md). Keyboard
completion is still partial pending the remaining K01/K02 acceptance cases.

[ADR 0071](decisions/0071-native-keyboard-focus-and-default-order.md) adds
root-window keyboard focus, input arbitration before editor shortcuts and
canonical Tab/SetFocus/Space default ordering. Full M7A.4 remains in progress.


[ADR 0072](decisions/0072-native-form-accelerators.md) adds native persisted
accelerators and canonical Accelerator properties, selected eligible Alt actions
for Label/buttons/booleans/Tab/Page, validation/callback continuations and inert
caption underlines. Duplicate routing explicitly refuses; non-ASCII/platform and
further ordering cases remain open. Checklist K03 is PARTIAL; M7A.4 remains in
progress, edited persistence/removal remain M7A.5/.6 and Office observations remain
M8.5 NOT_RUN.


[ADR 0073](decisions/0073-native-mouse-callbacks.md) adds typed native mouse events,
indexed Tab/MultiPage arguments, mutable DblClick cancellation and canonical
completion receipts. Selected shared pointer capture/default paths preserve
Stop/Continue, Reset, identity and keyboard focus; remaining text/list/numeric/
indexed-tab/platform defaults and full event ordering keep M01 PARTIAL. M7A.4
remains incomplete and fresh Office observations remain M8.5 NOT_RUN.

[ADR 0074](decisions/0074-native-text-pointer-selection.md) adds selected native
TextBox pointer caret/drag/word selection after mouse callbacks, cancellation and
focus redirects, with ordinary authorized selection commits. Canonical UTF-16,
CRLF, Stop/Reset and source/resource identity are preserved. Native AutoWordSelect
bit 27 is the inverse of the Boolean property; imported defaults and new dynamic
controls are distinguished. ComboBox/RefEdit pointer defaults, further platform/
word/selection cases and the full M7A.4 checklist remain open.

[ADR 0075](decisions/0075-native-composite-pointer-defaults.md) adds selected
ComboBox/RefEdit text and drop/picker, list/popup row, numeric step/repeat/drag and
indexed tab/page-header pointer defaults after ordinary mouse callbacks. Inert
modal-display metadata is separate from input ownership; popup input inherits the
execution barrier and tab identity is revalidated after mutation. ComboBox native
AutoWordSelect uses its inverse file bit. Canonical state and existing executable
command authorization remain authoritative. Full M01/M7A.4 acceptance stays open.
