# M7A.4 native runtime — current implementation and remaining work

Updated 2026-11-04. **IN PROGRESS. M7A.4 is not complete.** This record extends
[the first instance increment](m7a4-userform-instances.md) under
[ADR 0043](decisions/0043-native-controls-and-project-inspection.md).

Implemented in shared tiny-vba:

- [Native array readback](decisions/0054-native-list-array-readback.md): detached
  zero-based two-dimensional List/Column snapshots for the current ten-column
  unbound packet, singleton/empty cases, named Forms parameters and independent
  lifetime across Clear/Stop. Further shape/loading/binding parity remains open.
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
  intact. Multiple-selection/binding and further list properties stay open.

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

Remaining M7A.4 requirements, without reducing the milestone's scope:

- Additional default/control-reference alias and reload edge cases against Office;
  full Activate/Deactivate ordering and modeless focus ownership. The selected
  replacement/code-lifetime cases are implemented under ADR 0044.
- Additional implicit/alias behavior, As New arrays, empty-project/dynamic
  collection cases, direct enumerator cursor interfaces and structural-mutation
  parity. External class activation remains outside the project-only path.
  For Each uses the existing bounded snapshot profile.
- Further focus/validation edge cases across recursive redirection, window/container
  activation, boolean update timing and data binding; typed keyboard/mouse/accelerator ordering, further text/selection edge cases
  and remaining property/default/error/coercion contracts,
  including native dynamic TypeName. ADR 0048 implements the selected flat native
  focus/typed cancellation path, without claiming the full event/property matrix.
- Native Image and Excel RefEdit packets, including pictures, bindings and
  presentation, remain refused. ADR 0058 adds selected native MultiPage/Page;
  dynamic Pages, further transitions/properties/resources, programmatic selection/
  focus and lifecycle edge cases remain open.
  ADR 0057 implements selected native TabStrip records and ordinary Tab/Tabs;
  tab structural mutation, writable Index/Name and full event/presentation parity
  remain open. ADR 0056 implements selected native Frame parents; additional storage variants,
  properties, resource/presentation and full event parity remain open.
  ComboBox and ListBox now have selected unbound packets; further array-shape/loading cases, multiple selection, bindings and further presentation remain open. The selected numeric packet does not close all
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
