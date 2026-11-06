# 0072 — Native form accelerators

Date: 2026-11-06. Status: adopted for the implemented cases below. M7A.4 remains
in progress; checklist K03 is PARTIAL. Fresh Office observations remain M8.5 NOT_RUN.

## Ownership and contract

The native reader decodes the MS-OFORMS Label bit 11, CommandButton bit 8 and
CheckBox/OptionButton/ToggleButton MorphData bit 29 Unicode accelerator fields.
It validates characters and alignment before attachment, preserving original
resources separately. Unsupported masks retain their existing refusals. No
legacy `.tvform` extension or edited native persistence is introduced.

The IDE generates ordinary canonical Accelerator properties for these controls
and Tab/Page, retaining the first UTF-16 unit on assignment. Alt+character runs
after applicable KeyDown callbacks; cancellation suppresses default action.
Labels move focus to the next eligible tab control in their immediate scope;
buttons focus and Click; booleans use their existing value/event paths; Tab/Page
keys focus their owning control and select the surviving eligible item. Effective
ancestor/page eligibility, focus validation/cancellation and stale member checks
apply. These operations use ordinary VBA frames and existing authorization,
continuation and resource limits. Stop/resume cannot replay the action.

The UI queues raw Alt input even without authored keyboard handlers, pairs web
text/key input, and decorates matching ASCII caption characters using injected
fonts and colors. Rendering inspects fields without executing getters. The VM
has no Office-specific names or additional mutable registry. Receipts remain
canonical fields and are invalidated by Reset/source changes.

Duplicate eligible matches return an explicit unsupported SDK result without
selecting an arbitrary target. Exact duplicate cycling, non-ASCII/platform
shortcut delivery and underlining, all key/event ordering edge cases and full
window/lifecycle behavior remain open. This refusal is not Office parity and
is not an invented VBA error. Ctrl+Alt is excluded from Alt routing.

## Evidence

Independently authored native records cover persisted values, MorphData padding,
malformed surrogate refusal and original-byte retention. Eight model cases cover
Label/button/boolean/tab/page actions, disabled/hidden/duplicate cases, validation
cancellation, Ctrl+Alt, dynamic WithEvents/property changes and Stop/Reset.
Three additional real egui input cases cover no-handler routing, label/boolean
actions and browser text-before-Alt pairing. The Chromium harness imports native
resources, runs Alt+N/Alt+A, pauses/resumes Click and checks unchanged source.
Validation receipts are recorded in implementation progress, separately from
reference-derived expectations and unrun Office observations.

## Primary references

- [MSForms Accelerator](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/accelerator-property)
- [MSForms Click](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/click-event)
- [Tab/Page rename dialog](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/the-rename-dialog-box-microsoft-forms)
- [Label mask](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/81e14dea-a6c6-4fdc-b226-e9719ef53f66)
- [CommandButton mask](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/5c5ad1f1-d511-4f2e-b9c2-35cc31be8061)
- [MorphData mask](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/19014e19-67fa-4060-8c81-d1463809f117)
