# ADR 0092: Native AutoTab through ordinary focus continuations

Date: 2026-11-08. Status: adopted; shared/browser and both pinned host acceptance gates passed. Full M7A.4 remains open.

TextBox and ComboBox expose ordinary Boolean AutoTab state with False defaults.
MS-OFORMS VariousPropertiesBitfield bit 30 seeds it before authored initialization.
Only those two native kinds admit the bit; RefEdit and non-applicable/fixed flags
retain their refusal rules. Normal Boolean coercion and errors 13/94 apply.
Property changes and programmatic Text/Value/SelText assignments do not tab.

The IDE provider owns an ordinary generated VBA helper. After accepted user
editing and Change complete, it checks the current AutoTab/MaxLength, UTF-16 text,
collapsed selection, editable/unlocked state, eligible control, active window and
unchanged focus serial. Autocompletion alone must not count as entering the
remaining characters. The helper traverses current canonical navigation ranks,
skips ineligible controls/ancestors and inactive pages, descends containers and
wraps. It calls the existing VbaMoveFocus path: normal BeforeUpdate/AfterUpdate/
Exit/Enter, cancellation and reentrant guards apply without synthetic key events.
Stop/wait continuation is ordinary VBA execution; Reset discards it. There is no
new VM opcode, foreign capability, mutable host store or increased budget.

The shared renderer forwards the actual TextEdit mutation signal separately from
selection-only input. Replacing an autocompleted suffix can leave the canonical
text unchanged, so a final-text comparison cannot recover this cause. The new
InstanceTextEdit command and form_instance_text_edit API use the same validation,
authorization and callback path as existing text input. Existing callers of
form_instance_text retain comparison-based updates and passive selection. The
command remains executable under the existing default-deny host preparation;
pending-command ownership includes the new variant.

Independently authored model/GPU cases cover flags/defaults/coercion, inert import,
programmatic/passive selection, autocomplete, typing/paste, UTF-16, validation
cancellation, Stop/Continue/Reset, reentrant policy/focus changes, nested tab order,
wrap, resource preservation and root member shadowing. Browser/theme and fork
acceptance are recorded in implementation-progress.md after execution. No fresh
Microsoft Office observation is claimed; platform/container differences and the
unchanged full completion checklist remain open. Native edited save and legacy
format removal remain M7A.5/.6; fresh Office traces remain M8.5 NOT_RUN.

References: [Microsoft AutoTab](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/autotab-property),
[MS-OFORMS VariousPropertiesBitfield](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/c38cc3f9-969b-45da-ab41-b361c044431e).
