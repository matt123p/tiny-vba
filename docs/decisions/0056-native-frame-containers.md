# ADR 0056 — native Frame containers

Date: 2026-11-05. Status: adopted for the selected native Frame packet.
M7A remains IN PROGRESS.

vba-project decodes cached class 14 as a parent with its own FormControl `f`
and `o` streams in `i` + decimal site ID storage (minimum two digits). Parent
site object bytes must be zero; the selected site variant is nonstreamed,
promoted, with first-child selection. Child depth tables remain local depth zero.
Unknown masks, scrolling/zoom/picture records, missing streams, orphan resources
and other container variants refuse explicitly. The tree is bounded to eight
Frame levels and 128 total controls, with globally unique positive IDs/names.
Reused IDs across native parent scopes are currently refused. Original resources
and authored source remain byte-independent of the decoded model and execution.

The shared model adds immediate `Control.parent` identity, distinct from designer
grouping. Parents must be existing Frames preceding descendants; cycles, excessive
depth and invalid externally supplied descriptors fail before project mutation.
Coordinates and TabIndex belong to the immediate parent. Native Frame caption,
size, enabled state and parent-site visibility/tab/metadata initialize ordinary
class fields. This adds no IR, dependency edge, host object store or legacy codec
capability. Designer edits which violate parent-before-child order refuse atomically.

The IDE generates an ordinary Frame class implementing MSForms.Frame and Control.
Controls collections contain immediate children and share canonical control objects
with the root's bounded internal traversal collection. Parent is resolved through
an ordinary ByRef event into the existing root receiver, avoiding a stored child
to parent object cycle. UserForm.ActiveControl returns the outer Frame;
Frame.ActiveControl returns its immediate active child. Internal focus retains the
leaf and updates the ancestor path on committed focus changes. Frame.SetFocus
selects the first eligible descendant; empty/blocked targets raise 2110. TabIndex
changes reorder only siblings, and option groups match both parent and GroupName.

Effective enabled/visible and keyboard TabStop checks walk bounded ancestors.
Programmatic values retain their existing behavior; physical input and focus use
existing authorization and modal ownership. Ancestor Exit runs inner to outer
and Enter outer to inner across the changed path. Typed cancellation, debugger
Stop/resume and focus serial checks protect existing continuations. Frame background
Click does not force leaf focus. Broader event ordering and public WithEvents
focus-event exposure remain compatibility work; authored focus handlers use the
existing generated dispatcher. Internal traversal uses indexed ordinary calls,
keeping public bounded snapshot enumeration and existing VM work limits intact.

The inert designer also accumulates parent positions and clips child previews;
selection never instantiates a form. Native edited persistence remains separate.
The shared runtime renderer accumulates live parent geometry, clips descendants to their
Frames and exposes actual accessible group ancestry. Effective visibility/input,
scoped navigation and default/cancel eligibility read canonical fields. Frame
painting uses injected theme tokens and host fonts, with per-instance IDs. The
renderer does not maintain another form state or invoke properties while painting.

Evidence consists of independently authored nested CFB fixtures, invalid-storage/
ancestry tests, collection/type/group/tab/focus/cancellation/runtime cases, actual
widget/accessibility input, five theme captures and native/browser/fork gates.
Fresh Office traces remain NOT_RUN under M8.5. MultiPage/TabStrip/Image/RefEdit,
remaining properties/resources/bindings/platform/lifecycle requirements, native
edited persistence and .tvform removal remain in the unchanged M7A scope.

Sources: [MS-OFORMS parent controls](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/5eec5f0d-d53e-479f-a82f-bbb688ebe65a),
[cached classes](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/5f649a6c-90df-4226-b8c7-2a7d98cd9647),
[SITE_FLAG](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/ed58f23c-ec1f-43f8-a593-df2626191d27),
[Controls collection](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/controls-collection-microsoft-forms),
[SetFocus](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/setfocus-method),
[Enter/Exit and ActiveControl example](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/enter-exit-events-activecontrol-property-example).

Reproduce nested browser input using the native fixture generator
`--frame-controls` and `tools/ui-browser/native-forms.cjs` with
`VBA_NATIVE_FRAME_INPUT=1` and the generated fixture directory.
