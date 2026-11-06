---
name: tiny-vba-conformance
description: Create or review tiny-vba test scenarios, external corpus use, oracle evidence, compatibility reports and release claims. Use for fixture adoption, semantic evidence or measuring implemented behaviour.
---

# Conformance and evidence

Read [methodology](../../../docs/conformance-methodology.md),
[compatibility matrix](../../../docs/compatibility-matrix.md) and
[current progress](../../../docs/implementation-progress.md). Before external corpus work, read
[corpus policy](../../../tests/corpora/README.md), the
[source lock](../../../references/sources.lock.json) and
[inventory](../../../tests/corpora/inventory.json).

M0 validates [schema 1](../../../docs/scenario-format.md) self-authored fixtures only.
`vba-cli conformance validate` reports VALID_FIXTURE and NOT_RUN; it does not parse/execute VBA.
`vba-cli conformance run` (M4,
[ADR 0009](../../../docs/decisions/0009-m4-runner-and-numeric-widths.md)) executes a validated
fixture and compares its typed expectation, printing RESULT with the shared status vocabulary
(MISMATCH exits 1). The asserted M0 inventory is 17 PASS / 3 UNSUPPORTED.
External/oracle metadata needs a reviewed schema extension. Keep the 20 fixture identities unique
and matching filenames; update owning integration tests when changing that suite.

## Select evidence deliberately

Keep syntax/binding, language runtime, generic foreign calls, source import, debugger and UI evidence
separate. Office model, real workbook mutation, package round trips and host trust need fork tests.
A fixture pin/hash proves integrity, not permission to execute or behavioural compatibility.

Prefer independently authored deterministic cases. For external material establish provenance/licence,
required capabilities, host profile and justified expected outcomes before adoption. Never edit upstream
submodules or run downloaded scripts/macros merely to obtain a result. Do not reuse other interpreters'
implementation code. Observe Office only as a permitted black box; never inspect proprietary binaries.

Record source/version/hash, scenario identity, exact Office build/OS/locale and expected types, values,
error numbers, output and side effects. Preserve array bounds and distinguish formulas from results.
Keep Mac and Windows profiles separate. Document inference; never invent an oracle observation or
silently rewrite a golden to match the engine.

## Report what actually ran

Use PASS, MISMATCH, UNSUPPORTED, BLOCKED_BY_POLICY, INVALID_FIXTURE, CRASH, TIMEOUT and NOT_RUN.
Do not turn unavailable or unsupported execution into a pass. Report suite/version and denominators;
compilation, candidate file counts and CLI help tests establish no executable VBA semantics.

Put Rust integration tests under the owning crate's tests/; root tests/ holds shared data. Cover
negative/error/budget paths and minimized crash regressions where relevant. Execution changes need
native/WASM runtime evidence when available; UI changes need rendered/interaction evidence as well.
Run the appropriate checks and repository CI/WASM gates; corpus integrity is a separate check.
Update compatibility/progress from actual outcomes, including unavailable checks and the next smallest
task. Never claim full Office compatibility from a narrow generic-host or visual demonstration.

M1's [reviewed grammar allowlist](../../../docs/m1-corpus-report.md) permits syntax-only ingestion
of five directly authored tree-sitter-vba examples, not execution or third-party subtree adoption.
All 20 M0 bodies parse, but execution statuses and the corpus inventory remain NOT_RUN.

M2 VERIFIED/diagnostic disassembly is structural compilation evidence only; see
[compiler tests and boundaries](../../../docs/compiler.md). Do not mark any execution scenario PASS
from compilation. All M0/external execution statuses remain NOT_RUN until a real VM runs assertions.

## Implemented M3 contract

Read [runtime](../../../docs/runtime.md) and [ADR 0007](../../../docs/decisions/0007-m3-execution-and-debugger.md).
The m3-core-v1 runtime is intentionally narrower than structural IR. Preserve explicit frame/alias
storage, one-visit checkpoint permits, saved fault PCs, generation checks and nonreplayed continuations.
Use vba-runtime for operations/edits. VM host dispatch must use checked precommit return/writeback
validation and explicit per-site permissions; foreign aliases remain unsupported. Run cargo xtask ci,
wasm and wasm-run (native plus actual WASI execution). Runtime example results do not mark unrun M0
or external scenario assertions PASS. M8/M10 graphical UI and optional DAP have authored interaction,
native/browser and protocol evidence in the M10 acceptance map. These are not fresh Office oracle
observations or Excel workbook compatibility proof. Current handlers belong to the M4 profile.

[ADR 0010](../../../docs/decisions/0010-m4-profile-and-verification-hardening.md) publishes
m4-scalars-v1; do not silently expand m3-core-v1. Keep failure classifications and nonzero
CRASH/TIMEOUT/MISMATCH exits intact, compare exact payloads, and check narrowing bounds before casts.
Confirm xtask's logged workspace matches the intended checkout; shared binaries may originate elsewhere.

Current M4 support supersedes earlier first-slice limits: read [M4 language/IR 2](../../../docs/m4-language.md)
and [ADR 0011](../../../docs/decisions/0011-m4-language-extension.md). Compile against m4-language-v1
VM capabilities. Preserve the IR 1 structural target and explicit IR 2 feature/version gates.
The authored inventory is 20 M0 plus 24 M4 PASS fixtures; no Microsoft oracle observation is claimed.

M4 external acceptance is now implemented under [ADR 0012](../../../docs/decisions/0012-m4-oracle-acceptance.md).
Read [the report](../../../docs/m4-oracle-report.md) and current language contract: 44 authored
plus 153 inherited upstream-reported Excel Mac scenarios PASS. Schema 2 and the reviewed allowlist
preserve raw results/environment/hashes; no fresh Office run is claimed. Use m5-host-v1
and explicit IR 3/4 capabilities, retaining IR 1/2 refusal rules. Unselected external cases remain NOT_RUN.

[ADR 0032](../../../docs/decisions/0032-m6-sign-off-and-reference-comparison-deferral.md)
signs off M6 for its tested integration profile by owner direction. Fresh
Excel/Word comparisons are deferred to M8.5 and remain NOT_RUN until observed;
they no longer block M7. Keep authored, inherited and fresh evidence distinct.

## M7 package evidence

Read [M7 acceptance](../../../docs/m7-acceptance.md) and
[ADR 0037](../../../docs/decisions/0037-m7-package-preservation-and-writeback.md).
The optional `cargo xtask m7` verifies reviewed sibling artifact hashes before
and after actual fork tests. Ordinary core CI remains sibling-independent.
Keep load/compile/run/save, canonical source/original project bytes and
authored/inherited/fresh evidence separate. Original binary preservation and
plain-source exports do not implement edited CFB writeback. Opaque signature
transport does not verify validity or grant trust. Native forms/controls and removal of `.tvform` are now scheduled in M7A;
fresh Office observations remain M8.5.

## Standing Office compatibility policy — M7A

[ADR 0038](../../../docs/decisions/0038-native-office-forms-and-compatibility-only.md)
supersedes the custom format direction and native-forms deferral. Mirror applicable
Excel/Word VBA behavior, formats and workflows; do not add custom user-facing
language features, formats or control semantics. Unknown behavior remains an
explicit gap pending evidence. M7A is in progress (see [inventory/ledger](../../../docs/m7a-inventory.md)); native `.frm`/`.frx`
and embedded form workflows, required native edited persistence, and removal of
`.tvform` precede M8 release acceptance. Existing codec references describe legacy
implementation, not permission to extend or retain that product format. Reuse
compatible internal layers and native-equivalent regressions. Original-byte
preservation alone does not satisfy saving native edits.

Initial FC1.1 native worksheet metadata lives in GridCraft XLSX, with no VM or
UI dependency and no second writable model. [ADR 0039](../../../docs/decisions/0039-m7a-native-control-metadata.md)
requires source DrawingML identity/assignment reconciliation before later native
rendering or activation. Decode tests are not execution evidence.

M7A.1/M7A.2 implement [native-userform-import-v1](../../../docs/m7a-control-contracts.md)
under [ADR 0040](../../../docs/decisions/0040-native-userform-import-and-attachment.md):
bounded native export/embedded Label/TextBox/CommandButton decoding and inert
shared/standalone/fork attachment. IDE -> project is a reviewed downward edge.
Keep original resources separate from authored source/design, validate handlers
before insertion, and never substitute .tvform or acknowledge edited native saves
without the M7A.5 encoder. Full semantics, broader controls and removal remain open.

M7A.3 completes [worksheet-form-controls-v1](../../../docs/m7a-worksheet-controls.md)
under ADR 0041 in GridCraft: native Shape identity/state, engine bindings/undo/recalc,
OOXML/VML edited persistence, optional ControlFormat APIs and trusted widget input.
Preserve original package data separately; failed joins stay inert and callbacks
use the existing bridge. This is not MSForms/ActiveX or UserForm edited-save support.
M7A.4 is in progress; fresh Office observations remain NOT_RUN under M8.5.

M7A.4 now also includes the [native runtime increment](../../../docs/m7a4-native-runtime.md)
under ADR 0043: six flat control families, selected property/input semantics,
Controls/UserForms lookup and bounded transient project inspection. Canonical
VM state remains authoritative; queries never load forms or invoke getters.
Keep intrinsic providers/wait routing intact and do not extend .tvform. The
milestone remains in progress with its remaining requirements recorded there.

Native Frame containers follow [ADR 0056](../../../docs/decisions/0056-native-frame-containers.md):
bounded native parent storages, immediate child Controls/Parent identity, scoped
TabIndex/option groups and nested ActiveControl/focus cancellation. Canonical state
remains ordinary classes; effective ancestor flags, live relative geometry,
clipping and accessible groups use the shared renderer. Original source/resources
stay inert and unchanged. Further container/page/image/binding/platform semantics,
native edited persistence and .tvform removal remain in M7A; fresh Office traces
remain NOT_RUN under M8.5.

[ADR 0057](../../../docs/decisions/0057-native-tabstrip.md) adds selected native TabStrip
records and separate ordinary Tab/Tabs identity, indexed/named lookup,
enumeration/default selection and authorized widget input. Canonical state stays
in ordinary classes; inert queries/rendering do not instantiate or execute VBA.
MultiPage, dynamic tab mutation/index/name semantics and the remaining M7A.4
ledger stay open. Native edited persistence and .tvform removal remain separate.

[ADR 0058](../../../docs/decisions/0058-native-multipage.md) adds selected native MultiPage
f/o/x reconciliation, distinct Page/Pages identity, immediate child collections,
selection-gated input/focus and actual page rendering. Internal tabs stay private;
canonical state remains ordinary classes and source/resources stay inert. Indexed
Click and repeated-selection behavior also correct the prior TabStrip contract.
Dynamic page/tab mutation, further persistence/property/event/lifecycle cases and
the full remaining M7A.4 ledger remain open. Native edited saving/removal stay
M7A.5/.6; fresh Office traces remain NOT_RUN under M8.5.

[ADR 0059](../../../docs/decisions/0059-native-image-pictures.md) adds selected native Image,
bounded inert BMP/GIF/JPEG/ICO decoding, canonical stdole picture identity and
assignment, display properties and nonfocusable Click. Immutable raster assets
and capped derivative texture caches preserve source/resources and existing VM
budgets. Further picture metadata/platform/format and event/presentation cases,
RefEdit, bindings, dynamic collections and remaining M7A.4 semantics stay open.
Native edited saving/.tvform removal remain M7A.5/.6; Office traces remain NOT_RUN
under M8.5.

[ADR 0060](../../../docs/decisions/0060-native-refedit.md) adds bounded inert RefEdit class-table
resolution, ordinary typed RefEdit text/drop state and selected focus/validation/
Change/DropButtonClick behavior. The shared renderer accepts host range-selection
availability and collapses/restores modal RefEdit; GridCraft owns worksheet
selection and authorization. Original source/resources stay separate. Broader
RefEdit/platform/property/event/binding semantics and the unchanged M7A.4 ledger
remain open. Native edited saving/removal stay M7A.5/.6; fresh Office traces stay
NOT_RUN under M8.5.

[ADR 0061](../../../docs/decisions/0061-native-tab-page-mutation.md) adds native Tabs/Pages
Add/Remove/Clear, writable Integer Index and runtime member Name. Scalar position
cells preserve member/selection identity without owning cycles; inert queries and
page input/rendering follow canonical ordering. Generic VM reference-scan metadata
keeps primitive slots out of object lifetime scans under unchanged budgets.
Dynamic Controls, bindings/multiple selection and the broader M7A.4 ledger remain
open; native edited saving/removal remain M7A.5/.6. Fresh Office traces are NOT_RUN.

[ADR 0062](../../../docs/decisions/0062-native-dynamic-controls.md) implements selected native
Controls.Add/Remove/Clear, runtime names, scoped collections/notifications and
WithEvents focus/validation. An inert runtime projection drives widgets and
input from ordinary canonical objects; imported source/design/resources stay
unchanged. Weak context requests avoid ownership cycles. Shared typed helpers,
indexed verifier metadata and generic dispatch optimizations preserve all existing
VM limits. Full M7A.4, native edited persistence/removal and M8.5 comparisons remain
separate requirements; CI/WASM/WASI validation of ADR 0062 passed.

[ADR 0063](../../../docs/decisions/0063-native-listbox-multiple-selection.md) adds native ListBox
multiple selection, independent focus/selection, row mutation and authorized
pointer/keyboard input. [ADR 0064](../../../docs/decisions/0064-native-runtime-type-names.md) adds
IR 12 inert runtime class names and native TypeName through canonical aliases.
State stays in ordinary classes, rendering stays inert, and budgets are unchanged.
Bindings and the remaining M7A.4 ledger remain open; M7A.5/.6 and M8.5 are separate.

[ADR 0065](../../../docs/decisions/0065-native-cell-and-row-bindings.md) defines native
ControlSource/RowSource through ordinary classes and the existing explicit host
range import. Import and rendering remain inert; GridCraft owns cell commands,
trust and revision-driven callbacks. The owner approved 4,096 static permission
entries on 2026-11-05; per-site authorization and executed-call/work/memory limits
remain in force. Further binding timing/types and the broader M7A.4 ledger remain open.


[ADR 0066](../../../docs/decisions/0066-foreign-runtime-type-names.md) adds optional
read-only runtime type names through the existing object resolver. Check full
identity/lifetime, bound names/work/allocation and preserve errors/fallback; never
invoke getters, host calls or activation. Forks own native names. No IR or budget
increase is implied. Broader M7A.4 and fresh M8.5 comparisons remain separate.


[ADR 0067](../../../docs/decisions/0067-vartype-default-value-inspection.md) adds
IR 13 optional default inspection for VarType. Preserve source-order evaluation,
normal class frames/authorized host calls, getter errors and one-shot Stop/wait
continuations. Resolver presence metadata is inert and identity checked; actual
getters remain executable. Older IR consumers refuse this capability. Watches
stay inert and all existing budgets remain. Full M7A.4/M8.5 are separate gates.


## Lazy project-class array elements

ADR 0068 array evidence is authored/reference-based, not fresh Office output. Cover fixed/dynamic arrays, ByRef order, element identity, stale/forged bytecode, private/static restore and constructor/For Each pauses natively and under WASI; native form identity also needs shared/fork regressions. See [ADR 0068](../../../docs/decisions/0068-lazy-project-class-array-elements.md). Existing budgets remain unchanged; full M7A.4 remains open.


[ADR 0069](../../../docs/decisions/0069-userforms-without-form-components.md)
selects minimal ordinary UserForms support without form components via bounded
code tokens; normal binding resolves shadows. Runtime selection uses verified
class/source identity, never mutable source during ticks. Preserve Reset, inert
inspection, wrapped callback waits/identity, existing budgets and no extra object
registry. Fresh comparisons remain M8.5; use the M7A.4 completion checklist.

ADR 0070 adds native typed keyboard events. Keep mutable ReturnInteger and
completion receipts in ordinary VBA frames/fields; UI queues only bounded raw
input and waits before default widget processing. Inspection must not execute
getters. Preserve Stop/Continue/Reset, injected ANSI services, distinct paste/IME
input and existing host trust checks. Keyboard/platform and the full M7A.4
completion checklist remain partial until their executable acceptance passes.

ADR 0071 arbitrates native raw input before editor shortcuts. Preserve canonical
VBA focus and callbacks, bounded raw held-key ownership, root widget identities,
paused character/release continuation and non-text Space ordering. Debugger
F5/F8 releases must not become form KeyUp callbacks. Root input eligibility
requires more than a missing TabStop. Full M7A.4 remains in progress.

ADR 0074 adds selected TextBox pointer selection after ordinary mouse callbacks.
Keep text/selection canonical and retain only immutable layout parameters/raw
anchors. Re-read values after callbacks, preserve cancelled/redirected focus and
Stop/Reset, and authorize InstanceText like other executable form commands.
Native AutoWordSelect bit 27 is inverted relative to its VBA property; do not
confuse persisted defaults with new dynamic controls. Full M01/M7A.4 remains open.
