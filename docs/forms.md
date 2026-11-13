# Native forms and persistence

The supported runtime profile is `native-forms-common-v1`. Native `.frm`/`.frx`
files and admitted embedded Office resources attach without executing VBA.
The retired custom form format has no shipping codec, converter or file route.

## Controls and runtime state

Common controls are Label, TextBox, CommandButton, CheckBox, OptionButton,
ToggleButton, ComboBox, ListBox, Frame and MultiPage/Page. Selected SpinButton,
ScrollBar, TabStrip, Image and Excel RefEdit behavior is also retained. Support
applies to tested combinations and admitted resource records, not every member
of the corresponding Office interface.

Native control classes use ordinary VM objects and fields. Form instance identity
is distinct from designer/source identity. Predeclared and explicit `New` instances
have independent state. `Load`, modal/modeless `Show`, `Hide`, `Unload`, close
cancellation and ordinary lifecycle handlers use the same frames and debugger
continuations as other VBA. Inspection does not instantiate objects or call getters.

| Area | Supported contract |
|---|---|
| Properties | Applicable Name/Caption, geometry, Enabled/Visible, TabStop/TabIndex, Tag and ControlTipText |
| Text | UTF-16 Text/Value and selection, MaxLength, Locked, multiline/password input, HideSelection, AutoTab, SelectionMargin and bounded TextBox viewports |
| Buttons | Click, Default/Cancel, TakeFocusOnClick; applicable Boolean Value/Null/TripleState and scoped OptionButton groups |
| Lists | Bounded row mutation, indexed List/Column load/readback, selection, ComboBox styles/matching/drop state |
| Containers | Parent/Controls/ActiveControl, nested focus and groups, Page/Tab identity, snapshot enumeration and selected dynamic collection mutation |
| Events/input | Common focus/validation cancellation, key/mouse callbacks, Tab/Space/Enter/Escape and selected accelerators |
| Appearance | Applicable OLE colors, typed Font identity/attributes, borders/effects, alignment and wrapping within fixed authored bounds |
| Binding | Explicit-capability single-cell ControlSource and contiguous RowSource links; the host owns real range access |

Executable input requires host authorization. Raw input/default processing waits
for ordinary callback completion; Stop/Continue must not replay an event or write.
Reset, unload, trust loss and context changes discard stale continuations. The
renderer reads bounded inert snapshots and must not replace host styles or fonts.
Detailed defaults, coercions, native flag mappings and refused cases are indexed
under [native form decisions](decisions/README.md#native-forms).

## Designer

The shared designer supports form/control caption, text, geometry,
enabled/visible state and eligible tab order. Toolbox addition/removal covers
Label, TextBox and CommandButton. Imported controls retain native identities and
resources. Designer state and authored source survive VM Reset; generated support
code is kept separate from saved authored source.

## Native Save and Export

A native form snapshot contains exact authored source, design, native identities
and resources. Save acknowledges only that revision after a successful commit;
Export retains the editor's modified state. A stale or foreign acknowledgement
cannot clear later edits.

The shell publishes the content-named `.frx` companion completely and exclusively
before atomically replacing `.frm`. A differing existing companion refuses, and a
failed commit retains the previous primary file. Existing private file modes are
preserved. A failed commit can leave an unused companion; remove it only when no
form refers to it. Codecs consume caller-provided bytes and perform no filesystem IO.

Browser export offers both files for download. Requesting a download does not
prove a successful disk save, so it does not acknowledge persistence.

## Edited workbook/document copies

GridCraft's **Export Edited XLSM Copy** and WordCraft's **Export Edited DOCM Copy**
compile the idle/reset project without execution, encode supported source/forms,
remove invalidated VBA signature parts and relationships, and reopen the result
to verify exact source/design and component/code-name/reference identity.
The chooser refuses the current document path, including canonical aliases.
Copies retain the original editor's modified state and imported binary bytes.
Ordinary document Save preserves the original VBA project; open the exported copy
to use persisted authored edits.

Source-only MS-OVBA output uses zero module text offsets and omits module prefixes,
SRP caches and stale free-sector cache bytes. Protected projects, read-only edits,
identity changes, unrepresentable code-page text and unsupported resource edits
refuse explicitly. No P-code execution, protection bypass, signature verification
or resigning is implemented. See [persistence decisions](decisions/README.md#persistence-and-release).

## Worksheet controls

The `worksheet-form-controls-v1` profile belongs to GridCraft. Nine standard
worksheet Form Control families use native Shape identity/state, engine
bindings, protection, undo/recalculation and OOXML/VML edited persistence.
The optional VBA adapter provides ControlFormat APIs, trusted launches and input;
failed identity joins remain inert. This is separate from UserForms and does not
establish worksheet MSForms/ActiveX support. See ADR 0041 in the decision index.

## Compatibility boundary

Exhaustive reentrant lifecycle and alias behavior, platform/IME/DBCS input,
non-ASCII/duplicate accelerators, wheel/repeat parity, AutoSize/container scrolling,
advanced binding types/timing, additional picture/RefEdit variants and worksheet
Office-supplied ActiveX are outside the accepted common profile. Third-party
controls and arbitrary COM/OCX/DLL activation are excluded. A defect in an included
common workflow remains a defect; this boundary does not reclassify it.

[Testing](testing.md) links the immutable acceptance selections and executable
regressions. Fresh Office comparison remains **NOT_RUN**. Historical M7A/P9 labels
in those selections describe evidence scope, not a current task list.
