# Intrinsic implementation design

The shared descriptor catalogue, argument plan and context contract keep binding,
runtime and editor metadata consistent. See the [function inventory](vba-intrinsic-functions.md)
for supported spellings and services.

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
