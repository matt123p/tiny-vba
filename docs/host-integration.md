# Host integration

Embed tiny-vba through revision-pinned dependencies at the application's upper
composition layer. The host owns document objects and trust; tiny-vba owns VBA
execution and the reusable editor/debugger. A sibling checkout is needed only for
optional cross-repository acceptance gates.

## Native calls and capabilities

`vba-foreign` supplies a host-neutral Rust protocol for calls, properties,
references, typed results/patches and asynchronous wait tokens. Supply imports,
roots and immutable reference/type metadata explicitly. Native object identity
and generation checks belong to the host. The VM checks protocol compatibility
and per-site permissions; compiling a call cannot authorize it.

Synchronous operations run in process. Slow operations return a wait token;
explicit resume polls it and completes the saved continuation exactly once.
Release document borrows before callbacks, waits, debugger pauses and rendering.
Cancellation must release pending host tokens. Effects already committed to the
host survive VBA failure and Reset. See [runtime](runtime.md) for exact lifecycle,
permission, budget and writeback contracts.

## References and events

Host metadata supports nominal object types, enums/constants, optional defaults,
default members and collection iteration without creating another Office registry.
Editor catalogues describe members without executing them. Source declaration
shadowing and explicit reference selection remain authoritative.

The host decides event order, captures typed arguments and authorizes delivery.
Document callbacks resolve verified code names; document display names are not
VBA identities. Host-object WithEvents subscriptions and ByRef outputs use the
existing callback path. The VM remains non-reentrant; supported suspended/modal
callbacks use explicit continuations. No automatic Open/New execution is implied
by importing or attaching source.

## Project and document ownership

Discover OOXML relationships and extract `vbaProject.bin` in the host. The shared
[project importer](project-import.md) decodes caller-supplied bytes inertly.
Keep original package/project/resources separate from extracted and edited source.
Reference paths must not trigger filesystem, registry or library activation.

The host owns trust prompts, storage, protection, signatures, undo/recalculation
and document lifecycle. Plain-source export, ordinary document Save and explicit
edited macro-enabled copy export are separate operations; see [forms](forms.md).
Closing a project cancels execution and releases pending host work. Never retain
a mutable document borrow in a paused session.

## Application integrations

| Host | Owned integration | Scope |
|---|---|---|
| GridCraft | Fork `crates/vba`, workbook engine/UI and XLSX packaging | Excel-compatible Application/Workbook/Worksheet/Range profiles, values/formulas, selected formatting/data/table operations, events, UDFs and worksheet controls |
| WordCraft | Fork `crates/vba`, document lifecycle/UI and DOCX packaging | Application/Document/Selection/live Range profiles for supported plain main-story paragraphs, selected events and native package workflows |
| DeckCraft | Intended host | No integration claimed |

Feature branches and build commands are in the [root README](../README.md).
Each fork's pinned dependency determines actual support. Tables/inline-object
position mapping and the broader Word model are outside the declared Word profile;
arbitrary Excel/Word macros are not guaranteed to run unchanged.

The maintained member-by-member scope belongs to the forks:
[GridCraft integration](https://github.com/matt123p/gridcraft/blob/feat/tiny-vba-integration/docs/vba.md),
[Excel audit](https://github.com/matt123p/gridcraft/blob/feat/tiny-vba-integration/docs/vba-excel-compatibility.md),
and [WordCraft integration](https://github.com/matt123p/wordcraft/blob/feat/vba-host-integration/docs/vba-integration.md).

For panel mounting, source buffers, host requests, save acknowledgements and
native/browser harnesses, use [the embedding guide](ui/embedding.md).
Office-shaped test doubles belong with the host adapter; shared tests use generic
objects. Optional real-host gates and evidence limits are in [testing](testing.md).
