# ADR 0012: M4 external oracle acceptance

Status: adopted for implementation, 2026-10-26.

M4 completion requires executing a growing, reviewed ReflexGrid pure-language subset with
Microsoft-verified expectations. The previous authored-only gate was insufficient. Schema 2
extends fixtures with provenance, exact source/expected/report hashes, case key and raw oracle
result, upstream Excel build/OS/date and explicitly recorded locale availability. Schema 1
remains authored/unmeasured only. A parser accepts metadata structure, not an oracle trust claim.
The reviewed selection and source-byte/hash check establish which bundled fixtures carry evidence.

Use only the MIT language case data and published expected/report data from pinned ReflexGrid;
never import its interpreter or execute Python oracle scripts. A data-only maintenance checker
may parse literal tables without executing them. Normal Rust tests use redistributed fixtures
without requiring submodules or Office. Include the upstream MIT notice.

The pinned report identifies Excel for Mac 16.114/build 26101123, macOS 15.3.1, 2026-10-15 and
oracle run commit 3758c56a39f0c8841fa43eedda0e074414fed3e3. Case/expected files are identical at
that revision and pinned revision 03a338b214df243f1bc68e50512936ddc5566c76. This is inherited
upstream-reported Microsoft evidence, not a new local observation. Locale is not recorded per
run; select binary/locale-independent cases. Exclude Windows-only, non-ASCII source and Mac
array TypeName quirks. Do not infer exact floating bits from rounded oracle text; prefer exact
representable numeric results and independently test floating boundaries separately.

Finish array parameters, ParamArray, Option Base and argument reference capture in IR 3, guarded by explicit extension features. Preserve the IR 1 and IR 2 structural targets;
new reference opcodes, array-layout metadata and intrinsic identifiers must not extend older versions silently. Publish m4-conformance-v1 instead of silently
expanding the earlier consumer profile. Canonical references remain locations; named calls must
capture array references before later arguments can resize their owner. Runtime owns coercion
corrections, and the verifier checks new shapes/features before execution.

M4 may be marked complete only after the reviewed selected oracle comparisons and the authored,
negative IR, lifecycle, native/WASI and release gates pass. Keep unselected external cases NOT_RUN.
