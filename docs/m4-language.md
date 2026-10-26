# M4 language profile and IR 2

The `m4-language-v1` profile implements the reviewed M4 semantic subset. It builds on the M3
explicit-frame VM, debugger, budgets and generic host protocol. [ADR 0011](decisions/0011-m4-language-extension.md)
is the adopted extension. This is an independently tested language profile, not full VBA or an
Office compatibility certification. Original supplied specifications remain design inputs.

## Values and coercion

All twelve scalar declaration types execute: Variant, Boolean, Byte, Integer, Long, LongLong,
Single, Double, Currency, Date, String and Object. Assignment, parameter binding, conversions and
debugger edits share vba-runtime. Numeric conversion uses ties-to-even rounding and checks the
actual destination bounds. LongLong integer arithmetic stays in checked i64; Currency addition,
subtraction and multiplication use scaled integers with i128 intermediates. Currency decimal
strings preserve the entire i64 boundary without passing through Double. Date serial conversion
accepts -657434 through values below 2958466. Date +/- day intervals preserves Date; Date-Date
returns Double. Single arithmetic with Single/Integer/Byte preserves Single. Mixed floating
operations use Double when its width is required. Integer/Long overflow remains error 6.

Empty supplies zero or an empty string in the corresponding context. Null propagates through
arithmetic/comparison, with determining False And / True Or cases and string concatenation rules.
Numeric conditions are supported; a Null condition is False. Missing, CVErr and Nothing remain
distinct: IsMissing/IsError/IsObject inspect them without raising. Invalid numeric use yields
448/13/91 respectively; typed assignment of Null yields 94. Object assignment still requires Set.

String data is UTF-16; Len and slicing count code units, including unpaired surrogates. Binary
comparison is deterministic. Ambient locale, time and codepages are not consulted. Non-ASCII case
conversion/codepage operations and locale-dependent Date formatting are explicitly Unsupported.

## Arguments, arrays and storage

Optional parameters may have constant scalar defaults. An Optional Variant without a default
receives Missing; a typed Optional requires a default. Named arguments bind case-insensitively,
reject duplicates/unknown names, and preserve source evaluation order before reordering.
Omitted arguments receive defaults; required arguments cannot be omitted. The IR ABI remains a
complete positional signature: optionality is elaborated in the compiler. Launch parameters still
require explicit ByVal values. Direct exact-type local/static/module/array-element ByRef aliases
share canonical locations. Parenthesized and evaluated ByRef expressions use typed private temporaries.

Arrays support Dim with fixed bounds, dynamic Dim(), ReDim, ReDim Preserve, Erase, typed element
reads/writes and direct ByRef elements. Dimensions retain signed lower bounds and column-major
order. Preserve may change only the final upper bound. Erase resets fixed cells or unallocates a
dynamic array. Rank is bounded at eight, count at 65,536 and payload at eight MiB. Structural
replacement invalidates old element locations; a later stale use is numbered error 9 and debugger
snapshots identify unavailable locals. Failed resizing preserves prior storage. Array element
representation tags are validated. Nested arrays are refused explicitly.

Module variables and Static local declarations share bounded persistent slots during calls.
Each fresh Session starts with defaults. `project_state()` exports state only after a terminal
invocation; `restore_project()` imports it before the first instruction, matching revision and
exact source identities/text. Starting fresh resets state. Completed writes survive failure;
restoration does not implement rollback. State is owned by the Session/caller, never a process global.

The profile currently refuses calls combining named/omitted or evaluated-temporary argument
capture with ByRef array elements, whole-array
parameters, ParamArray, UDT fields, Static procedure modifiers, implicit ReDim declarations,
Option Base 1, class/property/default-member dispatch, radix/extra-suffix numeric literals and
locale-sensitive behavior. These refusals are outside the reviewed M4 profile. Object host
dispatch and snapshot aliases retain the M3 restrictions; Office objects remain fork-owned.

## Intrinsic registry

The values layer owns closed intrinsic identifiers and arity. Runtime owns implementation and
shared conversions. Implemented deterministic operations are CVErr, IsMissing, IsEmpty, IsNull,
IsError, IsObject, IsArray, VarType, TypeName, CInt, CLng, CLngLng, CByte, CSng, CDbl, CCur, CDate,
CBool, CStr, CVar, Len, Abs, Int, Fix, Sgn, Round, Left, Right, Mid, Trim, LCase, UCase, Asc, Chr,
InStr, Replace, Split, Join, Array, LBound and UBound. Binary string comparison is supported;
locale-dependent comparison is refused. Debug.Print captures UTF-16 records in `Session::output()`;
comma-separated arguments are tab-separated in this profile. Semicolon formatting, output sinks,
formatting/date-text functions and ambient IO intrinsics are not enabled.

## Error state and resumption

On Error Resume Next, On Error GoTo label, On Error GoTo 0, Resume, Resume Next, Resume label,
Err.Number/Description/Source, Err.Clear, positional Err.Raise and numeric line labels/Erl execute.
On Error GoTo -1 is refused. Each frame has independently enabled/disabled mode and an active
fault checkpoint. A new error skips active handlers and searches callers; unwound frame-owned
locations expire, while completed caller/host writes survive. Resume retries the fault statement;
Resume Next uses its logical continuation, skipping a faulting If/For header's entire construct.
Resume label chooses a verified checkpoint. Resume without an active error yields 20. Err.Clear
clears properties while leaving handler activation intact. Resume clears activation and Err.

Only numbered VBA errors enter handlers. Unsupported operations, hard limits, cancellation and
host protocol/disconnect failures remain engine outcomes. A host's numbered error, including after
wait/resume, enters VBA handlers without invoking the host operation again. Break-on-error pauses
before handler delivery; continue applies it once. Fault inspection is read-only. Same-line
statements retain separate checkpoints. A retry can intentionally repeat effects inside the faulting
statement; effects of earlier completed statements are not replayed by ordinary continuation.

## IR 2 extension

IR_VERSION stays 1 for old clients; IR2_VERSION is 2. `ir1_structural()` retains old features/opcodes;
`ir2_structural()` adds the extension. The compiler defaults to the new structural target and emits
IR 1 for old instructions or IR 2 when an extension feature is required. Existing IR 1 instruction
shapes are unchanged. The runtime profile is explicitly new; compile against VM capabilities.

| Instruction | Stack effect / constraints |
|---|---|
| Intrinsic {function, arguments} | n values -> value; closed identifier and accepted arity |
| OnError {target, resume_next} | empty -> empty; target is a checkpoint; target and next are exclusive |
| Resume {target, next} | empty -> terminator; exclusive optional target/next; dynamic retry otherwise |
| ErrGet(field) | -> value; closed number/description/source/erl names |
| ErrClear | empty -> empty |
| Raise(n) | 1–3 values -> empty; transfer on numbered error |
| RefElement {rank, kind} | owner reference plus rank indices -> typed element reference |
| ArrayResize {rank,kind,preserve,initialize,dynamic} | owner reference plus lower/upper pairs -> empty |
| ArrayErase | owner reference -> empty |

New feature names are intrinsics-v2, error-handlers-v2, arrays-v2, persistent-v2 and
argument-defaults-v2. SlotRole::Persistent(key) shares a bounded project key across procedures;
the verifier checks type agreement and disallows parameter/result overlap. Missing constants and WriteRef(ArgumentBinding) are
allowed only in IR 2. ArgumentBinding uses shared parameter binding rules for captured values,
including Object/Nothing copies, and requires argument-defaults-v2. Live Object and Array constants remain forbidden.

All blocks begin with empty-stack checkpoints and terminate explicitly. Handler-capable procedures
conservatively include verified Resume Next and registered handler/resume target edges in bounded
reachability analysis. This overapproximation is not a handler-mode dataflow proof. Each target
still receives an empty stack; all indexes, arities, feature/version requirements and exact maximum
stacks are verified. The compiler records logical structured successors before flattening.

## Evidence

The 20 original fixtures and 24 M4 fixtures run through the CLI with exact typed judgments. M4
fixtures declare `requirements = m4-language-v1`; their evidence remains reasoned-not-oracle.
Owning runtime/value/compiler/IR/VM/host tests cover numeric extrema, canonical aliases, stale cells,
source evaluation order, handler propagation/resumption, budgets and explicit state reset/restore.
The native/WASI smoke runs arrays, Currency, Date, exact LongLong, optional/named calls, handlers
and project-state transfer alongside all original M3 debugger/host/cancellation checks.
External corpus candidates remain NOT_RUN. Microsoft oracle goldens remain an M9 obligation.
