# M7A.5 — Native editing and persistence

Updated 2026-11-08. Status: **COMPLETE** for the explicit workflows below. Contract: the accepted
[common profile](m7a4-common-profile.md), under
[ADR 0098](decisions/0098-native-form-encoding-and-source-only-writeback.md).
This is authored regression evidence, not fresh Office observation.

## Supported workflows

Standalone and embedded editors export native `.frm`/`.frx` files. Native Save
acknowledges the exact source/design snapshot only after a successful file commit;
Export leaves the editor modified. Browser transfer offers both binary files and
never assumes a download request saved them. The content-named companion is
published completely and exclusively before the atomic `.frm` replacement; an
existing differing companion refuses. Failure retains the previous primary file.
Existing private file modes are preserved. A failed commit can leave an unused
companion, which is safe to remove when no form references it.

GridCraft's **Export Edited XLSM Copy** and WordCraft's **Export Edited DOCM Copy**
compile the current idle/reset project without execution, encode its source and
form resources, remove invalidated VBA signature parts/relationships/content types,
write a macro-enabled copy and reopen it to verify the exact encoded project.
The native chooser refuses the current document path, including canonical aliases.
Copies do not clear the original editor's modified state or replace its imported
bytes. Ordinary document Save continues to preserve the original VBA project;
use the explicit copy action to persist authored VBA edits, then open that copy.
Browser package export requests a download and does not acknowledge a disk commit.

Existing controls retain native identities, untouched font/flags/resource records,
storage CLSIDs/state and nested Frame/MultiPage resources. The current designer
supports form/control captions, text, geometry, enabled/visible state and eligible
tab order, plus Label/TextBox/CommandButton toolbox addition/removal. Writers advance
NextAvailableID. Unrepresentable source/header characters, unsupported resource
edits and changed component/code-name/reference identities refuse explicitly.
Imported originals remain separate from encoded output and running VM values.
The uncommon extensions remain in [P9](p9-forms-compatibility.md).

Source-only MS-OVBA output has zero module text offsets, no module prefix or SRP
caches, and a cache-free `_VBA_PROJECT` header. Rebuilding the CFB removes old cache
bytes from free sectors too. Exact source, design, component/code-name/reference
identity is compared after decoding the result. Protected projects and read-only
source edits refuse; no P-code or protection bypass is implemented. The bounded
compressor refuses an incompressible partial chunk rather than padding source.

## Evidence and gates

| Boundary | Executable evidence |
|---|---|
| Native file records | `vba-project/tests/native_save.rs`: six cases cover edited source/text/caption/geometry/state, toolbox add/remove, code-page refusal, nested resources, empty resources and storage metadata |
| Source-only project | `vba-project/tests/writeback.rs`: four cases cover exact source/form writeback, structural offsets/cache removal, compression limits and identity/code-page refusal |
| Save ownership | `vba-ide/tests/native_save.rs`: four cases cover current/foreign/stale snapshots, native HostRequest routing, compile-checked project export, close releasing originals and invalidating tokens |
| Atomic file IO | standalone `source_io`: nine retained/new cases, including native pair collision/failure and private primary/companion modes |
| XLSM/DOCM package | each fork's `native_forms` suite edits actual source and embedded TextBox resources, exports/reopens and runs the edited handler only after trust; rejects stale snapshots and retains original bytes |
| Signature removal | Grid XLSX test retains shared nonsignature data while removing signature reachability; Word DOCX test strips legacy/agile/v3 forms while retaining VBA data; incomplete graphs/manifests refuse |
| Host UI | Grid `vba_forms` and Word `native_package_copy_ui` invoke copy actions and inspect output; original dirty state and bytes remain intact |
| Browser native files | `tools/ui-browser/native-save.cjs` edits the actual designer caption, downloads two files, reimports inertly and runs unchanged authored source; assertions check binary companion identity |

Shared native CI, strict workspace Clippy, WASM/WASI and native UI/Xvfb/web gates
pass; latest project tests pass natively and on Node 22 WASI (60 cases), retained
instance tests pass (99), and the four final IDE save cases pass natively/on WASI.
The nine file-IO cases pass. All 15 browser flows pass. Both hosts pass full CI, strict optional VBA/UI Clippy,
VBA-enabled web builds and native/WASM package suites (Grid 34/34, Word 29/29).
Grid passes six form UI cases; Word passes 13 retained VBA UI cases and the
hidden-editor, canvas and explicit copy-command cases. Host canvases and native
save/reopened views were inspected. Exact revisions and final checks are in
[implementation progress](implementation-progress.md). Source fixtures/builders
are original Rust; no Office-created binaries or external implementation were added.

## Remaining milestones and limits

M7A.6 removes the shipped legacy `.tvform` codec/routes/examples; it is not part of
this persistence gate. M7A.7 runs final combined acceptance. M8.5 fresh Excel/Word
comparisons remain **NOT_RUN**. Cryptographic signature verification/resigning,
protection changes, arbitrary OCX/COM loading and exhaustive native resource/member
parity are unsupported. No runtime/host permission, memory or execution limit was
raised for this work. Encoder snapshots are inert; runtime input retains existing
host trust and authorization checks. Exported copies have invalidated signatures
removed and must receive the normal import/trust decision when opened.
