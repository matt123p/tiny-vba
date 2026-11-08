# P9 — Extended Office forms compatibility

Status: PLANNED / deferred on explicit owner direction, 2026-11-08.
[ADR 0097](decisions/0097-common-forms-scope-and-p9.md) moves uncommon and exhaustive
compatibility work out of M7A.4. The owner-requested label is **P9**; existing M1–M8
IDs remain unchanged. P9 follows M8 and is not a prerequisite of M7A.5–.7 or the
current common-profile release gate. Existing implementations/regressions remain.

| Packet | Remaining work | Former checklist IDs | Owner |
|---|---|---|---|
| P9.1 | Unusual default/control aliases, reloads and implicit references after unload; structural mutation during enumeration, live cursor applicability and additional collection behavior | C02, C03 | tiny-vba IDE/runtime contracts |
| P9.2 | Exhaustive modeless/window/document activation ordering; arbitrary reentrant cross-window lifecycle/focus/validation/binding redirection and recursive cancellation combinations | L01, L02, F01 | shared IDE/UI; fork document activation |
| P9.3 | Exhaustive keyboard/platform/IME/DBCS, modifiers, held-repeat and shortcut behavior; duplicate/non-ASCII accelerators; remaining mouse ordering/background/drag/wheel/capture cases | K01, K02, K03, M01 | shared UI/IDE; platform adapters |
| P9.4 | Remaining property/default/error/coercion audit; AutoSize, form/container scrolling/Zoom/placement, Help/F1, mouse icons/pointers, font ambient/platform/Nothing/external cases, caret reflow after geometry/font changes and remaining appearance/selection variants | P01 | project metadata + shared IDE/UI |
| P9.5 | Further List/Column shapes/loading, ControlSource/RowSource value types, protected failure/update timing and reentrant binding cases beyond the accepted bounded links | B01 | shared forms; fork Range/document capability |
| P9.6 | Further Frame/MultiPage/TabStrip/Page/Tab resource/property/presentation/transition cases; additional numeric/list/combo members, resources and input contracts | W01, W04 | shared project/IDE/UI |
| P9.7 | Additional picture formats/metadata/platform loading/display/events and RefEdit platform/property/event/picker behavior | W02, W03 | shared project/IDE/UI; Gridcraft RefEdit |
| P9.8 | Applicable Office-supplied worksheet ActiveX packets, native metadata/runtime/host platform behavior and persistence when implemented | A01 | owning Office fork; shared contracts where reusable |

Every packet is OPEN for its remaining cases; implemented subsets below are
retained evidence, not a new completion claim. No refusal, byte retention or
inventory row counts as an implementation. The full member/resource inventory
in [control contracts](m7a-control-contracts.md), [native inventory](m7a-inventory.md)
and [runtime ledger](m7a4-native-runtime.md) remains available for P9 audits.
Any unimplemented uncommon variant previously required by those records belongs
here even when not individually named in the summary above.

## Migrated acceptance cases

The following original IDs preserve the previous checklist's full outstanding
requirements and partial evidence. Their remaining requirements now belong to
P9.1–P9.8 above; they no longer prevent common-profile M7A.4 completion.

| Original ID | Extended acceptance cases | Retained implementation evidence / outstanding state |
|---|---|---|
| C02 | Additional default/control-reference aliases and reloads; implicit reference behavior after unload and reentrant mutation | PARTIAL: ADR 0044/0067/0068; remaining cases open |
| C03 | Dynamic collections and structural changes during iteration; review direct enumerator cursor applicability against Office's exposed interfaces | PARTIAL: snapshot profile retained; ADR 0080 fixes Page parent lookup under runtime-created MultiPages; ADR 0096 retains captured input across transient writes and shifted peer indices; remaining mutation/enumeration parity open |
| L01 | Activate/Deactivate across Show/Hide, multiple modeless windows, document focus, repeated activation and unloading | PARTIAL: ADR 0077 selected Show/Hide/modeless ordering, unload and explicit executable document activation; ADR 0078 adds selected automatic host document input; full ordering remains open |
| L02 | Lifecycle handlers redirect focus, show/hide/unload another form, Stop/resume or Reset without stale ownership or replay | PARTIAL: ADR 0076 selected nested input; ADR 0077 reentrant window handlers, stopped activation and cross-modeless key transfer; ADR 0078 adds selected document-input cancellation/replay; remaining lifecycle/focus/binding cases open |
| F01 | Recursive SetFocus and cancellation, nested containers/page switches, dirty validation and binding callbacks, boolean update timing | PARTIAL: selected focus/validation implemented; ADR 0096 adds focus-preserving CommandButton clicks without prematurely committing dirty text; edge cases open |
| K01 | KeyDown/KeyPress/KeyUp with mutable ByVal MSForms.ReturnInteger and typed Shift; cancellation/default processing in ordinary frames | PARTIAL: ADR 0070 evidence plus ADR 0071 root/Tab/SetFocus/Space cases PASS (nine runtime, 11 new UI, Chromium, both hosts); remaining cases open |
| K02 | Held-key repeat; modifiers; Tab/navigation focus transitions; Default/Cancel Enter/Escape exceptions; ANSI input versus Unicode/IME/paste | PARTIAL: raw-input barrier and ADR 0071 focus/default ordering; remaining platform/shortcut/focus/IME cases open |
| K03 | Accelerators select/focus/activate the appropriate eligible control and preserve keyboard/event ordering | PARTIAL: ADR 0072 selected Label/button/boolean/Tab/Page actions; duplicate, non-ASCII/platform and further ordering cases open |
| M01 | Applicable MouseDown/MouseMove/MouseUp/Click/DblClick, typed button/modifier/coordinate/cancellation arguments, event order and container-relative coordinates | PARTIAL: ADR 0073 typed callbacks and selected simple-control capture/defaults; ADR 0074 selected TextBox caret/drag/word selection; ADR 0075 selected ComboBox/RefEdit/list/numeric/tab-header defaults; further ordering, background/drag/wheel/mutation and platform cases open |
| P01 | Remaining common properties, defaults, exact errors/coercions, text/selection edge cases | PARTIAL: ADR 0080 adds selected BackColor/ForeColor import, typed state and palette rendering; ADR 0081 adds selected Font data/objects/rendering; ADR 0082 adds typed Font reference assignment and reload; Nothing/external/other-assignment/ambient/platform font behavior remain open; ADR 0083 corrects Page ownership; ADR 0084 adds selected borders/effects/transparency and fixed-bound clipping; ADR 0085 adds published Font aliases; ADR 0086 adds selected TextAlign and matching pointer geometry; ADR 0087 adds selected caption WordWrap, file/runtime defaults and fixed horizontal text allocation; ADR 0088 adds Boolean-control WordWrap with accessible bounded captions; ADR 0090 adds CheckBox/OptionButton Alignment with separate caption clipping; ADR 0091 adds TextBox/ComboBox HideSelection and canonical inactive highlights; ADR 0092 adds selected AutoTab; ADR 0093 adds native SelectionMargin and deferred displayed-line input; ADR 0094 adds native TextBox ScrollBars and deferred bounded viewport input; ADR 0095 adds no-bar/room-hidden caret viewports and scrolled input; ADR 0096 adds native CommandButton TakeFocusOnClick and True defaults; defaults and remaining member/platform audit open |
| B01 | List/Column array shape/loading, binding value types and update timing, protected failures and reentrant handlers | PARTIAL: current selected array/binding packets implemented |
| W01 | Frame, TabStrip/Tabs, MultiPage/Pages: remaining storage/resource/property/presentation cases, transitions, focus and lifecycle | PARTIAL: ADR 0080 adds selected container colours and runtime Page lookup; ADR 0083 corrects Page appearance ownership; remaining cases open |
| W02 | Image/picture: remaining formats, metadata, loading/platform services and events/presentation | PARTIAL |
| W03 | RefEdit: remaining platform, property/event and picker behavior | PARTIAL |
| W04 | ListBox/ComboBox and ScrollBar/SpinButton: remaining property/resource/presentation/event contracts | PARTIAL |
| A01 | Applicable Office-supplied worksheet ActiveX/platform packets in the owning host | OPEN |

C01 is already accepted and stays in M7A.4. V01's safety, trust, bounds and
appropriate native/WASM/debugger/host evidence are mandatory for every future
packet; they are not deferred quality requirements. Bugs in included common
workflows remain fixes, not P9 scope transfers.

## Boundaries and scheduling

- Native editing/persistence for the accepted common profile stays **M7A.5**.
- Removal of the shipped `.tvform` codec/routes/examples stays **M7A.6**.
- Common-profile end-to-end native form/package acceptance stays **M7A.7**.
- Fresh Excel/Word comparisons for the declared release profile stay **M8.5**;
  current status is NOT_RUN. Fresh comparisons for later P9 extensions accompany
  their own acceptance and do not expand M8.5 back into exhaustive forms parity.
- Third-party controls, arbitrary COM/OCX/DLL loading and invented VBA semantics
  remain excluded. The existing host boundaries and all budgets remain intact.

Each P9 packet must define finite applicable inputs/members and evidence before
implementation. Do not reopen the finished common profile by requiring all of
P9 at once. No P9 implementation is claimed by this scheduling change.
