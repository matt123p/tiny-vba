# M6A implementation framework and Luna handoff

Updated 2026-11-01. Status: design and work breakdown; no runtime implementation
or test results are claimed. Read [the scope](vba-intrinsic-functions.md),
[the plan](implementation-plan.md) and
[ADR 0034](decisions/0034-intrinsic-implementation-framework.md) first.
M6A acceptance still blocks M7. Implement one packet at a time; preserve current
M6 acceptance and historical IR/runtime contracts.

## Start here

1. Read AGENTS.md and the development/language-semantics skills. Inspect the
   working tree; it already contains the M6A planning changes. Preserve them.
2. Complete packet F0 below. Its output is an audited inventory and the first
   implementation packet, not a claim that all intrinsics are supported.
3. Implement F1–F4 in order, with small reviewable changes. Then implement
   function packets in the prescribed order. Do not rewrite the whole interpreter.
4. Update the ledger after each packet with exact tests and remaining work.
   Continue to the next ready packet unless user direction or an actual blocker
   requires otherwise. No approval is needed for routine choices within this design.

## Verified code map and current gaps

Paths below are relative to the tiny-vba root; inspect symbols rather than relying
on line numbers. These facts were checked against the working tree on 2026-11-01.

| File / symbol | Existing behaviour | M6A change point |
|---|---|---|
| `crates/vba-values/src/lib.rs`: `Intrinsic`, `from_name`, `accepts` | Enum plus hand-maintained name/arity list; internal ArrayBaseOne exists | Central descriptor catalogue; retain enum identities and compatibility entry points |
| `crates/vba-runtime/src/coercion.rs`: `intrinsic`, `string_function`, `convert`, `text`, `integral` | Central pure evaluation/coercion, partial string functions | Reuse conversions; extract family modules without changing public legacy behaviour |
| `crates/vba-semantics/src/lib.rs`: module options and Apply binding | Shared bare/qualified intrinsic binder; Option Compare Text is captured for IR 8 contextual operations | Keep one intrinsic binder and preserve project/local/implicit-Me/import precedence |
| `crates/vba-syntax/src/ast.rs` / `parser.rs`: ModuleOption | Compare option already represented | Retain source/spans; audit `$`/byte/zero-argument syntax before adding new parsing |
| `crates/vba-compiler/src/lib.rs`: ExprKind::Intrinsic | Emits argument expressions in order and Intrinsic opcode; special-cases v3 intrinsics | Lower the explicit call plan; feature-gate extended calls |
| `crates/vba-ir/src/lib.rs`, `verify.rs` | Legacy opcode verifies arity and v2/v3 requirements; IR 8 verifies IntrinsicCall and contextual BinaryCompare | Preserve old contract; retain verifier gates on IR 8 context operations |
| `crates/vba-vm/src/lib.rs`: intrinsic instruction arm | Pops/reverses arguments, handles DebugPrint specially, calls runtime; accepts injected string-comparison service | Keep extended calls/context read-only and avoid duplicate evaluation/effects |
| `crates/vba-ide/src/code_aids.rs`, `metadata.rs`, `lib.rs` | Source/host metadata and safe evaluation paths | Derive intrinsic code aids from descriptors; audit each evaluation entry point |
| `crates/vba-runtime/tests/m4_matrix.rs`, `crates/vba-vm/tests/m4.rs` | Direct typed runtime tests and source-to-VM helpers | Models for new focused family tests; preserve historical suites |
| `xtask/src/main.rs`: wasm-run | Explicit WASI suite commands, not an automatic guarantee for every new test target | Add new targets to real WASI execution and inspect logged test counts |

`CDec` cannot be implemented faithfully merely by returning Double: the present
value model has no Decimal variant. Likewise pointer width, code pages, calendar
and locale are real contracts, not strings to hardcode. Track their prerequisites.
The intrinsic constants must be audited too; no compare-constant definitions were
found in the inspected semantics/VM files. Reuse the existing reference/constant
resolution path rather than adding a second global-name resolver.

## Shared implementation contracts

### One descriptor catalogue

Keep semantic metadata in `vba-values`, alongside Intrinsic (a sibling source
module is appropriate). It must not depend on syntax, runtime, IR, host or UI.
Use ordinary checked-in Rust data; no build-time code generation or new crate.

Each implemented public spelling needs:

- intrinsic identity and alias/return variant; distinguish `$` from Variant forms;
- display/canonical spelling, permitted qualification and zero-argument call form;
- parameter names in declaration order, required/optional/variadic rules and
  omitted-default policy (including function-specific overloads such as InStr);
- per-parameter value-context versus raw-Variant/object handling;
- pure/contextual/host-effect classification and read-only inspection policy;
- minimum IR version and capability, preserving current v2/v3 entries;
- human-readable signature/help and platform requirements.

`from_name` and `accepts` remain compatibility wrappers over this catalogue.
One public spelling resolves case-insensitively to one descriptor. Do not expose
internal ArrayBaseOne as a public alias. Do not register planned unimplemented
functions as executable: keep their audit inventory separate. Known restricted
functions may resolve to an explicit unsupported descriptor/diagnostic.
Add descriptor tests for duplicate spellings, arity/default consistency and aliases.

### One binder and explicit argument plan

Both bare intrinsic calls and `VBA.<name>` calls use one helper after their existing
name-resolution decisions. Preserve project/local/implicit-Me/import precedence;
do not make descriptors override a user declaration. Audit whether a local `VBA`
identifier shadows qualification rather than blindly special-casing its spelling.

The bound extended call records:

```text
IntrinsicCallPlan
  descriptor identity and return variant
  evaluated expressions in original source order
  declaration-order parameter bindings: Evaluated(index) or Omitted
  effective module comparison mode
```

Resolve names/default positions without evaluating arguments. Reject duplicate,
unknown, forbidden-order and missing-required arguments with their source spans.
Evaluate every supplied expression once in source order, then map results into
parameter order. Omitted is structural metadata, not Null/Empty or a user-supplied
Value::Missing; IsMissing must still observe an actual Missing Variant correctly.
No blanket default-member coercion: use the parameter's descriptor mode.
Function-specific overload selection is explicit and tested, not guessed from arity.

### Extended calls and capability boundary

Preserve `Instruction::Intrinsic` and its v2/v3 meaning for existing programs.
Introduce **IR 8**, capability **`intrinsics-m6a-v1`**, for extended intrinsic calls.
The new instruction carries the descriptor/variant identity, supplied-expression
count, declaration-order argument bindings and effective comparison mode. Reuse
the existing typed operand stack; evaluate arguments before emitting the call.
Do not put ASTs, source text, closures or host handles into these operands.

Verifier requirements: IR >= 8 and the new capability; executable descriptor;
valid variant/context; capped parameter/supplied counts; every evaluated index
in bounds, used exactly once; required slots present; only optional slots omitted;
legal variadic layout; correct stack consumption and one result. Malformed
bytecode must fail before runtime/host dispatch. Retain the current 256-argument
ceiling unless a separately justified function contract requires a smaller cap.

Compile new/extended calls to this form. Unchanged legacy calls can retain the old
form. An old capability target must refuse an extended call explicitly. Keep the
old runtime entry point working for existing callers/tests, with its existing
defaults; new contextual execution uses the extended path. Update the IR spec,
disassembly, capability negotiation, legacy-refusal tests and runtime docs together.

### Context and services

Use a read-only per-call context for module comparison and immutable locale,
calendar, code-page and target-width configuration. The compiler captures the
calling module's comparison mode in the verified call, not the entry module's.
Omitted compare and explicit -1 use it; dynamic compare expressions are evaluated
once. Preserve Option Compare Binary as default. Do not simply remove the Text
guard: module string operators and existing comparison intrinsics must agree with
the implemented option, or retain an explicit refusal for an incomplete profile.

Keep pure algorithms and shared coercion in `vba-runtime`; split family dispatch
into modules as functions are added. Keep the compatibility wrapper thin. Use
`VbaString::units()` for code-unit operations; UTF-8/lossy conversion cannot
silently alter unpaired surrogates or UTF-16 indexes.

VM-owned execution state holds seeded RNG state and injected service identity.
Use the existing native host wait/completion protocol for effectful calls; do not
create a second scheduler or mutate the pure runtime through global state.
Do not call host services from constant folding or read-only watches. Reset,
pause, cancellation and persisted-state rules must be explicit and tested.

Locale/text services must have deterministic test doubles and a documented
supported implementation for each accepted target. ASCII case folding is not a
locale service. NoService must yield a named unsupported capability, never a
fabricated result. Final acceptance still requires every required service row.
Choose providers only after dependency/licence/native/WASM review; do not add a
large dependency or assume Rust's host locale supplies VBA behaviour.

### Errors and budgets

Keep `Error::Vba`, `Unsupported` and `Resource` distinct. Coercion and error
precedence are per-function contracts: do not globally propagate Null before
validating every other argument. Assert exact numbered errors where established;
uncertain numbers remain a contract issue until reference research resolves them.
Check input-derived arithmetic and capacity before allocating. Numeric iteration
has an explicit iteration cap/convergence rule. Long intrinsic work must charge
or bound VM work rather than evading slice/cancellation budgets inside one opcode.

## Work packets and dependencies

Packet IDs below refine existing M6A tasks; they do not renumber the plan.
Each packet ends with working code/tests or a completed audit, a ledger update
and an exact next packet. Keep function packets to at most 1–4 closely related
functions; split a row below further when its tests or dependencies warrant it.

| Packet | Plan task | Depends on | Deliverable and exit condition |
|---|---|---|---|
| F0 | M6A.1 | Existing docs | Per-function inventory using the template below; mark actual code/support, variants, evidence and prerequisites. Establish StrComp's full contract. |
| F1 | M6A.1/M6A.7 | F0 | Descriptor catalogue and family-dispatch seam; migrate existing metadata without semantic changes. Historical tests and descriptor checks pass. |
| F2 | M6A.7 | F1 | IR 8 call-plan instruction, verifier/disassembly/capability support and runtime compatibility adapter. Positive and malformed/legacy negative tests pass. |
| F3 | M6A.7 | F2 | Shared bare/qualified binder, named/omitted argument planning, alias identity and code aids. Source-order side-effect and shadowing tests pass. |
| F4 | M6A.2/M6A.5 | F3 | Effective module comparison, constants and context seam; coherent Binary/Text comparison contract, locale double and explicit unavailable-service behaviour. |
| S1 | M6A.2 | F4 | StrComp end-to-end first; exact Integer/Null results, errors, default/-1/0/1 modes and real module context. |
| S2 | M6A.2 | S1 | InStr/InStrRev, including distinct omitted compare defaults, reverse-position bounds and injected Text comparison. |
| S3 | M6A.2 | S2 | Replace start/count/compare behavior and typed/error boundaries. |
| S4 | M6A.2 | S3 | Split/Join array shape, bounds, empty fields and defaults. |
| S5 | M6A.2 | S4 | Filter include/compare behavior and String array results. |
| S6 | M6A.2 | S5 | Add LTrim/RTrim and StrReverse with whitespace-edge, UTF-16 and Null boundaries. |
| S7 | M6A.2 | S6 | Implement Space/String count, character, Null, return-tag and bounded-output rules. |
| S8 | M6A.2 | S7 | Audit LCase/UCase coercion, Null, case range, return type and locale boundary. |
| S9 | M6A.2 | S8 | Implement AscW/ChrW over UTF-16 code units, with signed Integer returns and surrogate boundaries. |
| S10 | M6A.2 | S9 | Add Left$/Right$/Mid$ with fixed String return identity while plain forms retain Variant(String). |
| S11 | M6A.2 | S10 | Add Trim$/LTrim$/RTrim$ with fixed String return tags and correct plain Variant(String) metadata. |
| S12 | M6A.2 | S11 | Add Space$/String$ with fixed String return identity while plain forms retain Variant(String). |
| S13 | M6A.2 | S12 | Audit byte intrinsics and specify byte representation/provider before implementation; Unicode BSTR bytes and converted ANSI/MBCS bytes have distinct semantics. |
| C1 | M6A.3 | F3; relevant context | Portable scalar conversions and Variant-state/type inspection; exact width, ties-even, Null/Error states. Keep locale parsing/formatting, Decimal and target-width work explicit. |
| C2 | M6A.3 | C1; injected date service | Route CDate text parsing and IsDate text recognition through an explicit locale/calendar provider; no ambient OS locale. |
| C3 | M6A.3 | C2; injected locale conversion service | Add IsNumeric numeric-string recognition and CStr(Date) locale/calendar formatting; no ambient locale. |
| C4 | M6A.3 | C3; value/context foundation | Add an exact 96-bit coefficient/scale Decimal Variant payload and expose the configured 32/64-bit LongPtr alias without approximating Decimal arithmetic. |
| C5 | M6A.3 | C4 | Implement CDec exact scalar/string paths and CLngPtr for explicit 32/64-bit targets; refuse binary-float Decimal approximation and unsupported target widths. |
| C6 onward | M6A.3 | C5 | Continue remaining conversion and formatting functions in dependency order. |
| A1 onward | M6A.4 | F3 | Arrays/bounds and conditional selection/evaluation; preserve existing array storage/aliases and IIf eagerness. |
| N1 onward | M6A.4 | F3 | Pure numeric functions, then financial functions with bounded convergence and exact VBA error contracts. |
| D1 onward | M6A.5 | F4 | Calendar/date representation and clock seam, then constructors/extraction/arithmetic/names in small packets. |
| R1 | M6A.5 | Context/state seam | Rnd/Randomize deterministic state, reset and pause/resume tests; never ambient randomness. |
| H1 onward | M6A.6 | F3; existing host protocol | One bounded host-service family per packet; colour/CallByName separate. Specify unavailable/denied/cancelled behaviour and no replay. |
| Q1 | M6A.7 | Each implemented packet | Incremental shared IDE signatures/completion and inspection-policy tests; do not wait until the end to expose functions correctly. |
| X1 | M6A.8 | All required rows | Published shared revision, real pinned fork tests and documented acceptance; only then start M7. |

F2–F4 are shared infrastructure, not permission to implement all family functions
at once. Partial slices can be recorded, but S1 is not accepted until its full
declared contract passes. An unresolved architecture/provider issue is reported
precisely; continue independent ready work rather than guessing behaviour.

## Function packet template

Copy this into `docs/m6a-work-ledger.md` as the next packet becomes ready.
Keep one authoritative row per spelling in the inventory section of
`vba-intrinsic-functions.md`; link the packet rather than duplicating support claims.

```text
Packet ID / owning M6A task:
Function(s), aliases and exact public reference URLs:
Prerequisites and touched files/symbols:
Signature, defaults, argument modes, return tags:
Option/locale/platform requirements:
Empty / Null / Missing / Nothing / CVErr rules and error precedence:
Numbered errors and reference basis:
Allocation/work caps and cancellation policy:
Worksheet counterpart and discriminating cases:
Runtime vectors (typed input -> typed result/error):
Source-to-VM cases, qualification/shadowing/evaluation-order cases:
Native/WASI and affected IDE/fork checks:
Implemented / remaining / exact validation output:
Next packet:
```

Suggested per-function inventory columns:
`spelling | identity/variant | signature | owner | current limitation | required
disposition | prerequisites/packet | worksheet counterpart/difference | reference
| native result | WASI result | fork result | remaining gap`.
F0 expands every seed group and worksheet-counterpart group; neither an enum entry
nor successful parsing is a PASS. Keep NOT_AUDITED and NOT_RUN distinct.

## First semantic packet: StrComp test specification

F0 must establish coercion/error precedence, exact errors and locale expectations
before S1 implementation. The following authored contract vectors are ready to
turn into assertions; they are not Microsoft observations:

| Inputs / mode | Expected typed result |
|---|---|
| `"a", "a", 0` | `Value::Integer(0)` |
| `"a", "b", 0` | `Value::Integer(-1)` |
| `"b", "a", 0` | `Value::Integer(1)` |
| `"", "", 0` | `Value::Integer(0)` |
| `"", "a", 0` / `"a", "", 0` | Integer -1 / Integer 1 |
| `"A", "a", 0` | `Value::Integer(-1)` |
| `"A", "a", 1` with the declared text provider | `Value::Integer(0)` |
| `Null, "a", 0` / `"a", Null, 0` | `Value::Null` |
| omitted compare / explicit -1 | Same result as the calling module's declared mode |

Additional mandatory cases: non-ASCII and unpaired UTF-16 strings under binary
comparison; injected text-provider ordering; invalid/Null compare; Access-only
mode refusal; extra/missing/duplicate/unknown arguments; supported named argument
order; local/project shadowing; VBA qualification; exactly-once source-order
evaluation; On Error/Resume; statement pause/resume. A source test must return
`As Variant`, so an `As Long` return conversion cannot hide an incorrect tag.
Cross-module tests must use different comparison options in caller and callee.

## Validation and handoff reporting

Add focused tests under owning crates, for example
`vba-runtime/tests/intrinsics_strings.rs`, `vba-values/tests/intrinsics.rs`,
`vba-semantics/tests/intrinsics.rs`, `vba-ir/tests/intrinsics.rs` and
`vba-vm/tests/intrinsics.rs`. Those filenames are proposed, not existing tests.
Reuse the source/drive/typed-outcome patterns in `vba-vm/tests/m4.rs`; do not add
a JSON round trip or a second interpreter to test the first.

During a packet, run the affected crate's focused tests. Before marking a packet
complete, run the required `cargo xtask ci`, `cargo xtask wasm` and
`cargo xtask wasm-run`; add new targets to the latter explicitly where necessary.
Run affected UI/native/web gates for interaction/code-aid changes. Verify the
xtask workspace banner. Do not repeatedly run unchanged broad gates merely to
fill time; a failed or unavailable gate is recorded with its exact reason.
No new remote fixture/macro execution is authorized by this framework.

Final X1 acceptance includes both fork adapters and their application gates using
published immutable tiny-vba pins, including StrComp in a GridCraft worksheet UDF.
Do not publish, push or change fork pins as part of preparing this handoff.
At implementation time, follow the user's current publication instructions and
repository gates; no development path overrides count as final fork acceptance.

Report a packet's outcome in this form: implemented behaviour; exact tests run;
remaining gaps/blockers; next packet. Leave M6A planned/in-progress until every
required row and X1 pass. Fresh Office comparisons remain M8.5.
