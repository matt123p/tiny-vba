# Implementation progress

Updated 2026-10-28. The classic-editor/debugger profile is implemented; its
historical milestone is M10. Runtime remains m5-host-v1 with verified IR 1/2/3/4/5/6.
The [remaining plan](implementation-plan.md) retains stable **M2–M8** task IDs.
**M1.x is complete** for the declared registered-type/default/collection profile.
Next: **M2.1**, class/document runtime semantics. Office objects, document events,
trust and packaging remain fork-owned.

## New M1.2/M1.3 completion — defaults, iteration and real workbook acceptance

[ADR 0022](decisions/0022-default-members-and-enumeration.md) and the
[implementation/evidence log](m1-defaults-and-collections.md) record the full
contract and stages. IR 6 preserves generic Get/Let/Set intent, chained/indexed
applications, named/omitted/native ByRef arguments and value/reference contexts.
For Each has bounded owned host snapshots, live native array aliases, typed
controls, resize locks, source/Next checkpoints, error/resume cleanup and
canonical paused element edits. Safe watches remain inert. The headless
19-crate DAG and repository ownership boundary are unchanged.

GridCraft exclusively supplies six Excel object types, real Worksheets/Sheets
roots, Count/Item/defaults and cell iteration through native commands. Original
unchanged source verifies typed loops, names/order, Range defaults/indexes,
values/errors, handle budgets, stale references, undo and trust revocation.
A worksheet UDF loops over a two-dimensional value array and recalculates when
its precedents change. Broader M2–M8 compatibility is not claimed.

Core: 316 workspace tests, strict `cargo xtask ci`, headless WASM compilation and
130 actual WASI tests pass. GridCraft: 40 native and 40 actual JavaScript/WASM
adapter tests; full CI passes all 10 gates with 2,568 test/doctest results;
1,150 optional desktop VBA-window tests, strict window Clippy and browser VBA
compilation pass. Node 24.22.1 provides the successful JS/WASM run; an older
Node 24.14.0 shutdown crash is explicitly recorded, not counted as success.

Published feature branch in both repositories: `feat/m1-defaults-and-collections`.
GridCraft commit `55d3b1bb825499086581afdbd2e07eec87bc3c29` pins tested tiny-vba
commit `d80e0393b408e0421a8df7e16667dbbbc970783b` by Git revision, with no embedded
copy or path override. Main branches are unchanged. Completed M1 is removed
from the remaining plan; M2–M8 IDs are retained for follow-up references.

## New M1.1 completion — generic core and fork-owned acceptance

Completed nominal host binding under [ADR 0021](decisions/0021-nominal-host-object-types.md),
with [resumable implementation/evidence](m1-host-references.md). The core carries
opaque library/version/type identity and IR 5 constraints, and queries a host-owned
read-only resolver. It owns no Excel definitions, object store or GridCraft dependency.
The boundary audit finds only oracle metadata references to Excel in non-test
interpreter Rust; the reviewed 19-crate DAG remains headless and acyclic.

Storage/alias/array/temporary/return checks, host writeback precommit, debugger
edits, rootless embeddings, isolated snapshots, callbacks, persistent restore,
lifetime and resource budgets are verified. Code aids preserve library scope
through qualified/chained types rather than mixing same-named classes.

GridCraft owns its four Excel type descriptors and resolver over its existing
object store. Its authored unchanged Worksheet/Excel.Range source executes real
cell writes, typed calls/results and pause/edit/resume. Wrong classes fail with
13; stale/foreign/closed handles with 91. This is registered-type compatibility,
not a complete Excel typelib or general unchanged-workbook claim.

Core validation: 304 workspace tests and 118 actual WASI tests, strict CI and
headless WASM compilation pass. Fork validation and the immutable dependency pin
are recorded in the M1 log and GridCraft's `docs/vba-m1-types.md`. M1.2/M1.3 remain
the default-member/collection work; no implicit implementation is inferred here.
GridCraft's final pin is tiny-vba `98a88c3`, with adapter commit `139016d7`:
33 native and actual JavaScript/WASM adapter tests, 2,561 full-CI test/doctest
results, 1,150 optional window tests, strict window Clippy and browser feature
build pass. Both feature branches remain separate from main.

## First M1.1 explicit-reference increment — historical checkpoint

Implemented validated native reference snapshots, ordered namespace resolution,
registered enum declarations and executable typed constants. Compiler defaults
remain reference-free. IDE discovery/compilation and isolated Immediate evaluation
use the same explicit snapshot; reset guards prevent mid-session replacement.
See [the resumable work log](m1-host-references.md) and
[ADR 0020](decisions/0020-explicit-host-reference-snapshots.md).

Nominal object declarations remain explicitly unsupported rather than erased to
generic Object. M1.1 is **in progress**, not complete: next enforce nominal type
identity/assignability across storage paths, then prove unchanged typed source in
the forks. M1.2 default members and M1.3 collections have not started. No GridCraft
changes, Office oracle runs or new external macro execution are included.

Validation: 288 workspace tests (11 new), strict CI/layer/provenance checks,
headless WASM compilation and 102 actual WASI tests pass. The resumable log records
the supported profile and the next nominal-object checkpoint.

## Plan renumbering — 2026-10-28

On user direction, removed completed work from the plan, removed old M12/M13
(native extensions/translation and AI assistance), and excluded third-party
ActiveX/OLE controls not supplied with Office applications. Standard Office
UserForms/controls remain in scope. No product capability or runtime contract
changes in this documentation update; [ADR 0019](decisions/0019-remaining-plan-scope.md)
records the scope and ordering decision.

| Current remaining milestone | Previous milestone / scope |
|---|---|
| M1 — host types/default members/collections | Old M15 |
| M2 — classes/document semantics/standard Office UserForms | Old M11, narrowed control scope |
| M3 — range/worksheet coverage | Old M16 |
| M4 — identity/values/member semantics | Old M17 |
| M5 — document events/Range-aware UDFs | Old M18 |
| M6 — remaining host integration | Unfinished old M14, including WordCraft |
| M7 — unchanged-document/package acceptance | Old M19 plus Word document package acceptance |
| M8 — expanded conformance/hardening/release | Old M9, release after feature acceptance |

Existing reports/ADRs and the historical entries below retain their old IDs.
Completed setup and old M0–M8/M10 evidence remain here and in their acceptance
reports; they are not new tasks under the reused numbers. The completed GridCraft
baseline and old M14.10 audit remain in [the integration log](gridcraft-integration.md).
Dependencies now put shared type/class contracts before consumers, range behavior
before dependent events/UDFs, host integration before package acceptance, and
release last. Conformance/fuzz/performance work runs throughout, not only at release.

Validation for this documentation-only revision: contiguous M1–M8 headings/table
and subtasks, removed scopes absent from the active plan, 274 local Markdown
links checked, and `git diff --check` pass. Required `cargo xtask ci` (277 workspace
tests), `cargo xtask wasm` and `cargo xtask wasm-run` (95 actual WASI tests) pass.
No new product behavior, Office observation or corpus execution is claimed.

## M10 completion

Preserved the user's `8dd37dc` checkpoint and completed the work on
`feat/m10-completion`. Added shared bounded source history across views, scoped
literal search, stable bookmark/navigation anchors, procedure/split editing,
classic project commands, preferences/docking/toolbars and browser file transfer.
Metadata-backed constants, Quick/Parameter Info, definition/help navigation and
Object Browser use source/host catalogues without loading libraries.

Quick Watch, scoped conditional/change watches, typed local array edits, trapping
controls, verified straight-line Set Next Statement and explicitly opted-in
isolated Immediate calls have positive, refusal and lifecycle regressions.
Optional DAP runs above the IDE with bounded stdio framing and real subprocess
acceptance. [ADR 0018](decisions/0018-m10-classic-tools-and-debugger-policy.md)
records the contracts; the [M10 report](ui/m10-implementation.md) maps every stage
to tests, final commands and limitations. No GridCraft changes or new Office
oracle observations are included. Next UI milestone: M11's class/document
prerequisite contract, before enabling a form designer Run command.

## M8 completion corrections

The latest user request makes working IntelliSense and syntax feedback part of M8. Added public
Microsoft VBA behavior references and independently authored examples to specification section 16;
four additional reference pages are cached only under ignored `.cache/evidence/vbe/`, with hashes
and factual observations tracked. No downloaded implementation, artwork or reference screenshot is used.

Implemented scoped design-time symbol/typed-receiver completion, an editor-anchored list with real
keyboard/pointer acceptance and cancellation, project signature help, and live red/underlined parser
feedback that clears on correction without moving the typing caret. Compatible same-host runs
transfer VM project state; Reset/source or host identity changes isolate it. Find Next advances and
wraps. CR/CRLF/LF display mapping retains untouched source bytes and compiler/debugger coordinates.
The Windows checkout regression now compares filesystem identity rather than short/long path spelling.

Local validation: **236 workspace tests** (17 new regressions), strict Clippy/format, layer/provenance/
corpus checks, UI tests and eleven original rendered variants, native Xvfb capture, headless WASM
compile/execution and the extended Chromium typing workflow. The 33 IDE/embedding tests run headless;
21 egui interaction tests verify actual typing, completion, error underlines/recovery and CR editing.
All six [remote CI jobs](https://github.com/matt123p/tiny-vba/actions/runs/40552813785) passed for implementation commit `e24d07a`: Windows, Linux, macOS, WASM, UI and corpus verification. The documentation closure changes no product/test code.
Advanced completion/reference/class resolution remains M10; forms/extensions/AI/fork acceptance
remain M11–M14. Earlier 219-test counts below describe the initial implementation.

## Earlier M8 completion re-audit (historical failures)

After pushing `f462959` on 2026-10-27, reran the required baseline gates and added targeted audit
probes. Locally, the 219 existing tests and UI/WASM gates pass, but all three extra probes fail: normal Run
relaunch discards Static state; Find Next stays on the current match; lone-CR source cannot set
statement breakpoints with the correct line coordinates. These are M8 defects, not M10–M14 features.
[The audit report](ui/m8-completion-audit.md) records expected/actual behaviour, source locations and
`python3 tools/audit_m8_completion.py` for reproduction. At that point full M8 completion was not confirmed.
The pushed GitHub workflow additionally fails its Windows checkout-location regression while UI,
corpus, WASM, Linux and macOS jobs pass; the audit report records the failing assertion and cached
log provenance. Resolve that CI gate as well. The earlier completion record below is historical;
the correction record above supersedes this historical failure state.

## GridCraft integration — selected spreadsheet profile complete

The 2026-10-28 desktop-window correction is complete in GridCraft `a3f77434`:
separate native `vba-window` and embedded `vba` presentations share one project/VM;
Excel-style Developer/View/Macro Edit routes replace permanent launch buttons.
Full GridCraft CI (2,553 tests), optional window UI (1,150 tests), both feature
lint/build paths and browser fallback pass. Native Windows windows and original
snapshots were inspected. The [work log](gridcraft-integration.md) records exact
evidence and remaining platform limits. Shared tiny-vba code is unchanged.

GridCraft commit `b496325b` adds the requested editor/toolbar/menu/keyboard launch,
assigned worksheet shapes and worksheet-callable Functions. Its full gate passes
2,553 tests; the full optional UI suite passes 1,143 (18 VBA cases), and all 25
adapter cases pass natively and in JavaScript-WASM. Public behavior references,
source/assignment persistence, read-only UDF policy, exact logs, launch instructions
and compatibility limits are in [the final acceptance record](gridcraft-integration.md).
tiny-vba remains a separate dependency pinned to merged product revision `ea38c3f`.
Broader M14/WordCraft/events/classes remain separate planned work.

Work is on `feat/gridcraft-integration` here and `feat/tiny-vba-integration` in the
GridCraft fork. The staged plan, decisions, exact validation and resumption notes
are in [the integration work log](gridcraft-integration.md). Office objects remain
fork-owned; no application dependency is added here. Workbook tests exposed the
root-import member-dispatch defect fixed under [ADR 0017](decisions/0017-imported-root-members.md).
The native VM regression covers exact Get/Call/Let/Set dispatch and named arguments.
Windows xtask selects `python` rather than assuming a `python3` executable.
GridCraft consumes shared revision `4d80a9584b3daeb75d589c629eff272ecd5881fe`
as a pinned Git dependency, with corpus/reference downloads explicitly opt-in.
Fourteen real-workbook tests pass natively and as JavaScript-hosted WASM; eight
shared-editor/control/source-save-discard tests pass. Light/dark/narrow/scaled
actual-paused-state snapshots were rendered and inspected. The fork's full gate,
exact commands, remaining profile limits and resumption state are recorded in
the work log. Broader M14/WordCraft/events/forms/signatures are still planned.

Continuation stage completed in GridCraft commit `8168bda4`: native typed cell
contents share the engine command/edit lifecycle, replacing JSON mutation dispatch
and removing the Text-format (`@`) write restriction. All 2,540 GridCraft gate
tests, five targeted engine cases, 15 native/JavaScript-WASM adapter cases and
eight editor/control cases pass. Shared tiny-vba code and its Git pin are unchanged.

## Post-M8 milestone allocation

On 2026-10-27, assigned the remaining UI specification work to explicit **planned** milestones
in [the post-M8 roadmap](ui/post-m8-roadmap.md): **M10** classic editing/code aids/advanced debugger
and DAP; **M11** class/document runtime and UserForm editor; **M12** native Craft extensions,
non-Microsoft enhancements and Python translation preview; **M13** optional Copilot-style assistance;
**M14** cross-repository Gridcraft/Wordcraft integration acceptance. Each has owners, dependencies,
acceptance gates and a mapping to the specification's phases/scenarios. M8's delivered baseline
and M9's next conformance/release work are unchanged. No deferred feature becomes implemented
through this planning update; no sibling repository work is started by it.

## M8 reusable editor/debugger and standalone harnesses

Implemented [ADR 0016](decisions/0016-m8-embeddable-editor.md) following the user-supplied
[public-source UI specification](ui/craft_vba_editor_debugger_ui_spec.md) and the
[implementation plan written before code](ui/m8-implementation-plan.md). `vba-ide` owns toolkit/IO-free
source state, compilation, command metadata and real debugger sessions; `vba-ui-egui` mounts the
Classic shell/panes in a caller-owned Ui; `vba-ide-app` owns native/browser source transfer,
preferences and window lifecycle. `vba-cli debug` uses the same model and accepts command scripts.
No application sibling is a dependency. The 19-node DAG enforces GUI/platform boundaries.

Delivered source editing/navigation, lexer colours, undo, bounded find/replace, actual statement
breakpoints/highlighting, bounded run/pause/continue/reset, nested into/over/out stepping, selected
stack/local inspection, validated active-frame edits, safe expression watches/Immediate and output.
Generic-host embedding tests prove one native write through break/edit/resume, exactly-once waits
and cancellation without rollback. Edited compiled source requires reset/recompile; invalid,
stale/out-of-scope and unsafe evaluation states remain explicit. Multiple views have independent
widget namespaces; injected themes never replace global context visuals/fonts/scaling.

Validation from this checkout: `cargo xtask ci` passes **219 workspace tests**, formatting, strict
all-target Clippy, the reviewed 19-node DAG, seven source pins/646 candidate hashes, M4 evidence and
UI cache/provenance/pin/font-notice checks. The 42 added tests comprise **25 IDE/embedding**, **12 UI
interaction/accessibility/layout**, **4 shell source-IO** and **1 scripted CLI debugger** regressions.
All 197 authored/reviewed language fixtures retain PASS. `cargo xtask wasm` compiles every library;
`cargo xtask wasm-run` passes native/WASI language/host/debugger smoke plus the 14 importer,
17 callback and 25 headless IDE/embedding tests. `cargo xtask ui` passes product tests and renders
seven original actual-paused-state PNG variants. The native eframe app runs/captures under Xvfb;
the Chromium harness verifies F8 Break, F5 Completed, output 10 and no page errors. Browser packaging
uses `cargo xtask web` and matching wasm-bindgen-cli 0.2.126. `git diff --check` passes.

Visual review covered light/dark, narrow, scaled, injected host theme, focused Immediate and dark
Object Browser output and corrected glyph availability, selector naming, header clipping,
per-document IDs and debugger presentation. Generated product images remain ignored in `reports/ui/`.
Downloaded public reference pages remain ignored in `.cache/evidence/vbe/`; only public URL/date/hash
observations and licensed redistribution notices are tracked. No proprietary/VBE clone code,
resource extraction, reference screenshots or Office installation was used. This does not establish
pixel identity, full C0/C1 feature coverage, another operating system or real workbook integration.
See [the explicit feature/acceptance matrix](ui/m8-acceptance.md) and [embedding guide](ui/embedding.md).

Next smallest task: fix M8-A1–A3 and add permanent regression tests, then M9 conformance/release work; separately wire these reusable panels into a fork's
real document lifecycle and verify workbook effects/undo/recalc/events without duplicating the UI.
Remaining classic features now belong to M10, forms/class support to M11, native enhancements
and translation to M12, optional AI to M13, and fork integration acceptance to M14. See the
[post-M8 roadmap](ui/post-m8-roadmap.md); all remain planned.

## M7 bounded generic callbacks and lifecycle

Implemented [ADR 0015](decisions/0015-m7-callbacks-and-lifecycle.md). The VM stays single-ownership
and non-reentrant: hosts schedule callbacks by name (`Session::schedule_callback`, FIFO, bounded by
`Limits.callbacks` default 256, name/arity/payload validated) and the VM delivers each as a fresh
top-level invocation through `run_next_callback` only after the current program is terminal. A
delivery drives to a boundary in one call (soft quota yields continue internally), shares
module/static state, bindings, output and the session instance, applies breakpoints and the
checkpoint predicate exactly like `run_slice`, and reports `counters().callbacks_run`. Host waits
inside a callback use the existing exactly-once resume path. `cancel`, `disconnect` and `host_lost`
drain the queue without executing pending callbacks; scheduling is refused once the lifecycle is
closed (cancelled/disconnected), not merely after normal completion.

Lifecycle additions: `Session::reset_state()` clears persistent/static storage, Err and captured
output, bumps the generation and prepares the original entry with its original bound arguments,
refused unless terminal, open and current; `Session::rebind_roots()` lets the host replace
root references (its own lifetime/generation model) under the same launch validation. A stale root
at delivery surfaces the host's reference error 91 with no repeated dispatch. Generic-host
reentrancy, drain, budget, failure-recovery, debugger-pause-in-callback and stale-reference tests
live in crates/vba-vm/tests/callbacks.rs. Event ordering, `Application.EnableEvents`, document
events, UDF evaluation and any Office-shaped catalogue remain fork-owned over this generic queue.

Verification from this checkout: cargo xtask ci passes formatting, strict all-target Clippy,
**177 integration tests**, the 16-node workspace DAG, seven source pins/646 candidate hashes and
M4 evidence checks. All 197 authored/reviewed language fixtures retain PASS. cargo xtask wasm
compiles all libraries; cargo xtask wasm-run passes native/WASI language/host/debugger acceptance,
all **14 importer tests** and all **17 callback/lifecycle tests** on WASI. git diff --check passes.

M7 acceptance audit corrections (2026-10-27): closing an already terminal invocation now closes
its lifecycle permanently; invalid delivery quota/source/host/lifecycle requests preserve queue
and outcome; reset prepares the original entry again with original bound arguments and fresh
static/output/Err state. Reset retains queued callbacks, instance/debugger identities and lifetime
work budgets and cannot reopen a closed session. Added native/WASI regressions, shared cumulative
queue payload/metadata accounting, upfront argument conversion, owned top-level ByRef arguments,
and clean frame-start resource failure. Corrected the plan's unsupported claim of fork event-order
acceptance: Office event/workbook gates remain separate and unverified by this checkout.

Next: the required reusable graphical editor/debugger slice on these VM APIs; M8 completes its
acceptance gate.

## M6 project source decoding

Implemented [ADR 0014](decisions/0014-m6-project-decoding.md) and the
[bounded import contract](project-import.md) in vba-project, finishing the incoming decoder slice.
The read-only API consumes controlled vbaProject.bin CFB bytes and returns project information,
registered/project/control references and standard/class/document modules with distinct module/
stream/code-name identities, source offsets, flags, descriptions and preserved source/attributes.
Caller-owned SourceId allocation preserves diagnostic locations without inventing global identities.
Strict code-page conversion retains the encoding identity; Unicode metadata/name maps must agree.

Protection envelopes are checked only to refuse restricted/password/hidden projects. Unknown
encodings/records and designers fail explicitly. Corrected PROJECTlk's classification to unsupported
ActiveX licence metadata, rather than protection. Added complete CFB-sector validation after a
truncation regression showed the underlying library accepts missing final padding. Hard and
caller-lowered bounds cover input/streams/decompression/output/counts/depth; metered CFB reading
and strict cycle checks bound hostile metadata work. Cache consistency is explicitly NotChecked;
P-code, trust decisions, compilation, execution and binary writing remain absent from this API.

Reviewed/pinned cfb 0.14.0 (MIT) and encoding_rs 0.8.42 ((Apache-2.0 OR MIT) AND BSD-3-Clause).
The dependency checker now distinguishes reviewed external pins from workspace DAG nodes; only
vba-project gains those normal dependencies. No interpreter implementation or proprietary fixture
was imported. Fourteen authored integration tests build controlled binary records in memory and
cover exact metadata/source results, Unicode identities and diagnostic spans, representative
encodings, optional records, empty projects, protected/unsupported/corrupt inputs, every directory
truncation, selected header mutations, CFB tree/FAT cycles, compression vectors and all limits.
Source containing Auto_Open/unknown host calls remains inert; the crate has no VM/host/compiler edge.

Verification from this checkout: cargo xtask ci passes formatting, strict all-target Clippy,
**160 integration tests**, the 16-node workspace DAG, seven source pins/646 candidate hashes and
M4 evidence checks. All 197 authored/reviewed language fixtures retain their existing PASS results.
cargo xtask wasm compiles all libraries for wasm32-unknown-unknown. cargo xtask wasm-run passes
native/WASI language/host/debugger acceptance and now executes the same **14 importer tests on
WASI**, with no preopened directories. git diff --check passes. No separate MSRV, macOS/Windows,
external-workbook import, browser runtime or Office observation is claimed.

M7 delivered the bounded generic callback queue, reset ownership and lifecycle cleanup under
[ADR 0015](decisions/0015-m7-callbacks-and-lifecycle.md).

## M5 native host integration contract

Implemented [ADR 0013](decisions/0013-m5-host-contract.md). `vba-foreign` owns protocol version 1
(`API_VERSION`, `ProtocolVersion`, `check_protocol`); the foreign session and the VM both refuse an
incompatible host protocol before any dispatch, with a double-check regression removed from the
invoke path. `Import`/`validate_imports` bound generic source imports: ≤64 entries, ASCII identifier
names, case-insensitive uniqueness, Call/Get only. The compiler accepts them through
`CompileOptions::imports` and lowers each to a hidden IR 4 `HostRoot` slot plus a fixed member
call/get; imports resolve only as call targets and cannot be written or captured.

`vba-ir` gained `IR4_VERSION`, the `host-roots-v4` feature and `SlotRole::HostRoot` with structural
rules (IR 4 only, Object, non-array, hidden, result-free, read-only under `RefLocal`); older IR 1/2/3
targets and `Capabilities::ir3_structural()` stay closed. The VM's `HostBinding` gained case-folded
named roots (cap 64, host-identity matching); missing roots refuse `Session::new`, mismatched-host
roots refuse launch, and a stale root surfaces the host's own reference error (91) with no repeated
dispatch. `HostBinding::for_program` derives the whole site allowlist from a verified program, and
`Session::limits()` exposes budgets to embedding drivers.

The runtime profile is now `m5-host-v1`; the compiler's structural default is `ir4-structural` while
IR 1/2/3 remain selectable and verified. `vba-values`/`vba-debug` publish `API_VERSION` consts.
`vba-testkit` supplies the reusable bounded bridge driver (`run_to_boundary`, `statement`), and
vba-vm/tests/embedding.rs proves the fork integration shape in-repo: an import-driven FillCells-style
call against a scripted fake host, breakpoint/inspect/edit/resume with exactly one dispatch, and
hidden-root refusal cases. Architecture DAG edges (semantics→foreign, compiler→foreign, testkit
debug/ir/vm) are reviewed in tools/architecture.json.

Verification from this checkout: cargo xtask ci passes formatting, strict all-target Clippy,
**146 integration tests**, the 16-crate acyclic DAG and seven pins/646 hashes. All 197 authored and
reviewed fixtures still assert PASS under the renamed profile, and the same tests pass in release.
cargo xtask wasm builds every library and cargo xtask wasm-run passes the identical native/WASI
acceptance. Not claimed: the fork-owned FillCells proof with a real Office layer remains Gridcraft's
gate, as ADR 0013 requires.

## M4 external acceptance completion

The earlier authored-only completion claim did not establish the supplied M4 gate. Corrected it
by reviewing and adopting **153 MIT ReflexGrid pure-language cases** with published Microsoft
Excel Mac expectations under [ADR 0012](decisions/0012-m4-oracle-acceptance.md). All 153 now PASS,
alongside the original **44 authored PASS** scenarios. [The acceptance report](m4-oracle-report.md)
records every selected case and raw expected result, source/expected/report hashes and provenance.

The upstream report names Excel for Mac 16.114/build 26101123, macOS 15.3.1 and 2026-10-15.
Fetched its recorded run revision read-only and verified that the case/expected files are
byte-identical at the pinned source revision. Locale is explicitly not recorded per run.
This is inherited upstream-reported Microsoft evidence, not a fresh local Office run. Excluded
Windows-only/locale/non-ASCII/Mac naming cases and all unselected specimens remain NOT_RUN.
Only fixture literals and selected VBA snippets were consumed; no upstream interpreter or Python
script ran. The exact upstream Err-state probe is preserved in a small fixture adapter.

Oracle cases corrected Byte widths/overflow, Currency addition, Single division, Boolean arithmetic,
scalar text precision/exponents, 0/0 and power overflow, Empty/string comparison, eager IIf,
empty-array bounds and physical-line label parsing. Added required typed whole-array ByRef,
ParamArray packing (including empty 0..-1), Option Base and qualified VBA.Array, and canonical named
array-reference capture. Typed whole-array assignments, debugger edits and host patches preserve
element kinds. Captured references retain source evaluation order and expire after resizing.

IR 3 carries the new array layouts/parameters, reference captures and intrinsic identifiers;
IR 1/2 structural capabilities and supported-version sets are preserved explicitly. The compiler
selects the minimum required version. Schema 2 requires complete external metadata and matching
raw/typed results, while Schema 1 remains authored/unmeasured. CI verifies the immutable selection,
license notice, source adaptations and identical native/WASI case bundle, with no corpus checkout
required for ordinary tests.

Final verification: cargo xtask ci (formatting, strict Clippy, **133 integration tests**, 16-crate
DAG, seven pins/646 hashes and M4 evidence verification); the same 133 tests pass in release.
All libraries compile for wasm32-unknown-unknown. Native and WASI execute all 153 inherited oracle
cases and expanded array/argument tests alongside every M3 debugger/host/cancellation/isolation
proof. No remote cross-platform or separate MSRV run is claimed. Remaining language limitations
are explicit in [the current contract](m4-language.md); no complete-VBA or full-corpus claim is made.

Next: M5 public API/bridge versioning, with graphical IDE work under M8. M9 grows oracle coverage
and obtains fresh observations; it no longer defers the original M4 external-subset gate.

## Historical M4 authored-subset completion

Implemented the adopted [IR 2 language extension](decisions/0011-m4-language-extension.md) across
syntax, semantics, compiler, verifier, values, runtime and explicit-frame VM. The original IR 1
structural capabilities remain available; the compiler's new structural target is ir2-structural,
and execution explicitly uses m4-language-v1. New instructions/assignment modes require IR 2 and
feature gates. No Office or UI dependency was introduced.

Added centralized coercions for all scalar widths, exact LongLong integer operations, scaled
Currency arithmetic/decimal-boundary conversion, Date serial operations, Empty/Null rules,
Missing/Nothing/CVErr distinctions, UTF-16 string operations and deterministic intrinsic registry.
Optional/default/named/omitted calls preserve source-order evaluation. Canonical ByRef locals,
module/static slots and array cells share locations; evaluated arguments use private typed temporaries.
Argument capture uses binding rules, preserving Object/Nothing without changing ordinary Let/Set.

Added bounded fixed/dynamic arrays with signed bounds, column-major layout, Preserve/Erase,
generation-checked cells, representation checks, copy budgets and allocation precharges. Module
and Static local storage survives calls; explicit project state export/restore checks revision and
exact source identity, while fresh sessions reset. Debug.Print captures bounded UTF-16 records.

Added per-frame On Error modes/active faults, Err/Erl, retry/next/label Resume, exception edges,
structured-statement continuation metadata and caller propagation/unwinding. Err.Clear leaves an
active handler active. Host errors after waits enter VBA handlers without reinvocation. Break-on-error
continues into a handler once. Engine/resource/protocol failures are not swallowed. A stale array
alias originally escaped while constructing its failure snapshot; the minimized regression now
produces runtime error 9 and explicitly lists unavailable locals.

Verification from this checkout: cargo xtask ci passes formatting, strict all-target Clippy,
**119 integration tests**, 16-crate acyclic layering and seven pins/646 hashes. The same 119 tests
pass in release. cargo xtask wasm builds every library; cargo xtask wasm-run passes identical native
and WASI acceptance, including arrays Long(44), static/module state Long(23), restored state Long(47),
Currency/Date/exact LongLong, optional/named calls and handler Resume Long(12), alongside every M3
host/debugger/cancellation/isolation check. git diff --check passes.

The 20 original scenarios now assert **20 PASS / 0 UNSUPPORTED**. Added 24 independently authored,
uniquely identified M4 scenarios tagged m4-language-v1: **24 PASS**. These are reasoned expectations,
not Microsoft oracle observations; external corpus execution remains NOT_RUN. Profile limitations
are explicit in the M4 contract (including locale/codepage behavior, nested/whole-array parameters,
class/default-member binding and selected argument combinations). This completes the reviewed M4
gate without claiming the entire supplied VBA wishlist. No remote cross-platform or separate MSRV
run is claimed. Next: M5 API/versioned bridge contract and the required graphical IDE slice.

## M4 scalar/runner hardening and corrected validation

Continued the incoming M4 first slice after closing the M3 instance-isolation audit. Reproduced and
fixed two correctness regressions: Double-to-Long conversion could wrap past the 32-bit destination,
and the scenario runner labeled malformed/invalid source UNSUPPORTED with success status. Resource/
engine failures also no longer exit successfully. Float/Date judgments compare exact payload bits;
UTF-16 report text preserves lone surrogates and escapes line breaks.

Published m4-scalars-v1 for the expanded numeric matrix instead of mutating m3-core-v1 in place.
Added Double negation/mixed comparisons and native/WASI boundary assertions. Existing M3-tagged
programs must be recompiled; IR 1 is unchanged. Source/engine failures are classified explicitly,
and known unimplemented intrinsic names retain UNSUPPORTED without hiding unknown-name errors.

Found a verification-root bug: a cached xtask binary could run an older checkout embedded at build
time. Historical pass claims alone were therefore insufficient. xtask now finds and logs the current
checkout at runtime, has a failing-before/fixed-after cross-checkout regression, and honors custom
WASM target paths. Re-ran the current code, not the cached checkout: 89 tests pass in debug and
release; cargo xtask ci passes format/strict Clippy/layers and seven pins/646 hashes; cargo xtask wasm
passes all libraries. An isolated CARGO_TARGET_DIR native/WASI smoke agrees on Long(42), Long(18),
edited Long(41), one invocation/two polls, cancellation/session isolation, Double(3.5), Boolean(true)
and Long overflow error 6. No remote cross-platform, separate MSRV or Office result is asserted.

The 20 scenario assertions remain 17 PASS / 3 UNSUPPORTED. [ADR 0010](decisions/0010-m4-profile-and-verification-hardening.md)
records profile/exit/report/gate contracts. M4 is still in progress: next is the handler-capable IR
specification before On Error implementation, alongside the remaining arrays/arguments/builtins.

## M4 first slice: assertion runner and numeric widths

Implemented `conformance run <scenario>`: the CLI parses the manifest, compiles only the embedded
VBA body with the VM's actual capability set, executes host-free and compares the outcome with the
manifest's typed expectation through new `vba-conformance` judgment APIs (`judge_value`,
`judge_error`, `describe_value`). It prints `RESULT <id>: <STATUS> (<detail>)`; MISMATCH exits 1,
PASS/UNSUPPORTED exit 0, invalid fixtures exit 2. `conformance validate` still reports NOT_RUN and
never executes.

Enabled real division and Double widths under existing IR 1 instructions through an explicit
`m3-core-v1` capability update ([ADR 0009](decisions/0009-m4-runner-and-numeric-widths.md)): `/`
always produces Double (zero divisor 11, non-finite 6), Add/Subtract/Multiply promote to Double
only with a Double operand, Double declarations/assignments widen exactly, narrowing rounds half to
even with overflow 6 out of range, and decimal/exponent literals bind as finite Doubles. The
integer operator matrix and its error numbers are unchanged. The M3 CLI negative case `Answer=1/2`
became correct behavior (Double 0.5 narrowed to Long 0), so the unsupported-operator case uses `^`.

The 20-fixture inventory moved from 20 NOT_RUN to an asserted 17 PASS plus 3 explicit UNSUPPORTED
(cverr: builtins; named: named/omitted direct arguments; optional-missing: optional
parameters/IsMissing). External corpora remain NOT_RUN; no oracle observation is claimed.

Validation: cargo xtask ci (81 integration tests, fmt, strict Clippy, layers, corpus pins),
cargo test --workspace --release --locked, cargo xtask wasm and cargo xtask wasm-run (native and
WASI smoke pass with unchanged M3 results). No dependency graph, IR version or scenario-schema
change. Remaining M4 work: the handler-capable IR contract for On Error/Resume/Err/Erl before
implementing handlers; arrays; optional/named direct-call binding; builtins; Currency/Date/Byte/
LongLong/Single arithmetic; persistent/static state.

Next smallest task: specify the versioned handler-capable IR extension (frame handler state,
exceptional edges, resumption metadata, nested-handler tests) before enabling On Error constructs.

## M2/M3 acceptance audit and session-isolation correction

On 2026-10-26, reviewed every adopted M2 gate against the binder/compiler/IR implementation and
acceptance tests. M2's limited IR 1 scope is complete; no M2 code change was needed. The
[gate-to-evidence audit](m2-m3-acceptance-audit.md) records source maps, deterministic goldens,
negative IR cases, bounded verification and valid self-loop evidence.

Continued M3 with a minimized execution-reuse regression. It failed before the correction: a stale
handle from a dropped/cancelled session could read/edit a replacement session with the same client
label/frame/pause generation. A retained controller's breakpoints and step target had the same
label-reuse problem. Added opaque, non-reused SessionInstance identities and used them for handles,
location ownership, breakpoints and stepping, preserving the client's routing label. ADR 0008 records
the runtime API refinement; no IR version or instruction change is involved.

Verification: cargo xtask ci (75 integration tests, fmt/strict Clippy/layers/corpus integrity),
release workspace tests (75), cargo xtask wasm and cargo xtask wasm-run. The same replacement-session
regression runs natively and in WASI alongside typed results, host waits, edits and cancellation.
No separate MSRV, remote cross-platform or Office observation is claimed. M4 remains next; the
scenario assertion runner, broader coercions, On Error and graphical UI remain separate work.

## M3 completion

Implemented vba-runtime's documented Integer/Long/Boolean/String/Variant/Object assignment and
operator subset with exact scalar tags, overflow/division errors and explicit Unsupported results.
The new m3-core-v1 capability profile is rechecked at VM launch using borrowed immutable IR; the
structural M2 profile cannot silently enable runtime operations. Added CLI run and generic Object-
local member Call/Get/Let/Set binding/lowering without Office globals or default-member inference.

Implemented explicit frames, typed owned locations and canonical direct ByRef aliases, frame-local
operand stacks, quotas/hard work and copy budgets, saved call/Invoke fault PCs, fresh checkpoint
visits and single-use continue permits. Added source breakpoints, into/over/out stepping, generation-
checked read-only inspection, validated active-frame edits and source staleness checks. Stop and
optional read-only fault pauses continue without replay. Bounded snapshots explicitly report local
truncation. Cancellation, host disconnect/loss, recursion/storage/stack/payload limits are explicit.

Connected generic native host dispatch with per-site permissions, exactly-once wait/resume continuations
and cancellation. vba-foreign's checked APIs validate return payload and all typed patches before
its sole commit loop. Foreign alias snapshots are refused; direct ByRef remains canonical. Tests
preserve successful direct alias/host effects after errors and distinguish VBA numbers from engine
failures. The VM still has no compiler/debug/UI dependency and no ambient IO.

Validation: cargo xtask ci, cargo xtask wasm, cargo xtask wasm-run and release workspace tests.
The workspace has 74 integration tests: prior 54 plus 3 runtime, 10 VM execution, 6 VM host and
1 additional CLI execution case. Native and actual wasm32-wasip1 execution run the same Rust
m3_smoke acceptance under Node 18.19.1/Rust 1.99.1: Long(42), If/For/calls/ByRef Long(18), host
pause/edit/resume Long(41), one invoke/two polls, stale handles, host-token and infinite-loop
cancellation. Core libraries also compile for wasm32-unknown-unknown; CI now runs the shared
native/WASI smoke gate using Node 22. Corpus integrity remains seven pins/646 hashes. No Office
oracle, browser UI, remote cross-platform run or separate MSRV test is claimed by these local checks.

See [runtime contract](runtime.md) and [ADR 0007](decisions/0007-m3-execution-and-debugger.md).
The original M0 fixture manifests and external execution inventory remain NOT_RUN; there is still
no fixture assertion runner. M3's explicit runtime examples/tests have actual execution evidence.
The required graphical editor/debugger, DAP and fork-owned Office integration remain later gates.

Next smallest task: superseded — the M4 runner and numeric widths are implemented above; see the
M4 section for the current next task.

## Pre-push verification of M0–M2 (historical stage record)

On 2026-10-26, reran `cargo xtask ci` (formatting, warning-free all-target Clippy, all 54
workspace integration tests, layer checks and available corpus integrity), `cargo xtask wasm`
(all 14 libraries), and `cargo test --workspace --release --locked` (the same 54 tests in release).
All passed. Strict corpus verification checked all seven pins and 646/646 hashes; the opt-in
five-file grammar_corpus example matched all reviewed syntax classifications. `git diff --check`
passed. Corrected the fixture-schema documentation's stale claim that compile is unavailable.
These checks cover implemented M0–M2 behavior. VM/Office execution remains unavailable; WASM
runtime, other operating systems and separate MSRV testing are not claimed by these local runs.

### Cross-platform checkout correction

The first pushed M1/M2 commit passed Linux/macOS, WASM and corpus CI; Windows exposed Git's
CRLF checkout conversion changing source byte offsets and golden line endings. Added root
`.gitattributes` with `text=auto eol=lf` so tracked text fixtures retain their reviewed bytes on
all platforms while auto-detected binaries remain untouched. Explicit CRLF/lone-CR parser
inputs stay covered by authored tests. A fresh `core.autocrlf=true` checkout preserved LF fixture
bytes and passed `cargo xtask ci`, including all 54 tests, without corpus downloads. Cross-platform
CI is rerun after this fix; source-map tests stay exact rather than normalizing parser input.

## M2 completion (historical stage record)

Implemented ordered module/procedure scopes and case-insensitive lookup, Option Explicit, scalar
parameters/locals/Function result slots, implicit Variant locals, literal constants, positional ByVal
and exact-type local ByRef binding. Unsupported persistent storage, class/foreign binding, deeper
argument and constant/numeric forms fail explicitly with source spans. Binding preflights AST depth
and budgets source/token/node payload and constant substitution before copying.

Added vba-compiler for parse/bind orchestration and checkpoint-based lowering, including If/ElseIf,
restricted positive integral-step For, direct nested calls, Set/Let modes, Exit and Stop. Retained
same-line/control statement identities and full statement tables after unreachable code removal.
Constants are deduplicated by exact tag/payload in first emitted-use order. Parameters/result/user/
scratch slots have explicit roles. New reviewed edges preserve IR -> diagnostics/values only;
semantics now shares ScalarKind from values. No external Rust dependency or corpus execution added.

Implemented every IR 1 instruction's structural verification, immutable opaque VerifiedProgram,
launch/signature/slot/call/foreign/source ownership checks, V/Location stack simulation, exact maxima,
iterative reachability, bounded verification and exact-tag deterministic disassembly. M2's explicit
ir1-structural capability target is not a runtime implementation. Unknown versions/features/op
capabilities, malformed IR, unreachable blocks and over-budget input cannot create VerifiedProgram.

CLI check/compile/disasm now work. Arithmetic's full reviewed golden matches the nine-instruction
IR 1 example with Integer 6/7, Long result and max_stack=3. M2Control.bas independently demonstrates
If/For/nested calls and ByRef lowering. Execution remains unavailable and all M0/external scenario
execution results remain NOT_RUN. See [compiler contract](compiler.md) and
[ADR 0006](decisions/0006-m2-binding-and-verification.md) for supported forms, budgets and limitations.

Validation: `cargo xtask ci` and `cargo xtask wasm`; compiler/IR/binder/CLI tests and documented
compile/disasm examples. The workspace now has 54 integration tests (8 compiler, 9 IR, 4 binder,
7 CLI, 13 frontend, and 13 existing M0 library tests). Native tests include malformed-IR mutations,
all instruction stack shapes, each verifier budget dimension, exact source diagnostics and bounded
constant amplification. Library WASM evidence is compilation only. No Office oracle, runtime/WASM
execution, macOS/Windows or separate MSRV run is claimed. Corpus integrity remains seven pins and
646/646 candidate hashes; the M1 five-file syntax report is unchanged.

Next smallest task: M3 VM explicit frames/typed local storage/checkpoint dispatcher with hard/slice
budgets and actual capability checks; execute Arithmetic.Answer and assert exact Long(42). Then
implement direct alias calls, debugger/host continuations and native/WASM pause/resume proofs.

## M1 completion (historical stage record)

Implemented a pure-Rust bounded lexer/parser in vba-syntax with retained source and all trivia tokens,
exact checked spans, spelling-preserving names and distinct same-line statement identities. The AST
covers scalar declarations/parameters, Sub/Function, module/class attributes/options, expressions,
assignment/Let/Set, If/ElseIf/Else, numeric For/Next, explicit/implicit calls, Exit and Stop. Named,
omitted and optional syntax is preserved without claiming binding or IR 1 execution support.
SourceFile coordinates now handle LF/CRLF/lone CR consistently. Parser recovery never reports syntax
success, and source/token/nesting/tree/argument/diagnostic limits bound adversarial input.

Added `vba-cli parse <source.bas|source.cls>` with bounded UTF-8 reads, exact source diagnostics and
nonzero rejection. Both self-authored examples and all 20 M0 VBA bodies parse, as do the new class
metadata/control fixtures. All execution results stay NOT_RUN; no binder, IR verifier, VM or UI exists.
The development CLI -> syntax edge is recorded in the reviewed DAG; no dependency was added to syntax.
See [frontend support and limits](frontend.md) and [ADR 0005](decisions/0005-m1-frontend.md).

Reviewed five directly authored MIT tree-sitter-vba examples for explicit syntax-only ingestion.
The two nominally valid modules are UNSUPPORTED (Select Case/property procedures); all three broken
examples are REJECTED at source diagnostics. Five classifications matched review; zero executable
compatibility passes are claimed. [The report](m1-corpus-report.md) records pins, hashes, rights,
capability/profile restrictions and reproducible invocation. Normal tests need no corpus checkout.
No upstream implementation, generated parser, downloaded script or macro was executed or copied.

Validation: `cargo xtask ci`, `cargo xtask wasm`, the opt-in grammar_corpus example and documented
parse commands passed. Thirty-two integration tests now cover infrastructure/frontend/CLI, including
13 frontend tests for preservation, exact source spans, control metadata, call/precedence distinctions,
unsupported/malformed input, nested control statement identities and token/tree/recovery/identifier limits.
Member identifiers enforce the same 128-byte limit as standalone identifiers. The deterministic malformed/truncated
stress check is not a fuzzing campaign. Corpus integrity remains seven pins and 646/646 hashes.
WASM evidence is compilation only; macOS/Windows and a separate MSRV run were not executed locally.

Next smallest task: M2 bound module/procedure scopes and Option Explicit/local/result/parameter binding
against the M1 AST, then the IR 1 program layouts, verifier/disassembler and real compiler/lowering crate.
Numeric semantic legality, property/Select Case/array syntax and the broader grammar remain unsupported
as documented; M1 completion covers its reviewed subset, not the full supplied grammar wishlist.

## Bytecode/VM contract prepared before M1 (historical stage record)

Authored the adopted IR 1 bytecode/VM implementation specification and ADR 0004. The document
specifies exact instruction operands/stack effects, program/constants/signatures/slots, source
checkpoint mapping, canonical direct ByRef, restricted M0 host adaptation and single-use host
continuations. It defines verifier/disassembly rules, explicit execution states and errors,
checkpoint visit permits, stepping/inspection generations, resource defaults, lowering examples
and M2/M3 positive/negative acceptance obligations. A planned upper-layer vba-compiler owns
compilation/lowering when implemented; no package/dependency graph change is made now.

Updated architecture, debugger, plan, compatibility, development entry points and relevant skills.
This is documentation only: no instruction enum, verifier, compiler, VM, debugger or runtime
behavior was added. All M0 VBA fixtures and external corpus candidates retain their prior status.
M2/M3 remain unimplemented. M1's bounded lexer remains the next smallest implementation task.

Validation: documentation links and instruction/reference consistency reviewed; `cargo xtask ci`
and `cargo xtask wasm` passed after the documentation change. Existing 16 integration tests still
cover M0 only; no bytecode acceptance case is claimed to have run.

## M0 completion

- Implemented SourceId, per-source StatementId, bounded SourceFile, checked UTF-8 byte spans and
  attached diagnostics, including Unicode/CRLF/identity/bounds tests.
- Implemented scalar Value states/widths, scaled Currency/serial Date, exact UTF-16 strings and
  bounded, invalidatable owner-scoped locations. No runtime coercion or arrays yet.
- Implemented bounded schema-1 fixture parser/validator and 20 independently authored MIT pure-language
  cases with typed expectations, rationale, requirements and explicit unmeasured evidence. All remain NOT_RUN.
- Added `vba-cli conformance validate <scenario>`; CLI tests validate all 20 fixtures and reject malformed
  input. Execution commands still fail explicitly; fixture validation is not VBA syntax validation.
- Proved generic Call/Get/Let/Set, omitted/named/ByRef snapshots and ordered delayed writebacks against
  a deterministic scripted host. Tested errors, repeated waits without replay, cancel, stale locations/
  references, wrong host identity, invalid/non-ByRef patches, alias order and resource limits.
- Recorded adopted source/scenario/foreign prototype contracts, lifecycle requirements and M4/M5
  limitations in ADR 0003; updated architecture, reviewed dependency graph, layer checker and focused skills.
  The development CLI's conformance dependency is explicitly allowed; shipping libraries remain isolated.
- No third-party Rust dependencies, downloaded macro execution, Office observations, parser, VM or UI added.

Verification: `cargo xtask ci` passed (format, warning-free Clippy, workspace tests, layers and corpus
integrity); `cargo xtask wasm` passed for all 13 libraries. Sixteen Rust integration tests cover the
M0 infrastructure and CLI, including validation of all 20 fixtures. Corpus integrity remains seven
pins and 646/646 candidate hashes; this is not a corpus compatibility result. WASM runtime behavior,
Office compatibility, macOS/Windows execution and a separate MSRV run were not tested locally.

Next smallest task: M1 bounded lexer for Arithmetic.bas with exact spans and positive/negative token
fixtures, including comments, CRLF/continuations and colon-separated statements. Then implement the
source-preserving parser and record exact unsupported syntax; M0's cases stay NOT_RUN until execution exists.

## Setup history (pre-M0)


Created a standalone Cargo workspace with 13 library crates, vba-cli and xtask, each with src/.
Added current architecture, implementation plan, decisions, adapted AGENTS/CLAUDE and repository-local skill.
Preserved the supplied interpreter spec and copied the two workspace design documents unchanged.
Added test tier/fixture/example/fuzz/report locations and native/WASM/corpus CI jobs.

Cloned six documented test repositories and Microsoft VBA-Docs as pinned submodules.
Source lock tracks notices/revisions; candidate inventory tracks SHA-256, size and NOT_RUN status.
No downloaded macros, oracle scripts, interpreters or workbook files have been executed.
Contextures is tracked as a manual download source; no Word corpus was identified in the supplied docs.
At setup, no parser, binder, runtime, VM, fake-host implementation, project decoder or scenario runner existed.

Scope updated on 2026-10-26: this repository owns the reusable egui VBA editor/debugger and
standalone native/web IDE. ADR 0002 and the UI contract define ownership, Gridcraft design
alignment and the required visual acceptance gate. No UI packages or working UI exist yet.
The enforced Cargo graph is unchanged because this change defines scope and implementation gates.

Added seven shared Codex/OpenCode design skills under .agents/skills: development routing,
architecture, language semantics, debugger, editor UI, host integration and conformance.
AGENTS.md routes design/implementation work through relevant skills; docs/agent-skills.md records
shared discovery and usage. These are agent guidance, not new engine/UI implementation.
Validation: all seven skill names, descriptions, bodies and local routing links passed a
dependency-free structural check; bundled quick_validate.py could not run because PyYAML is absent.
`cargo xtask ci` and `cargo xtask wasm` passed after the skill changes. Discovery paths were
verified against official documentation; live Codex/OpenCode skill activation was not exercised.

The setup's next tasks (source identities, M0 fixture validation and 20 independent cases) are now
complete as recorded above. See the plan for remaining milestones.

## Setup validation

- `cargo xtask ci`: passed on Linux with Rust 1.99.1 (fmt, Clippy, workspace tests, layers and available corpus hashes).
- Two CLI integration tests passed; these check explicit refusal and help, not VBA execution.
- `cargo xtask wasm`: all 13 libraries compiled for wasm32-unknown-unknown.
- Strict corpus verification: seven Git pins and 646 candidate file hashes checked; no macros executed.
- Current documentation/skill links and unchanged source document copies checked.
- macOS/Windows jobs are configured, not run locally. No MSRV run, fuzzing or VBA runtime tests yet.
