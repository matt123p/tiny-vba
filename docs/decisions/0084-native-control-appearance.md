# ADR 0084: Native control borders, effects and backgrounds

Date: 2026-11-07. Status: selected shared and affected host acceptance passed. M7A.4 remains open.

## Ownership and data

vba-project retains Appearance on native Design/Control records: OLE BorderColor,
BorderStyle, SpecialEffect and BackStyle. Native Form, Label, MorphData and Image
records use their distinct widths/masks and published file defaults. VariousBits
retains transparency where applicable and refuses it on mandatory-opaque control
families. Enum/colour validation and VBFrame reconciliation refuse malformed or
conflicting data before attachment. Original source/resources stay unchanged.
Image's border/effect fields move from its picture packet into this common data;
there is no duplicated mutable or immutable authority. No custom format changes.

File Form defaults (flat) remain distinct from a newly created runtime Frame
(sunken). Header BorderColor/BorderStyle/SpecialEffect apply to empty forms too.
Page common records remain retained, but do not create Page appearance APIs
(ADR 0083). There are no border/background-style members on MultiPage or TabStrip.

## Ordinary runtime

The IDE emits only applicable members: borders on UserForm/Frame, Label, TextBox,
RefEdit, ComboBox, ListBox and Image; effects additionally on CheckBox/OptionButton;
background style on the documented transparent-capable text/button/image families.
ToggleButton's disabled SpecialEffect stays unavailable; its persisted value
remains the required sunken value. Unsupported members are not invented.

Appearance enum properties use Long values and registered Microsoft Forms enums;
BorderColor retains OLE Long bits and uses the shared colour validator. Nonzero
BorderStyle resets SpecialEffect, and nonzero SpecialEffect resets BorderStyle.
Zero leaves the other field unchanged. CheckBox/OptionButton allow only flat and
sunken. Invalid enums/coercions refuse before mutation. Transparency preserves
BackColor. Getters/setters use ordinary frames, root lazy loading and existing
coercion, debugger/Reset, lifetime, trust and work/memory/permission policies.

## View

Local painting reads canonical fields. Single borders use the requested colour;
raised/sunken/etched/bump edging uses injected system colours with theme-derived
fallbacks. Check/option edging applies to the indicator. Flat Frames ignore
BorderStyle as documented. Text/Combo/Image chrome no longer adds a second
unconditional outside border. Native controls clip paint to their authored bounds;
popups retain their separate layers and existing pointer regions.

Transparent backgrounds reveal already painted siblings/parents, without changing
stored colours or making hit regions transparent. Bitmap transparency is separate.
The RefEdit collapsed strip uses the same local attributes. Missing palette notices
exclude colours hidden by transparency or flat Frame border suppression. Caller
fonts/global styles and original data stay unchanged. Exact platform palette/glyph/
edge realization remains an embedding policy, not a fresh Office pixel claim.

Independent decode/frame, model and real GPU cases cover values, enum types,
errors, effects coupling, defaults, opacity/overlap, borders/system colours,
fixed-bound glyph clipping, Frame exceptions, Stop/Continue and Reset. Existing
34 pointer cases pass. Full shared/browser/host receipts belong in progress.
The broader P01/W01/member/platform checklist stays open; native edited saving is
M7A.5 and fresh Office comparisons M8.5 NOT_RUN.

References:

- [Forms appearance](https://learn.microsoft.com/en-us/office/vba/language/concepts/forms/ways-to-change-the-appearance-of-a-control)
- [BorderStyle](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/borderstyle-property)
- [SpecialEffect](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/specialeffect-property)
- [BackStyle](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/backstyle-property-microsoft-forms)
- [Transparent controls](https://learn.microsoft.com/en-us/office/vba/language/concepts/forms/create-a-transparent-control)
- [Native BorderColor](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/6896a3a4-9632-4ac5-950f-ef5f61db08e8)
- [Native BorderStyle](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/77d241d0-520b-47ea-ad24-071046e4cc2c)
- [Native SpecialEffect](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/96e57270-2409-4cd8-8586-a2219cf834d0)
- [VariousPropertiesBitfield](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/c38cc3f9-969b-45da-ab41-b361c044431e)
- [MorphDataDataBlock](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/8a95d726-067d-4cd0-8fd9-3a4b6cc6188d)
- [LabelDataBlock](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/1d97ecdc-e827-49e8-88be-657eb3007f7d)
- [FormDataBlock](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/096870e8-5263-44ed-885b-379d23471c4f)
- [ImageDataBlock](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/e004baf2-792b-403e-b589-c16ba25ee696)
