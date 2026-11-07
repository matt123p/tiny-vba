# M7A.4 completion checklist

Updated 2026-11-07. M7A.4 remains IN PROGRESS. This makes the existing runtime
ledger actionable; it does not replace or reduce the native control contracts,
inventory, or implementation plan. PASS below requires executable evidence;
reference-based expectations and fresh Office observations remain distinct.

| ID | Required acceptance cases | State |
|---|---|---|
| C01 | Projects with no form components: bare/qualified UserForms, Count=0, aliases/TypeName, empty For Each, Item bounds, Add of missing form, shadowing, Stop/Reset/source edits, document callbacks | PASS: ADR 0069; seven shared cases and both pinned host suites |
| C02 | Additional default/control-reference aliases and reloads; implicit reference behavior after unload and reentrant mutation | PARTIAL: ADR 0044/0067/0068; remaining cases open |
| C03 | Dynamic collections and structural changes during iteration; review direct enumerator cursor applicability against Office's exposed interfaces | PARTIAL: snapshot profile retained; ADR 0080 fixes Page parent lookup under runtime-created MultiPages; remaining mutation/enumeration parity open |
| L01 | Activate/Deactivate across Show/Hide, multiple modeless windows, document focus, repeated activation and unloading | PARTIAL: ADR 0077 selected Show/Hide/modeless ordering, unload and explicit executable document activation; ADR 0078 adds selected automatic host document input; full ordering remains open |
| L02 | Lifecycle handlers redirect focus, show/hide/unload another form, Stop/resume or Reset without stale ownership or replay | PARTIAL: ADR 0076 selected nested input; ADR 0077 reentrant window handlers, stopped activation and cross-modeless key transfer; ADR 0078 adds selected document-input cancellation/replay; remaining lifecycle/focus/binding cases open |
| F01 | Recursive SetFocus and cancellation, nested containers/page switches, dirty validation and binding callbacks, boolean update timing | PARTIAL: selected focus/validation implemented; edge cases open |
| K01 | KeyDown/KeyPress/KeyUp with mutable ByVal MSForms.ReturnInteger and typed Shift; cancellation/default processing in ordinary frames | PARTIAL: ADR 0070 evidence plus ADR 0071 root/Tab/SetFocus/Space cases PASS (nine runtime, 11 new UI, Chromium, both hosts); remaining cases open |
| K02 | Held-key repeat; modifiers; Tab/navigation focus transitions; Default/Cancel Enter/Escape exceptions; ANSI input versus Unicode/IME/paste | PARTIAL: raw-input barrier and ADR 0071 focus/default ordering; remaining platform/shortcut/focus/IME cases open |
| K03 | Accelerators select/focus/activate the appropriate eligible control and preserve keyboard/event ordering | PARTIAL: ADR 0072 selected Label/button/boolean/Tab/Page actions; duplicate, non-ASCII/platform and further ordering cases open |
| M01 | Applicable MouseDown/MouseMove/MouseUp/Click/DblClick, typed button/modifier/coordinate/cancellation arguments, event order and container-relative coordinates | PARTIAL: ADR 0073 typed callbacks and selected simple-control capture/defaults; ADR 0074 selected TextBox caret/drag/word selection; ADR 0075 selected ComboBox/RefEdit/list/numeric/tab-header defaults; further ordering, background/drag/wheel/mutation and platform cases open |
| P01 | Remaining common properties, defaults, exact errors/coercions, text/selection edge cases | PARTIAL: ADR 0080 adds selected BackColor/ForeColor import, typed state and palette rendering; ADR 0081 adds selected Font data/objects/rendering; ADR 0082 adds typed Font reference assignment and reload; Nothing/external/other-assignment/ambient/platform font behavior remain open; ADR 0083 corrects Page ownership; ADR 0084 adds selected borders/effects/transparency and fixed-bound clipping; ADR 0085 adds published Font aliases; ADR 0086 adds selected TextAlign and matching pointer geometry; ADR 0087 adds selected caption WordWrap, file/runtime defaults and fixed horizontal text allocation; ADR 0088 adds Boolean-control WordWrap with accessible bounded captions; defaults and remaining member/platform audit open |
| B01 | List/Column array shape/loading, binding value types and update timing, protected failures and reentrant handlers | PARTIAL: current selected array/binding packets implemented |
| W01 | Frame, TabStrip/Tabs, MultiPage/Pages: remaining storage/resource/property/presentation cases, transitions, focus and lifecycle | PARTIAL: ADR 0080 adds selected container colours and runtime Page lookup; ADR 0083 corrects Page appearance ownership; remaining cases open |
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

Next keyboard acceptance packet: remaining platform/shortcut/IME and DBCS
replacement cases, cross-window lifecycle/focus changes and remaining non-text
control defaults. Root focus and selected Tab/SetFocus/Space cases follow ADR 0071.
No refusal counts as a completed inventoried behavior.

Next implementation packet: remaining M01 platform/order/selection/drag/wheel and
reentrant collection/binding cases, tab/page background hits and the unchanged
lifecycle, keyboard and property/member checklist. ADR 0075 adds the selected
composite/list/numeric/header defaults with capture and ordinary continuation.
ADR 0073 implements the callback and selected simple-control capture/default paths;
ADR 0074 adds selected TextBox defaults without deferring the remaining K03
accelerator/platform or lifecycle cases.

Next lifecycle acceptance packet: activation through other chrome/keyboard routes,
arbitrary mixed pointer/key ordering and further reentrant focus/collection/binding
cases. ADR 0079 adds selected coalesced form-key/paste/IME prefixes and spanning
composition before document deactivation, including stopped callbacks and modal
redirection. ADR 0078 implements the selected authorized canvas
barrier, Stop/Continue, Reset/identity/geometry invalidation, repaint acknowledgment
and native keyboard ownership. RefEdit keeps its existing modal selection path.
The complete K/M/property/binding/platform requirements above remain open.

ADR 0078 implements the selected pre-document input barrier and debugger/Reset/
identity continuation cases described above. Broader lifecycle/binding/collection,
all-key/mixed-input and platform acceptance remain open; this does not narrow any
required control/member or resource contract.
