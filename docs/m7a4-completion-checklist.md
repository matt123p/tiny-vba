# M7A.4 completion checklist

Updated 2026-11-08. **COMPLETE for `native-forms-common-v1`.**
The owner explicitly reduced the gate to common properties and workflows under
[ADR 0097](decisions/0097-common-forms-scope-and-p9.md). The previous exhaustive
checklist and its outstanding cases moved to [P9](p9-forms-compatibility.md), with
original IDs and partial evidence preserved. This does not claim full MSForms parity.

[The common profile](m7a4-common-profile.md) defines the finite accepted interfaces,
workflows and limitations. The following acceptance uses implemented assertions,
not the mere reclassification of unsupported work as PASS.

| Gate | Required common behavior | Result / evidence |
|---|---|---|
| CF01 | Inert native attachment, original source/resources and atomic refusal | PASS: shared IDE/UI `native_forms`, both host native/WASM packages; ADR 0040 |
| CF02 | Default/explicit instances, modal/modeless Show, Hide/Unload, ordinary lifecycle and close cancellation | PASS: `form_instances`, `form_window`, UI `form_windows`, `empty_userforms`; ADRs 0042–0048, 0069, 0076–0079 |
| CF03 | Standard controls and basic name/caption/geometry/state/tab order | PASS: `form_instances`, `form_execution`, UI form suites; ADRs 0043, 0048, 0055–0058 |
| CF04 | Text editing/selection/limits/locking/password/multiline and current scrolling | PASS: model/UI text, selection, AutoTab and scrollbar suites; ADRs 0049, 0091–0095 |
| CF05 | Button roles/focus and Boolean state/groups | PASS: `form_instances`, model/UI `native_button_focus`, `native_boolean_word_wrap`; ADRs 0043, 0088, 0096 |
| CF06 | Bounded list/combo data/selection and existing basic host bindings | PASS: `form_instances`, pinned host package tests; ADRs 0051–0054, 0063, 0065 |
| CF07 | Child containers, lookup/snapshot enumeration and current dynamic collections | PASS: `form_instances`, UI `form_instances`/`form_mouse`; ADRs 0056–0058, 0061–0062, 0080, 0096 |
| CF08 | Common pointer/keyboard actions, Click/Change/focus/validation and cancellation | PASS: model/UI keyboard/mouse/accelerator tests; ADRs 0048, 0070–0075, 0096 |
| CF09 | Existing colors/fonts/borders/alignment/wrapping/tips and themed rendering | PASS: model/UI appearance suites, five theme/scale/host snapshots and Chromium flows; ADRs 0055, 0080–0095 |
| CF10 | Canonical state, covered coercion/errors, trust/lifetime/bounds, Stop/Continue/Reset and both hosts | PASS: shared full CI/WASM/WASI/UI/native/web, fourteen Chromium flows, full Gridcraft/Wordcraft gates; ADR 0096 receipt |

Accepted shared product: `bcf3dc859095df7a331760d1469b8047c00cacf4`.
Gridcraft: `624d73f45365b1e3661ea81ebed94befc0b5a58d`.
Wordcraft: `5f74fa087e6ea2ffc7db37b0a203f905d77eb54d`.
[Progress](implementation-progress.md) records exact gate counts and the focused
common-profile recheck. No product code, budget or test expectation changed for
this scope decision; the full accepted product/host evidence remains applicable.
Fresh Office evidence is still M8.5 NOT_RUN, not an inferred PASS.

Next: **M7A.5 native editing/persistence**, then M7A.6 `.tvform` removal and M7A.7
integration acceptance for the common profile. All former C02–A01 remaining cases
are explicitly mapped to P9; C01's accepted empty-UserForms behavior is retained.
Quality requirements from former V01 still apply to every change.
