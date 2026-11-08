# ADR 0094: Native TextBox ScrollBars and bounded viewports

Date: 2026-11-08. Status: adopted; increment accepted. Full M7A.4 remains open.

The project reader retains TextBox MorphData ScrollBars values 0–3 and refuses
unknown values before attachment. The IDE provides ordinary Long state, None
(default), validated assignment/errors and the published fmScrollBars constants.
Native metadata seeds it before authored initialization; inert inspection reads
the backing field only for TextBox receivers and preserves authored root names.
Changing the property never changes text, selection, events or original resources.
ListBox's required explicitly stored Both value is also admitted and validated;
its existing automatic list viewport does not gain a public ScrollBars member.
Other non-applicable MorphData flags remain refused.

The shared view owns only the viewport offset and derived render markers. These
are egui temporary state tagged with execution and control identity; they are not
a second text/selection model or public VBA scroll-position properties. Repaints
do not call VBA. Reset/recreated controls discard old offsets. Theme-local solid
scrollbars retain host chrome independently of TextBox ForeColor. Authored bounds
and accessible edit bounds remain fixed; source/resources remain unchanged.

Measured overflow, available room, MultiLine and WordWrap determine visible
axes. Multiline WordWrap suppresses the horizontal bar, single-line suppresses
the vertical bar, fitting contents hide unnecessary bars, and the public policy
retains its assigned value. Native ScrollBars=None keeps the existing no-bar
path. AutoSize, hidden-bar scrolling, repeat/platform details and broader
container scrolling remain open; they are not claimed by this increment.

Track/drag defaults use the existing authorized mouse callback and completion
path. Stop delays the default; current focus, policy, geometry and lifetime guard
it after callbacks. Scrolling changes only the viewport: canonical text/ranges
and Change/validation state remain independent. Derived geometry translates text
hits through the current offset, including margin selection. Ordinary selection
or text changes follow the caret; passive rendering preserves manual scrolling.
No new VM instruction, foreign capability, event, format or budget is added.

Eight authored model cases and eight GPU/input cases cover native records,
ListBox stored Both, defaults/constants, normal coercion/errors, applicability,
Stop/inert reads/Reset/resources, visible axes, real track and thumb input,
callback policy changes, scrolled selection geometry and programmatic selection
following. The existing 40 mouse, AutoTab and alignment suites pass. Full native CI,
strict Clippy, WASM/UI/Xvfb/web and twelve Chromium flows pass; the eight new
model cases also pass on Node 18 WASI. Five theme snapshots and browser
runtime/Break/scrolled views were inspected. Full Node 22 WASI passes. Both pinned host
acceptance suites pass, including full CI, strict optional Clippy, VBA-enabled
web, native/Node WASM package cases and inspected host canvases. Revisions and
exact case counts are recorded in implementation progress.
Fresh Office comparisons remain M8.5 NOT_RUN. Native edited save and legacy
removal remain M7A.5/.6; the full completion checklist remains authoritative.

References: [Microsoft ScrollBars](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/scrollbars-property),
[MS-OFORMS ScrollBars (MorphData)](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/1b674d2d-cd44-4c5a-a947-1dad39243fe5),
[MorphDataDataBlock](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/8a95d726-067d-4cd0-8fd9-3a4b6cc6188d).
