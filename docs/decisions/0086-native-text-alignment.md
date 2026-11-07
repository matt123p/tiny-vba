# ADR 0086: Native control text alignment

Date: 2026-11-07. Status: selected runtime/view implemented; shared/browser and both host acceptance recorded in progress, with the WASI runner limitation below. M7A.4 remains open.

## Contract and ownership

Label, TextBox, ListBox and ComboBox gain ordinary Long TextAlign properties and
the published fmTextAlign constants. The existing native TextProps ParagraphAlign
is immutable input, not an attribute of the shared runtime Font object. Its
PARAFORMAT values 1/2/3 mean left/right/centre; public fmTextAlign 1/2/3 mean
left/centre/right. Source generation translates those values explicitly before
authored initialization. File defaults and runtime-created controls use left.
Original metadata/source/resources remain unchanged.

TextAlign stays independent of Font assignment, text, selection and events.
Ordinary parameter coercion and existing error state apply; invalid values raise
380 before mutation. Inspection reads canonical fields without invoking code.
Leaf aliases do not shadow authored form fields; the audit also restricts existing
BackStyle/AutoSize inspector aliases to controls. There is no new VM operation,
host protocol, font store, resource format or increased budget. Native edited
saving remains M7A.5; legacy .tvform is not extended.

This packet admits those four families. Boolean and RefEdit TextAlign behavior
still needs its own applicability/behavior acceptance; it is not claimed absent
from their Office interfaces. Form/Frame, Page, CommandButton, TabStrip/MultiPage,
Image and numeric controls do not gain an invented TextAlign API. The remaining
member/default/presentation and full M7A.4 checklist remain open.

## View

The renderer reads canonical Long state locally. Native Label caption positioning
keeps the existing fixed clickable/accessibility bounds, accelerator decoration,
font and background policy. TextBox and editable ComboBox use matching glyph and
pointer geometry; the caret origin accounts for a centred/right galley's negative
bounds. Caller vertical alignment, global styles/fonts and UTF-16 values remain
unchanged. List cells align independently in the existing column layout. ComboBox
edit-region alignment does not change its popup rows. Non-editable ComboBox text
uses the same physical alignment policy. Broader wrap/AutoSize, bidi/IME, mutation
and platform comparison cases remain open; this is not full Office pixel parity.

Seven model cases cover enum/nominal types, raw/public mapping, invalid records,
coercions/errors, defaults, Font/text/selection independence, absent interfaces,
field alias scope, Stop/Continue and Reset/resource identity. Four real GPU/input
cases cover physical left/centre/right positions, fixed bounds, multiline captions,
multi-column lists, independent
ComboBox popup rows and pointer start/end positions through UTF-16 text. Existing
appearance/Font and pointer regressions are checked alongside these. Full shared,
browser and affected host receipts belong in implementation progress. Fresh
Office build/OS/locale observations remain M8.5 NOT_RUN.

Runner diagnostic: Node 22.23.4 exits with SIGSEGV in the existing boolean
accelerator WASI test using the default debug module. The byte-identical module
passes all eight accelerator cases on Node 18.19.1. A separately linked 8 MiB
diagnostic stack also passes the failing case on Node 22; that does not establish
the underlying cause. Shipping stack settings, execution/memory limits and the
restricted WASI runner remain unchanged. The full WASI receipt uses the documented
Node 18 runner; Node 22's default-stack failure remains a recorded limitation.

References:

- [Forms TextAlign](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/textalign-property)
- [Native ParagraphAlign](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/de5bbeab-a3dc-4e1d-bcbd-ff35f064fe01)
- [PARAFORMAT values](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/23d08734-76f0-48c7-b603-a4e589c7f429)
- [Label interface](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.labelclass?view=office-pia)
- [TextBox interface](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.textboxclass?view=office-pia)
- [ListBox interface](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.listboxclass?view=office-pia)
- [ComboBox interface](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.comboboxclass?view=office-pia)
