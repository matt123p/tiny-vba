# Implementation progress

## M7A.4 continued — native RefEdit (2026-11-05)

Implemented [ADR 0060](decisions/0060-native-refedit.md): bounded inert native
class-table resolution, RefEdit MorphData, typed ordinary RefEdit text/drop state,
focus/validation and Change/DropButtonClick continuations. Picker ownership is
derived from live modal state, including parked callbacks; Reset stays inert.
The shared reduce/restore UI uses original vector marks and an injected host
selection capability. GridCraft supplies authorized worksheet references through
the ordinary text-input path and retains debugger visibility across Stop.
Shared CI, WASM, actual WASI (79 runtime/20 decoder cases), UI (19 actual
widget cases), native capture and web gates PASS. Ten expanded/collapsed renders
were inspected across light/dark/narrow/scaled/injected-host themes; caption and
restore marks use injected tokens and original vector drawing. Actual Chromium
RefEdit text input, modal handler Stop/resume and exact authored source retention
PASS with no page errors. Original M7 artifact hashes PASS. Both applications pin
shared product revision `21898dce`: GridCraft `3d9af79a` (implementation `4b46c982`)
and WordCraft `5aed1365`. Full CI (10/8 steps), twelve native and twelve actual
Node/WASM package cases per application, strict optional VBA/UI Clippy and
optional VBA web checks PASS. GridCraft's three embedded-form UI/consent cases
cover real range-picker/worksheet pointer input, multi-area references, authored
workbook changes, refusal of cell editing while picking, hidden VBE and trust
revocation; its host capture was inspected. WordCraft's thirteen existing VBA UI
cases and native-form widget test PASS; the latter covers hidden VBE, real button
input and Stop/F5 continuation without replay. Form windows/advancement are now
independent of VBE visibility in both hosts. Exact original XLSM/DOCM project bytes
survive save/reopen; source/resources remain separate. All repositories are
committed and pushed. This accepts the selected RefEdit and hidden-editor work;
M7A.4 remains IN PROGRESS for dynamic
collections, bindings, remaining control/platform/property/event/lifecycle
contracts and broader RefEdit semantics. Native edited saving/removal remain
M7A.5/.6. Fresh Office traces remain NOT_RUN under M8.5. Next: dynamic native
Tabs/Pages collection mutation and the remaining semantic ledger.

## M7A.4 continued — native Image/pictures (2026-11-05)

Implemented [ADR 0059](decisions/0059-native-image-pictures.md): cached native
Image records and inert bounded BMP/GIF/JPEG/ICO resources, ordinary Image and
stdole picture objects, typed assignment/clearing and shared immutable assets.
Clip/stretch/zoom, corner/centre alignment, repeat tiling and selected AutoSize/
border/effect properties use canonical fields. Nonfocusable Click preserves dirty
text focus and resumes once after Stop; inspection does not load or execute forms.
Native container focus now skips controls without tab properties and caches kind
reads, preserving existing VM budgets. Original source/resources stay independent.
Shared CI, WASM, actual WASI (77 runtime/19 decoder cases), UI (18 actual
widget cases), native capture and web gates PASS. Five light/dark/narrow/scaled/
injected-host image renders were inspected. Actual Chromium Image clicks, picture
assignment/alignment, modal Stop/resume and exact source retention PASS. Zero-size
zoom/tiling and one-pixel repeat regressions keep painting bounded. Both applications
pin shared product revision `4455df40`: GridCraft `7d23b368` and WordCraft
`e3a77b94`. Full CI (10/8 steps), eleven native and eleven actual Node/WASM
package cases per application, strict optional VBA/UI Clippy and optional VBA
web checks PASS. Both GridCraft embedded-form UI/consent regressions PASS.
The new package cases verify inert import, nominal Image/stdole picture access,
shared immutable resource identity, nonfocusable authorized clicks and exact
original XLSM/DOCM project bytes after save/reopen; WordCraft also verifies trust
revocation. All three repositories are committed and pushed. The reviewed
headless project -> image codec edge and explicit feature/license/budget policy
are recorded in the ADR and architecture.
M7A.4 remains IN PROGRESS for further picture format/metadata/platform/event/
presentation cases, RefEdit, dynamic collections, bindings, platform controls and
remaining lifecycle/property/default/error contracts in the unchanged ledger.
Native edited saving/removal remain M7A.5/.6. Fresh Office traces remain NOT_RUN
under M8.5. Next: RefEdit applicability/native-format and GridCraft range selection,
then the remaining semantic requirements. The published
[RefEdit persistence contract](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oshared/45556c0e-0f42-4c79-864e-503cd04e38ba)
uses TextBox MorphData with Always/Reduce drop-button properties; native class
resolution and the GridCraft-owned range picker remain unimplemented.

## M7A.4 continued — native MultiPage/Page (2026-11-05)

Implemented [ADR 0058](decisions/0058-native-multipage.md): bounded native
f/o/x reconciliation, distinct canonical Page/Pages identity, immediate child
Controls/Parent, indexed/named lookup, enumeration, default Value/SelectedItem,
selected-page input/focus gating and themed actual widget rendering. Physical
selection waits for dirty-field validation/cancellation across Stop/resume.
Internal navigation snapshots preserve object identity without raising VM limits.
The published indexed Click signature and same-selection suppression also fix
prior TabStrip behavior. Original source/resources remain independent and inert.
Shared CI, WASM, actual WASI (74 instance/16 decoder cases), UI (16 actual
widget cases), native capture and web gates PASS. Five light/dark/narrow/scaled/
injected-host captures were inspected. Actual Chromium page arrow input, modal
Stop/resume and exact authored source retention PASS, without page errors.
Original M7 fixture hashes and changed Markdown links PASS. A subsequent Page
API audit removes inherited Frame methods/events and UserForm page aliases, keeps
site geometry internal, and verifies Integer Index and explicit method refusal.
All repeat shared gates and actual Chromium page input/modal Stop/resume PASS.
Both applications pin shared product revision `18e412ca`: GridCraft
`00207414` and WordCraft `a0d78dc4`. Full CI (10/8 steps), ten native and ten
actual Node/WASM package cases per application, strict optional VBA/UI Clippy
and optional VBA web builds PASS. Both GridCraft embedded-form UI/consent
regressions PASS. New package cases exercise native page selection, selected-page
child input/refusal and exact original XLSM/DOCM resources after save/reopen;
WordCraft also verifies trust revocation. All three repositories are committed
and pushed. This closes the selected MultiPage increment, not M7A.4.
M7A.4 remains IN PROGRESS for dynamic page/tab mutation, broader native page/
Frame/resource/property/event/lifecycle semantics, Image/RefEdit, list/binding/
platform controls and the unchanged detailed ledger. Native editing/removal
remain M7A.5/.6; fresh Office observations remain NOT_RUN under M8.5. Next:
native Image resources/Picture layout, then Excel RefEdit and the remaining ledger.

## M7A.4 continued — native TabStrip (2026-11-05)

Implemented [ADR 0057](decisions/0057-native-tabstrip.md): selected native
TabStrip records, distinct nominal Tab/Tabs objects, canonical indexed/named
lookup, enumeration, selection/default properties and authorized actual widget
input. Change commits before Stop; Click resumes once. Tabs remain separate
from the form's Controls and do not contain nearby controls. Source/resource
bytes remain independent. Shared CI, WASM, actual WASI (69 instance/14 decoder
cases), UI (15 runtime widget cases), native capture and web gates PASS. Five
light/dark/narrow/scaled/injected-host captures were inspected, including visible
keyboard focus and preserved missing-glyph text. Chromium TabStrip arrow input,
modal Stop/resume and exact source retention PASS, alongside Frame, focus, list,
ComboBox, metadata/control-tip, editor and legacy form regressions. Application
integration pins shared revision `fe86257c`: GridCraft `4d044524` and WordCraft
`225b4ca4`. Both full CI gates (10/8), nine native and nine actual Node/WASM
package cases per application, strict optional VBA/UI Clippy and optional VBA
web builds PASS. Both GridCraft embedded-form UI/consent cases PASS. The new
package cases cover inert import, distinct tab identity/defaults, disabled input
refusal, trusted selection and exact original XLSM/DOCM resource preservation;
WordCraft also verifies trust revocation. M7 artifact/hash and changed Markdown
checks PASS. A malformed uncompressed character-count guard
and native/WASI regression prevent counts from aliasing the compression flag.
M7A.4 is not complete. MultiPage/Page, Image/RefEdit, tab structural
mutation and writable Index/Name, further property/event/binding/platform and
lifecycle semantics remain required. Native saving/removal remain M7A.5/.6.
Fresh Office traces remain NOT_RUN under M8.5. Next: native MultiPage f/o/x
reconciliation and Page identity/selection, followed by remaining control and
semantic requirements in the unchanged milestone.

## M7A.4 continued — native Frame containers (2026-11-05)

Implemented [ADR 0056](decisions/0056-native-frame-containers.md): nested native
Frame storages, immediate child Controls/Parent identity, nested ActiveControl,
scoped TabIndex/option groups and ancestor focus/validation continuations. The
renderer uses live relative geometry, parent clipping, effective input flags and
accessible group ancestry. Original source/resources remain inert and independent.
Shared CI, WASM, actual WASI (67 instance/13 decoder cases), UI (14 runtime
widget and two native designer/import cases), native capture and web gates PASS.
Light/dark/narrow/scaled/injected-host Frame captures were inspected. Chromium
editor, legacy form, native focus/text/numeric/boolean/list/ComboBox/metadata
regressions PASS; native Frame Unicode input, Tab/Shift+Tab, modal Stop/resume and source retention PASS.
Both forks pin shared product revision `e246411f`: GridCraft `fd631b3f` and
WordCraft `ed52222d`. Full CI (10/8 gates), eight native and eight actual Node 22
WASM package cases per fork, strict optional VBA/UI Clippy and optional VBA web
builds PASS. The Frame package cases verify inert import, nominal access, scoped
collections/Parent/ActiveControl, trusted nested input and exact original XLSM/
DOCM project bytes after reopen; WordCraft also verifies trust revocation.
Both existing GridCraft embedded-form UI/consent cases PASS. Original M7 artifact
hashes and changed Markdown targets PASS. No full milestone acceptance is claimed. Remaining M7A.4 includes MultiPage/TabStrip/Image/RefEdit, further
property/event/resource/binding/platform and lifecycle cases. Native edited saving,
.tvform removal and full acceptance remain M7A.5–M7A.7. Fresh Office observations
remain NOT_RUN under M8.5. Next: native TabStrip records/Tabs and MultiPage f/o/x page identity, collections
and selection, building on the bounded Frame ancestry path.

## M7A.4 continued — array readback and native metadata (2026-11-04)

Implemented [ADR 0054](decisions/0054-native-list-array-readback.md) and
[ADR 0055](decisions/0055-native-form-metadata-and-control-tips.md): detached native
List/Column snapshots, documented named parameters, inert metadata decoding and
ordinary Tag/ControlTipText/HelpContextID state with actual runtime control tips.
The 60 native form-instance, 11 decoder and 13 actual widget cases PASS. Hover
preserves focus/output; source/resource state stays independent. Light/dark/
narrow/scaled/injected-host tooltip renders were inspected. Shared CI, WASM,
actual WASI (60 instance/11 decoder cases), UI/native/web gates PASS. Chromium
native control-tip hover and List/Column snapshot round-trip/input/modal Stop/
resume PASS; editor smoke and legacy form browser regressions also PASS. The
actual normal-delay browser tooltip was visually inspected. Original M7 fixture/
hash checks and changed Markdown targets PASS. Both forks pin shared runtime
revision `5cda307d`: GridCraft `00a3b295` and WordCraft `33de3c18`. Full CI
(10/8 gates), seven native and seven actual Node 22 WASM package cases per fork,
strict optional VBA/UI Clippy and optional VBA web builds PASS. Existing list/
ComboBox cases now round-trip detached snapshots; the seventh package case
covers decoded metadata, nominal Control access, runtime writes, authorized
input and exact original XLSM/DOCM source/resource retention. The detailed
breakdown's obsolete FC0/FC1 next-step footer now matches current progress.
M7A remains IN PROGRESS for further array/list/binding semantics, native containers/
pages/images/RefEdit, applicable platform controls and remaining lifecycle/input/
property cases; actual Help/F1 routing is still unsupported. Native edited
persistence (M7A.5), .tvform removal (M7A.6) and full acceptance (M7A.7) remain
required. Fresh Office traces remain NOT_RUN under M8.5. Next after this packet:
native container/resource/collection work while retaining the complete inventory.

## M7A.4 continued — native list array loading (2026-11-04)

Implemented [ADR 0053](decisions/0053-native-list-array-loading.md): shared native
ListBox/ComboBox two-dimensional List loading, transposed Column loading and
indexed/current-row Column access. Source arrays retain their bounds/types and
remain independent. All cell conversion and limit checks precede canonical row/
selection replacement; existing ordinary Change continuations retain Stop/resume.
Four new authored regressions cover typed/nonzero-bound arrays, transpose/copy,
current-row selection, failed-load atomicity and pause/resource preservation.
Shared CI, WASM, actual WASI (56 form-instance cases), UI (12 actual widget
cases), native capture and web gates PASS. Chromium array-loaded ListBox and
transposed ComboBox input/modal Stop/resume PASS; editor smoke and legacy form
regressions also PASS. Array-loaded row/text renders were inspected. Original M7
fixture/hash checks and changed Markdown targets PASS. Both forks pin shared
runtime revision `3688113b`: GridCraft `9351fdef` and WordCraft `07774ca7`.
Full CI (10/8 gates), all six native and six actual Node 22 WASM package cases
per fork, strict optional VBA/UI Clippy and optional VBA web builds PASS. The
existing ListBox/ComboBox package cases now cover nonzero source bounds, bulk/
transposed loading, source-copy isolation and current-row Column reads before
authorized row/text input and exact original XLSM/DOCM project preservation.
M7A.4 remains IN PROGRESS. Next: whole-array readback/empty-list shape and remaining
loading/list semantics, followed by containers/resources/bindings. Native edited
persistence and .tvform removal remain M7A.5/M7A.6. Fresh Office comparison is
NOT_RUN under M8.5; the selected errors/selection rules are authored expectations.

## M7A.4 continued — native ComboBox (2026-11-04)

Implemented [ADR 0052](decisions/0052-native-combobox.md): editable/drop-list
records, shared ordinary list rows, Value/Text/selection/matching and VM-owned
drop state. Physical input uses existing authorization, focus/validation and
Change/Click/DropButtonClick continuations. F4 belongs to a focused ComboBox;
Enter closes its list and navigation keeps it open. Native resource, five runtime
and two actual widget cases PASS. Shared CI, WASM, actual WASI (52 form-instance
and ten decoder cases), UI/native/web gates PASS, with 12 actual widget cases.
Chromium ComboBox completion/F4/navigation/Enter/modal Stop/resume PASS, as do
list/focus/text/numeric/boolean, editor smoke and legacy form browser regressions.
Light/dark/narrow/scaled/injected-host popup renders were inspected. The popup
uses the injected theme rather than inheriting a mismatched global frame.
Original M7 fixture/hash checks and changed Markdown targets PASS. Both forks
pin shared runtime revision `6534f4fc`: GridCraft `2c84a6ad` and WordCraft
`04d2b58b`. Full CI (10/8 gates), all six native and actual Node 22 WASM package/
trust/input cases, strict optional VBA/UI Clippy and optional VBA web builds
PASS. The additional XLSM/DOCM regression checks MSForms.ComboBox access,
trusted row selection and UTF-16 free text with MatchEntry=None, exact original
project bytes after reopen, and WordCraft authorization revocation.
Full M7A.4 remains IN PROGRESS for further list APIs/bindings/resources,
containers/pages/images/RefEdit, platform and remaining lifecycle/property cases.
Next: whole-array List/Column APIs and remaining list semantics before container
resource/collection packets. Fresh Office comparisons remain NOT_RUN under M8.5.

## M7A.4 continued — native unbound ListBox (2026-11-04)

Implemented [ADR 0051](decisions/0051-native-unbound-listbox.md): cached native
ListBox records and selected columns, ordinary bounded indexed rows and selection,
AddItem/RemoveItem/Clear, Value/Text/columns and authorized actual pointer/key
input. Change commits before an authored Stop and Click resumes once. Stable
focus no longer resets the widget arrow-key filter on each render. Two decoder,
four native runtime and one actual widget regression pass. Shared CI, WASM,
actual WASI (47 native form-instance and nine decoder cases), UI/native/web gates
PASS, with ten actual form-instance widget tests. Chromium native list/focus/
numeric input and modal Stop/resume PASS; Unicode text, boolean, editor smoke
and legacy form browser regressions also PASS. Light/dark/narrow/scaled/injected-host
multi-instance list renders were inspected. Original M7 fixture/hash validation
and changed Markdown targets PASS. Both forks pin shared revision `c578131f`:
GridCraft `1cdbe9c3` and WordCraft `71048414`. Their full CI (10/8 gates), five
native and actual Node 22 WASM package/trust/input cases, strict optional VBA/UI
Clippy and optional VBA web builds PASS. The new package case checks typed
MSForms.ListBox access, trusted selection and exact original XLSM/DOCM project
bytes on reopen; WordCraft also checks authorization revocation. The inventory
now records the selected numeric and ListBox packets rather than its obsolete
unsupported labels. Full M7A.4 remains IN PROGRESS, including
ComboBox, further list/binding APIs, containers/pages/images/platform and
remaining lifecycle/property contracts in [the runtime record](m7a4-native-runtime.md).
Next: native ComboBox editable/drop-list behavior using the same canonical list
model, followed by the remaining list array/binding/container packets.

## M7A.4 continued — native numeric controls (2026-11-04)

Implemented [ADR 0050](decisions/0050-native-spinbutton-and-scrollbar.md): native
SpinButton/ScrollBar resource packets, Long properties, orientation/constants,
normal/reversed ranges and canonical Change/spin/drag-only Scroll events. UI
arrows/track/thumb/keyboard/repeat submit authorized ordinary callbacks; a paused
Change commits before the spin continuation without replay. Three native runtime,
two decode and two actual widget input cases cover the new paths. Native CI,
WASM/WASI and UI/native/web gates PASS; all 43 instance and seven decode cases
PASS in native/WASI execution, with nine actual instance/input widget cases.
Chromium numeric, Unicode text, boolean and focus/cancel/Stop/resume gates PASS.
A focus regression found by immediate typing after resume is fixed by preserving
stable paused control IDs and restoring focus before input. Stationary dragging
also remains inert after a Scroll handler resets Value. Host glyph policy remains
unchanged; tests explicitly opt into placeholders when fonts lack their Unicode.
Both forks pin accepted shared revision `2ce3f205`: GridCraft `53f230cd` and
WordCraft `37106fc3`. Full CI (10/8 gates respectively), four native and actual
Node 22 WASM package/trust/input cases, strict optional VBA/UI Clippy and optional
VBA web builds PASS. The additional XLSM/DOCM case checks typed numeric controls,
trusted small/large steps and exact original project bytes after reopen. An
unchanged WordCraft proofing timing case passed on an isolated retry and the
subsequent full serial CI run. Chromium output was visually inspected; M7
fixture/hash validation and changed Markdown links PASS. Unrelated GridCraft
ribbon work remains outside these commits.

Full M7A.4 remains IN PROGRESS for the unchanged [remaining scope](m7a4-native-runtime.md).
Next implementation packet: unbound native ListBox/ComboBox, beginning with
MS-OFORMS MorphData masks/DisplayStyle and ordinary canonical list/selection
properties; follow through authorized UI input, debugger continuation and both
forks. Container/page/image/binding/platform and remaining lifecycle contracts
still require implementation; these selected numeric/text increments do not
constitute full milestone completion.

## M7A.4 continued — native TextBox text and selection (2026-11-04)

Implemented [ADR 0049](decisions/0049-native-textbox-selection-and-input.md):
UTF-16 selection/replacement, multiline/wrap, Enter/Tab behavior, field-selection
constants and native/custom password masking. Input commits text and selection
atomically before ordinary Change callbacks and preserves typed validation/focus.
A real Unicode widget regression validates caller-selected glyph fallback without
changing host fonts/global policy. Four new native instance and one decode case pass; actual multiline/selection,
persisted password and isolated-surrogate widget cases cover the UI. Shared
`cargo xtask ci`, `wasm`, `wasm-run`, `ui`, `ui-native` and `web` PASS, including
40 native instance tests in native/WASI execution and seven actual widget cases.
Chromium native text/password/focus/boolean input and editor/legacy form gates
PASS; rendered text and password output was inspected. Original M7 hashes and
changed Markdown links PASS. Both forks pin accepted revision 1411bd16: GridCraft 76c135ba and WordCraft
49841a60 PASS full CI, three native/actual Node 22 WASM package/trust/input
cases, strict optional VBA/UI Clippy and optional VBA web builds. The new
XLSM/DOCM case covers native text flags/mask, typed selection/replacement,
trusted input and exact original VBA bytes on reopen. Unrelated GridCraft
ribbon edits remain outside the integration commits.
Full M7A.4 remains IN PROGRESS with its unchanged remaining control/property/
binding/platform requirements in [the runtime record](m7a4-native-runtime.md).

## M7A.4 continued — native container focus and validation (2026-11-04)

Implemented [ADR 0048](decisions/0048-native-container-focus-and-validation.md):
ActiveControl/SetFocus and selected Enter/Exit/BeforeUpdate/AfterUpdate delivery
with typed mutable ByVal cancellation, ordinary input/focus frames and authorized
UI commands. Cancellation keeps canonical and widget focus together; native
text/click input follows validation. Programmatic writes do not schedule update
events. Four new IDE cases and an actual Tab/cancel/Shift+Tab test PASS locally.
Shared `cargo xtask ci`, `wasm`, `wasm-run`, `ui`, `ui-native` and `web` PASS.
All 36 native instance cases PASS natively and in actual WASI execution; four
actual instance/input widget cases PASS. Chromium native typed focus and boolean
input gates PASS, including subsequent typing after cancellation and event order
through Stop/resume. Native/legacy forms and general browser smoke regressions
also PASS; rendered cancellation output was inspected. Default/Cancel keyboard
activation preserves the current control focus. Original M7 fixture hashes remain
intact. Both forks pin accepted shared revision 69d433c2: GridCraft fdeee9aa and
WordCraft a7c188dc PASS full CI, native/actual Node 22 WASM package/trust/input
checks, strict optional VBA/UI Clippy and optional VBA web builds. The package
case additionally asserts initial TextBox and subsequent canonical checkbox focus,
with trust refusal and exact VBA bytes preserved. Unrelated GridCraft ribbon edits
remain outside these integration commits. Full M7A.4 remains
IN PROGRESS for its unchanged [remaining scope](m7a4-native-runtime.md).
Next smallest runtime packet: TextBox selection/keyboard/property behavior and
native multiline/password flags, preserving typed cancellation and UI focus.
Further native control/container/list/resource families and worksheet ActiveX
remain required; the implemented flat profile is not a milestone scope reduction.

## M7A.4 continued — Forms type bindings and library access (2026-11-04)

Implemented [ADR 0047](decisions/0047-native-forms-nominal-types-and-library-access.md):
generic IR 11 class-interface identity/event exposure, native MSForms provider
metadata, UserForm/Control/Controls/leaf type storage and ordinary mutable return
objects. WithEvents uses canonical native controls and hides private bridge
events; generic non-source connections refuse with catchable error 459. Explicit
source-function aliases supply VBA.UserForms lookup/enumeration while honoring
project shadows and existing references. Six generic interface, four alias and
32 native instance regressions PASS natively and in actual WASI execution. Shared
`cargo xtask ci`, `wasm`, `wasm-run`, `ui`, `ui-native` and `web` PASS. Chromium
smoke, legacy forms and native forms PASS. The boolean-input harness additionally
checks an authored MSForms.CheckBox WithEvents handler, typed control enumeration
and qualified VBA.UserForms access during Initialize. Input, Stop/resume, exact
source and page-error checks PASS; the rendered result was inspected. Both forks
accepted immutable revision d94a7a33: GridCraft f7ac1279 and
WordCraft ee3d8a90 PASS full CI, native/actual Node 22 WASM package/trust/input
tests, optional VBA/UI Clippy and optional VBA web builds.
Broader native focus/control/property/platform scope
remain open in [the runtime record](m7a4-native-runtime.md); M7A.4 is not complete.

Both forks accepted the preceding enumeration/As New revision b95a38a2:
GridCraft 2b2615d5 and WordCraft d67bb55d pin it and PASS full CI, native/actual
Node 22 WASM package/trust/input cases and strict optional VBA/UI Clippy.
The authored package case verifies no loaded form before first As New reference,
canonical control/UserForms enumeration and exact original VBA bytes on reopen.
Unrelated GridCraft Developer-ribbon edits were preserved outside these commits.

## M7A.4 continued — lazy class/form variables (2026-11-04)

Implemented [ADR 0046](decisions/0046-lazy-project-class-variables.md): scalar
As New for constructible project classes and native forms, including canonical
module/static/instance variables, ByRef preparation, Nothing re-creation and
constructor Stop/failure/retry. IR 10 validates exact nominal storage and refuses
older consumers. Debugger inspection stays inert; there is no location/object
registry or external activation. Designer launch now explicitly constructs native
forms whose predeclared attribute is False. A modal callback regression also
proves enumeration-snapshot lifetimes in parked frames. Ten generic and 25 native
instance cases PASS natively and in WASI. Shared `cargo xtask ci`, `wasm`,
`wasm-run`, `ui`, `ui-native` and `web` PASS. Chromium smoke, legacy forms and
native forms PASS; the native boolean-input harness additionally runs an authored
As New probe after the handler Stop, constructs/releases it without replay and
preserves exact source and expected values. The milestone remains
IN PROGRESS for the unchanged [remaining scope](m7a4-native-runtime.md).

## M7A.4 continued — class and native collection enumeration (2026-11-04)

Implemented [ADR 0045](decisions/0045-class-and-native-collection-enumerators.md):
IR 9 validates standard DISPID_NEWENUM metadata and invokes Object-returning
getters through ordinary frames. Native Controls/UserForms now support bounded
For Each, empty/nested loops and canonical child/form identity. Retained native
enumerators own their collections; active/parked snapshots keep their objects
alive through unload, Stop and resume. Forwarding chains are bounded and retain
normal host permissions, error, wait and cancellation behavior. Document classes
without a project enumerator continue forwarding to their bound host owner.
No .tvform extension or second mutable object store is added.

Authored regressions: five generic enumeration tests and 22 native form instance
tests, including three new collection ownership/iteration cases. The full M7A.4
remaining scope is preserved in [the runtime record](m7a4-native-runtime.md);
focus/typed events, qualified and nominal bindings, additional implicit/alias cases,
As New arrays, further
control families and platform packets remain required. Fresh Office evidence
remains NOT_RUN under M8.5. Shared `cargo xtask ci`, `wasm`, `wasm-run`,
`ui`, `ui-native` and `web` PASS. All five generic and 22 native-instance
regressions PASS natively and in actual WASI execution. Existing Chromium smoke,
legacy forms, native forms and native boolean input harnesses PASS. A further
authored browser fixture enumerates all seven controls during Initialize before
the same modal input/Stop/resume/source-identity checks; it also PASSes. The
rendered result was inspected. M7's two authored fixtures retain all twelve hashes.
Fork immutable-pin integration and package/bridge enumeration checks follow.


## M7A.4 continued — native control properties and collections (2026-11-04)

Implemented [ADR 0043](decisions/0043-native-controls-and-project-inspection.md)
and the [current runtime scope](m7a4-native-runtime.md): native CheckBox,
OptionButton and ToggleButton decode/display/input/run, canonical Boolean/Null
state and ordinary Change/Click delivery, root option groups and Locked; Integer
TabIndex with reordering/clamping; Default/Cancel exclusivity and Enter/Escape;
TextBox MaxLength; Controls Count/index/name lookup; loaded UserForms Count/Item/
Add without constructing defaults; Object-valued Load/Unload; explicit construction
of non-predeclared forms; contextual property names. The form wrapper now preserves
embedding intrinsic providers/waits. Generic bounded read-only project views avoid
a second mutable object registry and do not execute getters or private fields.
New native families deliberately have no legacy .tvform encoding.

[ADR 0044](decisions/0044-userform-window-and-code-lifetime.md) separates window
and referenced code lifetimes: named control properties reload initial controls,
unload releases the matching default root while aliases retain code fields, and
Terminate follows actual last-reference cleanup. Bounded internal continuations
keep this release outside VM slices and protect document roots; cancelled close
and paused Reset retain their normal ownership semantics.

M7A.4 remains IN PROGRESS. Additional default/control alias edge cases, typed focus and
validation/events, qualified collection access/type binding, the remaining native
control/resource/binding/presentation families and platform packets remain open.
M7A.5 edited native saving and M7A.6 removal are unchanged. No fresh Office run or
full milestone sign-off is claimed. The runtime record keeps the completion gaps
explicit rather than reducing the milestone to the completed subset.

Validation: shared `cargo xtask ci`, `wasm`, `wasm-run`, `ui`, `ui-native` and
`web` PASS. Nineteen native instance/property/lifecycle tests, the existing fifteen
form execution tests, three actual multi-window/keyboard/boolean widget tests,
new native decode/parser tests and both generic VM view/root-release regressions
PASS, including actual WASI execution. Existing browser smoke/form/native harnesses
PASS. A new six-family Chromium harness imports original native resources, clicks
CheckBox/OptionButton/ToggleButton widgets, verifies group exclusion through the
real VBA handler, pauses/resumes without replay, and preserves exact source.
Light/dark/narrow/scaled/host renders were inspected; boolean glyph contrast was
corrected using injected tokens. The original M7 fixture checker retains all twelve
hashes. No Office-produced artifact or downloaded macro is included/executed.
Both forks now pin shared `d2de831892e5d2b17557de6b7abb00a83fdfc292`:
GridCraft `8c7acc9a`, WordCraft `81b49a81`. Their new native boolean-instance
package tests PASS natively and in actual Node 22 JavaScript/WASM, preserving
original XLSM/DOCM project bytes. Real editor dispatch regressions PASS for
untrusted launch and revoked-trust instance input, including pure VBA handlers.
Both optional VBA web builds and strict optional VBA/UI Clippy PASS. GridCraft
full CI passes all ten steps with the existing licensed craft-fonts input
(`aced4137c022f1f587624d48e81ec77e1914a9cc`); the missing-input run failed two
glyph tests. WordCraft full CI passes all eight steps with RUST_TEST_THREADS=1;
a PDF timing check failed under concurrent load, then passed in isolation and
in the complete rerun without code/threshold changes. These are regression and
selected-profile integration results, not full M7A.4 completion or fresh Office
observations.

## M7A.4 started — explicit native UserForm instances (2026-11-04)

The repositories were clean and synced before work began; M7A.3 was already
committed/pushed in GridCraft (6d061ede) and tiny-vba (89b0b90).

Implemented the first shared FC3.5 packet under [ADR 0042](decisions/0042-userform-instance-routing.md):
explicit Set/New native forms now construct independent controls and handlers,
coexist with default instances, and have separate runtime windows, focus and
input identities. Modeless window ownership and normal last-reference cleanup
reuse generated ordinary class state. Modal/nested callbacks retain their actual
instance through debugger breaks and parked callers. Reset stays inert and source/
design/native resources are preserved. Generic VM receiver/parked-frame lifetime
roots fix premature collection without Office/UI dependencies or a new IR version.

See [the scope/evidence record](m7a4-userform-instances.md). M7A.4 is not complete:
default-instance replacement/alias details, the loaded UserForms collection/type
surface, exact lifecycle/focus/property/event behavior, broader controls and fork
integration remain open. Fresh Office observations remain NOT_RUN.

Validation: shared `cargo xtask ci`, `wasm`, `wasm-run`, `ui`, `ui-native` and
`web` PASS. Six new IDE native-instance tests, one actual egui multi-window/input
test and one generic VM receiver-lifetime regression PASS; existing form/default/
host-wait tests remain green. Existing browser smoke, form and native-form harnesses
PASS, including a real modal handler breakpoint/resume and unchanged source.
These browser regressions do not constitute a new multi-instance Chromium case.
Light/dark/narrow/scaled/injected-host multi-instance views were rendered and
inspected. The original M7 check retains all 12 artifact hashes. No external
macro/Office-produced binary or vendor asset was added. Consumer fork pins are
unchanged pending their integration packet; fresh Office observations stay NOT_RUN.

## M7A.3 — native worksheet Form Controls (2026-11-04)

Completed FC1–FC2 for [worksheet-form-controls-v1](m7a-worksheet-controls.md)
under [ADR 0041](decisions/0041-worksheet-form-controls.md). GridCraft now joins
worksheet/VML/ctrlProp/DrawingML identities into one existing Shape for Button,
CheckBox, OptionButton, DropDown, ListBox, ScrollBar, Spinner, Label and GroupBox.
Source conflicts stay inert. The model owns bindings/groups, engine commands own
state/link writes, undo/recalc and placement, and the optional adapter/shell own
selected Shapes/ControlFormat APIs and accessible native input. There is no new
VM capability, shared dependency pin, mutable control registry or custom format.

Assignments validate consent, source target and owner before mutation, commit the
linked value before running one Sub, and retain Application.Caller/owner scope
through break/resume. Busy/paused/waiting inputs refuse; Reset never replays them.
Property writes stay inert. Native save patches known worksheet/VML/property fields,
retains untouched parts/unknown XML and original VBA project data, and supports
create/edit/delete/reopen. This is worksheet OOXML persistence, not UserForm resource
encoding or binary VBA project writeback.

Validation: GridCraft `cargo xtask ci` PASS (all 10 steps), optional VBA/UI
clippy with `-D warnings` PASS, and regenerated/reference-checked VBA catalogue
PASS (5,996 pinned pages). Focused native suites PASS: seven engine, ten XLSX,
one real VBA, six actual-widget/trust/debugger/owner tests and the print regression.
The broader optional UI regression run passed its existing tests; the new widget
suite passed after dismissing the expected consent message through the actual UI.
Actual Node 22.23.4 JS/WASM PASS: seven engine, ten XLSX and one VBA control test.
The optional VBA web build PASS (existing wasm-only dead-code warning retained).
Light/dark/scaled native captures were rendered and inspected; a worksheet-caption
contrast issue under dark chrome was corrected. Shared `cargo xtask ci`, `wasm`
and `wasm-run` PASS; the original M7 fixture check retains all 12 hashes. Relative
Markdown links and whitespace checks PASS. No WordCraft implementation changes
were needed; its worksheet-free host stays unchanged.
Original independently authored Rust fixtures carry repository MIT OR Apache-2.0;
no Office-produced file, asset or downloaded macro is included/executed. Fresh
Office observations remain NOT_RUN under M8.5.

Next: **M7A.4**, native UserForm instance/lifecycle/input/property semantics and
broader standard MSForms controls. M7A.5 native UserForm editing/persistence,
M7A.6 removal and M7A.7 full acceptance remain open; worksheet ActiveX is unsupported.

## M7A.1 and M7A.2 — native inventory/contracts and UserForm import/attachment (2026-11-04)

Implemented `native-userform-import-v1` under
[ADR 0040](decisions/0040-native-userform-import-and-attachment.md) and the
[control/property/event/format/host contracts](m7a-control-contracts.md).
The [acceptance record](m7a-import-acceptance.md) distinguishes native export,
embedded import, shared/host attachment and later persistence/semantics work.

`vba-project` now returns distinct native UserForm components and bounded,
inert VBFrame/f/o/site/control data for immediate Label/TextBox/CommandButton
children. Export import accepts `.frm` plus explicitly supplied `.frx`, with no
filesystem access or library activation in the decoder. Embedded import separately
uses MODULESTREAMNAME storage, matching BaseClass/VB_Name/frame identity and exact
source attributes/line endings. Original resources remain independent snapshots.
Unknown controls/hierarchy/resources/behavior masks and malformed inputs refuse.
Existing protected-project, PROJECTlk, container/decompression and work limits remain.

The shared IDE maps native controls into its existing validated designer/runtime,
Project Explorer, View Object/View Code, Properties and debugger. A reviewed
IDE -> project edge is the only production dependency change. Native export
attributes remain in authored source; generated support supplies missing attributes
without duplicating them. Candidate handler validation precedes project mutation.
Failed imports keep the project and source-ID allocation unchanged. Native forms
cannot fall back to .tvform export or acknowledge modified native resources as
saved without the future encoder.

Standalone native IO reads the bounded validated companion; browser IO accepts
explicit paired selection. GridCraft and WordCraft attach selected embedded forms
inertly to their owner project and route native import through existing source
workflows. Explicit trust still precedes host execution; modal handlers and
breakpoint/resume/Reset use the existing runtime. Authored XLSM/DOCM regressions
retain original project bytes on unchanged save/reopen. This is original-byte
retention, not edited native persistence.

Validation: shared `cargo xtask ci`, `cargo xtask wasm`, `cargo xtask wasm-run`
and `cargo xtask ui` PASS; all nine new core codec/IDE tests run under actual WASI.
Native imported-form renders PASS (seven light/dark/narrow/scaled/host/runtime
captures, inspected), as do `cargo xtask ui-native` and an additional real native
`.frm`/`.frx` capture. `cargo xtask web` PASS with isolated wasm-bindgen 0.2.126
matching the repository pin. Chromium native-pair upload/designer/modal-handler/
Stop/continue/source-integrity PASS; existing browser smoke and legacy-form
regressions PASS. No page errors were observed.

GridCraft full CI PASS (10 steps); WordCraft full CI PASS (8 steps). New native
XLSM/DOCM attachment regressions PASS both natively and in actual Node 22.23.4
JavaScript/WASM (one per fork). Optional VBA UI tests PASS (9 GridCraft, 12 WordCraft), as does Word's `vba`
browser build. Original M7 selection still verifies all 12 hashes unchanged; this integrity
check does not execute the historical macros. Relative documentation links and
whitespace checks PASS. Consumer source pins are updated to the shared implementation
commit; final pin resolution uses the local commit in Cargo's git cache, without
committing local source overrides.

### Completed M7A.1 and M7A.2 plan scope

- **M7A.1 — Inventory and contracts (FC0):** completed actual path audit, native
  control/property/event/format/host matrix, lawful original fixture provenance,
  platform/capability/dependency/identity/resource/atomicity/persistence contracts
  and acceptance ledger. Broader runtime packets retain explicit unsupported and
  unobserved dispositions rather than invented Office behavior.
- **M7A.2 — Native UserForm codecs and attachment (FC3.1–FC3.4):** completed for
  the reviewed flat Label/TextBox/CommandButton profile, with separate native
  export-format and embedded import evidence, exact authored source and original
  resources, shared designer/debugger/runtime and standalone/fork attachment.
  Native export writing remains M7A.5. Containers/additional control/property
  variants and full native default-instance/multi-instance/lifecycle parity remain
  M7A.4; none are implied by import acceptance.

At the M7A.2 checkpoint, worksheet source identity/reconciliation and rendering
were the next task; M7A.3 below completes that worksheet profile. Shared next work is FC3.5/FC4 native
lifecycle and broader property/control packets. M7A.5 native edited persistence,
M7A.6 `.tvform` removal and full M7A acceptance remain open. Fresh Office build/
OS/locale comparisons remain NOT_RUN under M8.5.


## M7A started — inventory and first native Button metadata slice (2026-11-04)

Completed the initial FC0.1 code-path audit and published the
[control/property/event seed matrix and work ledger](m7a-inventory.md).
[ADR 0039](decisions/0039-m7a-native-control-metadata.md) records the first native
reader contract and owner boundaries. M7A.1 remains in progress: detailed control
signatures and native UserForm resource/encoding contracts are still needed.

GridCraft XLSX now exposes `read_worksheet_controls` over the existing preserved
worksheet/VML/ctrlProp graph. It joins native shape IDs/Office spid, names, caption
text and macro sources without creating a second model or granting execution.
A strict namespace-aware reader bounds input, stored text, nodes, depth, graph
metadata and control counts; DTDs, spoofed namespaces, ambiguous identities,
conflicting assignments and malformed graphs refuse without mutation. Caption
mixed text/entities preserve order. Known x14 markup-choice/fallback selection
produces one record. Preserved fragments now retain their inherited default XML
namespace as well as prefixes, fixing a blocker for proper native namespace checks.

Eight original native package regressions and the same eight actual Node 22
JavaScript/WASM tests PASS. They cover save/reopen/rename, exact metadata, explicit
unsupported controls, negative graph/namespace cases and resource limits. This is
partial FC1.1 decode evidence, not display/input/run or edited native save support.
Validation: GridCraft `CRAFT_FONTS_DIR=/tmp/gridcraft-acceptance-fonts cargo xtask ci`
PASS (all 10 steps, including full tests, strict Clippy and WASM). The focused
`cargo test -p gridcraft-xlsx --locked form_controls` and actual
`wasm32-unknown-unknown` tests via `wasm-bindgen-test-runner` PASS (8 each;
Node 22.23.4). Shared `cargo xtask ci`, `cargo xtask wasm` and
`cargo xtask wasm-run` PASS. `python3 -O tools/check_m7.py` verifies all 12
historical artifact hashes unchanged; it does not execute the M7 fixtures.
Both repositories pass `git diff --check`. WordCraft code is unchanged.

The audit also found DrawingML source IDs are replaced by model IDs. Preserve and
reconcile those IDs/assignments before enabling native Button rendering/activation.
Native UserForms (`.frm`/`.frx` and embedded resources), full control semantics,
required native encoding/writeback and `.tvform` removal are still open. No legacy
format extension, new user-facing format, VM/IR capability or fresh Office result
was introduced. Next: complete native worksheet representation reconciliation;
shared native UserForm resource contracts can proceed in their owning layer.

## M7A added — native Office forms and removal of .tvform (2026-11-04)

Owner direction inserts **M7A before M8** and supersedes the native-forms deferral
and custom `.tvform` format direction. [ADR 0038](decisions/0038-native-office-forms-and-compatibility-only.md)
records the standing policy: mirror Excel/Word behavior, formats and workflows;
no new user-facing dialect, exchange format or invented control semantics.
The [main plan](implementation-plan.md) defines stable M7A.1–M7A.7 tasks; the
[activated forms breakdown](office-forms-controls-implementation-plan.md) retains
FC0–FC5 IDs and adds FC6 for `.tvform` removal.

Scope includes native `.frm`/`.frx` and embedded UserForm resources, worksheet
forms/controls, Office instance/property/event behavior, native editing/export/
package persistence, and removal of the custom codec/APIs/routes/filters/examples.
The necessary reviewed encoder/writeback/signature work is an M7A prerequisite;
original-byte preservation and a custom export cannot stand in for saving edits.
Reuse compatible shared designer/runtime/debugger internals and replace useful
regressions with native fixtures. Historical M2/M7 evidence remains unchanged;
M7A has **not started implementation**, and `.tvform` is still present today.

Next: **M7A.1 / FC0.1** inventory and native compatibility contracts, followed by
the worksheet Button slice and the reviewed native UserForm codec work. M8 task
IDs remain stable; its release acceptance now depends on M7A. Fresh Office
comparisons remain NOT_RUN under M8.5. This is a planning/documentation change;
no native support or format removal is claimed from it.

Validation: `cargo xtask ci`, `cargo xtask wasm` and `cargo xtask wasm-run` PASS.
New scope/plan links and unique M7A.1–M7A.7/M8.1–M8.5 definitions verified;
`python3 -O tools/check_m7.py` and `git diff --check` PASS. No product code,
dependency or existing fixture/evidence hash changed.

## M7 — authored unchanged-document/package acceptance (2026-11-04)

Completed M7.1–M7.4 for `m7-unchanged-v1`; see the
[self-contained acceptance record](m7-acceptance.md),
[reviewed specimen selection](../tests/fixtures/m7/selection.json) and
[ADR 0037](decisions/0037-m7-package-preservation-and-writeback.md).
Two original, licensed generated packages carry fixed source hashes, explicit
capabilities/references and independent typed expectations. Both real forks verify
load, compile, authorized execution and save/reopen separately, preserving raw
source/hidden metadata and original binary project identity. Both supplied
context matrices have per-row PASS/UNSUPPORTED dispositions with authored bases;
fresh Office comparisons remain NOT_RUN under M8.5.

GridCraft now captures the bounded VBA-related payload/relationship graph and
refuses to reuse it with a replaced captured project. WordCraft attaches decoded
standard/class/ThisDocument source inertly and refuses invalid/unsupported owners
transactionally. Its adapter and editor enforce execution trust before VBA entry
while permitting untrusted source editing/compile/export and resetting live work
on trust revocation. WordCraft explicitly drops/logs package-wide OPC signature
roots on rewrite; VBA signatures remain opaque transport data. No cryptographic
verification, edited-source CFB encoder, binary form import or new runtime/IR
capability is claimed. M7.3's deliverable is the reviewed writeback design.

Validation: `cargo xtask m7` PASS (5 GridCraft package cases, 6 WordCraft package
cases, 2 package graph regressions; 12 reviewed artifact hashes). Both full fork
CI gates and their WASM stages PASS; GridCraft requires the existing optional
`CRAFT_FONTS_DIR=/tmp/gridcraft-acceptance-fonts` input for its two glyph-dependent
UI tests. Shared CI/WASM/WASI PASS. Actual Node 22 JavaScript/WASM package tests
PASS (5 Excel, 6 Word); host Node 18 aborts in the test runtime. Optional VBA UI
checks PASS (GridCraft 9 filtered integration tests; WordCraft 12 rendered/model
tests); final Word optional-feature strict Clippy and browser-shell compilation
PASS. Exact commands, scope and exclusions are recorded in acceptance.

At M7 sign-off, the next code task was M8.2's payload accounting review.
The subsequent owner-directed M7A addition takes precedence. The M8.5 fresh Office comparison remains an
explicit release gate. No external corpus candidate was newly executed.

## README and license alignment (2026-11-03)

Rewrote the public README around ArtCraft's clean-room VBA purpose, the Excel
compatibility adoption requirement, familiar editor/debugger workflows, and
opt-in GridCraft/WordCraft walkthroughs with small authored macros. Added original,
regenerated UI renders with provenance in `docs/images/`. Current support and
remaining package/reference-comparison gates remain explicit; no new runtime or
Office compatibility support is claimed. The next implementation task remains M7.1.

On explicit maintainer direction, aligned licensing with GridCraft and WordCraft:
MIT OR Apache-2.0. Preserved the existing MIT notice in `LICENSE-MIT`, added
`LICENSE-APACHE`, and updated `LICENSE`, Cargo metadata and the repository gate.
[ADR 0035](decisions/0035-dual-license.md) supersedes the original MIT-only decision.

Validation: `cargo xtask ci`, `cargo xtask wasm` and `cargo xtask wasm-run` PASS.
`cargo run --locked --offline -p vba-ui-egui --example snapshots` PASS; the two
selected renders were visually inspected. Local README/provenance links resolve,
the MIT notice is unchanged and the Apache text matches GridCraft. The README's
CLI execution example returns `Function(Long(42))`. `git diff --check` PASS.
Windows checks used a workspace-local TEMP/TMP and the installed Python 3.13
directory on PATH after the default WindowsApps Python alias failed. No fresh
GridCraft/WordCraft GUI walkthrough or Microsoft Office comparison was run.

Current status, 2026-11-04: the classic-editor/debugger profile is implemented;
its historical milestone is M10. Runtime host profile remains m5-host-v1 with
verified IR 1–8 and the accepted intrinsic extension. **M1–M6, M6A and M7 are
complete for their declared profiles.** The [remaining plan](implementation-plan.md)
contains M7A followed by M8. Fresh Excel/Word comparisons remain NOT_RUN and are a release gate
under M8.5. Office objects, document events, trust and packaging remain fork-owned.
The [M7 acceptance record](m7-acceptance.md) defines original-project preservation,
source exports and unsupported edited binary writeback separately.

## Suspended generic host calls (2026-11-03)

[ADR 0036](decisions/0036-suspended-host-project-calls.md) adds bounded public-procedure,
document-event and publisher-event calls while a project waits on its exact host token.
Typed results, callback ByRef outputs, catchable runtime errors and persistent state survive
parking; Reset cancels every wait. Debugger and cumulative limits remain authoritative.
IR 8 `host-procedure-defaults-v1` retains optional standard-procedure defaults; older targets
retain their contract. `prepare_host_events` prepares idle subscription context without running
user code. Qualified external type members accept keyword spellings. No ambient IO or Office
objects were added. M7/package acceptance and reference comparisons remain separate work.

## Deferred Office forms and controls plan (2026-11-02)

Owner requested a separate future [implementation plan](office-forms-controls-implementation-plan.md)
for Excel worksheet Form Controls, native VBA UserForm import, broader shared
controls and persistence. Status: DEFERRED / NOT STARTED, outside the current
M7–M8 delivery profile. FC0–FC5 define ownership, dependencies and acceptance;
they do not add implemented capabilities or change M2/M6 sign-off. M7.1 remains
next. When activated, start with FC0.1 and then the worksheet Button slice.
Read-only inspection of local Food.xlsm identified a worksheet Form Control
assigned to CopyIngredients; no workbook macro was executed or modified.

## Intrinsic milestone inserted before M7 (2026-11-01)

Owner direction added **M6A — VBA intrinsic-function coverage** before M7,
preserving existing M7/M8 IDs and M6 sign-off. The
[intrinsic-function document](vba-intrinsic-functions.md) defines the seed
inventory, audit requirements, implementation groups, host/platform boundaries
and acceptance. [ADR 0033](decisions/0033-intrinsics-before-package-acceptance.md)
records the sequencing change. Audit existing partial support alongside missing
functions; worksheet equivalents do not establish VBA semantics. M6A.8 and M6A
acceptance completed 2026-11-02; see [the acceptance record](m6a-acceptance.md)
and [work ledger](m6a-work-ledger.md). Fresh Office observations remain M8.5.

The Luna handoff: [implementation framework](m6a-implementation-framework.md)
maps inspected code paths to descriptor/binder/IR/runtime/IDE responsibilities,
specifies IR 8 extended calls without changing legacy contracts, and breaks work
into F0–F4 foundations plus small function packets and typed StrComp vectors.
[ADR 0034](decisions/0034-intrinsic-implementation-framework.md) records the
design. Its packet guidance is retained for future intrinsic work; packet status
and evidence are recorded in the [completed ledger](m6a-work-ledger.md).

## Reported Food.xlsm For-counter correction (2026-11-01)

Read-only source extraction confirmed CopyIngredients uses implicit Variant
counters Index, d and t. The binder's legacy Integer/Long-only guard rejected
those valid numeric loop controls. It now accepts scalar Variant controls while
rejecting arrays and nonnumeric declared control types. Lowering evaluates and
normalizes Variant bounds once through shared numeric conversion/arithmetic
before writing the counter; no IR opcode/profile change or host-specific code.
Authored regressions cover 4×5×40 nested iterations, explicit/implicit controls,
Next, Exit For, zero iterations, numeric strings, invalid/Null bounds and
checkpoint pause/resume with exact counter tags. The existing positive integral
literal-Step limitation and numeric arithmetic limits remain explicit.

The user workbook was not executed or modified. Clipboard/selection/sort host
operations in its source remain separate compatibility work. This is a targeted
language correction; M7 remains untouched. Validation: `cargo xtask ci` PASS
(419 workspace tests), `cargo xtask wasm` PASS, `cargo xtask wasm-run` PASS
(222 actual WASI tests, including the full execution regression suite).
See [ADR 0031](decisions/0031-variant-for-counters.md). Shared fix `f680441`
is published on main and pinned in both forks. Final adapter checks pass
(GridCraft 75, WordCraft 28), as do both full application CI gates.
Next smallest compatibility task: StrComp, followed by a separate audit of the
workbook’s Excel selection/clipboard/sort calls; no full-workbook run is claimed.

## M6 implementation and acceptance audit (2026-11-01)

Implemented generic live host-object WithEvents subscriptions (IR capability
`host-events-v1`), and WordCraft delivery for DocumentBeforeClose,
DocumentBeforeSave and WindowSelectionChange. Delivery validates all signatures
before effects, retains connection order/ByRef state through pause/resume and
uses one host undo batch. WordCraft also fixes Alt+F11 double handling, asynchronous
source-save acknowledgements and selection-event recursion after debugger pauses.

M6.9–M6.11 now parse owned module/member metadata and bounded class headers,
retain canonical raw source, project metadata out of normal code views and map
editing/debugging back to raw coordinates. Component code names, descriptions,
read-only Instancing and a raw metadata inspector use the same shared editor.
The forks refuse document code-name renames explicitly. The exact authored Macro2
invocation attribute executes without metadata checkpoints in shared and real
WordCraft/GridCraft sessions. See [ADR 0029](decisions/0029-host-event-subscriptions.md),
[ADR 0030](decisions/0030-attribute-metadata-and-code-projection.md) and the
[M6 acceptance record](m6-acceptance.md) for evidence and current integration gates.

Shared CI, WASM, actual WASI, UI, native UI and web-build gates pass. Both forks’
full CI, Git-pinned adapter suites and optional-feature checks pass. M6 is signed
off for this declared integration profile by owner direction on 2026-11-01.
Fresh Office observations remain NOT_RUN and are now tracked under M8.5;
this is a scheduling/acceptance change, not a new compatibility result.

## Plan addition — hidden VBA attributes and document code view (2026-11-01)

Completed the [online investigation](vba-attributes-and-editor-projection.md)
of the reported procedure `Attribute` diagnostic and raw `ThisWorkbook.cls`
header display. Microsoft documents stored workbook attributes, module headers,
code names and Macro Options; the VBE_Extras author's first-hand account documents
hidden-member metadata and Excel's invocation-string encoding. Local inspection
found top-level attribute parsing but no matching procedure-body path, plus raw
source presentation and filename-based Properties display.

Originally added M6.9–M6.12 tasks to the [remaining plan](implementation-plan.md):
metadata parsing/ownership, lossless code-view mapping, document-component
presentation/properties and native/WASM/UI/fork regression evidence. M7.2 now
includes hidden metadata preservation. The investigation identified M6.9 first: a minimal
authored procedure carrying the exact invocation attribute, with no executable
metadata checkpoint. That investigation was documentation only; the implementation update above
records the subsequent runtime work. No new Excel oracle observation is claimed. Existing unrelated working-tree implementation edits
were left untouched.

## M6 fix — real-world document-module cookies decode (2026-11-01)

A real Excel-authored workbook (`Food.xlsm`, user-supplied) was refused by the
importer with `invalid document module cookie`: the PROJECT-stream check required
eight bare hex digits, while MS-OVBA §2.3.1.4/§2.1.1.6 specify the document
module's `DocTlibVer` as `HEXINT32 = "&H" 8HEXDIG` — Excel writes
`Document=Sheet1/&H00000000`. The controlled authored fixture had baked in the
non-spec form, so every authored test asserted the wrong grammar. Fixed the check
to the specified form (case-insensitive `&H` per ABNF string-literal semantics;
exactly eight hex digits), switched the fixture to the spec form, and added
positive/negative regressions (lowercase prefix accepted; bare, short, overlong
and nested forms refuse). `Food.xlsm`'s project now decodes: `Module1` plus
`ThisWorkbook`/`Sheet1`/`Sheet2` document modules with preserved source.
Validation: `cargo xtask ci` PASS, `cargo xtask wasm-run` PASS (27 groups). No
execution claim: import stays inert and the workbook's recorded macro is never run.

## M6 module-context, resolution and host-object completion (2026-10-31)

Implemented the M6.5–M6.8 scope defined by the [per-layer check](m6-context-resolution-check.md)
against the two supplied specifications. Shared: staged bare-name resolution
(locals/parameters/module/project declarations, then host-declared document-module implicit
`Me` members, then host imports) with project-before-global shadowing; the `With` statement
end to end (parser, binder, hidden receiver slots, innermost-only leading dots, debugger-
invisible slots, native and WASI execution); and IDE receiver typing for host objects —
host-declared import result types, call-shaped receivers, `Me`/code-name document types and
catalogue member walks — still read-only and metadata-driven. CompileOptions gained
`document_members`; the IDE gained `configure_document_context`. Fixed a pre-existing strict
Clippy failure in vba-ui-egui's form Format buttons (theme styling) that blocked the gate.

Fork acceptance (in GridCraft/WordCraft under their M6 branches): GridCraft declares
worksheet implicit members and proves the specifications' unshadowed module-kind binding and
live host-global consultation cases against real workbook sessions; WordCraft declares
`ThisDocument` implicit members and pins the same shared revision. Remaining M6 items are
explicitly refused/NOT_RUN in the check (Application.Run, add-ins, chart sheets, Office
reference observations) rather than claimed.

Validation from this checkout: `cargo xtask ci` PASS (393 workspace tests; strict Clippy,
fmt, layers, provenance, assets and corpora), `cargo xtask wasm` PASS, `cargo xtask wasm-run`
PASS (188 actual WASI tests). New regressions: 3 parser (With/leading dots/unterminated),
5 binder (With slots, dot refusal, shadowing, implicit members, member writes on implicit
receivers), 4 VM end-to-end (`with_resolution.rs`), 4 IDE receiver typing
(`host_completion.rs`), 1 UI localization render regression (`drawn_chrome_text_goes_through_
the_injected_localizer`) plus the existing 31 vba-ui-egui interaction tests. IR versions, the
19-crate DAG and the m5-host-v1 profile are unchanged; no fresh Office oracle observation is
claimed. Two follow-up commits on the same day: member writes on implicit `Me` receivers keep
the requested member application (found by WordCraft's acceptance workflows), and the shared
editor's remaining chrome labels route through the injected localizer with the render
regression above — the change that turns WordCraft's translation-audit gate green.

## Plan addition — module-context and name-resolution specifications (2026-10-31)

User-supplied specifications [vba_module_types_and_execution_contexts](vba_module_types_and_execution_contexts.md)
and [vba_excel_context_and_object_resolution](vba_excel_context_and_object_resolution.md) are
indexed in [docs/README.md](README.md). The [remaining plan](implementation-plan.md) adds M6.5 —
a per-layer check of implemented, refused and missing behavior against current M1–M5 evidence —
plus M6.6/M6.7 for the resulting shared resolution and fork host-global/context gaps, and M7.4
for the two documents' differential matrices as unchanged-document acceptance. Documentation-only
change: no product code, runtime contract or compatibility claim. M5's accepted profile
(document code names, `Me`-bound event owners, Range-aware UDF context) is unchanged; M6.5 must
reconcile these specifications against that evidence before new implementation starts.

Reviewing the reported IntelliSense gap found the shared cause: vba-ide receiver typing
(crates/vba-ide/src/metadata.rs `receiver_type`) resolves only bare source symbols and catalogue
owners, and the completion prefix scan (crates/vba-ide/src/code_aids.rs `completion_prefix`)
drops call-shaped receivers, so configured host imports (`ThisWorkbook`, `ActiveSheet`) and call
receivers (`Range("A1")`, `Worksheets("Data")`) never reach member completion. Typed locals
(`Dim ws As Worksheet` + `ws.`) and the bare owner `Application.` do complete; GridCraft's only
completion regressions cover the typed-local path (gridcraft crates/vba/tests/nominal.rs), and
WordCraft has no embedded editor/catalogue yet (M6.2). The plan adds M6.8 for shared receiver
typing plus fork-declared catalogue coverage. Documentation-only change; no product code or
compatibility claim.

Validation for this documentation-only revision: local link targets verified and
`git diff --check` passes in this checkout.

## M3/M4 completion — range, identity, values and members (2026-10-29)

The [self-contained acceptance record](m3-m4-host-acceptance.md) closes the stale
“M3 not started” status: GridCraft already contained M3.1 range/materialization
separation, M3.2 relative/named/multi-area references and M3.3 navigation/search/
native structural commands. The audit retains independently authored source,
negative/resource tests and native engine undo/protection/recalculation evidence.

M4.1 now supplies stable sheet/document ownership, concrete captured handles and
fresh active workbook/sheet/cell/selection roots, cross-document scoped targets,
per-target consent and per-document undo groups. M4.2 implements Date/Currency
Value versus Value2 reads/writes, array lower bounds/cropping/CVErr fill, General
versus Text input conversion and optional Address A1/R1C1/relative/external forms.
M4.3 supplies live selected formatting, protected rename, real application
calculation/settings and truthful supported/unsupported metadata.

[ADR 0025](decisions/0025-getter-alias-member-dispatch.md), committed as `22cc845`,
is the only shared language change. The generic two-object regression checks
fresh getter resolution, exact targets/Get/Call/Let intent and named arguments,
retaining method shortcuts. Tiny-vba CI passes **375 tests**, headless WASM
compilation passes, and **181 actual WASI tests** pass. The 19-crate DAG, native
protocol and IR versions are unchanged. Office names/state remain fork-owned.

GridCraft's current authored native adapter suite passes **58 tests** (ten new
M4 workflows), with native structural/style acceptance and full gates recorded in
the acceptance report. No fresh Office oracle observation is claimed. Profiles
retain explicit unsupported operations, resource limits and broader M5–M8 gates.

The shared prerequisite branch `feat/m4-getter-alias-dispatch` is published.
GridCraft pins all 13 shared packages to immutable tiny-vba
`22cc845e5ee74a1bcd15eec32cdb8a8f18d316f8`; its lockfile contains Git sources,
no local path override, and no unrelated dependency updates. Final pinned
native/WASM/full-CI/window checks close the acceptance gate, as recorded in the
acceptance report. GridCraft’s integrated implementation is committed as `f320728d`.
Both repositories retain their separate ownership and commits.
M3 and M4 were removed from the remaining plan without renumbering M5–M8. M5 is
now also complete for its declared profile; current work starts at M6.

## M5.1 groundwork — explicit document code names (2026-10-29)

The IDE model now preserves a document module's VBA code name separately from its
source/display name. `add_document_module` validates the supplied identifier and
requires it to match the module's `VB_Name` attribute; adding source remains inert.
This shared editor groundwork preserves identity without executing source. GridCraft now binds the
resolved code name to the live host object for callback execution; M5.1 signature/member acceptance,
the broader event policy and UDF acceptance remain open.

## M2 completion — selected classes and standard forms (2026-10-29)

[ADR 0024](decisions/0024-standard-form-execution-and-exchange.md) and the
[self-contained acceptance map](m2-acceptance.md) close M2.1–M2.4. Typed project
member/visibility/signature and coherent property binding, canonical external
field ByRef and property assignment-value semantics now have regressions.
The shared egui designer mounts the existing model with Toolbox, real drag/resize,
Properties, keyboard/tab order, Format/group/z-order, zoom and bounded history.

Selected Label/TextBox/CommandButton events execute through ordinary native
classes and source Event/WithEvents frames. Modal/modeless Show, Load/Hide/Unload,
QueryClose cancellation, nested modal input ownership, source break/inspect/edit/
resume and Reset/close cleanup are exercised. Explicit suspended callbacks retain
waiting callers and host tokens without replay or reentrancy. Original source,
design identity/groups/geometry and independent paint/tab order round-trip through
strict `.tvform` v1 native/browser IO and revision-acknowledged host requests.
Malformed/unknown resources, stale saves and invalid imports refuse atomically.

Final validation: `cargo xtask ci` **PASS (374 workspace tests)**;
`cargo xtask wasm` **PASS**; `cargo xtask wasm-run` **PASS (180 actual WASI tests)**;
`cargo xtask ui` **PASS (127 tests, 24 original renders)**; native debugger/form
captures, real web build and both Chromium smoke/form workflows **PASS**.
Original light/dark/narrow/scaled/host-theme, native and browser images were
inspected. The 19-crate headless DAG is unchanged; no sibling repository changed.

This is the declared predeclared-form profile with `.tvform` exchange, not
universal VBA/UserForm or binary Office resource compatibility. Explicit `New`
form display, unselected controls/events, Friend/As New/Implements, COM activation
and binary Office form/package writeback remain outside the profile. The report
lists exact boundaries. No fresh Office oracle result is claimed.

M2 is removed from the remaining plan without renumbering M3–M8. Next smallest
planned task: **M3.1** range-reference/materialization budget separation in
GridCraft. It has not started under this tiny-vba-only task.

## Historical M2 completion audit after feature merges (2026-10-29)

Merged main `cadc744` is **not M2-complete**. The
[M2 acceptance audit](m2-classes-and-forms.md#completion-audit-after-feature-merges-2026-10-29)
confirms the missing visual designer, signed form handlers/runtime and form
source/resource persistence/round trips. Static project-member/field ByRef
acceptance also remains open. Fresh targeted verification passes nine IDE
class/form tests and 23 VM class tests. The merge's full CI and WASM gates
pass but do not demonstrate those absent acceptance surfaces. M3 has not begun;
it is GridCraft-owned under the remaining plan. Next: mount the headless form
model in the shared egui designer and verify its interactions.

## M2.1 runtime checkpoint (2026-10-29; M2 remains incomplete)

tiny-vba main was fast-forwarded and pushed to `2ee181d` as requested; GridCraft
was not merged. New work is on `feat/m2-classes-and-forms`. [ADR 0023](decisions/0023-project-classes-and-forms.md)
and the [staged implementation record](m2-classes-and-forms.md) document ownership,
the working profile and outstanding acceptance. Class/property construction,
canonical fields, lifecycle, identity, snapshots, lazy document state and actual
instance callbacks and declared Event/WithEvents/RaiseEvent delivery execute through ordinary VM/controller frames. Class events
pause/edit/resume through the IDE; safe watches stay inert. The existing 19-crate
normal DAG and external object ownership are preserved.

The editor now inserts class modules, distinguishes component kinds and exposes
all three error-trapping modes. Source events preserve connection ordering,
canonical ByRef, pause/resume, state restore and handler-fault cleanup.

The toolkit/IO-free `vba_ide::forms` model now supplies selected standard controls,
atomic layout/Properties edits, groups, independent tab/z-order, zoom and bounded
history, with six headless workflow tests. It is not connected to a visual designer.
After this model increment, mandatory CI passes 349 workspace tests, headless
WASM compilation passes and WASI passes 162 actual tests.

M2 is not complete: static member/field ByRef review, visual form
designer, handler generation, full form runtime/modal lifecycle and form source/
resource round trips still need implementation and acceptance. Checkpoint validation: 343 workspace tests and 156 actual WASI tests pass;
mandatory CI/WASM plus UI, native capture, web build and Chromium smoke pass.
No form Run action is enabled. M3 has not been started ahead of the requested M2 completion.

## New M1.2/M1.3 completion — defaults, iteration and real workbook acceptance

[ADR 0022](decisions/0022-default-members-and-enumeration.md) and the
[implementation/evidence log](m1-defaults-and-collections.md) record the full
contract and stages. IR 6 preserves generic Get/Let/Set intent, chained/indexed
applications, named/omitted/native ByRef arguments and value/reference contexts.
For Each has bounded owned host snapshots, live native array aliases, typed
controls, resize locks, source/Next checkpoints, error/resume cleanup and
canonical paused element edits. Safe watches remain inert. The headless
19-crate DAG and repository ownership boundary are unchanged.

GridCraft exclusively supplies six Excel object types, real Worksheets/Sheets
roots, Count/Item/defaults and cell iteration through native commands. Original
unchanged source verifies typed loops, names/order, Range defaults/indexes,
values/errors, handle budgets, stale references, undo and trust revocation.
A worksheet UDF loops over a two-dimensional value array and recalculates when
its precedents change. Broader M2–M8 compatibility is not claimed.

Core: 316 workspace tests, strict `cargo xtask ci`, headless WASM compilation and
130 actual WASI tests pass. GridCraft: 40 native and 40 actual JavaScript/WASM
adapter tests; full CI passes all 10 gates with 2,568 test/doctest results;
1,150 optional desktop VBA-window tests, strict window Clippy and browser VBA
compilation pass. Node 24.22.1 provides the successful JS/WASM run; an older
Node 24.14.0 shutdown crash is explicitly recorded, not counted as success.

Published feature branch in both repositories: `feat/m1-defaults-and-collections`.
GridCraft commit `55d3b1bb825499086581afdbd2e07eec87bc3c29` pins tested tiny-vba
commit `d80e0393b408e0421a8df7e16667dbbbc970783b` by Git revision, with no embedded
copy or path override. Main branches are unchanged. Completed M1 is removed
from the remaining plan; M2–M8 IDs are retained for follow-up references.

## New M1.1 completion — generic core and fork-owned acceptance

Completed nominal host binding under [ADR 0021](decisions/0021-nominal-host-object-types.md),
with [resumable implementation/evidence](m1-host-references.md). The core carries
opaque library/version/type identity and IR 5 constraints, and queries a host-owned
read-only resolver. It owns no Excel definitions, object store or GridCraft dependency.
The boundary audit finds only oracle metadata references to Excel in non-test
interpreter Rust; the reviewed 19-crate DAG remains headless and acyclic.

Storage/alias/array/temporary/return checks, host writeback precommit, debugger
edits, rootless embeddings, isolated snapshots, callbacks, persistent restore,
lifetime and resource budgets are verified. Code aids preserve library scope
through qualified/chained types rather than mixing same-named classes.

GridCraft owns its four Excel type descriptors and resolver over its existing
object store. Its authored unchanged Worksheet/Excel.Range source executes real
cell writes, typed calls/results and pause/edit/resume. Wrong classes fail with
13; stale/foreign/closed handles with 91. This is registered-type compatibility,
not a complete Excel typelib or general unchanged-workbook claim.

Core validation: 304 workspace tests and 118 actual WASI tests, strict CI and
headless WASM compilation pass. Fork validation and the immutable dependency pin
are recorded in the M1 log and GridCraft's `docs/vba-m1-types.md`. M1.2/M1.3 remain
the default-member/collection work; no implicit implementation is inferred here.
GridCraft's final pin is tiny-vba `98a88c3`, with adapter commit `139016d7`:
33 native and actual JavaScript/WASM adapter tests, 2,561 full-CI test/doctest
results, 1,150 optional window tests, strict window Clippy and browser feature
build pass. Both feature branches remain separate from main.

## First M1.1 explicit-reference increment — historical checkpoint

Implemented validated native reference snapshots, ordered namespace resolution,
registered enum declarations and executable typed constants. Compiler defaults
remain reference-free. IDE discovery/compilation and isolated Immediate evaluation
use the same explicit snapshot; reset guards prevent mid-session replacement.
See [the resumable work log](m1-host-references.md) and
[ADR 0020](decisions/0020-explicit-host-reference-snapshots.md).

Nominal object declarations remain explicitly unsupported rather than erased to
generic Object. M1.1 is **in progress**, not complete: next enforce nominal type
identity/assignability across storage paths, then prove unchanged typed source in
the forks. M1.2 default members and M1.3 collections have not started. No GridCraft
changes, Office oracle runs or new external macro execution are included.

Validation: 288 workspace tests (11 new), strict CI/layer/provenance checks,
headless WASM compilation and 102 actual WASI tests pass. The resumable log records
the supported profile and the next nominal-object checkpoint.

## Plan renumbering — 2026-10-28

On user direction, removed completed work from the plan, removed old M12/M13
(native extensions/translation and AI assistance), and excluded third-party
ActiveX/OLE controls not supplied with Office applications. Standard Office
UserForms/controls remain in scope. No product capability or runtime contract
changes in this documentation update; [ADR 0019](decisions/0019-remaining-plan-scope.md)
records the scope and ordering decision.

| Current remaining milestone | Previous milestone / scope |
|---|---|
| M1 — host types/default members/collections | Old M15 |
| M2 — classes/document semantics/standard Office UserForms | Old M11, narrowed control scope |
| M3 — range/worksheet coverage | Old M16 |
| M4 — identity/values/member semantics | Old M17 |
| M5 — document events/Range-aware UDFs | Old M18 |
| M6 — remaining host integration | Unfinished old M14, including WordCraft |
| M7 — unchanged-document/package acceptance | Old M19 plus Word document package acceptance |
| M8 — expanded conformance/hardening/release | Old M9, release after feature acceptance |

Existing reports/ADRs and the historical entries below retain their old IDs.
Completed setup and old M0–M8/M10 evidence remain here and in their acceptance
reports; they are not new tasks under the reused numbers. The completed GridCraft
baseline and old M14.10 audit remain in [the integration log](gridcraft-integration.md).
Dependencies now put shared type/class contracts before consumers, range behavior
before dependent events/UDFs, host integration before package acceptance, and
release last. Conformance/fuzz/performance work runs throughout, not only at release.

Validation for this documentation-only revision: contiguous M1–M8 headings/table
and subtasks, removed scopes absent from the active plan, 274 local Markdown
links checked, and `git diff --check` pass. Required `cargo xtask ci` (277 workspace
tests), `cargo xtask wasm` and `cargo xtask wasm-run` (95 actual WASI tests) pass.
No new product behavior, Office observation or corpus execution is claimed.

## M10 completion

Preserved the user's `8dd37dc` checkpoint and completed the work on
`feat/m10-completion`. Added shared bounded source history across views, scoped
literal search, stable bookmark/navigation anchors, procedure/split editing,
classic project commands, preferences/docking/toolbars and browser file transfer.
Metadata-backed constants, Quick/Parameter Info, definition/help navigation and
Object Browser use source/host catalogues without loading libraries.

Quick Watch, scoped conditional/change watches, typed local array edits, trapping
controls, verified straight-line Set Next Statement and explicitly opted-in
isolated Immediate calls have positive, refusal and lifecycle regressions.
Optional DAP runs above the IDE with bounded stdio framing and real subprocess
acceptance. [ADR 0018](decisions/0018-m10-classic-tools-and-debugger-policy.md)
records the contracts; the [M10 report](ui/m10-implementation.md) maps every stage
to tests, final commands and limitations. No GridCraft changes or new Office
oracle observations are included. Next UI milestone: M11's class/document
prerequisite contract, before enabling a form designer Run command.

## M8 completion corrections

The latest user request makes working IntelliSense and syntax feedback part of M8. Added public
Microsoft VBA behavior references and independently authored examples to specification section 16;
four additional reference pages are cached only under ignored `.cache/evidence/vbe/`, with hashes
and factual observations tracked. No downloaded implementation, artwork or reference screenshot is used.

Implemented scoped design-time symbol/typed-receiver completion, an editor-anchored list with real
keyboard/pointer acceptance and cancellation, project signature help, and live red/underlined parser
feedback that clears on correction without moving the typing caret. Compatible same-host runs
transfer VM project state; Reset/source or host identity changes isolate it. Find Next advances and
wraps. CR/CRLF/LF display mapping retains untouched source bytes and compiler/debugger coordinates.
The Windows checkout regression now compares filesystem identity rather than short/long path spelling.

Local validation: **236 workspace tests** (17 new regressions), strict Clippy/format, layer/provenance/
corpus checks, UI tests and eleven original rendered variants, native Xvfb capture, headless WASM
compile/execution and the extended Chromium typing workflow. The 33 IDE/embedding tests run headless;
21 egui interaction tests verify actual typing, completion, error underlines/recovery and CR editing.
All six [remote CI jobs](https://github.com/matt123p/tiny-vba/actions/runs/40552813785) passed for implementation commit `e24d07a`: Windows, Linux, macOS, WASM, UI and corpus verification. The documentation closure changes no product/test code.
Advanced completion/reference/class resolution remains M10; forms/extensions/AI/fork acceptance
remain M11–M14. Earlier 219-test counts below describe the initial implementation.

## Earlier M8 completion re-audit (historical failures)

After pushing `f462959` on 2026-10-27, reran the required baseline gates and added targeted audit
probes. Locally, the 219 existing tests and UI/WASM gates pass, but all three extra probes fail: normal Run
relaunch discards Static state; Find Next stays on the current match; lone-CR source cannot set
statement breakpoints with the correct line coordinates. These are M8 defects, not M10–M14 features.
[The audit report](ui/m8-completion-audit.md) records expected/actual behaviour, source locations and
`python3 tools/audit_m8_completion.py` for reproduction. At that point full M8 completion was not confirmed.
The pushed GitHub workflow additionally fails its Windows checkout-location regression while UI,
corpus, WASM, Linux and macOS jobs pass; the audit report records the failing assertion and cached
log provenance. Resolve that CI gate as well. The earlier completion record below is historical;
the correction record above supersedes this historical failure state.

## GridCraft integration — selected spreadsheet profile complete

The 2026-10-28 desktop-window correction is complete in GridCraft `a3f77434`:
separate native `vba-window` and embedded `vba` presentations share one project/VM;
Excel-style Developer/View/Macro Edit routes replace permanent launch buttons.
Full GridCraft CI (2,553 tests), optional window UI (1,150 tests), both feature
lint/build paths and browser fallback pass. Native Windows windows and original
snapshots were inspected. The [work log](gridcraft-integration.md) records exact
evidence and remaining platform limits. Shared tiny-vba code is unchanged.

GridCraft commit `b496325b` adds the requested editor/toolbar/menu/keyboard launch,
assigned worksheet shapes and worksheet-callable Functions. Its full gate passes
2,553 tests; the full optional UI suite passes 1,143 (18 VBA cases), and all 25
adapter cases pass natively and in JavaScript-WASM. Public behavior references,
source/assignment persistence, read-only UDF policy, exact logs, launch instructions
and compatibility limits are in [the final acceptance record](gridcraft-integration.md).
tiny-vba remains a separate dependency pinned to merged product revision `ea38c3f`.
Broader M14/WordCraft/events/classes remain separate planned work.

Work is on `feat/gridcraft-integration` here and `feat/tiny-vba-integration` in the
GridCraft fork. The staged plan, decisions, exact validation and resumption notes
are in [the integration work log](gridcraft-integration.md). Office objects remain
fork-owned; no application dependency is added here. Workbook tests exposed the
root-import member-dispatch defect fixed under [ADR 0017](decisions/0017-imported-root-members.md).
The native VM regression covers exact Get/Call/Let/Set dispatch and named arguments.
Windows xtask selects `python` rather than assuming a `python3` executable.
GridCraft consumes shared revision `4d80a9584b3daeb75d589c629eff272ecd5881fe`
as a pinned Git dependency, with corpus/reference downloads explicitly opt-in.
Fourteen real-workbook tests pass natively and as JavaScript-hosted WASM; eight
shared-editor/control/source-save-discard tests pass. Light/dark/narrow/scaled
actual-paused-state snapshots were rendered and inspected. The fork's full gate,
exact commands, remaining profile limits and resumption state are recorded in
the work log. Broader M14/WordCraft/events/forms/signatures are still planned.

Continuation stage completed in GridCraft commit `8168bda4`: native typed cell
contents share the engine command/edit lifecycle, replacing JSON mutation dispatch
and removing the Text-format (`@`) write restriction. All 2,540 GridCraft gate
tests, five targeted engine cases, 15 native/JavaScript-WASM adapter cases and
eight editor/control cases pass. Shared tiny-vba code and its Git pin are unchanged.

## Post-M8 milestone allocation

On 2026-10-27, assigned the remaining UI specification work to explicit **planned** milestones
in [the post-M8 roadmap](ui/post-m8-roadmap.md): **M10** classic editing/code aids/advanced debugger
and DAP; **M11** class/document runtime and UserForm editor; **M12** native Craft extensions,
non-Microsoft enhancements and Python translation preview; **M13** optional Copilot-style assistance;
**M14** cross-repository Gridcraft/Wordcraft integration acceptance. Each has owners, dependencies,
acceptance gates and a mapping to the specification's phases/scenarios. M8's delivered baseline
and M9's next conformance/release work are unchanged. No deferred feature becomes implemented
through this planning update; no sibling repository work is started by it.

## M8 reusable editor/debugger and standalone harnesses

Implemented [ADR 0016](decisions/0016-m8-embeddable-editor.md) following the user-supplied
[public-source UI specification](ui/craft_vba_editor_debugger_ui_spec.md) and the
[implementation plan written before code](ui/m8-implementation-plan.md). `vba-ide` owns toolkit/IO-free
source state, compilation, command metadata and real debugger sessions; `vba-ui-egui` mounts the
Classic shell/panes in a caller-owned Ui; `vba-ide-app` owns native/browser source transfer,
preferences and window lifecycle. `vba-cli debug` uses the same model and accepts command scripts.
No application sibling is a dependency. The 19-node DAG enforces GUI/platform boundaries.

Delivered source editing/navigation, lexer colours, undo, bounded find/replace, actual statement
breakpoints/highlighting, bounded run/pause/continue/reset, nested into/over/out stepping, selected
stack/local inspection, validated active-frame edits, safe expression watches/Immediate and output.
Generic-host embedding tests prove one native write through break/edit/resume, exactly-once waits
and cancellation without rollback. Edited compiled source requires reset/recompile; invalid,
stale/out-of-scope and unsafe evaluation states remain explicit. Multiple views have independent
widget namespaces; injected themes never replace global context visuals/fonts/scaling.

Validation from this checkout: `cargo xtask ci` passes **219 workspace tests**, formatting, strict
all-target Clippy, the reviewed 19-node DAG, seven source pins/646 candidate hashes, M4 evidence and
UI cache/provenance/pin/font-notice checks. The 42 added tests comprise **25 IDE/embedding**, **12 UI
interaction/accessibility/layout**, **4 shell source-IO** and **1 scripted CLI debugger** regressions.
All 197 authored/reviewed language fixtures retain PASS. `cargo xtask wasm` compiles every library;
`cargo xtask wasm-run` passes native/WASI language/host/debugger smoke plus the 14 importer,
17 callback and 25 headless IDE/embedding tests. `cargo xtask ui` passes product tests and renders
seven original actual-paused-state PNG variants. The native eframe app runs/captures under Xvfb;
the Chromium harness verifies F8 Break, F5 Completed, output 10 and no page errors. Browser packaging
uses `cargo xtask web` and matching wasm-bindgen-cli 0.2.126. `git diff --check` passes.

Visual review covered light/dark, narrow, scaled, injected host theme, focused Immediate and dark
Object Browser output and corrected glyph availability, selector naming, header clipping,
per-document IDs and debugger presentation. Generated product images remain ignored in `reports/ui/`.
Downloaded public reference pages remain ignored in `.cache/evidence/vbe/`; only public URL/date/hash
observations and licensed redistribution notices are tracked. No proprietary/VBE clone code,
resource extraction, reference screenshots or Office installation was used. This does not establish
pixel identity, full C0/C1 feature coverage, another operating system or real workbook integration.
See [the explicit feature/acceptance matrix](ui/m8-acceptance.md) and [embedding guide](ui/embedding.md).

Next smallest task: fix M8-A1–A3 and add permanent regression tests, then M9 conformance/release work; separately wire these reusable panels into a fork's
real document lifecycle and verify workbook effects/undo/recalc/events without duplicating the UI.
Remaining classic features now belong to M10, forms/class support to M11, native enhancements
and translation to M12, optional AI to M13, and fork integration acceptance to M14. See the
[post-M8 roadmap](ui/post-m8-roadmap.md); all remain planned.

## M7 bounded generic callbacks and lifecycle

Implemented [ADR 0015](decisions/0015-m7-callbacks-and-lifecycle.md). The VM stays single-ownership
and non-reentrant: hosts schedule callbacks by name (`Session::schedule_callback`, FIFO, bounded by
`Limits.callbacks` default 256, name/arity/payload validated) and the VM delivers each as a fresh
top-level invocation through `run_next_callback` only after the current program is terminal. A
delivery drives to a boundary in one call (soft quota yields continue internally), shares
module/static state, bindings, output and the session instance, applies breakpoints and the
checkpoint predicate exactly like `run_slice`, and reports `counters().callbacks_run`. Host waits
inside a callback use the existing exactly-once resume path. `cancel`, `disconnect` and `host_lost`
drain the queue without executing pending callbacks; scheduling is refused once the lifecycle is
closed (cancelled/disconnected), not merely after normal completion.

Lifecycle additions: `Session::reset_state()` clears persistent/static storage, Err and captured
output, bumps the generation and prepares the original entry with its original bound arguments,
refused unless terminal, open and current; `Session::rebind_roots()` lets the host replace
root references (its own lifetime/generation model) under the same launch validation. A stale root
at delivery surfaces the host's reference error 91 with no repeated dispatch. Generic-host
reentrancy, drain, budget, failure-recovery, debugger-pause-in-callback and stale-reference tests
live in crates/vba-vm/tests/callbacks.rs. Event ordering, `Application.EnableEvents`, document
events, UDF evaluation and any Office-shaped catalogue remain fork-owned over this generic queue.

Verification from this checkout: cargo xtask ci passes formatting, strict all-target Clippy,
**177 integration tests**, the 16-node workspace DAG, seven source pins/646 candidate hashes and
M4 evidence checks. All 197 authored/reviewed language fixtures retain PASS. cargo xtask wasm
compiles all libraries; cargo xtask wasm-run passes native/WASI language/host/debugger acceptance,
all **14 importer tests** and all **17 callback/lifecycle tests** on WASI. git diff --check passes.

M7 acceptance audit corrections (2026-10-27): closing an already terminal invocation now closes
its lifecycle permanently; invalid delivery quota/source/host/lifecycle requests preserve queue
and outcome; reset prepares the original entry again with original bound arguments and fresh
static/output/Err state. Reset retains queued callbacks, instance/debugger identities and lifetime
work budgets and cannot reopen a closed session. Added native/WASI regressions, shared cumulative
queue payload/metadata accounting, upfront argument conversion, owned top-level ByRef arguments,
and clean frame-start resource failure. Corrected the plan's unsupported claim of fork event-order
acceptance: Office event/workbook gates remain separate and unverified by this checkout.

Next: the required reusable graphical editor/debugger slice on these VM APIs; M8 completes its
acceptance gate.

## M6 project source decoding

Implemented [ADR 0014](decisions/0014-m6-project-decoding.md) and the
[bounded import contract](project-import.md) in vba-project, finishing the incoming decoder slice.
The read-only API consumes controlled vbaProject.bin CFB bytes and returns project information,
registered/project/control references and standard/class/document modules with distinct module/
stream/code-name identities, source offsets, flags, descriptions and preserved source/attributes.
Caller-owned SourceId allocation preserves diagnostic locations without inventing global identities.
Strict code-page conversion retains the encoding identity; Unicode metadata/name maps must agree.

Protection envelopes are checked only to refuse restricted/password/hidden projects. Unknown
encodings/records and designers fail explicitly. Corrected PROJECTlk's classification to unsupported
ActiveX licence metadata, rather than protection. Added complete CFB-sector validation after a
truncation regression showed the underlying library accepts missing final padding. Hard and
caller-lowered bounds cover input/streams/decompression/output/counts/depth; metered CFB reading
and strict cycle checks bound hostile metadata work. Cache consistency is explicitly NotChecked;
P-code, trust decisions, compilation, execution and binary writing remain absent from this API.

Reviewed/pinned cfb 0.14.0 (MIT) and encoding_rs 0.8.42 ((Apache-2.0 OR MIT) AND BSD-3-Clause).
The dependency checker now distinguishes reviewed external pins from workspace DAG nodes; only
vba-project gains those normal dependencies. No interpreter implementation or proprietary fixture
was imported. Fourteen authored integration tests build controlled binary records in memory and
cover exact metadata/source results, Unicode identities and diagnostic spans, representative
encodings, optional records, empty projects, protected/unsupported/corrupt inputs, every directory
truncation, selected header mutations, CFB tree/FAT cycles, compression vectors and all limits.
Source containing Auto_Open/unknown host calls remains inert; the crate has no VM/host/compiler edge.

Verification from this checkout: cargo xtask ci passes formatting, strict all-target Clippy,
**160 integration tests**, the 16-node workspace DAG, seven source pins/646 candidate hashes and
M4 evidence checks. All 197 authored/reviewed language fixtures retain their existing PASS results.
cargo xtask wasm compiles all libraries for wasm32-unknown-unknown. cargo xtask wasm-run passes
native/WASI language/host/debugger acceptance and now executes the same **14 importer tests on
WASI**, with no preopened directories. git diff --check passes. No separate MSRV, macOS/Windows,
external-workbook import, browser runtime or Office observation is claimed.

M7 delivered the bounded generic callback queue, reset ownership and lifecycle cleanup under
[ADR 0015](decisions/0015-m7-callbacks-and-lifecycle.md).

## M5 native host integration contract

Implemented [ADR 0013](decisions/0013-m5-host-contract.md). `vba-foreign` owns protocol version 1
(`API_VERSION`, `ProtocolVersion`, `check_protocol`); the foreign session and the VM both refuse an
incompatible host protocol before any dispatch, with a double-check regression removed from the
invoke path. `Import`/`validate_imports` bound generic source imports: ≤64 entries, ASCII identifier
names, case-insensitive uniqueness, Call/Get only. The compiler accepts them through
`CompileOptions::imports` and lowers each to a hidden IR 4 `HostRoot` slot plus a fixed member
call/get; imports resolve only as call targets and cannot be written or captured.

`vba-ir` gained `IR4_VERSION`, the `host-roots-v4` feature and `SlotRole::HostRoot` with structural
rules (IR 4 only, Object, non-array, hidden, result-free, read-only under `RefLocal`); older IR 1/2/3
targets and `Capabilities::ir3_structural()` stay closed. The VM's `HostBinding` gained case-folded
named roots (cap 64, host-identity matching); missing roots refuse `Session::new`, mismatched-host
roots refuse launch, and a stale root surfaces the host's own reference error (91) with no repeated
dispatch. `HostBinding::for_program` derives the whole site allowlist from a verified program, and
`Session::limits()` exposes budgets to embedding drivers.

The runtime profile is now `m5-host-v1`; the compiler's structural default is `ir4-structural` while
IR 1/2/3 remain selectable and verified. `vba-values`/`vba-debug` publish `API_VERSION` consts.
`vba-testkit` supplies the reusable bounded bridge driver (`run_to_boundary`, `statement`), and
vba-vm/tests/embedding.rs proves the fork integration shape in-repo: an import-driven FillCells-style
call against a scripted fake host, breakpoint/inspect/edit/resume with exactly one dispatch, and
hidden-root refusal cases. Architecture DAG edges (semantics→foreign, compiler→foreign, testkit
debug/ir/vm) are reviewed in tools/architecture.json.

Verification from this checkout: cargo xtask ci passes formatting, strict all-target Clippy,
**146 integration tests**, the 16-crate acyclic DAG and seven pins/646 hashes. All 197 authored and
reviewed fixtures still assert PASS under the renamed profile, and the same tests pass in release.
cargo xtask wasm builds every library and cargo xtask wasm-run passes the identical native/WASI
acceptance. Not claimed: the fork-owned FillCells proof with a real Office layer remains Gridcraft's
gate, as ADR 0013 requires.

## M4 external acceptance completion

The earlier authored-only completion claim did not establish the supplied M4 gate. Corrected it
by reviewing and adopting **153 MIT ReflexGrid pure-language cases** with published Microsoft
Excel Mac expectations under [ADR 0012](decisions/0012-m4-oracle-acceptance.md). All 153 now PASS,
alongside the original **44 authored PASS** scenarios. [The acceptance report](m4-oracle-report.md)
records every selected case and raw expected result, source/expected/report hashes and provenance.

The upstream report names Excel for Mac 16.114/build 26101123, macOS 15.3.1 and 2026-10-15.
Fetched its recorded run revision read-only and verified that the case/expected files are
byte-identical at the pinned source revision. Locale is explicitly not recorded per run.
This is inherited upstream-reported Microsoft evidence, not a fresh local Office run. Excluded
Windows-only/locale/non-ASCII/Mac naming cases and all unselected specimens remain NOT_RUN.
Only fixture literals and selected VBA snippets were consumed; no upstream interpreter or Python
script ran. The exact upstream Err-state probe is preserved in a small fixture adapter.

Oracle cases corrected Byte widths/overflow, Currency addition, Single division, Boolean arithmetic,
scalar text precision/exponents, 0/0 and power overflow, Empty/string comparison, eager IIf,
empty-array bounds and physical-line label parsing. Added required typed whole-array ByRef,
ParamArray packing (including empty 0..-1), Option Base and qualified VBA.Array, and canonical named
array-reference capture. Typed whole-array assignments, debugger edits and host patches preserve
element kinds. Captured references retain source evaluation order and expire after resizing.

IR 3 carries the new array layouts/parameters, reference captures and intrinsic identifiers;
IR 1/2 structural capabilities and supported-version sets are preserved explicitly. The compiler
selects the minimum required version. Schema 2 requires complete external metadata and matching
raw/typed results, while Schema 1 remains authored/unmeasured. CI verifies the immutable selection,
license notice, source adaptations and identical native/WASI case bundle, with no corpus checkout
required for ordinary tests.

Final verification: cargo xtask ci (formatting, strict Clippy, **133 integration tests**, 16-crate
DAG, seven pins/646 hashes and M4 evidence verification); the same 133 tests pass in release.
All libraries compile for wasm32-unknown-unknown. Native and WASI execute all 153 inherited oracle
cases and expanded array/argument tests alongside every M3 debugger/host/cancellation/isolation
proof. No remote cross-platform or separate MSRV run is claimed. Remaining language limitations
are explicit in [the current contract](m4-language.md); no complete-VBA or full-corpus claim is made.

Next: M5 public API/bridge versioning, with graphical IDE work under M8. M9 grows oracle coverage
and obtains fresh observations; it no longer defers the original M4 external-subset gate.

## Historical M4 authored-subset completion

Implemented the adopted [IR 2 language extension](decisions/0011-m4-language-extension.md) across
syntax, semantics, compiler, verifier, values, runtime and explicit-frame VM. The original IR 1
structural capabilities remain available; the compiler's new structural target is ir2-structural,
and execution explicitly uses m4-language-v1. New instructions/assignment modes require IR 2 and
feature gates. No Office or UI dependency was introduced.

Added centralized coercions for all scalar widths, exact LongLong integer operations, scaled
Currency arithmetic/decimal-boundary conversion, Date serial operations, Empty/Null rules,
Missing/Nothing/CVErr distinctions, UTF-16 string operations and deterministic intrinsic registry.
Optional/default/named/omitted calls preserve source-order evaluation. Canonical ByRef locals,
module/static slots and array cells share locations; evaluated arguments use private typed temporaries.
Argument capture uses binding rules, preserving Object/Nothing without changing ordinary Let/Set.

Added bounded fixed/dynamic arrays with signed bounds, column-major layout, Preserve/Erase,
generation-checked cells, representation checks, copy budgets and allocation precharges. Module
and Static local storage survives calls; explicit project state export/restore checks revision and
exact source identity, while fresh sessions reset. Debug.Print captures bounded UTF-16 records.

Added per-frame On Error modes/active faults, Err/Erl, retry/next/label Resume, exception edges,
structured-statement continuation metadata and caller propagation/unwinding. Err.Clear leaves an
active handler active. Host errors after waits enter VBA handlers without reinvocation. Break-on-error
continues into a handler once. Engine/resource/protocol failures are not swallowed. A stale array
alias originally escaped while constructing its failure snapshot; the minimized regression now
produces runtime error 9 and explicitly lists unavailable locals.

Verification from this checkout: cargo xtask ci passes formatting, strict all-target Clippy,
**119 integration tests**, 16-crate acyclic layering and seven pins/646 hashes. The same 119 tests
pass in release. cargo xtask wasm builds every library; cargo xtask wasm-run passes identical native
and WASI acceptance, including arrays Long(44), static/module state Long(23), restored state Long(47),
Currency/Date/exact LongLong, optional/named calls and handler Resume Long(12), alongside every M3
host/debugger/cancellation/isolation check. git diff --check passes.

The 20 original scenarios now assert **20 PASS / 0 UNSUPPORTED**. Added 24 independently authored,
uniquely identified M4 scenarios tagged m4-language-v1: **24 PASS**. These are reasoned expectations,
not Microsoft oracle observations; external corpus execution remains NOT_RUN. Profile limitations
are explicit in the M4 contract (including locale/codepage behavior, nested/whole-array parameters,
class/default-member binding and selected argument combinations). This completes the reviewed M4
gate without claiming the entire supplied VBA wishlist. No remote cross-platform or separate MSRV
run is claimed. Next: M5 API/versioned bridge contract and the required graphical IDE slice.

## M4 scalar/runner hardening and corrected validation

Continued the incoming M4 first slice after closing the M3 instance-isolation audit. Reproduced and
fixed two correctness regressions: Double-to-Long conversion could wrap past the 32-bit destination,
and the scenario runner labeled malformed/invalid source UNSUPPORTED with success status. Resource/
engine failures also no longer exit successfully. Float/Date judgments compare exact payload bits;
UTF-16 report text preserves lone surrogates and escapes line breaks.

Published m4-scalars-v1 for the expanded numeric matrix instead of mutating m3-core-v1 in place.
Added Double negation/mixed comparisons and native/WASI boundary assertions. Existing M3-tagged
programs must be recompiled; IR 1 is unchanged. Source/engine failures are classified explicitly,
and known unimplemented intrinsic names retain UNSUPPORTED without hiding unknown-name errors.

Found a verification-root bug: a cached xtask binary could run an older checkout embedded at build
time. Historical pass claims alone were therefore insufficient. xtask now finds and logs the current
checkout at runtime, has a failing-before/fixed-after cross-checkout regression, and honors custom
WASM target paths. Re-ran the current code, not the cached checkout: 89 tests pass in debug and
release; cargo xtask ci passes format/strict Clippy/layers and seven pins/646 hashes; cargo xtask wasm
passes all libraries. An isolated CARGO_TARGET_DIR native/WASI smoke agrees on Long(42), Long(18),
edited Long(41), one invocation/two polls, cancellation/session isolation, Double(3.5), Boolean(true)
and Long overflow error 6. No remote cross-platform, separate MSRV or Office result is asserted.

The 20 scenario assertions remain 17 PASS / 3 UNSUPPORTED. [ADR 0010](decisions/0010-m4-profile-and-verification-hardening.md)
records profile/exit/report/gate contracts. M4 is still in progress: next is the handler-capable IR
specification before On Error implementation, alongside the remaining arrays/arguments/builtins.

## M4 first slice: assertion runner and numeric widths

Implemented `conformance run <scenario>`: the CLI parses the manifest, compiles only the embedded
VBA body with the VM's actual capability set, executes host-free and compares the outcome with the
manifest's typed expectation through new `vba-conformance` judgment APIs (`judge_value`,
`judge_error`, `describe_value`). It prints `RESULT <id>: <STATUS> (<detail>)`; MISMATCH exits 1,
PASS/UNSUPPORTED exit 0, invalid fixtures exit 2. `conformance validate` still reports NOT_RUN and
never executes.

Enabled real division and Double widths under existing IR 1 instructions through an explicit
`m3-core-v1` capability update ([ADR 0009](decisions/0009-m4-runner-and-numeric-widths.md)): `/`
always produces Double (zero divisor 11, non-finite 6), Add/Subtract/Multiply promote to Double
only with a Double operand, Double declarations/assignments widen exactly, narrowing rounds half to
even with overflow 6 out of range, and decimal/exponent literals bind as finite Doubles. The
integer operator matrix and its error numbers are unchanged. The M3 CLI negative case `Answer=1/2`
became correct behavior (Double 0.5 narrowed to Long 0), so the unsupported-operator case uses `^`.

The 20-fixture inventory moved from 20 NOT_RUN to an asserted 17 PASS plus 3 explicit UNSUPPORTED
(cverr: builtins; named: named/omitted direct arguments; optional-missing: optional
parameters/IsMissing). External corpora remain NOT_RUN; no oracle observation is claimed.

Validation: cargo xtask ci (81 integration tests, fmt, strict Clippy, layers, corpus pins),
cargo test --workspace --release --locked, cargo xtask wasm and cargo xtask wasm-run (native and
WASI smoke pass with unchanged M3 results). No dependency graph, IR version or scenario-schema
change. Remaining M4 work: the handler-capable IR contract for On Error/Resume/Err/Erl before
implementing handlers; arrays; optional/named direct-call binding; builtins; Currency/Date/Byte/
LongLong/Single arithmetic; persistent/static state.

Next smallest task: specify the versioned handler-capable IR extension (frame handler state,
exceptional edges, resumption metadata, nested-handler tests) before enabling On Error constructs.

## M2/M3 acceptance audit and session-isolation correction

On 2026-10-26, reviewed every adopted M2 gate against the binder/compiler/IR implementation and
acceptance tests. M2's limited IR 1 scope is complete; no M2 code change was needed. The
[gate-to-evidence audit](m2-m3-acceptance-audit.md) records source maps, deterministic goldens,
negative IR cases, bounded verification and valid self-loop evidence.

Continued M3 with a minimized execution-reuse regression. It failed before the correction: a stale
handle from a dropped/cancelled session could read/edit a replacement session with the same client
label/frame/pause generation. A retained controller's breakpoints and step target had the same
label-reuse problem. Added opaque, non-reused SessionInstance identities and used them for handles,
location ownership, breakpoints and stepping, preserving the client's routing label. ADR 0008 records
the runtime API refinement; no IR version or instruction change is involved.

Verification: cargo xtask ci (75 integration tests, fmt/strict Clippy/layers/corpus integrity),
release workspace tests (75), cargo xtask wasm and cargo xtask wasm-run. The same replacement-session
regression runs natively and in WASI alongside typed results, host waits, edits and cancellation.
No separate MSRV, remote cross-platform or Office observation is claimed. M4 remains next; the
scenario assertion runner, broader coercions, On Error and graphical UI remain separate work.

## M3 completion

Implemented vba-runtime's documented Integer/Long/Boolean/String/Variant/Object assignment and
operator subset with exact scalar tags, overflow/division errors and explicit Unsupported results.
The new m3-core-v1 capability profile is rechecked at VM launch using borrowed immutable IR; the
structural M2 profile cannot silently enable runtime operations. Added CLI run and generic Object-
local member Call/Get/Let/Set binding/lowering without Office globals or default-member inference.

Implemented explicit frames, typed owned locations and canonical direct ByRef aliases, frame-local
operand stacks, quotas/hard work and copy budgets, saved call/Invoke fault PCs, fresh checkpoint
visits and single-use continue permits. Added source breakpoints, into/over/out stepping, generation-
checked read-only inspection, validated active-frame edits and source staleness checks. Stop and
optional read-only fault pauses continue without replay. Bounded snapshots explicitly report local
truncation. Cancellation, host disconnect/loss, recursion/storage/stack/payload limits are explicit.

Connected generic native host dispatch with per-site permissions, exactly-once wait/resume continuations
and cancellation. vba-foreign's checked APIs validate return payload and all typed patches before
its sole commit loop. Foreign alias snapshots are refused; direct ByRef remains canonical. Tests
preserve successful direct alias/host effects after errors and distinguish VBA numbers from engine
failures. The VM still has no compiler/debug/UI dependency and no ambient IO.

Validation: cargo xtask ci, cargo xtask wasm, cargo xtask wasm-run and release workspace tests.
The workspace has 74 integration tests: prior 54 plus 3 runtime, 10 VM execution, 6 VM host and
1 additional CLI execution case. Native and actual wasm32-wasip1 execution run the same Rust
m3_smoke acceptance under Node 18.19.1/Rust 1.99.1: Long(42), If/For/calls/ByRef Long(18), host
pause/edit/resume Long(41), one invoke/two polls, stale handles, host-token and infinite-loop
cancellation. Core libraries also compile for wasm32-unknown-unknown; CI now runs the shared
native/WASI smoke gate using Node 22. Corpus integrity remains seven pins/646 hashes. No Office
oracle, browser UI, remote cross-platform run or separate MSRV test is claimed by these local checks.

See [runtime contract](runtime.md) and [ADR 0007](decisions/0007-m3-execution-and-debugger.md).
The original M0 fixture manifests and external execution inventory remain NOT_RUN; there is still
no fixture assertion runner. M3's explicit runtime examples/tests have actual execution evidence.
The required graphical editor/debugger, DAP and fork-owned Office integration remain later gates.

Next smallest task: superseded — the M4 runner and numeric widths are implemented above; see the
M4 section for the current next task.

## Pre-push verification of M0–M2 (historical stage record)

On 2026-10-26, reran `cargo xtask ci` (formatting, warning-free all-target Clippy, all 54
workspace integration tests, layer checks and available corpus integrity), `cargo xtask wasm`
(all 14 libraries), and `cargo test --workspace --release --locked` (the same 54 tests in release).
All passed. Strict corpus verification checked all seven pins and 646/646 hashes; the opt-in
five-file grammar_corpus example matched all reviewed syntax classifications. `git diff --check`
passed. Corrected the fixture-schema documentation's stale claim that compile is unavailable.
These checks cover implemented M0–M2 behavior. VM/Office execution remains unavailable; WASM
runtime, other operating systems and separate MSRV testing are not claimed by these local runs.

### Cross-platform checkout correction

The first pushed M1/M2 commit passed Linux/macOS, WASM and corpus CI; Windows exposed Git's
CRLF checkout conversion changing source byte offsets and golden line endings. Added root
`.gitattributes` with `text=auto eol=lf` so tracked text fixtures retain their reviewed bytes on
all platforms while auto-detected binaries remain untouched. Explicit CRLF/lone-CR parser
inputs stay covered by authored tests. A fresh `core.autocrlf=true` checkout preserved LF fixture
bytes and passed `cargo xtask ci`, including all 54 tests, without corpus downloads. Cross-platform
CI is rerun after this fix; source-map tests stay exact rather than normalizing parser input.

## M2 completion (historical stage record)

Implemented ordered module/procedure scopes and case-insensitive lookup, Option Explicit, scalar
parameters/locals/Function result slots, implicit Variant locals, literal constants, positional ByVal
and exact-type local ByRef binding. Unsupported persistent storage, class/foreign binding, deeper
argument and constant/numeric forms fail explicitly with source spans. Binding preflights AST depth
and budgets source/token/node payload and constant substitution before copying.

Added vba-compiler for parse/bind orchestration and checkpoint-based lowering, including If/ElseIf,
restricted positive integral-step For, direct nested calls, Set/Let modes, Exit and Stop. Retained
same-line/control statement identities and full statement tables after unreachable code removal.
Constants are deduplicated by exact tag/payload in first emitted-use order. Parameters/result/user/
scratch slots have explicit roles. New reviewed edges preserve IR -> diagnostics/values only;
semantics now shares ScalarKind from values. No external Rust dependency or corpus execution added.

Implemented every IR 1 instruction's structural verification, immutable opaque VerifiedProgram,
launch/signature/slot/call/foreign/source ownership checks, V/Location stack simulation, exact maxima,
iterative reachability, bounded verification and exact-tag deterministic disassembly. M2's explicit
ir1-structural capability target is not a runtime implementation. Unknown versions/features/op
capabilities, malformed IR, unreachable blocks and over-budget input cannot create VerifiedProgram.

CLI check/compile/disasm now work. Arithmetic's full reviewed golden matches the nine-instruction
IR 1 example with Integer 6/7, Long result and max_stack=3. M2Control.bas independently demonstrates
If/For/nested calls and ByRef lowering. Execution remains unavailable and all M0/external scenario
execution results remain NOT_RUN. See [compiler contract](compiler.md) and
[ADR 0006](decisions/0006-m2-binding-and-verification.md) for supported forms, budgets and limitations.

Validation: `cargo xtask ci` and `cargo xtask wasm`; compiler/IR/binder/CLI tests and documented
compile/disasm examples. The workspace now has 54 integration tests (8 compiler, 9 IR, 4 binder,
7 CLI, 13 frontend, and 13 existing M0 library tests). Native tests include malformed-IR mutations,
all instruction stack shapes, each verifier budget dimension, exact source diagnostics and bounded
constant amplification. Library WASM evidence is compilation only. No Office oracle, runtime/WASM
execution, macOS/Windows or separate MSRV run is claimed. Corpus integrity remains seven pins and
646/646 candidate hashes; the M1 five-file syntax report is unchanged.

Next smallest task: M3 VM explicit frames/typed local storage/checkpoint dispatcher with hard/slice
budgets and actual capability checks; execute Arithmetic.Answer and assert exact Long(42). Then
implement direct alias calls, debugger/host continuations and native/WASM pause/resume proofs.

## M1 completion (historical stage record)

Implemented a pure-Rust bounded lexer/parser in vba-syntax with retained source and all trivia tokens,
exact checked spans, spelling-preserving names and distinct same-line statement identities. The AST
covers scalar declarations/parameters, Sub/Function, module/class attributes/options, expressions,
assignment/Let/Set, If/ElseIf/Else, numeric For/Next, explicit/implicit calls, Exit and Stop. Named,
omitted and optional syntax is preserved without claiming binding or IR 1 execution support.
SourceFile coordinates now handle LF/CRLF/lone CR consistently. Parser recovery never reports syntax
success, and source/token/nesting/tree/argument/diagnostic limits bound adversarial input.

Added `vba-cli parse <source.bas|source.cls>` with bounded UTF-8 reads, exact source diagnostics and
nonzero rejection. Both self-authored examples and all 20 M0 VBA bodies parse, as do the new class
metadata/control fixtures. All execution results stay NOT_RUN; no binder, IR verifier, VM or UI exists.
The development CLI -> syntax edge is recorded in the reviewed DAG; no dependency was added to syntax.
See [frontend support and limits](frontend.md) and [ADR 0005](decisions/0005-m1-frontend.md).

Reviewed five directly authored MIT tree-sitter-vba examples for explicit syntax-only ingestion.
The two nominally valid modules are UNSUPPORTED (Select Case/property procedures); all three broken
examples are REJECTED at source diagnostics. Five classifications matched review; zero executable
compatibility passes are claimed. [The report](m1-corpus-report.md) records pins, hashes, rights,
capability/profile restrictions and reproducible invocation. Normal tests need no corpus checkout.
No upstream implementation, generated parser, downloaded script or macro was executed or copied.

Validation: `cargo xtask ci`, `cargo xtask wasm`, the opt-in grammar_corpus example and documented
parse commands passed. Thirty-two integration tests now cover infrastructure/frontend/CLI, including
13 frontend tests for preservation, exact source spans, control metadata, call/precedence distinctions,
unsupported/malformed input, nested control statement identities and token/tree/recovery/identifier limits.
Member identifiers enforce the same 128-byte limit as standalone identifiers. The deterministic malformed/truncated
stress check is not a fuzzing campaign. Corpus integrity remains seven pins and 646/646 hashes.
WASM evidence is compilation only; macOS/Windows and a separate MSRV run were not executed locally.

Next smallest task: M2 bound module/procedure scopes and Option Explicit/local/result/parameter binding
against the M1 AST, then the IR 1 program layouts, verifier/disassembler and real compiler/lowering crate.
Numeric semantic legality, property/Select Case/array syntax and the broader grammar remain unsupported
as documented; M1 completion covers its reviewed subset, not the full supplied grammar wishlist.

## Bytecode/VM contract prepared before M1 (historical stage record)

Authored the adopted IR 1 bytecode/VM implementation specification and ADR 0004. The document
specifies exact instruction operands/stack effects, program/constants/signatures/slots, source
checkpoint mapping, canonical direct ByRef, restricted M0 host adaptation and single-use host
continuations. It defines verifier/disassembly rules, explicit execution states and errors,
checkpoint visit permits, stepping/inspection generations, resource defaults, lowering examples
and M2/M3 positive/negative acceptance obligations. A planned upper-layer vba-compiler owns
compilation/lowering when implemented; no package/dependency graph change is made now.

Updated architecture, debugger, plan, compatibility, development entry points and relevant skills.
This is documentation only: no instruction enum, verifier, compiler, VM, debugger or runtime
behavior was added. All M0 VBA fixtures and external corpus candidates retain their prior status.
M2/M3 remain unimplemented. M1's bounded lexer remains the next smallest implementation task.

Validation: documentation links and instruction/reference consistency reviewed; `cargo xtask ci`
and `cargo xtask wasm` passed after the documentation change. Existing 16 integration tests still
cover M0 only; no bytecode acceptance case is claimed to have run.

## M0 completion

- Implemented SourceId, per-source StatementId, bounded SourceFile, checked UTF-8 byte spans and
  attached diagnostics, including Unicode/CRLF/identity/bounds tests.
- Implemented scalar Value states/widths, scaled Currency/serial Date, exact UTF-16 strings and
  bounded, invalidatable owner-scoped locations. No runtime coercion or arrays yet.
- Implemented bounded schema-1 fixture parser/validator and 20 independently authored MIT pure-language
  cases with typed expectations, rationale, requirements and explicit unmeasured evidence. All remain NOT_RUN.
- Added `vba-cli conformance validate <scenario>`; CLI tests validate all 20 fixtures and reject malformed
  input. Execution commands still fail explicitly; fixture validation is not VBA syntax validation.
- Proved generic Call/Get/Let/Set, omitted/named/ByRef snapshots and ordered delayed writebacks against
  a deterministic scripted host. Tested errors, repeated waits without replay, cancel, stale locations/
  references, wrong host identity, invalid/non-ByRef patches, alias order and resource limits.
- Recorded adopted source/scenario/foreign prototype contracts, lifecycle requirements and M4/M5
  limitations in ADR 0003; updated architecture, reviewed dependency graph, layer checker and focused skills.
  The development CLI's conformance dependency is explicitly allowed; shipping libraries remain isolated.
- No third-party Rust dependencies, downloaded macro execution, Office observations, parser, VM or UI added.

Verification: `cargo xtask ci` passed (format, warning-free Clippy, workspace tests, layers and corpus
integrity); `cargo xtask wasm` passed for all 13 libraries. Sixteen Rust integration tests cover the
M0 infrastructure and CLI, including validation of all 20 fixtures. Corpus integrity remains seven
pins and 646/646 candidate hashes; this is not a corpus compatibility result. WASM runtime behavior,
Office compatibility, macOS/Windows execution and a separate MSRV run were not tested locally.

Next smallest task: M1 bounded lexer for Arithmetic.bas with exact spans and positive/negative token
fixtures, including comments, CRLF/continuations and colon-separated statements. Then implement the
source-preserving parser and record exact unsupported syntax; M0's cases stay NOT_RUN until execution exists.

## Setup history (pre-M0)


Created a standalone Cargo workspace with 13 library crates, vba-cli and xtask, each with src/.
Added current architecture, implementation plan, decisions, adapted AGENTS/CLAUDE and repository-local skill.
Preserved the supplied interpreter spec and copied the two workspace design documents unchanged.
Added test tier/fixture/example/fuzz/report locations and native/WASM/corpus CI jobs.

Cloned six documented test repositories and Microsoft VBA-Docs as pinned submodules.
Source lock tracks notices/revisions; candidate inventory tracks SHA-256, size and NOT_RUN status.
No downloaded macros, oracle scripts, interpreters or workbook files have been executed.
Contextures is tracked as a manual download source; no Word corpus was identified in the supplied docs.
At setup, no parser, binder, runtime, VM, fake-host implementation, project decoder or scenario runner existed.

Scope updated on 2026-10-26: this repository owns the reusable egui VBA editor/debugger and
standalone native/web IDE. ADR 0002 and the UI contract define ownership, Gridcraft design
alignment and the required visual acceptance gate. No UI packages or working UI exist yet.
The enforced Cargo graph is unchanged because this change defines scope and implementation gates.

Added seven shared Codex/OpenCode design skills under .agents/skills: development routing,
architecture, language semantics, debugger, editor UI, host integration and conformance.
AGENTS.md routes design/implementation work through relevant skills; docs/agent-skills.md records
shared discovery and usage. These are agent guidance, not new engine/UI implementation.
Validation: all seven skill names, descriptions, bodies and local routing links passed a
dependency-free structural check; bundled quick_validate.py could not run because PyYAML is absent.
`cargo xtask ci` and `cargo xtask wasm` passed after the skill changes. Discovery paths were
verified against official documentation; live Codex/OpenCode skill activation was not exercised.

The setup's next tasks (source identities, M0 fixture validation and 20 independent cases) are now
complete as recorded above. See the plan for remaining milestones.

## Setup validation

- `cargo xtask ci`: passed on Linux with Rust 1.99.1 (fmt, Clippy, workspace tests, layers and available corpus hashes).
- Two CLI integration tests passed; these check explicit refusal and help, not VBA execution.
- `cargo xtask wasm`: all 13 libraries compiled for wasm32-unknown-unknown.
- Strict corpus verification: seven Git pins and 646 candidate file hashes checked; no macros executed.
- Current documentation/skill links and unchanged source document copies checked.
- macOS/Windows jobs are configured, not run locally. No MSRV run, fuzzing or VBA runtime tests yet.

## M5 callback and UDF implementation slice — accepted profile (2026-10-30)

[ADR 0026](decisions/0026-document-module-code-name-model.md) records document-module code names
in the IDE. [ADR 0027](decisions/0027-document-callback-delivery.md) now specifies the shared
host-authorized callback seam: a verified predeclared document class is selected by code name,
its instance is created lazily, and callbacks (including first-use initialization) run through the
existing bounded session queue and debugger frames. The host remains responsible for trust and
captured arguments. ADR 0027 attaches the live host owner to the VM document instance; VM-authored
members remain local and unresolved `Me` members use the ordinary foreign permission/dispatch path.

Shared regressions cover inert code-name validation and callback selection by VBA code name, including
lazy `Class_Initialize` ordering. Both pass on native and WASI. `cargo xtask ci`, `cargo xtask wasm`
and `cargo xtask wasm-run` pass; the new IDE regression and the complete VM class suite pass natively.
These checks establish the generic shared callback contract only; they do not establish Office event,
recalculation or unchanged-workbook acceptance.

GridCraft's working `feat/m5-document-events-udfs` branch adds workbook/sheet `codeName` persistence,
inert project import, explicit open/change queue APIs, bounded `EnableEvents`, selected UI and VBA
write notifications, declared Range/Object UDF parameters as read-only Range handles, `Caller`,
`Volatile`, and persistent UDF project state. Variant and scalar UDF parameters keep the regular
M4 value/array conversion while the calculation call retains reference provenance. The UDF VM now
receives GridCraft's shared object type resolver. The calculation graph tracks per-formula UDF
volatility; unknown UDF calls no longer force volatility, so a later `Application.Volatile(False)`
stops unrelated-edit recalculation. UDF `Range.Value` and `Value2` share the regular typed/untyped
M4 read conversion.

The UDF read-only host now exposes Caller/Range Address, Row/Column, Count, Worksheet, Offset and
Resize, along with worksheet Name/CodeName and parent workbook CodeName. It continues to refuse
cell writes, workbook mutations and asynchronous calls.

M5 is accepted for the declared profile. Worksheet callbacks scope unqualified
Range/Cells/Rows/Columns construction to the event owner without changing active selection. Live
`Me` forwarding is exercised against a real GridCraft workbook while another sheet is active.
Independently authored tests cover inert document import, explicit Workbook_Open,
FIFO/exactly-once event delivery, EnableEvents tail cancellation/re-enable, a full 256-event chain
followed by a fresh chain, breakpoint pause/resume, Range identity, Caller, Static state, volatility
on/off and native/direct-control/cell-editor/drag-cut edit sources. The VM document object remains a
VBA instance forwarding unresolved members to the captured host owner. GridCraft event notifications
cover the reviewed native mutation set; reset/close/trust cancellation, queue exhaustion, callback
partial effects and form callbacks pass authored acceptance. A verified idle IDE session
now lets callbacks run from Design mode without an unrelated Main invocation. `Workbook_Open` is
exposed through an explicit, trust-checked command and never runs on import. Native event coverage includes cell/range writes, AutoFill/Series, directional fills, Flash Fill, sort, remove-duplicates, Text to Columns, content clears, Clear All and paste; formats-only paste/fill is suppressed. Callback failure cancels the remaining queued batch while already committed writes remain grouped for undo. On queue-budget
exhaustion, native UI edits remain committed but the callback is reported as not queued, while VBA
Range writes preflight and refuse before mutation. This is now recorded in GridCraft's VBA contract.
Cell-editor commits, direct control commands and drag-cut now enter the same post-commit event hook,
so these UI paths do not silently bypass the notification policy.
The idle session compiles the full project with a temporary inert standard entry stub; private
document handlers remain callback-only, and authored breakpoints are rebound before delivery.
GridCraft synchronizes sheet identities before write-event preflight and checks the bound workbook
still exists without requiring it to be the active workbook. Its embedded IDE
reuses the M2 standard-form runtime through the same host; `.tvform` remains external exchange data,
not workbook VBA metadata. The 256-event budget now resets at the idle boundary before a new native
or explicit Workbook_Open event is preflighted, so a fully used prior chain cannot starve later work.
Native notifications now include reviewed command sources for content writes, paste/clear,
fill/series, Flash Fill, sort, duplicate removal and Text to Columns; that command reports its
committed output bounds, including a custom destination. GridCraft authored VBA tests, a native
cell-edit UI test and calc tests now pass. Event tests cover breakpoint/resume exactly once;
Reset/close/trust cancellation; queue exhaustion before VBA mutation; and callback failure cancelling
the tail while retaining an undoable earlier write. Worksheet UDF tests now cover Range identity,
Caller, Row/Column, Address, Offset/Resize, Date/Currency typed `Value` versus numeric `Value2`, and
refusal of `Application.OnTime` with no workbook mutation. GridCraft now imports `.tvform` files via
the embedded IDE Import action and routes TextBox/button callbacks through the workbook host; the
modal completion/close test leaves no wait token. M5 acceptance E01–E10 now pass. Embedded UI
acceptance covers native and direct-control cell edits, the cell-editor commit, an actual drag-cut
gesture with source and destination callbacks, 15 content-edit commands, format-only suppression
and no-op duplicate removal. See [`m5-acceptance.md`](m5-acceptance.md) for the exact scope and
test record.

## M6 shared callback output API (2026-10-30)

[ADR 0028](decisions/0028-callback-byref-output.md) defines the shared return path for host-scheduled
callback `ByRef` parameters. The VM captures final parameter values at normal callback completion,
preserving their argument indices, and the IDE exposes one-shot FIFO consumption and explicit
clearing to host adapters. Results are bounded by the existing session payload/work and callback
budgets; callbacks without ByRef parameters retain no output snapshot. Failed/cancelled callbacks
do not create an output, and reset clears retained results.

Independently authored VM regressions verify callback `ByRef` mutation is returned exactly once
and that a failed callback does not expose a partial output. `cargo xtask ci` passes across the
complete tiny-vba workspace after this API. WordCraft uses the result for its explicit
save/close cancellation extensions. These document-module hooks are not standard Word
events: Word's cancellable `DocumentBeforeSave` and `DocumentBeforeClose` events belong
to Application. WordCraft records the canonical `Document_Close()` path and its extension
contracts in its own M6 integration profile. This shared API does not itself establish Office behavior.

## M6 contextual named argument labels (2026-10-30)

WordCraft's independently authored `Range.SetRange End:=..., Start:=...` workflow
exposed a parser refusal of reserved-word argument labels. The generic syntax parser
now uses its existing contextual member-name grammar for the label before `:=`.
It preserves exact spelling/spans and identifier budgets; ordinary declarations still
reject reserved identifiers. A generic parser regression covers `End:=` and `Type:=`;
the real document execution and argument/error contract remain WordCraft-owned.

## Completed M5/M6 plan scope

Moved from the remaining plan on M6 sign-off, 2026-11-01. Task IDs and the
implemented profile remain traceable; fresh Office comparison is now M8.5.

### M5 completion record — Document modules, events and Range-aware UDFs

Owner: tiny-vba shared class/document/callback semantics; GridCraft document
bindings, event ordering, standard Office controls and calculation context.
Depends on applicable M2–M4 capabilities; reuse shared contracts.

- **M5.1:** Execute workbook/worksheet modules with code-name roots, correct
  signatures and code-owner context while keeping import inert.
- **M5.2:** Deliver Workbook_Open/Worksheet_Change and selected document/standard
  control events with defined ordering, EnableEvents, reentrancy/recursion
  budgets, cancellation and partial-side-effect policy. Connect M2 forms.
- **M5.3:** Add Range/reference-aware UDF arguments and Caller/Volatile/context
  behavior, persistent state and recalculation rules. Enforce the declared UDF
  mutation policy without silently substituting value arrays for references.

Accepted for the declared profile. Unchanged authored handlers and UDFs execute
against real GridCraft state. Pause/inspect/resume, exactly-once dispatch,
reset/close/trust revocation and calculation dependencies pass. Workbook_Open
never executes on import. The [M5 acceptance map](m5-acceptance.md) records all
ten passing scenarios and the exact native, WASM and authored-test evidence.

### M6 — Completed GridCraft and WordCraft integration

Owner: application forks for Office objects, UI/storage/trust/events/undo/recalc;
tiny-vba for shared embedding APIs. M1 supplies generic types; M2 supplies shared
classes/forms. Excel workflow acceptance consumes the applicable M3–M5 behavior.

Completed for the declared profile (2026-10-31), with the [M6.5 per-layer
check](m6-context-resolution-check.md) as the scope-driving record:

- **M6.5:** the check records implemented, refused and missing behavior per owned layer
  against [module types and execution contexts](vba_module_types_and_execution_contexts.md) and
  [Excel context and object resolution](vba_excel_context_and_object_resolution.md), and the
  [compatibility matrix](compatibility-matrix.md) now carries the result.
- **M6.6:** staged identifier resolution over locals/parameters/module/project declarations,
  host-declared implicit `Me` members for document modules, then host globals; per-module-kind
  binding; local/member shadowing with project-before-global precedence; innermost-only
  leading-dot `With` receivers that never change `Me` or active state; standard-module-only
  bare cross-module callees; no implicit current-workbook frame parameter.
- **M6.7:** the forks dispatch declared host globals per invocation against live application
  state with distinct `ThisWorkbook`/`ActiveWorkbook`/`ActiveSheet`/`Me` identities and stable
  code names; GridCraft proves the specifications' unshadowed binding and live-consultation
  cases in real workbook workflows, and WordCraft supplies the document roots under its M6.1
  profile. `Application.Run`, add-in context and chart sheets stay refused/NOT_RUN in the check.
- **M6.8:** IDE member completion types receivers from host-declared import result types and
  call shapes (`ThisWorkbook.`, `Range("A1").`, `Worksheets("Data").`, `Cells(1, 1).`) plus
  `Me`/code-name document types; each fork declares the member/result types it supports in its
  catalogue. Code aids stay read-only and metadata-driven; they never invoke host members.

M6 is signed off for its tested profile. Fresh Office comparisons formerly
associated with M6.4 moved to M8.5 by owner decision; they remain NOT_RUN.

#### Attribute and document-module corrections (2026-11-01; implemented)

The [attribute/editor investigation](vba-attributes-and-editor-projection.md)
records online sources, the two screenshot failures and local code findings.
M6.9–M6.11 are implemented under [ADR 0030](decisions/0030-attribute-metadata-and-code-projection.md).
M6.12 has native/WASM/WASI, GUI and actual fork regression evidence, recorded in
[acceptance](m6-acceptance.md). Office observations remain unavailable.
Document code-name renames use a host acknowledgement contract; the current forks
explicitly refuse them because binary project writeback is outside this profile.

- **M6.9:** tiny-vba syntax/semantics/compiler: parse module and member attributes
  as owned metadata at their documented positions, including procedure-local
  `Macro2.VB_ProcData.VB_Invoke_Func = " \n14"` and signed member IDs. Preserve raw
  spans/values and associate targets; validate placement and literal forms.
  Retain opaque metadata without claiming semantics; diagnose unsupported
  semantic requirements. Generate no executable statements/checkpoints for
  attributes. Ordinary unsupported VBA must keep its diagnostics.
- **M6.10:** tiny-vba IDE/shared UI: default to a code view omitting recognized
  attribute/export-header ranges while retaining canonical raw source. Provide
  a bounded reversible mapping for edits, CR/LF/CRLF, Unicode, diagnostics,
  completion, search/replace, navigation, breakpoints and full/procedure views.
  Preserve hidden records through insertion, member rename/delete, undo/redo
  and revision-checked plain-source save/export. Add a read-only raw metadata
  inspector; do not strip source in the read-only importer.
- **M6.11:** tiny-vba IDE/shared UI plus fork integration: an attribute-only
  `ThisWorkbook` opens as blank code while the document component remains in
  the explorer. Present its code name in explorer/Properties `(Name)`, separate
  from `.cls` source/export filenames; preserve event navigation and host-bound
  identity. Route rename through host validation and consistent metadata updates.
  Expose supported descriptions/Instancing without making host-owned attributes
  arbitrary editable code. GridCraft owns Excel Macro Options and shortcut policy.
- **M6.12:** regress the exact member-attribute shape with a minimal independently
  authored executable procedure, invalid/unknown metadata, signed IDs, default
  member/enumerator semantics or explicit refusals, attribute-only documents,
  mapped debugging/editing and exact raw-source retention. Run native/WASM/WASI,
  shared UI/native/browser gates and real fork workflows. Keep the workbook's
  unrelated host-method gaps separate. Office reference-build observations stay
  NOT_RUN until actually collected; import never executes handlers.

Acceptance: both forks reuse the same editor and shared interpreter repository.
Real document workflows verify effects and lifecycle without replay or retained
locks; tiny-vba-only snapshots and generic doubles cannot satisfy this gate. The
M6.5 record drove M6.6/M6.7 scope, and acceptance includes the specifications'
unshadowed module-kind binding and live host-global consultation cases in real
fork workflows. Declared host globals and typed/call receivers complete their
members in the shared editor without host invocation. Package round trips and
signatures are the next milestone's separate gate.

Validation for ADR 0036: `cargo xtask ci`, `cargo xtask wasm` and `cargo xtask wasm-run` passed. All seven suspended-call tests, 15 compiler tests and the qualified-type regression also passed under Node WASI. These are authored generic contract tests, not fresh Office oracle evidence.

ADR 0036 fatal-policy refinement: exact-token `abort_host_wait` stops all parked/modal invocations with a runtime diagnostic when the embedding requires it. Generic error-return behavior remains available; Office error policy stays in GridCraft.

## Completed M7 plan scope

Moved from the remaining plan on 2026-11-04 after authored-profile acceptance.
The following definitions preserve task IDs and the original requested scope;
[M7 acceptance](m7-acceptance.md) records actual results and profile limits.

### M7 — Unchanged-document and package acceptance

Owner: fork acceptance/package layers, coordinated with tiny-vba source-import
and evidence contracts. Starts after M6A acceptance and depends on each
specimen's M1–M6 and intrinsic capabilities; form
specimens require M2 and the corresponding host event/persistence integration.
Native Excel worksheet Form Controls, binary UserForm import and expanded control
support are deferred outside this delivery profile; see the
[Office forms and controls plan](office-forms-controls-implementation-plan.md).
Classify specimens needing those capabilities as UNSUPPORTED/NOT_RUN rather than
expanding M7 into their implementation. Existing `.tvform` acceptance stays scoped
to its supported shared/host workflow.

- **M7.1:** Curate licensed, reviewed Excel workbooks and Word documents with
  declared language/object/reference/standard-control requirements and independently
  established expected results. Keep candidate/unrun material separate.
- **M7.2:** Report load, compile, run and save separately for unchanged source.
  Verify source identity, typed values/errors, effects/events, code names,
  package relationships/payloads, trust and lifecycle for .xlsm/.docm workflows.
  Include M6.9–M6.12 attribute retention and code-view/raw-source mapping:
  unchanged headers/member metadata survive load and plain-source export,
  and code edits preserve attached hidden records. Report these separately
  from original-package preservation and edited binary writeback.
- **M7.3:** Design binary VBA project writeback and signature preservation/
  invalidation as a separate reviewed contract before implementation. Distinguish
  original-byte preservation from edited-source writeback. No protection/P-code
  bypass or unrestricted binary add-ins.
- **M7.4:** Execute the two specifications' module-context and name-resolution
  matrices ([module types](vba_module_types_and_execution_contexts.md),
  [object resolution](vba_excel_context_and_object_resolution.md)) as
  unchanged-document evidence: per-module-kind binding with a different workbook
  active, `With` versus bare identifiers, shadowing, code names after tab rename,
  late binding, and chart-sheet/add-in cases where licensed specimens permit.
  Exercise the adopted implementation contracts and record
  PASS/MISMATCH/UNSUPPORTED/NOT_RUN per case, with each expectation’s basis
  (authored contract, public specification or inherited observation). Fresh
  comparisons against Excel/Word reference builds belong to M8.5; do not label
  these M7 checks as fresh Office observations or change precedence from an
  unverified assumption.

Acceptance: per-document profile/version/evidence records distinguish PASS,
MISMATCH, UNSUPPORTED and NOT_RUN. Verify declared supported forms/controls, source
attachment, no untrusted auto-run and signature validity/loss decisions.
Run both repositories' required CI/WASM gates and real fork workflows.
Publish incompatibilities; subset success does not imply universal compatibility.
