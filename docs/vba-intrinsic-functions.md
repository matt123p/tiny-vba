# VBA intrinsic functions — M6A

Updated 2026-11-01. Status: planned; implementation and acceptance are not claimed.
Owner: tiny-vba language/runtime, with explicit host services supplied by embedding
applications. **M6A must be accepted before M7 starts.** Existing M7/M8 IDs stay
stable. See [the implementation plan](implementation-plan.md) and
[ADR 0033](decisions/0033-intrinsics-before-package-acceptance.md).

Implementation entry point: [M6A framework and Luna handoff](m6a-implementation-framework.md),
with exact code paths, shared extension contracts, dependent work packets and
StrComp test vectors. [ADR 0034](decisions/0034-intrinsic-implementation-framework.md)
records the design; the [work ledger](m6a-work-ledger.md) tracks execution.

## Purpose and scope

The reported `unknown procedure StrComp` is a missing shared-language intrinsic,
not an Excel workbook-object member. GridCraft currently pins tiny-vba `f680441`,
whose intrinsic registry does not contain StrComp. Worksheet-function coverage
does not establish VBA intrinsic coverage.

This milestone implements and audits VBA's intrinsic library, prioritizing
functions unavailable as built-in worksheet formulas. Include overlapping names
because signatures, coercion, errors and evaluation can differ. Inventory functions
already implemented as well as missing ones; a recognized name is not proof of
complete semantics. Do not route VBA calls through GridCraft's formula engine.

Functions with differently named worksheet equivalents, and same-name functions
with different behaviour, are required M6A scope. The comparison matrix below
is part of the inventory and acceptance work, not an optional worksheet appendix.

Public references, consulted 2026-11-01:

- [Microsoft VBA function index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications)
- [Microsoft worksheet function index](https://support.microsoft.com/en-us/excel/excel-functions-alphabetical)
- [StrComp contract](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/strcomp-function)
- [Type conversion functions](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions)

These are specification sources, not fresh Office execution evidence. Use
MS-VBAL and each function's public reference to resolve individual contracts.

## Inventory and implementation groups

The following is the required seed inventory, not an audited exhaustive catalogue.
M6A.1 must reconcile the public index, related function pages, syntax variants and
actual registry into a per-function matrix in this document. Do not assume that
every name here is missing or that a worksheet analogue has identical semantics.

| Group | Names to inventory and implement/audit |
|---|---|
| Comparison, search and text | StrComp, InStr, InStrRev, Replace, Left, Right, Mid, Len, LCase, UCase, Trim, LTrim, RTrim, Space, String, StrReverse, StrConv |
| Character and byte variants | Asc, AscW, AscB, Chr, ChrW, ChrB, LeftB, RightB, MidB, LenB; documented `$` string-returning forms |
| Conversion and formatting | CBool, CByte, CCur, CDate, CDbl, CDec, CInt, CLng, CLngLng, CLngPtr, CSng, CStr, CVar, CVErr, Val, Str, Hex, Oct, Format, FormatCurrency, FormatDateTime, FormatNumber, FormatPercent |
| Value/type inspection | IsArray, IsDate, IsEmpty, IsError, IsMissing, IsNull, IsNumeric, IsObject, TypeName, VarType, Error |
| Arrays and selection | Array, LBound, UBound, Split, Join, Filter, Choose, IIf, Switch, Partition |
| Numeric | Abs, Atn, Cos, Exp, Int, Fix, Log, Rnd, Sgn, Sin, Sqr, Tan, Round; Randomize as the related statement |
| Date and time | Date, Time, Now, Timer, DateAdd, DateDiff, DatePart, DateSerial, DateValue, TimeSerial, TimeValue, Day, Month, Year, Hour, Minute, Second, Weekday, MonthName, WeekdayName |
| Financial | DDB, FV, IPmt, IRR, MIRR, NPer, NPV, Pmt, PPmt, PV, Rate, SLN, SYD |
| Colour and dynamic invocation | RGB, QBColor, CallByName |
| Explicit host/environment services | MsgBox, InputBox, DoEvents, Command, Environ, Dir, CurDir, EOF, FileAttr, FileDateTime, FileLen, FreeFile, GetAttr, Input, Loc, LOF, Seek, GetSetting, GetAllSettings |
| Platform/restricted facilities | Shell, CreateObject, GetObject, IMEStatus, MacID, MacScript; classify platform availability and existing policy restrictions |
| Context-dependent output | Spc, Tab and existing Debug.Print lowering; distinguish function syntax from output statements |

Record each spelling/variant's signature and defaults, argument names, return tag,
module-option dependencies, platform/locale requirements, worksheet-name overlap,
owning layer, present support/limitations, implementation task, tests and evidence.
Use separate columns for implemented capability and observed test outcome.
Mark unexamined rows NOT_AUDITED and unexecuted scenarios NOT_RUN. Every row must
have a disposition before milestone acceptance; all portable core rows above are
required implementation work, including fixes to existing partial support.

The current `vba-values::Intrinsic` registry and `vba-runtime` implementation are
the starting code audit, together with binder/lowering/VM and IDE call paths.
Internal enum entries such as ArrayBaseOne are not additional public VBA functions.

## Worksheet equivalents and semantic differences

These are conceptual counterparts, not interchangeable implementations. Some
worksheet alternatives require recent Excel versions or several composed
functions. M6A.1 must expand grouped entries into per-function inventory rows and
record the applicable version/platform and exact reference contract. Every VBA
function below is already included in the implementation groups above.

| VBA function(s) | Worksheet counterpart(s) | Difference to implement and test |
|---|---|---|
| LCase, UCase | LOWER, UPPER | Different names; preserve VBA coercion, Null propagation and locale rules. |
| InStr, InStrRev | FIND, SEARCH; composed reverse search | VBA supports comparison modes and returns 0 when not found; FIND/SEARCH raise a worksheet error. InStrRev searches backwards. |
| StrComp | EXACT; composed comparison expressions | StrComp returns ordering (-1/0/1) or Null and supports comparison modes; EXACT returns case-sensitive equality only. |
| Replace | SUBSTITUTE, REPLACE | VBA replaces matching substrings with start/count/compare options; SUBSTITUTE selects matching occurrences, while worksheet REPLACE replaces by character position. |
| Trim, LTrim, RTrim | TRIM; composed expressions | VBA Trim removes edge spaces and retains internal repeats; worksheet TRIM also collapses internal runs. LTrim/RTrim remove only one edge. |
| Space, String | REPT | VBA constructs spaces or repeats a character; REPT repeats an entire supplied text value. Audit String's character-code and string arguments. |
| Chr, ChrW, Asc, AscW | CHAR, UNICHAR, CODE, UNICODE | Code-page versus UTF-16/code-point contracts differ. Worksheet ASC is a width-conversion function, not VBA Asc's character-code operation. |
| StrConv | PROPER and width/case conversions | VBA offers several conversion modes and locale/platform dependencies; no single worksheet function covers them all. |
| Split, Join | TEXTSPLIT, TEXTJOIN | Delimiter/limit/compare options, array shape/lower bounds and empty-item handling differ; worksheet availability is version-dependent. |
| Filter | FILTER | VBA filters a one-dimensional string array by substring with include/compare options; worksheet FILTER uses an inclusion array. |
| Array, LBound, UBound | Array constants, ROWS, COLUMNS | VBA arrays have explicit dimensions and lower bounds; worksheet dimensions do not reveal VBA subscripts. |
| CInt, CLng and other C* conversions | VALUE, NUMBERVALUE, INT, TRUNC and coercion | VBA conversions produce specific types with width/overflow rules. Integer conversions round; INT/TRUNC are not equivalent conversions. |
| Val | VALUE, NUMBERVALUE | Val parses a numeric prefix using its own syntax; worksheet text-to-number conversion and locale handling differ. |
| Format, FormatCurrency, FormatDateTime, FormatNumber, FormatPercent | TEXT, DOLLAR, FIXED | Format grammars, optional arguments, locale and date/number handling differ; output is text rather than a cell number format. |
| Hex, Oct | DEC2HEX, DEC2OCT, BASE | Signed representation, accepted ranges and result widths differ; do not reuse worksheet limits. |
| IsNumeric, IsEmpty, IsError | ISNUMBER, ISBLANK, ISERROR | Test VBA value states, coercion and Variant errors separately from worksheet cell/reference rules. IsNumeric accepts some numeric strings that ISNUMBER rejects. |
| CVErr, Error | Error values, ERROR.TYPE | CVErr constructs a Variant error; Error returns an error description. ERROR.TYPE classifies worksheet errors instead. |
| IIf, Switch, Choose | IF, IFS, SWITCH, CHOOSE | IIf evaluates both result arguments. VBA Switch selects by condition/value pairs; worksheet SWITCH matches an expression against cases. Audit argument evaluation and index coercion for each. |
| Sqr, Atn, Sgn | SQRT, ATAN, SIGN | Different names; verify domain errors, coercion and exact VBA result types. |
| Log | LN, LOG | VBA Log is the natural logarithm, like LN; worksheet LOG defaults to base 10 and accepts a base argument. |
| Fix, Int | TRUNC, INT | Fix truncates toward zero; Int rounds down, including negatives. VBA typing/coercion differs from worksheet numbers. |
| Round | ROUND | VBA uses ties-to-even rounding; worksheet ROUND rounds ties away from zero. Assert positive/negative halfway cases and precision. |
| Rnd | RAND | VBA has argument-dependent seeded generator state, paired with Randomize; RAND has no VBA-style seed argument. |
| Date, DateSerial | TODAY, DATE | VBA Date returns the current date; worksheet DATE constructs one. DateSerial is the constructor analogue; audit year/range/calendar rules. |
| Time, TimeSerial, Now | MOD(NOW(),1), TIME, NOW | VBA Time returns the current time; worksheet TIME constructs one. Preserve VBA Date tags and explicit clock injection. |
| DateAdd, DateDiff, DatePart | EDATE, DATEDIF, date arithmetic/extraction | No universal one-to-one mapping: interval boundaries, week rules, calendar options and result types differ. |
| DateValue, TimeValue, Day, Month, Year, Hour, Minute, Second, Weekday | DATEVALUE, TIMEVALUE, DAY, MONTH, YEAR, HOUR, MINUTE, SECOND, WEEKDAY | Audit locale, serial-date epoch, return types and optional weekday/calendar arguments despite matching names. |
| MonthName, WeekdayName | TEXT with date formats | VBA accepts month/weekday indexes and abbreviation/week-start options, rather than a worksheet date serial plus format. |
| DDB, FV, IPmt, IRR, MIRR, NPer, NPV, Pmt, PPmt, PV, Rate, SLN, SYD | Same-name financial functions | Audit VBA signatures/defaults, array versus range inputs, timing, convergence, errors and result types individually. Matching names alone do not establish equivalence. |
| Abs, Cos, Exp, Sin, Tan, Left, Right, Mid, Len | Same-name worksheet functions | Audit VBA coercion/Null/error behaviour, argument defaults and numeric/string return tags even where ordinary results agree. |

Function-level sources supplementing the public indexes:
[VBA Round](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/round-function),
[Excel/VBA Trim distinction](https://learn.microsoft.com/en-us/office/vba/api/excel.worksheetfunction.trim),
[VBA Replace](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/replace-function),
[VBA Log](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/log-function)
and [VBA IIf](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/iif-function).
These comparisons guide required contract tests; no differential Office run is claimed.

## Semantic contract to establish

- Preserve UTF-16 strings, numeric widths, Currency/Date, arrays/lower bounds and
  Empty/Null/Missing/Nothing/CVErr. Reuse shared coercion and error handling.
- Specify positional/named/omitted arguments, defaults, evaluation order and
  case-insensitive binding, qualification and shadowing under VBA's rules.
  Include documented `$`, byte and wide-character variants separately.
- Specify binary/text comparison, Option Compare, code page/locale/calendar,
  pointer width and platform restrictions. ASCII lowercasing is not a complete
  locale-aware comparison implementation.
- Audit same-name differences: VBA Filter versus worksheet FILTER, VBA Date
  versus worksheet DATE, and VBA Round's rounding rules. IIf evaluates both
  branches; worksheet IF is not a substitute. Formatting and conversions require
  their own contracts rather than aliases to worksheet TEXT/VALUE.
- Inject clock, randomness, locale and prompts; define deterministic seeded/time
  fixtures and host service lifetimes. Host-dependent functionality must use
  explicit bounded interfaces, with refusal when the service is unavailable.
- Bound allocations, search/replacement work, output length and numeric iteration.
  Integrate cancellation, pause/resume and runtime error propagation without
  repeating effects. Inspection/watch evaluation must remain read-only.

### First implementation slice: StrComp

Implement `StrComp(string1, string2, [compare])` through normal shared binding,
lowering and execution. The documented result is a Variant containing an Integer
(-1, 0 or 1), or Null when a string argument is Null. Omitted comparison uses
the module's Option Compare; -1 requests that option explicitly, 0 is binary and
1 is text. Null comparison raises an error; value 2 is Access-specific.

Tests must cover equal/less/greater, empty strings, mixed case, non-ASCII and
UTF-16 boundaries, coercion, Null operands, invalid/Null comparison, module
defaults and explicit comparison constants. Establish exact error numbers and
error precedence from the applicable reference/evidence. Do not guess locale
ordering or silently implement Access database comparison in Excel/Word profiles.

## Ownership and platform boundaries

Syntax/semantics own source forms and binding; values/runtime own intrinsic
identities, typed results, coercion and pure algorithms; compiler/IR/VM own
verified lowering, capability checks and execution state. The shared IDE owns
matching completion/signature help and safe evaluation policy. Add versioned
IR/runtime capabilities only where the existing contract requires them.

The VM must not acquire ambient filesystem, network, shell or COM access.
Embedding shells/forks provide explicit host services. Office objects,
WorksheetFunction, workbook selection/clipboard/sort, document trust, packaging
and undo/recalculation remain fork-owned. This milestone does not authorize
arbitrary COM/DLL activation, P-code or third-party ActiveX controls.

M6A.1 must classify each host/platform row as an existing service, a required
bounded service implementation, or an explicit refusal required by current scope.
Publish the exact function, platform, reason and diagnostic for each refusal;
do not count refusal as implemented VBA behaviour. An unavailable service must
not appear merely as an unknown user procedure. Platform-specific and restricted
rows cannot silently disappear from the inventory or become implicit M7 work.

## Acceptance before M7

1. Publish the reconciled per-function inventory with no NOT_AUDITED rows,
   signatures, scope dispositions, platform limits and linked evidence.
   Include every worksheet-counterpart row above, with different-name and
   different-behaviour mappings expanded and linked to the owning M6A task.
2. Implement the portable core groups and required injected services; retain
   explicit documented refusals for unsupported platforms/restricted facilities.
   No required implementation row may remain UNSUPPORTED or NOT_RUN at sign-off.
3. Assert exact types, values, errors and effects, including negative paths,
   boundary/budget cases, naming/argument variants and debugger pause/resume.
   Add discriminating cases for each documented worksheet difference (for
   example Trim's internal spaces, Round's ties, Log's base and InStr's no-match
   result); do not treat a worksheet implementation's tests as VBA acceptance.
   Retain all prior language/import/editor regressions.
4. Run shared `cargo xtask ci`, `cargo xtask wasm` and `cargo xtask wasm-run`;
   run affected UI/native/web gates when code aids or interaction change.
5. Publish the shared revision and verify pinned GridCraft and WordCraft adapter
   suites, required application CI/WASM gates and authored intrinsic scenarios
   in both real hosts, including a GridCraft worksheet UDF using StrComp.
6. Record acceptance and remaining explicit platform/policy limitations. Keep
   authored, inherited and fresh Office evidence distinct. Fresh Excel/Word
   observations remain M8.5 under ADR 0032; this milestone does not claim them.

Next task: M6A.1 inventory and contract audit, followed by M6A.2 StrComp. This
planning change adds no runtime support and does not execute user workbooks.
