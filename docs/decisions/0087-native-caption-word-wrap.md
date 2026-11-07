# ADR 0087: Native caption wrapping and TextBox defaults

Date: 2026-11-07. Status: selected implementation; shared/browser and both host
acceptance recorded in implementation progress.
M7A.4 remains open.

## Contract and ownership

Native Label and CommandButton gain ordinary Boolean WordWrap state. Their
VariousPropertyBits bit 23 seeds it without executing code. File defaults differ:
Label wraps; CommandButton does not. Runtime-created Label, CommandButton and
TextBox use the published True default. This corrects the previous dynamic
TextBox False default; imported TextBox flags still initialize existing text
state. No new RefEdit or Boolean-control property acceptance is implied.

Ordinary field assignment retains central VBA coercion/error behavior and does
not edit text, selection, focus, geometry or events. Import remains bounded and
inert; other unsupported flag combinations still refuse atomically. Existing
immutable metadata/source/resource bytes are distinct from canonical VM state.
No new bytecode, host operation, mutable property store or increased budget is
introduced. A persisted ComboBox/ListBox WordWrap bit is not permission to invent
public properties on those classes. Their required file bits remain enforced.

## Local view

Native Label/CommandButton captions use the existing styled empty button for
fixed interaction/accessibility bounds and frame painting, followed by the
original font/accelerator-decorated text job. WordWrap chooses the authored content
width or unwrapped layout; clipping remains fixed and explicit newlines survive.
Label keeps its canonical TextAlign; CommandButton keeps centred captions. Global
styles/fonts and legacy form rendering remain unchanged.

The pixel audit found that multiline unwrapped TextEdit expanded its interaction/
accessibility width to the line even though native painting was already clipped.
A local copy of the shaped galley now measures only the native horizontal viewport
for allocation. Glyph/cursor rows and original text remain complete; alignment
anchors stay left/centre/right and the font cache is untouched. The existing
pointer adapter uses the corresponding galley origin. This is UI measurement,
not another runtime store. Further vertical overflow, scrolling, AutoSize, trailing
space, bidi/IME and platform/ambient presentation cases remain open.

Six model cases cover file/runtime defaults, native flag combinations, metadata
independence, refused unrelated flags, exact Boolean errors, text/selection/event
independence, inert inspection, Stop/Continue, Reset and source/resource identity.
Three real GPU/input cases cover wrapped/unwrapped rows, explicit newlines,
single-line TextBox policy, fixed caption/TextBox rectangles and clipped multiline
UTF-16 pointer positions. Existing pointer, alignment, appearance and Font cases
remain acceptance requirements. Full shared/browser and affected host receipts
are recorded in implementation progress. Fresh Office observations remain M8.5
NOT_RUN; no full milestone or Office pixel-parity claim is made.

Boolean-control WordWrap and broader member/default/AutoSize and presentation
requirements remain on the unchanged M7A.4 ledger. Native edited saving stays
M7A.5; legacy .tvform is not extended and removal stays M7A.6.

References:

- [WordWrap property](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/wordwrap-property)
- [VariousPropertiesBitfield](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/c38cc3f9-969b-45da-ab41-b361c044431e)
- [File-format defaults](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/7a72ac4a-39d9-4e2b-829e-19e3e9a1f60d)
