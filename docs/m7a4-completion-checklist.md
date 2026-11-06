# M7A.4 completion checklist

Updated 2026-11-06. M7A.4 remains IN PROGRESS. This makes the existing runtime
ledger actionable; it does not replace or reduce the native control contracts,
inventory, or implementation plan. PASS below requires executable evidence;
reference-based expectations and fresh Office observations remain distinct.

| ID | Required acceptance cases | State |
|---|---|---|
| C01 | Projects with no form components: bare/qualified UserForms, Count=0, aliases/TypeName, empty For Each, Item bounds, Add of missing form, shadowing, Stop/Reset/source edits, document callbacks | PASS: ADR 0069; seven shared cases and both pinned host suites |
| C02 | Additional default/control-reference aliases and reloads; implicit reference behavior after unload and reentrant mutation | PARTIAL: ADR 0044/0067/0068; remaining cases open |
| C03 | Dynamic collections and structural changes during iteration; review direct enumerator cursor applicability against Office's exposed interfaces | PARTIAL: snapshot profile retained; remaining parity open |
| L01 | Activate/Deactivate across Show/Hide, multiple modeless windows, document focus, repeated activation and unloading | PARTIAL: full ordering/focus ownership open |
| L02 | Lifecycle handlers redirect focus, show/hide/unload another form, Stop/resume or Reset without stale ownership or replay | OPEN |
| F01 | Recursive SetFocus and cancellation, nested containers/page switches, dirty validation and binding callbacks, boolean update timing | PARTIAL: selected focus/validation implemented; edge cases open |
| K01 | KeyDown/KeyPress/KeyUp with mutable ByVal MSForms.ReturnInteger and typed Shift; cancellation/default processing in ordinary frames | OPEN |
| K02 | Held-key repeat; modifiers; Tab/navigation focus transitions; Default/Cancel Enter/Escape exceptions; ANSI input versus Unicode/IME/paste | OPEN |
| K03 | Accelerators select/focus/activate the appropriate eligible control and preserve keyboard/event ordering | OPEN |
| M01 | Applicable MouseDown/MouseMove/MouseUp/Click/DblClick, typed button/modifier/coordinate/cancellation arguments, event order and container-relative coordinates | OPEN |
| P01 | Remaining common properties, defaults, exact errors/coercions, text/selection edge cases | PARTIAL: remaining contract/member audit open |
| B01 | List/Column array shape/loading, binding value types and update timing, protected failures and reentrant handlers | PARTIAL: current selected array/binding packets implemented |
| W01 | Frame, TabStrip/Tabs, MultiPage/Pages: remaining storage/resource/property/presentation cases, transitions, focus and lifecycle | PARTIAL |
| W02 | Image/picture: remaining formats, metadata, loading/platform services and events/presentation | PARTIAL |
| W03 | RefEdit: remaining platform, property/event and picker behavior | PARTIAL |
| W04 | ListBox/ComboBox and ScrollBar/SpinButton: remaining property/resource/presentation/event contracts | PARTIAL |
| A01 | Applicable Office-supplied worksheet ActiveX/platform packets in the owning host | OPEN |
| V01 | Every implementation above has appropriate native/WASM, debugger continuation, trust/lifetime and affected host regression evidence | Required per change; no blanket milestone PASS |

The native control/property/event matrix remains authoritative for the member
and resource audit. An unsupported behavior required by that inventory remains
open; describing its refusal does not complete it. Third-party controls and
arbitrary COM/OCX loading retain their existing exclusions.

The keyboard contract is specifically Microsoft Forms: physical key events use
ReturnInteger and Shift flags 1/2/4; ANSI keypress delivery has its own eligibility
and repeat rules. Default/Cancel buttons suppress the corresponding Enter/Escape
physical key events. Focus changes can change the receiving control. An adapter
must not infer key events from a final Text/Value diff: paste, navigation and IME
need distinct input handling. The two reference pages describe focus-transition
KeyPress eligibility differently; preserve that as an explicit comparison case
rather than treating Access/Outlook controls as the MSForms oracle.

Native edited persistence is M7A.5; .tvform removal is M7A.6. Fresh Office
build/OS/locale comparisons remain M8.5 NOT_RUN. Neither moves unimplemented
M7A.4 behavior out of this checklist.

References and owning records:

- [Runtime ledger](m7a4-native-runtime.md)
- [Control contracts](m7a-control-contracts.md)
- [Native inventory](m7a-inventory.md)
- [MSForms physical key events](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/keydown-keyup-events)
- [MSForms KeyPress](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/keypress-event)
