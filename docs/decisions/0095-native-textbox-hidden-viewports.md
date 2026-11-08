# ADR 0095: Native TextBox caret viewports without visible bars

Date: 2026-11-08. Status: adopted; increment accepted. M7A.4 remains open.

The UI now separates scrollable text axes from visible scrollbar axes. A native
TextBox always has a fixed, clipped text viewport: horizontal for single-line or
unwrapped multiline text, vertical for multiline text. ScrollBars remains the
ordinary validated VBA policy from ADR 0094; None does not make the tail of long
text unreachable. WordWrap suppresses horizontal movement. A horizontal bar
hidden for lack of room also retains caret-following behavior.

The existing egui ScrollArea owns viewport mechanics with its automatic chrome
hidden. The renderer reserves only the space required by measured visible bars,
paints those bars from the caller's current widget style and exposes the same
accessible ScrollBar roles. This permits a hidden axis to move without moving a
visible bar on the other axis. Track/drag input still uses the existing authorized
MouseScope callback/default continuation; drawing a bar never executes VBA.
No control, property, event, foreign call, file format or execution budget is added.

The execution/control-tagged UI view retains only offset and derived render
markers. Text, UTF-16/CRLF selection and policy remain canonical ordinary VBA
fields. Selection changes, real navigation and typing reveal the caret; passive
rendering preserves manual offsets. Pointer geometry includes the offset for
normal and selection-margin hits. Stop postpones pointer defaults, Continue
completes them once, and Reset discards both the continuation and old viewport.
The previous local unwrapped-galley rectangle clamp is unnecessary: the viewport
clips complete glyph/cursor rows without altering shared font caches.

Seven new GPU/input cases cover no-bar single/multiline caret movement and return,
a hidden horizontal axis alongside a visible vertical bar, astral UTF-16/CRLF
scrolled margin selection, Stop/Continue/Reset, insufficient bar room, and actual
Ctrl+End followed by typing. Untouched initial right/centre-aligned overflow
keeps its alignment; a later explicit caret move can reveal either end. The
existing clipped-pointer expectation and a new caret-zero pixel test cover this.
The eight existing viewport cases and retained
mouse, alignment, HideSelection and AutoTab suites also pass. Full shared CI, strict workspace/all-targets Clippy, WASM/WASI/UI/Xvfb/web,
thirteen Chromium flows and both pinned host acceptance suites pass. The host
suites include full CI, strict optional Clippy, VBA-enabled web, native/Node WASM
package cases and inspected no-bar canvases. Exact revisions and case counts are
recorded in implementation progress.
The new Chromium flow already passes stopped Ctrl+End, continuation and typing
at the revealed tail with exact source preservation. Five theme variants and
browser runtime/Break/scrolled views were inspected.

This is authored/specification-derived evidence, not a fresh Office observation.
AutoSize, repeat/wheel/platform details, geometry/font-change edge cases and the
remaining control/member/container audit stay open. The full completion checklist
remains authoritative. Native edited saving/removal remain M7A.5/.6; fresh Office
comparisons remain M8.5 NOT_RUN.

References: [MSForms ScrollBars](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/scrollbars-property),
[MSForms SelStart](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/selstart-property),
[selection and keyboard example](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/enabled-enterfieldbehavior-sellength-selstart-seltext-properties-example).
