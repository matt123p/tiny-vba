# Reference and corpus sources

Recorded 2026-10-26. Exact upstream revisions, URLs, roles, notice-file hashes and review state are in
[sources.lock.json](sources.lock.json). Submodules retain their upstream notices. No upstream interpreter
implementation has been incorporated into new source. Pins are provenance, not fixture approval.

MicrosoftDocs/VBA-Docs: Microsoft Corporation and contributors; documentation is CC BY 4.0 under
LICENSE, samples are MIT under LICENSE-CODE. The upstream checkout is unmodified. Follow attribution
and indicate changes if adapting prose; preserve sample notices where used. Office catalogue generation
belongs in the application forks, not this interpreter.

ReflexGrid: Milo Chen and contributors, MIT top-level notice. Use scenario/oracle data after specimen
review; do not incorporate its interpreter. VBA-JSON: Tim Hall, MIT. VBA Project Starter: J.D. Landgrebe,
MIT. Tree-sitter VBA: harumiWeb, MIT; bundled third-party fixtures may have other rights.
chapmjs/vba-samples and dtripathi18/Excel-VBA-Projects have no top-level licence file in the pinned
checkouts. Marked NOASSERTION and pending review; no redistribution or execution permission is inferred.

The original user-supplied design inputs remain in Git history; maintained contracts live under docs/.
Gridcraft AGENTS.md/CLAUDE.md supplied the conventions adapted into independently worded tiny-vba guidance;
Gridcraft's upstream licensing is MIT OR Apache-2.0. tiny-vba is dual-licensed MIT OR Apache-2.0.

Normative reference leads: Microsoft MS-VBAL, MS-OVBA and MS-CFB URLs in the supplied interpreter spec.
They are not downloaded or version-pinned by this setup. Pin the actual specification revision/hash when
selecting normative cases; do not claim current online versions from the historical design alone.

The unavailable ReflexGrid checkout has been replaced by four exact MIT
oracle-data/notice files under `oracle-sources/reflex-grid/`. Source revision,
original URL and all file hashes remain in the source lock and oracle selections.
This snapshot is data for verification; its Python is never imported or executed.
