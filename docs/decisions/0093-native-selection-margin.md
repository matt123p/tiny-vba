# ADR 0093: Native SelectionMargin and deferred line selection

Date: 2026-11-08. Status: adopted; shared/browser acceptance passed; pinned host checks pending. Full M7A.4 remains open.

TextBox and ComboBox expose ordinary Boolean SelectionMargin state with True
file/runtime defaults. Native VariousPropertiesBitfield bit 26 seeds it before
authored initialization. Only those two kinds admit a variable flag; required
fixed bits on other classes and non-applicable RefEdit flags retain strict
validation. Normal Boolean coercion/errors apply. Property assignment preserves
text, UTF-16 ranges, events, source/resources and all existing budgets.

The shared edit widget reserves a thin left strip when enabled, adding to the
current theme's padding without changing global style or font caches. False
returns that width to text. Accessibility retains the control name; empty native
edit fields render blank rather than showing an invented name placeholder.

Primary margin input selects a displayed line through the existing mouse callback
barrier and selection-only command. Current layout determines wrapped rows;
UTF-16 conversion preserves CRLF and surrogate widths. Dragging extends over
whole displayed rows, and Shift extends the existing range. Locked fields remain
selectable. Normal caret/word input uses the remaining edit region. AutoTab is
not triggered by selection-only movement.

The current text, layout, focus and property are read after callbacks finish.
Stopped MouseDown therefore delays selection and uses resumed state; changing
the policy or redirecting focus suppresses a stale margin action. Existing
capture/ownership, debugger continuation, Reset, authorization and bounded queues
remain authoritative. There is no VM/foreign change, second canonical store,
format extension, budget increase or native edited-save implementation.

Independently authored model/input cases exercise flags/defaults/coercion,
instance independence, inert stopped reads, source/resources, CRLF/UTF-16 rows,
wrapping, drag, editable ComboBox, locked text and stopped/policy-changing
callbacks. Full shared/browser/theme and pinned host results are recorded in
implementation-progress.md after execution. Printing, remaining text/selection/
platform cases and the full M7A.4 checklist stay open. Fresh Office evidence is
M8.5 NOT_RUN; native edited saving and legacy removal remain M7A.5/.6.

References: [Microsoft SelectionMargin](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/selectionmargin-property),
[MS-OFORMS VariousPropertiesBitfield](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/c38cc3f9-969b-45da-ab41-b361c044431e).
