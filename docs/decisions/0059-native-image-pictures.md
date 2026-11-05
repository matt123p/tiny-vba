# ADR 0059 — native Image and immutable picture resources

Date: 2026-11-05. Status: adopted for the selected native raster profile.
M7A.4 remains IN PROGRESS; this decision preserves the full remaining ledger.

vba-project decodes cached Image (12), its distinct ImageControl record and
GuidAndPicture/StdPicture resources. Import stays inert. Validate masks, markers,
lengths, dimensions and original resource bounds. BMP, static GIF, JPEG and ICO
raster decoding uses pinned image 0.25.10 in memory, without GUI, platform
handles, filesystem access or control activation. PNG outside ICO is not an
MS-OFORMS StdPicture format and refuses. WMF/EMF and custom mouse resources
remain unsupported. The selected VariousPropertyBits profile requires an opaque
control; colours/decoration follow the existing themed presentation policy.

A picture has at most 4096 pixels per dimension and 1,048,576 pixels total;
combined decoded RGBA across a form is capped at 4 MiB. Encoded resources retain
the existing 4 MiB cap. Header and decoder phases apply explicit dimensions and
16 MiB codec allocation limits, with area checked before pixel decoding. RGBA is
an immutable Arc asset; descriptor clones and instances share those bytes.
BMP physical resolution supplies HIMETRIC extents. Other supported rasters use
the renderer's 96 logical DPI fallback; further metadata/platform DPI cases need
comparison. GIF is a static picture, with no animation feature introduced.

This introduces the reviewed project -> image edge. image is a portable raster
library, previously used for shell PNG capture; it is not a UI dependency. Keep
egui/eframe/UI/app/web-sys above the toolkit-independent model. The architecture
gate now allows the pinned in-memory codec and checks explicit bmp/gif/jpeg/ico/
png features with default features disabled. No toolkit/platform boundary is
relaxed. New lockfile dependencies gif 0.14.2 (MIT OR Apache-2.0) and color_quant
1.1.0 (MIT) are licensed raster-codec support; image retains MIT OR Apache-2.0.
The existing pinned JPEG/PNG codec dependencies remain unchanged.

The IDE generates ordinary Image and picture classes. stdole.IPictureDisp and
StdPicture are nominal reference metadata, not COM activation. Picture get/set
uses the same nominal interface, preserving Nothing and ordinary object identity.
Invalid interface assignment fails through existing VBA type conversion before
mutation. Width/Height are read-only Long HIMETRIC values; Type is Integer.
Picture objects name immutable source resources using bounded internal fields,
without a second mutable host model. Inert inspection reads canonical VM fields
and resolves the original descriptor without executing getters or loading forms.
Assignment between controls shares the picture object; clearing one reference
does not change another control, source or resources. Platform handles and
LoadPicture/file services remain unsupported.

Image has no Caption, SetFocus or public tab-order surface. Click uses ordinary
event frames and preserves text focus/dirty state; it never synthesizes Enter,
Exit or validation. Stop/resume does not replay input. Native container traversal
skips nonfocusable objects without evaluating nonexistent tab properties. Internal
kind values are read once per scan, retaining the existing VM work/location
budgets. AutoSize, PictureSizeMode (0/1/3), PictureAlignment (0..4), PictureTiling
and selected border/effect properties remain ordinary instance state.

The shared renderer reads those fields, draws the original raster with clip,
stretch, zoom, corner/centre alignment and repeat tiling, and respects ancestor
clipping, visibility/enabled state and injected theme/fonts. Repeat UVs bound
paint work even for one-pixel pictures. GPU texture caches contain derivative
view data, keyed by weak immutable asset identity and capped at 128 entries;
they contain no canonical control state. Pointer input is executable through
the existing authorized command path and accessible Image identity.

Original independently authored Rust bitmaps and form/container records are
MIT OR Apache-2.0 test material, not vendor assets or Office-produced files.
Tests establish authored expectations and portability, not fresh Office traces.
Full mouse/drag/drop/picture/presentation/metadata/platform variants, RefEdit,
bindings, dynamic collections and lifecycle/property cases remain M7A.4.
Native edited saving and .tvform removal remain M7A.5/.6. Fresh Office build/OS/
locale observations remain NOT_RUN under M8.5.

Sources: [StdPicture](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/8c1c088a-9775-49de-9b3b-298eeb1ddc19),
[Image interface](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.image),
[picture assignment](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/picture-property),
[alignment](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/picturealignment-property),
[size modes](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/picturesizemode-property),
[AutoSize](https://learn.microsoft.com/en-us/office/vba/api/outlook.image.autosize),
[codec limits](https://docs.rs/image/0.25.10/image/struct.Limits.html).
