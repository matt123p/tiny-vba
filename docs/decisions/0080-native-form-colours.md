# ADR 0080: Native form and control colours

Status: implemented and verified for the selected colour profile; full M7A.4 remains in progress.
Date: 2026-11-07.

## Ownership and representation

The native decoder now retains BackColor and ForeColor as signed OLE_COLOR Long
values, including the original type/palette bits. Form/Frame/Page containers,
MorphData, Label/CommandButton, numeric, TabStrip and Image records retain their
class-specific file defaults. Image has BackColor and no ForeColor member.
VBFrame colour properties admit bounded decimal and native hexadecimal values;
conflicting explicitly stored frame/binary colours refuse before attachment.
Invalid colour encodings also refuse before project mutation.

These are native metadata fields, not additions to the legacy .tvform codec.
Original source and resources remain separate and unchanged. Runtime edits do
not implement native edited persistence; that remains M7A.5.

The IDE generates ordinary BackColor/ForeColor properties and canonical fields.
Root access retains existing unloaded-form initialization; control access keeps
ordinary identity, aliases and lifetime. Assignments use normal Long coercion and
validate RGB/default, palette-relative RGB, bounded client-palette indices and
defined system-colour indices. Invalid values raise 380 without replacing the
previous value; normal type/Null/overflow errors remain in the common coercion
path. Colour assignments do not raise text Change events. Inert inspection reads
the fields and never invokes getters or loads a form.

One generated standard-module validator serves all colour setters. Static child
attachment uses the existing Controls/Pages collections directly, removing three
redundant container forwarding methods. GridCraft MultiPage regressions exposed
the nominal metadata cap: these source reductions preserve its 4,096-entry bound
without weakening typed fields or changing compiler/VM limits.

No compiler, VM, foreign-protocol or IR capability is added. Generated support
uses decimal expressions, including the existing Long-minimum expression, rather
than extending the language's literal profile. No execution, memory, callback or
static-permission budget is increased.

## Presentation

The shared native renderer reads those canonical fields for form and control
backgrounds and applicable text/arrows. RGB values keep their authored colour.
Default system colours project onto the caller's injected theme roles; this is
not a claim of measured Windows/macOS palette equality.

Optional Theme.form_color supplies a pure, read-only UI palette resolver for
exact platform or application palette values. Explicit RGB bypasses the resolver. It is not a VBA callback, host
capability or colour-value store. A client-palette entry the caller cannot
resolve keeps its canonical Long and shows a visible unavailable-palette message
while using the existing themed fallback. No palette/font files are acquired,
and neither global visuals nor host fonts are replaced.

This packet does not implement Font, BorderColor/Style/SpecialEffect, BackStyle,
all ambient/toolbox defaults, selected/hover/disabled presentation or complete
platform appearance. Their existing M7A.4 requirements remain open. File-format
defaults must not be relabelled as measured VBE toolbox defaults.

## Runtime-created MultiPage lookup

The colour tests exposed a preexisting parent-lookup omission for Pages added to
a runtime-created MultiPage. Root lookup now searches the existing bounded Page
collections of runtime-created MultiPages after its ordinary direct lookup.
There is no second object registry. A nested control's Parent resolves to the
actual Page; ordinary colour state and continuation survive Stop/Continue.
Further collection/removal/reentrancy cases remain in C03/W01.

## Evidence and scope

Independently authored fixtures cover record defaults, RGB/system/palette bits,
Long minimum, native frame reconciliation and invalid input. Runtime tests cover
typed/late-bound properties, instance isolation, coercion/errors, all selected
dynamic control families, Pages, parent identity, Stop/Continue and Reset/source/
resource preservation. Eight model cases also check attachment-time member-name
collisions and preservation of legacy authored inspection fields. Actual GPU-rendered tests assert authored pixels, local
style ownership and unresolved/injected palettes. Final native/WASM/WASI/browser
and consumer receipts belong in implementation progress.

Shared verification: cargo xtask ci (strict Clippy and native tests), wasm, ui,
Xvfb ui-native, web, repository checks, native colour snapshots and actual Chromium
smoke/forms/native-colors all pass. Five theme/scale views and both host document
views were inspected. The final wasm-run wrapper exited 143 during its last IDE
suite; all preceding suites passed and the remaining native_colors, native_forms,
nominal, projection, references, search, suspended_calls and workflows WASI suites
were rerun successfully with cargo test on wasm32-wasip1. This records complete
case coverage across runs, not a zero exit for the interrupted wrapper.

Both consumers pin shared product d3c99baab86fad9e33333fd4204066757baec63d.
GridCraft integration 384b66ea passes full CI (10 steps), strict optional VBA/UI
Clippy, five form UI cases, VBA web and 29 native/29 Node-WASM package cases.
WordCraft integration a3dc31e9 passes full CI (8 steps, isolated serial tests),
strict optional VBA/UI Clippy, 13 VBA UI cases, hidden-editor and canvas-prefix/
Reset flows, VBA web and 24 native/24 Node-WASM package cases. Final full host CI
used each checkout's native target/debug/xtask ci entry point after terminated
cargo wrappers; the task implementations and checks are the same.

P01/W01 and full M7A.4 remain IN PROGRESS. No refusal or themed approximation
closes an unimplemented member/platform requirement. Fresh Office build/OS/locale
comparisons, including palette readback and ambient defaults, remain M8.5 NOT_RUN.

References: [MSForms BackColor](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/backcolor-property-microsoft-forms),
[MSForms ForeColor](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/forecolor-property-microsoft-forms),
[native BackColor defaults](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/e7bd4bba-ba34-4f8a-9792-3d327d997c73),
[native ForeColor defaults](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/ad83bd19-7e70-4465-963d-65058eb724f8),
[OLE_COLOR](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/4b8f4be0-3fff-4e42-9fc1-b9fd00251e8e),
[OLE colour types](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/55b9612d-3a9a-4e62-9f68-27af8e479210),
[system-colour indices](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsyscolor).

Correction: [ADR 0083](0083-page-appearance-ownership.md) removes the earlier
non-Office Page Font/BackColor/ForeColor exposure. Page children and client painting
use the owning MultiPage; the Font-capable control count is twelve, excluding Page.
