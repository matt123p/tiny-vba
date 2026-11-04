# M7A.4 native runtime — current implementation and remaining work

Updated 2026-11-04. **IN PROGRESS. M7A.4 is not complete.** This record extends
[the first instance increment](m7a4-userform-instances.md) under
[ADR 0043](decisions/0043-native-controls-and-project-inspection.md).

Implemented in shared tiny-vba:

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
The legacy encoder refuses the three added native control families. Native edited
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
- [Actual multi-instance render builder](../crates/vba-ui-egui/examples/native_instances_snapshots.rs)

Validation and consumer integration status are recorded in
[implementation progress](implementation-progress.md).

Remaining M7A.4 requirements, without reducing the milestone's scope:

- Additional default/control-reference alias and reload edge cases against Office;
  full Activate/Deactivate ordering and modeless focus ownership. The selected
  replacement/code-lifetime cases are implemented under ADR 0044.
- Qualified collection access, generic UserForm/
  MSForms type bindings, additional implicit/alias behavior and As New arrays.
  External class activation remains outside the supported project-only path. Direct enumerator
  cursor interfaces and collection structural-mutation parity also remain open;
  For Each uses the existing bounded snapshot profile.
- ActiveControl/SetFocus, Enter/Exit and Before/AfterUpdate with the native typed
  cancel arguments; keyboard/mouse/accelerator ordering, text selection, multiline,
  password and the remaining property/default/error/coercion contracts.
- Native ComboBox, ListBox, Frame, MultiPage, TabStrip, ScrollBar, SpinButton,
  Image and Excel RefEdit packets, including real container/resource identity,
  list/page collections, pictures, bindings and presentation. These remain refused.
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
