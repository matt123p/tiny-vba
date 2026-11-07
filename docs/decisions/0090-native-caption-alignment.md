# ADR 0090: Native CheckBox/OptionButton caption alignment

Date: 2026-11-07. Status: implementation; shared/browser acceptance passed, host acceptance in progress. Full M7A.4 remains open.

CheckBox/OptionButton expose ordinary Long Alignment state and the published
fmAlignmentLeft=0/fmAlignmentRight=1 constants. File and dynamic defaults are
Right. The decoder translates native VariousPropertiesBitfield bit 13 inversely:
a set bit places the caption on the left and seeds public value 0. That bit is
accepted only for the two applicable control kinds. ToggleButton's documented
disabled property does not gain an enabled member or permissive file flag.

The IDE provider owns ordinary VbaAlignment fields/getters/setters, standard Long
coercion and error 380 for values outside 0..1. Native flags seed state before
authored initialization. Inert queries read fields without invoking getters or
loading an instance; authored root members remain independent. Value, Null,
TripleState, option groups, WordWrap, events and original resources do not change
when Alignment changes. No compiler/VM/foreign/schema or budget change is needed.

The shared view keeps real CheckBox/RadioButton widgets for indicator styling,
input, focus and accessibility. Local physical direction positions the indicator;
an ordinary paint atom lays out left-aligned caption text within its own bounded
rectangle. It clips wrapped/unwrapped text before the indicator cell and preserves
full glyph data, font caches, authored bounds and global embedding styles. Full
accessible labels and selected/indeterminate state are retained; empty captions
use the runtime control name as their accessible label. ToggleButton keeps its
centred fixed caption path. The legacy codec and rendering are not extended.

Seven authored model cases cover defaults, inverse flags, published constants,
Long types/errors, inert stopped inspection, one-time continuation, Reset,
resource identity, Value/Null/group independence and unrelated flag refusal.
Four new GPU/input cases extend the four existing Boolean-caption cases: both
sides with wrapped/unwrapped clipping, actual stopped side changes and right-side
OptionButton indicator clicks and absence of indicator/bevel pixels on the
unused side. Five-theme snapshot review exposed the old left-side bevel after
the real widget moved right. Its pixel regression fails with the old placement
and passes when the decorative bevel follows canonical Alignment. Pixel comparison subtracts an empty-caption render
so indicator pixels cannot be miscounted as text. Full shared/browser and pinned
host receipts belong in implementation progress. Fresh Office observations remain
M8.5 NOT_RUN; no remaining checklist requirement is reduced.

Native edited persistence remains M7A.5; legacy format removal remains M7A.6.
Other properties, AutoSize/scrolling and lifecycle/input/binding/platform work stay
open under the existing M7A.4 checklist.

References:

- [Alignment property](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/alignment-property)
- [VariousPropertiesBitfield](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/c38cc3f9-969b-45da-ab41-b361c044431e)
