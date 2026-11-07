# ADR 0091: Native HideSelection and inactive text highlights

Date: 2026-11-07. Status: implementation; shared/browser acceptance passed, host acceptance in progress. Full M7A.4 remains open.

TextBox and ComboBox expose ordinary Boolean HideSelection state. File and dynamic
defaults are True. Native VariousPropertiesBitfield bit 29 seeds that value before
authored initialization; explicit False is accepted for those two control kinds.
Required fixed bits for Boolean controls/ListBox retain their existing validation.
Unknown/non-applicable flags still refuse atomically; RefEdit does not gain an
unreviewed public property or broader flag acceptance.

The IDE provider owns the ordinary field and normal Boolean coercion, preserving
errors 13/94 and the previous value on failed assignment. Changing the property
never changes Text/Value, SelStart/SelLength/SelText or events. Source/resources,
Reset, instance independence and authored root names remain separate. Inert field
queries neither invoke getters nor load another object. There is no new VM,
foreign call, opcode, schema, mutable host store or budget.

The view paints the canonical selected range when unfocused and HideSelection is
False. Focused TextBox/editable ComboBox keep normal widget selection painting;
non-editable ComboBox text uses the same local galley selection primitive. Visible
highlighting does not grant edit permission or change focus, rows or drop state.
Selection shaping uses existing UTF-16/CRLF cursor conversion and clipped bounds,
including wrapped/multiline and password display. Only local galley clones are
painted; shared font caches and global styles remain untouched. Focus/style
queries and selection decoration occur outside the font lock. RefEdit and legacy
rendering retain their prior policy.

Seven authored model cases cover native flags, both ComboBox styles, True file/
dynamic defaults, independent instances, Boolean types/errors, text/range/event
independence, Stop/Continue/Reset, inert inspection, resource retention and authored
root members. Six fail against the previous implementation. Three GPU/input cases
cover active/inactive policy for all three presentations, fixed bounds, a selected
emoji/CRLF range and actual focus transfer with a once-only callback. Browser and
pinned host receipts belong in implementation progress. These are reference-based
authored checks; fresh Office observations remain M8.5 NOT_RUN.

Remaining selection/scrolling, AutoSize, property/member/default, lifecycle/input,
binding and image/RefEdit/ActiveX/platform work remains on the unchanged M7A.4
checklist. Native edited persistence is M7A.5; legacy format removal is M7A.6.

References:

- [HideSelection property](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/hideselection-property)
- [VariousPropertiesBitfield](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/c38cc3f9-969b-45da-ab41-b361c044431e)
