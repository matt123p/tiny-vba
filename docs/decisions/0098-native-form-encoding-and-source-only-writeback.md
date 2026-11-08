# ADR 0098: Native form encoding and source-only project writeback

Date: 2026-11-08. Status: adopted; M7A.5 implementation/acceptance IN PROGRESS.

Owner direction activates M7A.5 for the accepted common forms profile. This
implements the separately reviewed encoder boundary required by ADR 0037; it does
not turn ordinary document saves into implicit binary writeback. vba-project now
owns explicit inert form and source-only project encoders beside its decoder.
The compiler/VM remain independent; no crate, IR, foreign call, host authority or
budget changes. Office package/signature/trust policy and actual IO remain in hosts.

The native form writer reconstructs admitted Form/OleSite/leaf property blocks,
retains untouched native flags/font/class/resource data and storage class/state
metadata, advances NextAvailableID past persisted control identities, and writes current authored
designer state rather than VM widget values. Existing common controls support
basic designer edits; the current Label/TextBox/CommandButton toolbox supports
addition/removal. Existing nested Frame/MultiPage resources are retained and child
geometry can change. Unsupported resource edits refuse; they never fall back to
`.tvform`. Output reopens through the native decoder and must match the target
design and exact authored source. Code-page errors refuse instead of replacing
characters. Native source/resource input/output and all record sizes remain bounded.

Native export consists of a standard `.frm` plus its explicitly referenced `.frx`.
The companion basename includes a deterministic content discriminator, used only
for immutable-file naming, never authentication. A filename collision must compare
all bytes and refuse differing content. Standalone native IO stages/syncs complete
files, publishes the companion exclusively, then atomically replaces the `.frm`
commit point. It never overwrites an existing companion. Failed commits leave the
previous `.frm` and its resources intact; an unused new companion may remain.
Existing private file permissions also apply to its replacement and new companion.
Browser transfer downloads both binary files and does not infer a disk-save
acknowledgment from merely requesting downloads.

IDE snapshots bind source/design revisions and an opaque project-instance owner.
Stale, removed or foreign snapshots cannot clear modified state. Successful project
close releases native originals and invalidates outstanding save-owner tokens. Native exports
remain separate from the exact imported original resources. Binary project
snapshots compile exact authored state without execution and require an idle/reset
session. Fixed component/code-name/reference identities are retained; component
insertion/renaming and unrepresentable/unsupported source/resources refuse.

The MS-OVBA writer updates directory MODULEOFFSET fields through structural
parser positions, sets every module offset to zero, and writes compressed source
with no module performance prefix. It removes all `__SRP_` streams and writes a
seven-byte `_VBA_PROJECT` header with Version=0xFFFF and no PerformanceCache.
The CFB is rebuilt from live streams, preserving storage class/state metadata;
removed cache bytes cannot survive in free sectors. No P-code is inspected or executed. The prior import's NotChecked state is not
accepted as cache consistency: fresh output is constructed without those caches,
then all extracted sources, component/code names, references and form designs are
compared after reopen. Literal/copy compression is bounded; an incompressible
partial chunk that would require padding exact source bytes is explicitly refused.
Protected projects and read-only source edits remain refused.

The public project encoder is an inert format API; callers must compile their
snapshot before invoking it. The IDE's project save API enforces that compilation
and validates current original bytes/revisions before caller commit. A project-save
acknowledgment is allowed only after successful package/file commit and removal of
all VBA signature forms/relationships/content types by the owning fork. This shared
encoder alone does not implement or certify host package/signature policy.

Tests and exact gate results are recorded in implementation progress. Fresh Office
comparisons remain M8.5 NOT_RUN. M7A.5 is not signed off until both native export
and required edited macro-enabled package workflows have executable evidence.
M7A.6 legacy-format removal, M7A.7 final integration and P9 extensions stay separate.

Format references: Microsoft's [Form stream](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-oforms/cb5df5d6-e090-4bf3-a328-c4edaff0c66b),
[_VBA_PROJECT write contract](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/ef7087ac-3974-4452-aab2-7dba2214d239),
[MODULE record](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/f6475ef5-0436-47c6-b31f-ea8f23810dc6),
[compression](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/4742b896-b32b-4eb0-8372-fbf01e3c65fd)
and [decompressed chunk bounds](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/ec9b2c7b-c68e-4246-8d68-7eef2458180b).
These are published format expectations, not fresh Office observations.
