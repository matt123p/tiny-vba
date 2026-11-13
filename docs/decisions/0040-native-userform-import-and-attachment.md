# ADR 0040 — Native UserForm import and attachment

Date: 2026-11-04. Status: adopted for M7A.1/M7A.2 under ADR 0038.

## Ownership and selected profile

`vba-project` owns `native-userform-import-v1`: inert `.frm`/`.frx` export and
embedded MS-OVBA/MS-OFORMS decoding, native component identity, original resources,
authored source and diagnostics. `ModuleKind::UserForm` is distinct from Class;
`Module::form` carries its decoded resource result. The project decoder still
refuses protected envelopes and PROJECTlk licence data and never reads executable
cache implementation data. Standard/class/document behavior is unchanged.

The initial admitted hierarchy is a UserForm root with immediate cached Label,
TextBox and CommandButton children. Native site IDs, names, positions, sizes,
caption/value, Enabled, Visible, TabStop and TabIndex are decoded. Containers,
non-cached classes, pictures, bindings, default/cancel, accelerators and additional
behavior variants refuse explicitly. These are declared profile limits, not
claims that those Office features are absent or an authorization to invent them.
Broader controls and lifecycle semantics remain M7A.4. See the
[property/event/format/host matrix](../forms.md).

`vba-ide` now depends on `vba-project`, adding one reviewed downward edge. It maps
validated native data into the existing shared form designer/runtime. Neither
project decoding nor the generic VM depends on the IDE or a fork. There is no
new crate, bytecode version, Office object store, runtime control library or
external production dependency. The reviewed graph follows the actual manifests.

## Identity, source and resources

MS-OVBA directory ordinal `ModuleId` stays project-local. MODULENAME/PROJECT
BaseClass/VB_Name/VBFrame designer name must agree. MODULESTREAMNAME is independently
preserved and selects the designer storage; it need not equal the component name.
Hosts allocate `SourceId`, keyed to the attached form/class and handler source.
Native site IDs remain separate from caption, name, tab index, painting order
and caller identities. For v1 the hierarchy is flat; deeper sites and container
streams refuse, so hierarchy is never flattened or guessed.

Export source is the exact decoded text after the designer header, including
attributes and line endings. Embedded source uses the existing decompression and
code-page decoder. Original export/frame/companion/embedded streams are retained
separately from editable source/design. Original resource snapshots do not change
when runtime values or the design change. Generated support only supplies missing
identity attributes and never enters authored source. Native names that cannot
be represented by the current shared designer fail before attachment.

## Input and failure contract

The codec accepts caller-owned bytes, an explicit code page and companion basename.
It never opens files, traverses paths, resolves references or loads COM/OCX/DLLs.
OleObjectBlob permits a local `.frx` basename and bounded hexadecimal offset only;
the caller supplies exactly the named companion. No signature scanning occurs.
The export reader admits CFB at that exact offset or its checked length-prefixed
OLE envelope; this is authored envelope evidence, not a fresh VBE export result.
Embedded decoding separately requires the native designer storage and VBFrame/f/o
streams. Export success never substitutes for embedded evidence.

Limits: existing project input/stream/source/entry/work bounds remain. Each form
has at most 128 controls, 4 MiB resource data, 16 KiB native strings, strict
record/string/site/object lengths and no recursive container descent. Standalone
resource CFB has at most 512 entries, depth four and 200,000 metered IO operations;
aggregate project designer resources are at most 8 MiB. IDE attachment caps retained
resources at 8 MiB across forms and retains its 2 MiB design budget. Lower caller
limits remain effective. Invalid encodings, IDs, duplicate names, malformed/trailing
data, unknown classes/streams, unsupported masks and conflicting captions fail.

An import builds and validates a complete candidate before changing module/source
identity, selection or design. Handler validation precedes insertion. Whole-package
attachment constructs a fresh IDE before publishing it in each host. Failed import
never fires Initialize, changes trust, publishes a partial component or consumes
a source ID. Failures use existing CORRUPT/UNSUPPORTED/PROTECTED/LIMIT diagnostics.

Formatting/font/design metadata admitted by the decoder remains in original
resources with a visible diagnostic where the current renderer does not interpret
it. Unknown runtime behavior properties refuse; they are not silently dropped.
HIMETRIC positions/sizes convert to points only at shared designer attachment.
Full Office font/decoration/focus/lifecycle parity remains M7A.4.

## Workflow, lifetime and persistence

`Ide::attach_native_form`, `import_native_form_export` and `import_source_bundle`
feed the existing Project Explorer, View Object/View Code, Properties, handler
navigation and debugger. Standalone native selection reads the validated sibling
companion in its IO layer; browser selection supplies the explicit pair together.
The export shell profile is Windows-1252, with explicit other project code pages
available through the Rust codec/API. Embedded projects use PROJECTCODEPAGE.
GridCraft and WordCraft attach native package forms through the same inert decoder;
document ownership and trust remain host-owned. Native byte decoding is portable
to native/WASM/WASI; it does not depend on Office being installed.

After explicit execution authorization, existing modal callbacks, debugger
continuations, exact handler source and Reset/close cancellation apply. Import
does not itself instantiate a form. Multi-instance/lifecycle compatibility changes
belong to M7A.4, rather than being inferred from native format support.

Native form export/save requests refuse until M7A.5; no request may silently save
the custom `.tvform` archive or acknowledge edited binary resources as persisted.
Unchanged package saving retains original bytes. Native edited source/resource
encoding, atomic file commit, source/design revision acknowledgment, executable-cache
handling and signature invalidation remain the separate ADR 0037/M7A.5 contract.
Removal of legacy routes remains M7A.6; this work adds none.

## Evidence and references

Original Rust fixture builders synthesize native records and CFB packages under
MIT OR Apache-2.0. The same design is asserted independently through export,
embedded, shared IDE, standalone shell, rendered UI and fork package attachment.
Negative tests truncate every byte boundary of f/o streams and exercise unknown
types, bindings, flags, lengths, missing companions, protection and atomic failure.
Authored tests establish the implemented subset; fresh Office build/OS/locale
comparisons remain NOT_RUN under M8.5.

Format basis: [MS-OFORMS](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/9c79701a-8c3e-4429-a139-b60ac3a1d50a)
v20250819 §§2.1.1.2, 2.1.2, 2.2.1/4/5/10, 2.3, 2.4.5/7/14, 2.5;
[MS-OVBA designer storages](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/f614ae64-1b3d-47be-a166-0e10b8230026)
and [VBFrame](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/76ecef26-5e1c-4729-8074-c76584e25b6b).
The public specification PDF was read outside the repository; no implementation
code, vendor assets, Office-produced fixture or downloaded macro was adopted.
