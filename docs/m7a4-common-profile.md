# M7A.4 common forms profile

Accepted 2026-11-08 on explicit owner direction under
[ADR 0097](decisions/0097-common-forms-scope-and-p9.md).
Capability label: `native-forms-common-v1`. M7A.4 is COMPLETE for this finite
runtime profile. This is a planning/support boundary, not a new VBA API or format.
It supersedes the former exhaustive M7A.4 gate; it does not claim full MSForms parity.

## Included everyday behavior

The common controls are Label, TextBox, CommandButton, CheckBox, OptionButton,
ToggleButton, ComboBox, ListBox, Frame and MultiPage/Page. The property names below
apply only to the Office interfaces that expose them. Existing numeric controls,
TabStrip, Image and Excel RefEdit support is retained at its documented level;
further compatibility for these less common controls belongs to P9.

| Area | Included contract | Executable evidence |
|---|---|---|
| Native attachment | The admitted native `.frm`/`.frx` and embedded resource records attach inertly; source and original resources stay separate; malformed/unsupported records refuse before mutation | IDE `native_forms`; UI `native_forms`; both hosts' native package suites |
| Instances/lifetime | Predeclared and explicit New instances; independent state; Load, modal/modeless Show, Hide, Unload, UserForms/Controls lookup; ordinary Initialize/Activate/Deactivate/QueryClose/Terminate paths and close cancellation | IDE `form_instances`, `form_window`, `empty_userforms`; UI `form_instances`, `form_windows`; ADRs 0042–0048, 0069, 0076–0079 |
| Basic properties | Name/Caption, Left/Top/Width/Height, Enabled/Visible, TabStop/TabIndex, Tag/ControlTipText; runtime geometry and eligible tab order | IDE `form_instances`, `form_execution`; UI form suites; ADRs 0043, 0048, 0055 |
| Text editing | Text/Value, Locked, MaxLength, MultiLine, PasswordChar, SelStart/SelLength/SelText; existing Enter/Tab/field behavior, HideSelection, AutoTab, SelectionMargin and bounded TextBox scrolling with fixed bounds | IDE `form_instances`, `native_hide_selection`, `native_auto_tab`, `native_selection_margin`, `native_text_scroll`; UI counterparts; ADRs 0049, 0091–0095 |
| Buttons/choices | CommandButton Click, Default/Cancel, TakeFocusOnClick; Boolean Value/Null/TripleState and OptionButton grouping within the admitted interfaces | IDE `form_instances`, `native_button_focus`; UI `form_keyboard`, `native_button_focus`, `native_boolean_word_wrap`; ADRs 0043, 0088, 0096 |
| Lists/combo/binding | AddItem/RemoveItem/Clear, ListCount/ListIndex/Value/Text; current bounded List/Column loading/readback and selection; ComboBox styles/matching/drop paths; existing explicit-capability single-cell ControlSource and contiguous RowSource links | IDE `form_instances`; pinned host package suites; ADRs 0051–0054, 0063, 0065 |
| Containers/collections | Frame/MultiPage child Controls/Parent/ActiveControl, selected Page and normal nested focus/tab/grouping; current lookup, For Each snapshots and dynamic controls/pages/tabs | IDE `form_instances`; UI `form_instances`, `form_mouse`; ADRs 0056–0058, 0061–0062, 0080, 0096 |
| Common input/events | Click/Change, Enter/Exit and BeforeUpdate/AfterUpdate validation/cancellation; Tab/Shift+Tab, Space, Default/Cancel Enter/Escape, selected accelerators; current typed key/mouse callbacks and text selection defaults | IDE `form_keyboard`, `form_mouse`, `form_accelerator`; UI `form_keyboard`, `form_mouse`; ADRs 0070–0075, 0096 |
| Appearance | Current BackColor/ForeColor, Font attributes/reference assignment, applicable borders/effects, TextAlign, WordWrap and Boolean caption Alignment; local themes and fixed authored bounds | IDE/UI `native_colors`, `native_fonts`, `native_appearance`, `native_text_align`, `native_word_wrap`, `native_boolean_word_wrap`; ADRs 0080–0090 |
| Execution safety | Canonical VBA state, exact covered coercions/errors, Stop/Continue without replay, Reset/source/resource identity, bounded input/work and existing host authorization/trust loss; lower caller execution slices | Shared model/UI regressions, embedding suite and both pinned hosts; ADR 0096 |

These are the existing, tested contracts, not every combination of the named APIs.
The linked runtime/ADRs and tests define admitted inputs, bounds, defaults and exact
errors. In particular, collection enumeration retains its snapshot contract;
physical key callbacks do not establish exhaustive IME/platform equivalence;
Font assignment does not establish all Nothing/external font cases; bindings do
not establish every Excel range shape or value type. No existing feature is removed.

## Deferred boundary

[P9](p9-forms-compatibility.md) owns the old checklist's remaining exhaustive
member/default/resource audit, unusual reentrant alias/collection/lifecycle/focus
combinations, advanced binding timing/shapes, platform/IME/DBCS/accelerator/repeat/
wheel parity, AutoSize/container scrolling and geometry/font-change corner cases,
additional image/RefEdit/numeric/tab variants, and worksheet Office-supplied ActiveX.
Third-party controls and arbitrary COM/OCX/DLL activation remain excluded.

A defect in an included common workflow remains a defect to fix; calling it an
edge case must not silently move it to P9. New P9 work requires a named packet and
its own evidence. Unsupported P9 behavior remains unsupported; it is not PASS.

## Acceptance and next step

[The completion checklist](m7a4-completion-checklist.md) maps the common gate to
actual evidence. The accepted product is
`bcf3dc859095df7a331760d1469b8047c00cacf4`, with Gridcraft
`624d73f45365b1e3661ea81ebed94befc0b5a58d` and Wordcraft
`5f74fa087e6ea2ffc7db37b0a203f905d77eb54d` pinning it. The latest full native CI,
strict Clippy, WASM/WASI, UI/Xvfb/web, fourteen Chromium flows and both hosts' full
CI/native/WASM/UI gates pass; see the ADR 0096 progress receipt for exact counts.
This scope change changes no product code, dependency pin, limit or test expectation.
Focused rechecks pass: 129 native model and 26 GPU/input cases. Repository,
document consistency and formatting checks also pass.

M7A itself stays IN PROGRESS. Next is M7A.5: native editing/persistence for this
profile, then M7A.6: remove `.tvform`, and M7A.7: integration acceptance. Native
edited saving is not implemented by this sign-off. Fresh comparisons for the
declared release profile remain M8.5 NOT_RUN. P9 does not block those milestones.
