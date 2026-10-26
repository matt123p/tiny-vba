# ADR 0011: M4 language extensions

Status: adopted for implementation. Date: 2026-10-26.

Language elaboration remains in semantics/compiler, scalar rules in runtime, storage in values,
and execution/error state in the explicit-frame VM. Intrinsic identifiers are shared value-level
identifiers; their arity is verified before execution. New instructions require IR 2 and an explicit
feature. IR 1 keeps its existing instruction contract. Expanded execution uses m4-language-v1.

Optional defaults, omitted arguments and named calls elaborate to complete positional IR calls.
Present argument expressions evaluate in source order into typed temporary slots before parameter
reordering. ByRef locals retain canonical aliases; evaluated arguments get private call temporaries.
The runtime ABI still has required parameters; source optionality is a frontend contract.

Error handling is per frame: disabled, Resume Next, or a label target, plus an independent active
handler flag and saved fault checkpoint. Only numbered VBA failures enter handlers; engine, budget,
cancellation and protocol failures cannot be swallowed. Active handlers are skipped on propagation.
Unwinding invalidates owned callee locations while preserving committed caller/host effects.
Resume retries the fault checkpoint; Resume Next uses its verified source continuation; Resume label
uses a verified checkpoint. Err.Clear clears the error object without re-enabling an active handler.
A debugger fault stop precedes handler delivery and continuing applies that delivery once.

Arrays retain declared element kinds, signed lower bounds and bounded dimensions. Element references
must share the owning array storage and invalidate on structural replacement. Persistent project/static
storage has an explicit session/project lifetime and reset; it must never be process-global state.
These contracts are implementation obligations, not a claim that every listed feature is enabled.

All additions require exact typed/error/side-effect regressions and native/WASI validation. No
Microsoft oracle observation is inferred from authored expected results.

IR 2 adds Intrinsic, OnError, Resume, ErrGet, ErrClear, Raise, RefElement, ArrayResize and
ArrayErase. Persistent slots use a bounded shared project key in SlotRole. The verifier checks
cross-procedure key/type agreement. Handler-capable CFG verification conservatively includes
verified logical Resume Next edges and registered handler/explicit-resume targets. Every block
still starts at an empty-stack checkpoint. This is bounded reachability, not proof of handler
mode at every source point. The compiler records structured-statement successors explicitly.

The new semantics -> runtime dependency shares constant/default coercions with execution;
IR still depends only on values/diagnostics. Arrays are limited to eight dimensions, 65,536
scalar elements and eight MiB of payload; nested arrays are explicitly refused. Locale-dependent
Date text parsing/formatting and non-ASCII case/codepage operations remain Unsupported. Project
state restoration requires the same revision and exact source identities/text; fresh sessions reset.

IR 2 also gates WriteRef(ArgumentBinding) under argument-defaults-v2. Captured argument values
use parameter-binding semantics so Object/Nothing survive named reordering and evaluated ByRef
temporaries. Ordinary Let and Set instructions keep their separate assignment rules.
