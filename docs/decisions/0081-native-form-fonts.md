# ADR 0081: Native form fonts and bounded nominal metadata

Date: 2026-11-07. Status: implemented and verified for the selected profile. M7A.4 remains open.

## Ownership and values

vba-project retains native TextProps and StdFont data: face name, exact fractional
point size, effects, charset, weight, pitch/family and paragraph alignment. Size
uses integer ten-thousandths of a point; TextProps twips convert exactly. Defaults
are the published file defaults, not observations of a particular VBE toolbox.
An absent container font remains distinct from an explicitly persisted font.
StdFont enforces its published nonzero height ceiling, ASCII face length below
32 bytes, zero reserved/BOLD flag bits and bounded weight. TextProps retains its
own effects mask and exact twips conversion; the two records are not conflated.
Bounded VBFrame BeginProperty Font blocks reconcile with binary fonts; malformed,
duplicate and conflicting metadata refuses before attachment. Original source
and resource bytes remain separate and unchanged. No font file is opened.

The IDE supplies ordinary generated Font objects claiming MSForms.Font/NewFont
and the selected stdole Font/StdFont/IFontDisp automation interfaces. Each supported
form/control initially owns an independent object. Font.Name is its default
String property; Size is Currency; Weight and Charset are Integer; Bold, Italic,
Underline and Strikethrough are Boolean. Bold and Weight follow the documented
550 threshold and 700/400 assignments. Ordinary coercion, aliases, errors,
Stop/Continue, lifetime and Reset remain in the shared runtime. No VM opcode or
foreign host operation contains Office names.

Font getters preserve lazy form loading. Runtime-added controls and Pages copy
their container's current font fields into their own objects. Later edits do not
silently share those fields. Inert font inspection reads canonical fields and
does not invoke getters, load a form or acquire platform fonts. Image, SpinButton
and ScrollBar do not gain a Font member. Font object replacement, native edited
font persistence, complete ambient/toolbox defaults and platform realization are
not accepted by these property tests and remain explicit gaps.

Native/runtime font strings are bounded. Nonpositive sizes and invalid weights
or charsets refuse without changing the previous value. Size above the native
unsigned-size representation and over-budget strings use the existing resource
error; these resource boundaries are not fresh Office oracle observations.

## Static metadata ceiling

The compiler and verifier share MAX_NOMINAL_LOCALS = 8,192, replacing their old
4,096 global nominal-table ceiling. This is a static declaration table, not the
host-call permission table. Host permissions remain 4,096. The existing 64 MiB
verification metadata limit and work limits still charge every entry. VM work,
memory, locations, callbacks and authorization limits are unchanged. No declared
types, executable field aliases, event signatures or checks are dropped to fit.
The serialized representation and instruction semantics are unchanged; no IR
version or capability is added merely to adjust a bounded resource ceiling.

This changes the numeric profile ceiling recorded in ADRs 0073/0080. It does not
relax ADR 0073's event-compaction regression: that existing large typed field/event
case must still remain below 4,096 entries, so the larger ceiling cannot hide a
regression in Event alias generation. Separate authored compiler/verifier cases
exercise programs above the former limit and refusal beyond the new limit.

## Rendering and font policy

The shared UI reads canonical snapshots and applies typography only to the
control's child Ui. It preserves the embedding's global styles and font atlas.
Native point sizes convert at the form's existing 96/72 logical scale. Installed
named families can be selected directly; optional Theme.form_font resolves a
requested face/weight/style to a caller-owned installed family. Missing faces
substitute the caller's existing family, as a view decision; canonical names and
sizes do not change. No font acquisition, global font replacement or COM exists.
Italic, underline and strikethrough use local text layout; a supplied face resolver
owns italic/weight face selection. Exact bold faces and platform metrics require
the embedding's fonts; this is not measured Office glyph/layout equality. Glyph
realization is capped at 128 logical pixels and quantized to 1/64 pixel,
independently of retained font values. Sizes outside the display range show a
visible notice; their canonical Currency values remain unchanged. Unimplemented
ambient, font-substitution and platform cases do not count as milestone PASS.

## Evidence

Independently authored binary/frame cases cover defaults, fractional size/effect
retention, reconciliation, conflicts and original resources. Five runtime cases
cover typed/default properties, independent objects, Bold/Weight, coercion/errors,
dynamic container/Page defaults, aliases after unload, Stop/Continue, inert
inspection and Reset. Three GPU-rendered cases check authored typography, a
bounded-size notice, injected face selection, missing-face fallback, canonical
values and global-style isolation. Final shared and consumer acceptance receipts
belong in implementation progress.

Full M7A.4 remains IN PROGRESS. Fresh Office build/OS/locale observations remain
M8.5 NOT_RUN. Native edited saving remains M7A.5 and .tvform removal M7A.6.

References: [Forms Font](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/font-object-microsoft-forms),
[font attributes](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/bold-italic-size-strikethrough-underline-weight-properties),
[Forms NewFont](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.newfont?view=office-pia),
[TextProps](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/ebe980dd-676b-4940-b978-5c7c4644a520),
[StdFont](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/e219ce61-420f-403b-8216-63890c935aa1),
[FontHeight](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/012f7562-a80c-4b5b-96c0-8b9c36319b34),
[FontCharSet](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/6771b963-fd57-45a6-8c71-e124da4d460c).
