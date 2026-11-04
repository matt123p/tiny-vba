# ADR 0055 — native form metadata and control tips

Date: 2026-11-04. Status: adopted for the selected native flat-control packet.
Full M7A remains IN PROGRESS.

vba-project promotes retained OleSiteConcrete Tag, ControlTipText and
HelpContextID into bounded inert native control descriptors. VBFrame Tag and
HelpContextID become form metadata for export/embedded attachment. Source and
original resources remain independent. Invalid metadata fails before attachment;
string limits and positive-Long help IDs follow the existing import budgets.
No control activation, filesystem lookup, help-file load or new codec is added.

The IDE initializes ordinary per-instance Tag/ControlTipText String and
HelpContextID Long state before authored Initialize. Forms expose Tag and
HelpContextID through their existing lazy-load lifecycle. Control metadata uses
the same object identity through nominal MSForms.Control references. Negative
help IDs raise 380 before mutation; other coercion/errors use the existing VBA
field/parameter path. Generated-member collisions and invalid external descriptor
metadata refuse atomically. Read-only field aliases inspect backing state without
invoking properties, loading forms or executing handlers.

The shared renderer reads canonical ControlTipText and shows a bounded tooltip
at the pointer after the toolkit hover delay, scoped to the actual enabled control
rectangle and layer. Hover never transfers focus or submits a VBA command. Empty
text suppresses the tip. Tooltip frame/text use injected theme tokens, existing
host fonts and per-instance widget identity; no global style/font replacement or
mutable control registry is introduced. Native/browser presentation share this
path. The original fixture resources and native package bytes remain untouched.

HelpContextID is metadata storage only: host Help-file association and actual F1
routing remain unimplemented platform work. Native edited design/resource saving,
remaining control/container/image/binding/property semantics and legacy .tvform
removal remain M7A requirements. Evidence is authored resources, runtime/error/
isolation/atomic-import cases, actual widget hover, theme renders and browser
checks. Fresh Office comparisons remain NOT_RUN under M8.5.

Sources: [Tag](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/tag-property),
[ControlTipText](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/controltiptext-property),
[HelpContextID](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/helpcontextid-property),
[MS-OFORMS](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/).
