---
name: tiny-vba-language-semantics
description: Implement or review VBA syntax, binding, values, coercion, bytecode and runtime behaviour in tiny-vba. Use for language correctness and execution changes rather than Office object-model implementation.
---

# VBA language semantics

For the accepted M6A intrinsic profile, see
[the inventory](../../../docs/vba-intrinsic-functions.md),
[acceptance](../../../docs/m6a-acceptance.md) and
[implementation record](../../../docs/m6a-work-ledger.md). IR 8 extended calls,
the descriptor catalogue and argument planning are implemented. Retain shared
coercion, UTF-16 and exact types/errors; worksheet equivalents are not a VBA
implementation. Environmental services require explicit injection; documented
policy/platform limits remain distinct from supported behavior.

Use [the implementation framework](../../../docs/m6a-implementation-framework.md)
and [ADR 0034](../../../docs/decisions/0034-intrinsic-implementation-framework.md)
for the descriptor/binder/call-plan design. Implement the IR 8 extended form with
explicit capability checks; retain the old intrinsic instruction contract.
Structural omissions are distinct from Value::Missing. Evaluate supplied
arguments once in source order before parameter mapping. Start with F0 and
record results in the ledger; do not claim the designed IR is already supported.

New M1.1 adds explicit reference enum/constant binding under
[ADR 0020](../../../docs/decisions/0020-explicit-host-reference-snapshots.md).
Qualified type syntax preserves spans; registered enums use shared Long coercion
and existing IR. Reference constants must obey scope, qualification and missing-
reference rules in runtime expressions, defaults and bounds. Nominal objects now
carry verified IR 5 metadata under ADR 0021. Preserve exact declared ByRef identity,
typed temporaries and array tags through Variant/resize/Erase. Type checks use the
host's read-only resolver, not Office definitions in the engine; legacy IR rejects
nominal metadata. Calls/edits/restores must not bypass the shared validation path.

Read [architecture](../../../docs/architecture.md) and the relevant sections of the
[interpreter specification](../../../docs/vba-interpreter-rust-implementation-spec.md).
Use MS-VBAL and documented, profile-tagged observations for semantics; distinguish a proposed
expectation from a measured Microsoft oracle result.

M0 foundations are in [ADR 0003](../../../docs/decisions/0003-m0-source-scenarios-foreign.md):
checked UTF-8 source spans, caller-assigned source/statement identities, UTF-16 value strings,
scalar states and bounded owner-scoped locations. These are not parser/coercion/VM implementations.
Reuse these types rather than introducing a second value/source model.

The adopted [IR 1 specification](../../../docs/bytecode-vm-specification.md) defines exact stack effects,
canonical direct ByRef, empty-stack checkpoint blocks, verifier and version/feature refusal rules.
Use shared runtime semantics for operators/conversions; IR verification checks shape, not full VBA types.
Later instructions need an explicit IR-version/contract update. The specification itself is not support.

## Preserve the language model

- Preserve source spelling while resolving identifiers case-insensitively. Retain module/source
  identity, spans and individual statement IDs, including multiple statements on one source line.
- Keep Empty, Null, Missing, Nothing and CVErr distinct. Preserve numeric widths, Currency/Date,
  array element types, dimensions and lower bounds; do not reduce values to f64 or JSON.
- Centralize coercion, operators, argument binding, ByRef locations and error state. Use the same
  rules across execution and debugger evaluation. A language change must not create a parallel
  conversion implementation inside UI, DAP or a host bridge.
- Verify bytecode before execution. Use explicit frames, instruction position and per-frame error
  state so execution can pause at statement checkpoints and resume without replay.
- Inject time, randomness, locale and prompts. Bound input allocations, parsing/nesting and VM work;
  return structured diagnostics for unsupported constructs and malformed input.
- Implement generic foreign invocation through vba-foreign; Excel/Word names and defaults stay
  in the application's Office layer. Never add a workbook store to complete a language example.

Implement the smallest semantic slice with cases checking types, values, exact error numbers,
side effects and unsupported behaviour. Include same-line source mapping and pause/resume cases
when lowering or control flow changes. Put integration tests under the owning crate's tests/.
Use the conformance skill for external fixtures or oracle claims, and debugger skill for stepping
or inspection changes. Preserve native/WASM execution paths and report the exact supported subset.

## Implemented M1 frontend

Read [frontend](../../../docs/frontend.md) and [ADR 0005](../../../docs/decisions/0005-m1-frontend.md).
Reuse vba-syntax's owned source/tokens/AST and distinct StatementSites. A recovered tree with diagnostics
is never valid input to binding. Apply nodes remain unresolved; retain parentheses, call style,
Let/Set and omitted/named/optional syntax. M1 parsing does not make those IR 1 capabilities.
M2 implements the documented subset of binding/Option Explicit/types/call legality. No execution results change from parsing.

## Implemented M2 contract

Read [compiler](../../../docs/compiler.md) and [ADR 0006](../../../docs/decisions/0006-m2-binding-and-verification.md).
Reuse vba-values ScalarKind, vba-semantics bound symbols and vba-compiler lowering. The compiler always
passes vba-ir verification; only vba-ir constructs VerifiedProgram. Keep IR independent of binding and
VM independent of compilation/debug/UI. M2 ir1-structural capabilities describe verification shapes,
not runtime support. Actual execution must check the M3 runtime matrix. Preserve exact constant tags,
ordered slots/sites, complete nonexecutable statement metadata and explicit unsupported diagnostics.

## Implemented M3 contract

Read [runtime](../../../docs/runtime.md) and [ADR 0007](../../../docs/decisions/0007-m3-execution-and-debugger.md).
The m3-core-v1 runtime is intentionally narrower than structural IR. Preserve explicit frame/alias
storage, one-visit checkpoint permits, saved fault PCs, generation checks and nonreplayed continuations.
Use vba-runtime for operations/edits. VM host dispatch must use checked precommit return/writeback
validation and explicit per-site permissions; foreign aliases remain unsupported. Run cargo xtask ci,
wasm and wasm-run (native plus actual WASI execution). Runtime example results do not mark unrun M0
or external scenario assertions PASS. Graphical UI, DAP and VBA error-handler semantics remain later work.

[ADR 0008](../../../docs/decisions/0008-vm-instance-isolation.md) distinguishes the caller's routing
label from the non-reused SessionInstance. Key handles, locations, breakpoints and stepping by the
engine instance; caller-label/frame/generation equality alone cannot identify an execution. Include
the replacement-session regression in native/WASI evidence when changing these identities.

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

ADR 0022 adds IR 6 generic default invocation/value contexts and bounded For Each.
Read docs/m1-defaults-and-collections.md and the current compiler/runtime contract.
Office defaults, collections and signatures remain fork-owned. Preserve safe
watch evaluation, canonical array aliases, resize locks and wait/cancel semantics.

M2's selected class/standard-form profile is implemented under ADR 0023/0024.
Read docs/m2-acceptance.md for current scope, workflow, exchange format and gates;
docs/m2-classes-and-forms.md retains historical incomplete checkpoints. Preserve
IR7 class/event identity, canonical field ByRef, static visibility/signature checks,
property value ByVal semantics, inert safe inspection and all three error modes.

vba-ide owns the bounded shared design, authored source, .tvform codec and form
orchestration; vba-ui-egui renders the same designer/runtime in native/browser
hosts. Generated native control classes use ordinary Event/WithEvents frames.
Explicit suspended callbacks park a modal caller; existing callbacks remain
terminal-only. Retain parked locations, shared budgets, input ownership and host
tokens through pause/resume. Cancel every token on Reset/close; do not execute
user cleanup during abnormal shutdown. Design/history must survive VM Reset.

Form import is inert and validates before project mutation. Save/export preserve
exact authored source and design; revision acknowledgments must reject intervening
source/design edits. Unknown formats, controls and binary resources fail explicitly.
The selected Label/TextBox/CommandButton profile does not authorize COM/DLL or
third-party controls, binary Office form/resource compatibility or package writeback.
Office document bindings/events, storage, trust and package persistence stay in forks.
Run CI/WASM/WASI plus UI/native/web and both Chromium smoke/form acceptance gates
when changing these surfaces. M3 range/worksheet work is GridCraft-owned.

M6 metadata/code projection is defined by [ADR 0030](../../../docs/decisions/0030-attribute-metadata-and-code-projection.md).
Keep raw source canonical; use CodeProjection for visible edits/coordinates and
preserve hidden metadata/newline bytes through history and export. Attribute
records never create executable checkpoints. Document rename requires host
validation and revision-checked acknowledgement; unsupported hosts refuse.

Numeric For controls include implicit/explicit scalar Variant for the existing
positive integral literal-Step profile. Preserve numeric bound validation and
once-only capture before counter assignment; reuse shared arithmetic, exact
numeric tags, catchable errors and debugger checkpoints. Do not reintroduce the
legacy Integer/Long-only binder guard or accept arrays as scalar controls.

IR 8 `host-procedure-defaults-v1` under ADR 0036 retains standard optional/default
and trailing ParamArray metadata for explicit host calls. Validate its version/feature,
ByVal Variant-array position and literal defaults. Earlier targets retain expanded static
calls and required standard parameters. Qualified external type members may use contextual
keyword spellings; bare reserved identifiers remain invalid.

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

M7A.4 has started under [ADR 0042](../../../docs/decisions/0042-userform-instance-routing.md).
The [first instance packet](../../../docs/m7a4-userform-instances.md) removes the
predeclared-only restriction in shared tiny-vba. FormInstance keys separate native
runtime objects from designer/source IDs; UI input/window scopes use those keys.
Read-only VM class queries never instantiate objects; active and parked receivers
remain lifetime roots. Keep Reset/source/resources inert. Broader lifecycle,
control/property packets, fork integration and fresh Office evidence remain open.

M7A.4 now also includes the [native runtime increment](../../../docs/m7a4-native-runtime.md)
under ADR 0043: six flat control families, selected property/input semantics,
Controls/UserForms lookup and bounded transient project inspection. Canonical
VM state remains authoritative; queries never load forms or invoke getters.
Keep intrinsic providers/wait routing intact and do not extend .tvform. The
milestone remains in progress with its remaining requirements recorded there.

[ADR 0044](../../../docs/decisions/0044-userform-window-and-code-lifetime.md) separates
UserForm display/default ownership from referenced code lifetime. Named control
getters reload; inert inspection does not. Generic root release validates class/
object and protects document code names. Internal release waits remain bounded,
report Running and preserve slice quotas; Terminate runs only at normal finalization.

IR 9 `class-enumerators-v9` under
[ADR 0045](../../../docs/decisions/0045-class-and-native-collection-enumerators.md)
validates public zero-argument Object-returning class enumerators. Keep older IR
refusal, ordinary getter checkpoints, bounded forwarding roots, wait/cancel and
active/parked snapshot lifetimes. Native Controls/UserForms snapshot canonical
objects through the transient view; no second mutable registry or .tvform change.

IR 10 scalar As New is defined by
[ADR 0046](../../../docs/decisions/0046-lazy-project-class-variables.md). Preserve
exact nominal local/field/persistent metadata, inert declaration/inspection/Set
targets, source-order ByRef preparation, ordinary constructor frames and failed
initialization retry. No external activation or mutable registry is added.

IR 11 class interfaces and Forms library bindings follow
[ADR 0047](../../../docs/decisions/0047-native-forms-nominal-types-and-library-access.md).
Preserve exact nominal identity and event-source/exposure, bounded immutable
descriptors, restored-session validation and source-function alias shadowing.
MSForms names stay in the IDE provider; no new store, COM or unsafe evaluation.

Native unbound ListBox follows [ADR 0051](../../../docs/decisions/0051-native-unbound-listbox.md).
Keep list/selection state in ordinary VM fields, bounded indexed row operations,
executable input authorization, focus/validation and callback continuations. Do
not expand .tvform or claim full list/binding/ComboBox/M7A.4 parity.

Native ComboBox follows [ADR 0052](../../../docs/decisions/0052-native-combobox.md).
Reuse ordinary list rows, canonical UTF-16 text/selection/drop state and existing
locale services/refusals. Preserve authorized input, typed validation, source/
resource identity and nonreplayed Change/Click/drop continuations. No new codec,
COM or second mutable store; broader M7A.4 work stays explicit.

Native ListBox/ComboBox bulk loading follows [ADR 0053](../../../docs/decisions/0053-native-list-array-loading.md).
Keep source arrays independent, validate all cells before canonical replacement,
and use ordinary control refresh/event continuations. No error-handler rank probes,
new intrinsic or mutable host store; array reads and broader M7A.4 remain open.

Native array readback follows [ADR 0054](../../../docs/decisions/0054-native-list-array-readback.md):
copy canonical cells with explicit bounds, preserving Err, selection and events.
[ADR 0055](../../../docs/decisions/0055-native-form-metadata-and-control-tips.md) keeps native
metadata in ordinary fields and tooltip hover inert, bounded and theme-injected.
Help/F1 routing, native persistence/removal and the broader M7A scope stay open.
