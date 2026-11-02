# VBA intrinsic functions — M6A

Updated 2026-11-02. Status: M6A.8 cross-host verification is recorded, but M6A acceptance remains incomplete; required intrinsic rows and the shared tiny-vba CI gate remain open.
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
not an Excel workbook-object member. GridCraft and WordCraft are pinned to shared
tiny-vba revision `970ae9b81be0e015f89f08068551aa164db36895`; the shared runtime
provides deterministic UTF-16 Binary comparison, while Text comparison requires
an explicitly injected provider. Worksheet-function coverage does not establish
VBA intrinsic coverage.

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

## M6A.1 audited seed inventory (F0)

Audited against the current registry/runtime/binder/compiler/VM/IDE tree on
2026-11-01. `Partial` means a runtime arm exists but does not satisfy the full
reference contract (not a compatibility pass). `Absent` means there is no public
registry/runtime implementation. Portable functions are required work; host and
restricted functions are classified further in their owning M6A.6 packets. The
full signature contract remains linked to each function's Microsoft VBA reference
page through the [VBA function index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications);
each function packet must pin its exact page and vectors before implementation.
Every scenario not explicitly listed as an executed test remains `NOT_RUN`.

| Public spelling | Signature / call shape (reference form) | Current code audit | Disposition / packet | Worksheet counterpart and required discriminator | Evidence |
|---|---|---|---|---|---|
| StrComp | `(string1, string2, [compare])` | Implemented through IR 8; comparison requires an injected provider | S1 | EXACT; Variant(Integer) -1/0/1 or Null for Null string | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/strcomp-function) |
| InStr | `([start,] string1, string2, [compare])` | Implemented through IR 8; Text comparison requires an injected provider | S2 | FIND/SEARCH; no match returns 0; VBA returns one-based position | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/instr-function) |
| InStrRev | `(stringcheck, stringmatch, [start, [compare]])` | Implemented through IR 8; Text comparison requires an injected provider | S2 | Reverse composed search; reverse indexing and no-match 0; omitted compare defaults to Binary | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/instrrev-function) |
| Replace | `(expression, find, replace, [start, [count, [compare]]])` | Implemented through IR 8; Text comparison requires an injected provider | S3 | SUBSTITUTE/REPLACE; substring replacement, start/count and compare | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/replace-function) |
| Left | `(string, length)` | Implemented; Variant(String) | S2 complete | LEFT; plain form preserves Variant return identity | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Left$ | `(string, length)` | Implemented; fixed String | S10 | LEFT; `$` preserves String return identity | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Right | `(string, length)` | Implemented; Variant(String) | S2 complete | RIGHT; plain form preserves Variant return identity | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Right$ | `(string, length)` | Implemented; fixed String | S10 | RIGHT; `$` preserves String return identity | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Mid | `(string, start, [length])` | Implemented; Variant(String) | S2 complete | MID; UTF-16 indexing and optional length | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Mid$ | `(string, start, [length])` | Implemented; fixed String | S10 | MID; `$` preserves String return identity | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Len | `(expression)` | Implemented; coerces scalar values to text and counts UTF-16 code units; Null propagates | S8/S13 | LEN; VBA Variant/coercion rules | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| LenB | `(expression)` | Implemented; byte strings use preserved bytes, ordinary strings use the configured code page (UTF-16LE in the unconfigured profile); returns Long | S13 | LEN; byte length depends on representation | [Len/LenB](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/len-function) |
| LCase | `(string)` | Implemented; Unicode case mapping, Turkish/Azeri special casing and expansions; Null propagates; fixed String return | S8 | LOWER; locale comes from runtime context | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/lcase-function) |
| UCase | `(string)` | Implemented; Unicode case mapping, Turkish/Azeri special casing and expansions; Null propagates; Variant(String) return | S8 | UPPER; locale comes from runtime context | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ucase-function) |
| Trim | `(string)` | Implemented; Variant(String), Null propagates | S11 audit | TRIM; removes U+0020 edges | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ltrim-rtrim-and-trim-functions) |
| Trim$ | `(string)` | Implemented; fixed String, Null propagates | S11 | TRIM; `$` preserves String return identity | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ltrim-rtrim-and-trim-functions) |
| LTrim | `(string)` | Implemented; Variant(String), Null propagates | S6/S11 audit | TRIM composed; removes leading U+0020 only | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ltrim-rtrim-and-trim-functions) |
| LTrim$ | `(string)` | Implemented; fixed String, Null propagates | S11 | Leading U+0020 only; `$` preserves String return identity | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ltrim-rtrim-and-trim-functions) |
| RTrim | `(string)` | Implemented; Variant(String), Null propagates | S6/S11 audit | TRIM composed; removes trailing U+0020 only | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ltrim-rtrim-and-trim-functions) |
| RTrim$ | `(string)` | Implemented; fixed String, Null propagates | S11 | Trailing U+0020 only; `$` preserves String return identity | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ltrim-rtrim-and-trim-functions) |
| Space | `(number)` | Implemented; Variant(String), 32,768-unit engine limit | S7 | REPT; creates spaces with count rules | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/space-function) |
| Space$ | `(number)` | Implemented; fixed String, 32,768-unit engine limit | S12 | REPT; `$` preserves String return identity | [VBA](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/returning-strings-from-functions) |
| String | `(number, character)` | Implemented; Variant(String), 32,768-unit engine limit | S7 | REPT; character code or first character semantics | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/string-function) |
| String$ | `(number, character)` | Implemented; fixed String, 32,768-unit engine limit | S12 | REPT; `$` preserves String return identity | [VBA](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/returning-strings-from-functions) |
| StrReverse | `(expression)` | Implemented; reverses UTF-16 code units; Null error policy is engine error 94 | S6 | No direct counterpart; UTF-16 code unit and surrogate behavior | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/strreverse-function) |
| StrConv | `(string, conversion, [LCID])` | Implemented for case, width, kana and Unicode/code-page byte conversions; locale/LCID and code page come from runtime context; unsupported locale conversions return explicit errors | S13 | PROPER/width conversions; conversion flags and LCID | [StrConv](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/strconv-function) |
| Asc | `(string)` | Implemented for configured Windows-1252, Shift-JIS and supported encoding_rs code pages; unconfigured profile supports ASCII only | S13 | CODE is not code-page compatible | [Asc](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/asc-function) |
| AscW | `(string)` | Implemented; returns the first UTF-16 code unit as signed Integer | S9 | UNICODE; UTF-16 code unit, including surrogate behavior | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/asc-function) |
| AscB | `(string)` | Implemented; extracts first byte from preserved byte strings or configured code-page representation | S13 | No equivalent; byte extraction | [Asc/AscB](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/asc-function) |
| Chr / Chr$ | `(charcode)` | Implemented for configured code pages; DBCS values use two-byte code values; unconfigured profile supports ASCII only | S13 | CHAR differs by code page and range | [Chr](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/chr-function) |
| ChrW | `(charcode)` | Implemented; accepts signed/unsigned 16-bit values and returns one UTF-16 code unit | S9 | UNICHAR differs at UTF-16 boundaries | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/chr-function) |
| ChrB / ChrB$ | `(charcode)` | Implemented; constructs one-byte strings and preserves raw byte identity | S13 | No equivalent; byte construction | [Chr/ChrB](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/chr-function) |
| LeftB / LeftB$ | `(string, length)` | Implemented; slices raw or configured encoded bytes; byte string identity is preserved | S13 | LEFT; byte rather than character slicing | [Left/LeftB](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/left-function) |
| RightB / RightB$ | `(string, length)` | Implemented; slices raw or configured encoded bytes; byte string identity is preserved | S13 | RIGHT; byte rather than character slicing | [Right/RightB](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/right-function) |
| MidB / MidB$ | `(string, start, [length])` | Implemented; one-based byte slicing preserves raw bytes including partial DBCS sequences | S13 | MID; byte rather than character slicing | [Mid/MidB](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/mid-function) |
| CBool | `(expression)` | Implemented for Boolean, numeric and invariant True/False strings; Null errors 94 | C1 | VALUE/coercion; Boolean rules | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CByte | `(expression)` | Implemented; ties-even rounding and Byte range | C1 | VALUE; Byte rounding and range | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CCur | `(expression)` | Implemented for exact decimal text and supported numeric values; locale text remains limited | C1 | VALUE; fixed-point scale/rounding | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CDate | `(expression)` | Numeric serials; String parsing uses injected locale/calendar date service | C2 | DATEVALUE; parsing and serial epoch | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CDbl | `(expression)` | Implemented for supported numeric and invariant numeric-string inputs | C1 | VALUE; locale/coercion | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CDec | `(expression)` | Implemented exactly for Decimal, integral, Boolean, Empty and Currency values; String parsing uses injected locale service; Single/Double/Date conversion remains Unsupported to avoid invented rounding | C5 | NUMBERVALUE; exact decimal precision | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CInt | `(expression)` | Implemented; ties-even rounding and Integer range | C1 | INT/TRUNC not equivalent; ties-even and width | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CLng | `(expression)` | Implemented; ties-even rounding and Long range | C1 | INT/TRUNC not equivalent; Long width | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CLngLng | `(expression)` | Implemented; exact integer-string path and LongLong range; target availability policy remains | C1 | INT/TRUNC; 64-bit result | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CLngPtr | `(expression)` | Implemented using explicit target width; returns Long on 32-bit and LongLong on 64-bit targets | C5 | No equivalent; pointer-width result | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CSng | `(expression)` | Implemented for supported numeric and invariant numeric-string inputs | C1 | VALUE; Single precision | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CStr | `(expression)` | Locale-dependent Date formatting uses injected locale/calendar service; CVErr formatting remains engine-defined | C3 | TEXT; type and formatting rules | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CVar | `(expression)` | Implemented as Variant identity preserving scalar tag/state | C1 | VALUE; preserve Variant subtype/state | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CVErr | `(errornumber)` | Implemented; constructs Variant Error; accepted numeric range remains engine policy | C1 | ERROR.TYPE classifies; does not construct Variant Error | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Val | `(string)` | Implemented for decimal and `&H`/`&O` numeric prefixes; spaces, tabs and linefeeds are ignored; parsing is invariant | C6 | VALUE/NUMBERVALUE; numeric-prefix grammar | [Val](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/val-function) |
| Str / Str$ | `(number)` | Implemented for supported numeric tags with leading sign-space and invariant numeric text; plain result is Variant(String), `$` is fixed String | C6 | TEXT; leading sign-space and formatting | [Str](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/str-function) |
| Hex / Hex$ | `(number)` | Implemented with signed 32-bit bounds, rounded integer input, and two's-complement negative rendering; plain result is Variant(String), `$` is fixed String | C6 | DEC2HEX/BASE; signed widths and limits | [Hex](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/hex-function) |
| Oct / Oct$ | `(number)` | Implemented with signed 32-bit bounds, rounded integer input, and two's-complement negative rendering; plain result is Variant(String), `$` is fixed String | C6 | DEC2OCT/BASE; signed widths and limits | [Oct](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/oct-function) |
| Format / Format$ | `(expression, [format, [firstdayofweek, [firstweekofyear]]])` | Routed through injected locale/calendar formatting service; optional slots remain explicit; `$` has fixed String return tag. No built-in format grammar/provider | C7 | TEXT; VBA format grammar and locale | [Format](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/format-function-visual-basic-for-applications) |
| FormatCurrency | `(expression, [numdigitsafterdecimal, [includeleadingdigit, [useparensfornegative, [groupdigits]]]])` | Routed through injected locale/calendar formatting service; no built-in regional formatter | C7 | DOLLAR; currency locale/rounding | [FormatCurrency](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/formatcurrency-function) |
| FormatDateTime | `(date, [namedformat])` | Routed through injected locale/calendar formatting service; no built-in regional formatter | C7 | TEXT; named format and regional output | [FormatDateTime](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/formatdatetime-function) |
| FormatNumber | `(expression, [numdigitsafterdecimal, [includeleadingdigit, [useparensfornegative, [groupdigits]]]])` | Routed through injected locale/calendar formatting service; no built-in regional formatter | C7 | FIXED; locale and grouping | [FormatNumber](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/formatnumber-function) |
| FormatPercent | `(expression, [numdigitsafterdecimal, [includeleadingdigit, [useparensfornegative, [groupdigits]]]])` | Routed through injected locale/calendar formatting service; no built-in regional formatter | C7 | TEXT; percent scaling/locale | [FormatPercent](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/formatpercent-function) |
| IsArray | `(varname)` | Implemented; tests Array value tag | C1 | ISBLANK/array constants; VBA Variant tags | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsDate | `(expression)` | Implemented for Date/numeric values and String values through injected locale/calendar service | C2 | DATEVALUE; accepted text and serial range | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/isdate-function) |
| IsEmpty | `(expression)` | Implemented; tests Empty state | C1 | ISBLANK; Empty Variant state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsError | `(expression)` | Implemented; tests Variant Error state | C1 | ISERROR; Variant error values | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsMissing | `(argumentname)` | Implemented for Missing argument sentinel | C1 | ISBLANK; only optional Variant procedure arguments | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsNull | `(expression)` | Implemented; tests Null state | C1 | ISBLANK; Null state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsNumeric | `(expression)` | Implemented for numeric scalar values and provider-recognized numeric strings; Date and nonnumeric Variant states return False | C3 | ISNUMBER; numeric strings and Null | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/isnumeric-function) |
| IsObject | `(expression)` | Implemented; tests Object/Nothing tags | C1 | IS; object identity | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| TypeName | `(varname)` | Implemented for built-in scalar/array tags, Decimal, Object and Nothing; concrete host class names remain unavailable | C1/C4 | No direct counterpart; exact Variant/type name | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/typename-function) |
| VarType | `(varname)` | Implemented for built-in scalar/array tags and Decimal; user-defined-type tags unavailable | C1/C4 | No direct counterpart; exact subtype flags | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/vartype-function) |
| Error / Error$ | `([errornumber])` | Routed through injected formatting service; omitted number receives current runtime error number; descriptions/localization are provider-defined. Bare `Error()` expression syntax is currently rejected by parser | C8 | ERROR.TYPE; returns localized description | [Error](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/error-function) |
| Array | `([arglist])` | Implemented as a zero-based Variant array by default; unqualified spelling follows module Option Base, while `VBA.Array` remains zero-based; zero-argument array has zero elements | A1 | Array constants/ROWS; explicit bounds and subtype | [Array](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/array-function) |
| LBound | `(arrayname, [dimension])` | Implemented for engine arrays; omitted dimension defaults to 1; returns Long; dimension below 1 errors 5; missing dimension errors 9; non-array errors 13 | A1 | ROWS/COLUMNS do not report VBA subscripts | [LBound](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/lbound-function), [Error 5](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/invalid-procedure-call-or-argument-error-5), [Error 9](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/subscript-out-of-range-error-9) |
| UBound | `(arrayname, [dimension])` | Implemented for engine arrays; omitted dimension defaults to 1; returns Long; dimension below 1 errors 5; missing dimension errors 9; zero-element Array() yields its configured empty bound | A1 | ROWS/COLUMNS do not report VBA subscripts | [UBound](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ubound-function), [Error 5](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/invalid-procedure-call-or-argument-error-5), [Error 9](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/subscript-out-of-range-error-9) |
| Split | `(expression, [delimiter, [limit, [compare]]])` | Implemented through IR 8; Text comparison requires injected provider | S4 | TEXTSPLIT; limit/lower bound/empty parts | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/split-function) |
| Join | `(sourcearray, [delimiter])` | Implemented for one-dimensional arrays; String result; multidimensional arrays return Type mismatch | S4 | TEXTJOIN; array coercion and bounds | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/join-function) |
| Filter | `(sourcearray, match, [include, [compare]])` | Implemented through IR 8; Text comparison requires injected provider | S5 | FILTER; VBA returns zero-based String array and filters by substring | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/filter-function) |
| Choose | `(index, choice-1, [choice-2, ...])` | Implemented; rounds index ties-to-even, eagerly evaluates all choices; out-of-range and Null index return Null; selected value's Variant subtype is preserved | A2 | CHOOSE; VBA index coercion/evaluation | [Choose](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/choose-function) |
| IIf | `(expr, truepart, falsepart)` | Implemented; condition chooses return value after both branches evaluate left-to-right; Null condition follows False path; selected Variant subtype is preserved | A2 | IF; both branches are evaluated | [IIf](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/iif-function) |
| Switch | `(expr-1, value-1, [expr-2, value-2, ...])` | Implemented; requires expression/value pairs; returns value for first True condition or Null when none; argument expressions evaluate eagerly in source order | A2 | IFS/SWITCH differ in match form and evaluation | [Switch](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/switch-function) |
| Partition | `(number, start, stop, interval)` | Implemented; rounds each numeric argument ties-to-even; Null in any argument returns Null; validates nonnegative start, stop greater than start, and interval at least 1; returns padded Variant(String) ranges | A3 | No direct counterpart; range string contract | [Partition](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/partition-function) |
| Abs | `(number)` | Implemented for supported scalar numeric types; Null returns Null; result retains the input numeric type; Empty converts to Integer zero | N1 | ABS; VBA coercion/Null and result tag | [Abs](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/abs-function) |
| Atn | `(number)` | Implemented with numeric coercion, Null propagation, Double result, and non-finite/domain handling | N1 | ATAN; type/domain rules | [Atn](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/atn-function) |
| Cos | `(number)` | Implemented in radians with numeric coercion, Null propagation and Double result | N1 | COS; VBA coercion/result | [Cos](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/cos-function) |
| Exp | `(number)` | Implemented with numeric coercion, Null propagation and Double result; non-finite overflow reports error 6 | N1 | EXP; overflow contract | [Exp](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/exp-function) |
| Int | `(number)` | Implemented; floors negative fractional values; preserves supported integral/Currency/Single/Date result tags; Null returns Null | N1 | INT; negative input behavior | [Int/Fix](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/int-fix-functions) |
| Fix | `(number)` | Implemented; truncates toward zero; preserves supported integral/Currency/Single/Date result tags; Null returns Null | N1 | TRUNC; negative input behavior | [Int/Fix](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/int-fix-functions) |
| Log | `(number)` | Implemented as natural log; numeric coercion, Null propagation, Double result; nonpositive domain reports error 5 | N1 | LN/LOG; natural log and domain | [Log](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/log-function) |
| Rnd | `([number])` | Implemented with VBA-compatible 24-bit LCG, seeded per VM session; negative reseed and zero repeat semantics | M6A.5/R1 | RAND; seed/state/repeat semantics | [Rnd](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/rnd-function) |
| Randomize (statement) | `[number]` | Implemented; explicit numeric seed or injected clock seed; session reset restores initial state | M6A.5/R1 | RAND; explicit state mutation | [Randomize](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/randomize-statement) |
| Sgn | `(number)` | Implemented; returns Integer -1/0/1; Null returns Null | N1 | SIGN; VBA result tag/coercion | [Sgn](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/sgn-function) |
| Sin | `(number)` | Implemented in radians with numeric coercion, Null propagation and Double result | N1 | SIN; coercion/result | [Sin](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/sin-function) |
| Sqr | `(number)` | Implemented with numeric coercion, Null propagation and Double result; negative domain reports error 5 | N1 | SQRT; negative-domain error | [Sqr](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/sqr-function) |
| Tan | `(number)` | Implemented in radians with numeric coercion, Null propagation and Double result | N1 | TAN; overflow/precision | [Tan](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/tan-function) |
| Round | `(expression, [numdecimalplaces])` | Implemented with ties-to-even, Double result, Null propagation and positive/negative decimal-place counts | N1 | ROUND; tie direction/precision/type | [Round](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/round-function) |
| Date / Date$ | `()` | Implemented; injected clock; Date returns Date serial and Date$ requires locale formatter | M6A.5/D1 | TODAY; now versus constructor | [Date](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/date-function) |
| Time / Time$ | `()` | Implemented with subsecond Date serial and injected clock; Time$ requires locale formatter | M6A.5/D1 | MOD(NOW(),1); Date tag and clock | [Time](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/time-function) |
| Now | `()` | Implemented with injected clock and OLE Automation Date serial | M6A.5/D1 | NOW; serial/tag/timezone | [Now](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/now-function) |
| Timer | `()` | Implemented as Single seconds since midnight using injected clock | M6A.5/D1 | No direct counterpart; elapsed seconds | [Timer](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/timer-function) |
| DateAdd | `(interval, number, date)` | Implemented for yyyy/q/m/y/d/w/ww/h/n/s; Gregorian profile; month-end clamps and subsecond preservation | M6A.5/D4 | EDATE/date arithmetic; interval boundaries | [DateAdd](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/dateadd-function) |
| DateDiff | `(interval, date1, date2, [firstdayofweek, [firstweekofyear]])` | Implemented for yyyy/q/m/d/y/w/ww/h/n/s with week options; Gregorian profile | M6A.5/D4 | DATEDIF/date arithmetic; boundary rules | [DateDiff](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/datediff-function) |
| DatePart | `(interval, date, [firstdayofweek, [firstweekofyear]])` | Implemented for yyyy/q/m/y/d/w/ww/h/n/s; Gregorian profile and explicit week options | M6A.5/D4 | date extraction; week rules | [DatePart](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/datepart-function) |
| DateSerial | `(year, month, day)` | Implemented with Gregorian normalization, VBA two-digit year mapping, and year 100–9999 range | M6A.5/D2 | DATE; normalization/range | [DateSerial](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/dateserial-function) |
| DateValue | `(date)` | Numeric/Date serials supported; String parsing requires injected locale/calendar provider | M6A.5/D2 | DATEVALUE; accepted forms and epoch | [DateValue](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/datevalue-function) |
| TimeSerial | `(hour, minute, second)` | Implemented with signed component normalization and Date tag | M6A.5/D2 | TIME; normalization and Date tag | [TimeSerial](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/timeserial-function) |
| TimeValue | `(time)` | Numeric/Date serials supported; String parsing requires injected locale/calendar provider | M6A.5/D2 | TIMEVALUE; accepted forms and Date tag | [TimeValue](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/timevalue-function) |
| Day | `(date)` | Implemented for Date/numeric serials; String input uses injected parser | M6A.5/D3 | DAY; serial/locale | [Day](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/day-function) |
| Month | `(date)` | Implemented for Date/numeric serials; String input uses injected parser | M6A.5/D3 | MONTH; serial/locale | [Month](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/month-function) |
| Year | `(date)` | Implemented for Date/numeric serials; String input uses injected parser | M6A.5/D3 | YEAR; serial/locale | [Year](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/year-function) |
| Hour | `(time)` | Implemented for Date/numeric serials; String input uses injected parser | M6A.5/D3 | HOUR; serial/locale | [Hour](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/hour-function) |
| Minute | `(time)` | Implemented for Date/numeric serials; String input uses injected parser | M6A.5/D3 | MINUTE; serial/locale | [Minute](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/minute-function) |
| Second | `(time)` | Implemented for Date/numeric serials; String input uses injected parser | M6A.5/D3 | SECOND; serial/locale | [Second](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/second-function) |
| Weekday | `(date, [firstdayofweek])` | Implemented for Date/numeric serials, first-day option, and String parser provider | M6A.5/D3 | WEEKDAY; week-start numbering | [Weekday](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/weekday-function) |
| MonthName | `(month, [abbreviate])` | Implemented through injected locale calendar name provider | M6A.5/D3 | TEXT date format; index and locale names | [MonthName](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/monthname-function) |
| WeekdayName | `(weekday, [abbreviate, [firstdayofweek]])` | Implemented through injected locale calendar name provider | M6A.5/D3 | TEXT date format; index/week-start/locale | [WeekdayName](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/weekdayname-function) |
| DDB | `(cost, salvage, life, period, [factor])` | Implemented with optional factor default 2, bounded closed-form declining-balance calculation and salvage floor | N2 | Numerical/result/profile boundary; audited against documented signature | [DDB](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ddb-function) |
| FV | `(rate, nper, pmt, [pv, [due]])` | Implemented as Double with zero PV/due defaults; end/beginning timing | N2 | Numerical/result/profile boundary; audited against documented signature | [FV](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/fv-function) |
| IPmt | `(rate, per, nper, pv, [fv, [due]])` | Implemented as Double with zero FV/due defaults and period validation | N2 | Numerical/result/profile boundary; audited against documented signature | [IPmt](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ipmt-function) |
| IRR | `(values, [guess])` | Implemented for one-dimensional cash-flow arrays; default guess 0.1; 20 Newton iterations, 1e-7 rate tolerance | N2 | Numerical/result/profile boundary; audited against documented signature | [IRR](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/irr-function) |
| MIRR | `(values, finance_rate, reinvest_rate)` | Implemented for one-dimensional arrays with finance/reinvestment rates and positive/negative cash-flow validation | N2 | Numerical/result/profile boundary; audited against documented signature | [MIRR](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/mirr-function) |
| NPer | `(rate, pmt, pv, [fv, [due]])` | Implemented as Double with zero FV/due defaults and a separate zero-rate path | N2 | Numerical/result/profile boundary; audited against documented signature | [NPer](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/nper-function) |
| NPV | `(rate, values())` | Implemented as Double; each array/scalar cash flow is discounted from period one; accepts one-dimensional arrays | N2 | Numerical/result/profile boundary; audited against documented signature | [NPV](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/npv-function) |
| Pmt | `(rate, nper, pv, [fv, [due]])` | Implemented as Double with zero FV/due defaults; end/beginning timing | N2 | Numerical/result/profile boundary; audited against documented signature | [Pmt](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/pmt-function) |
| PPmt | `(rate, per, nper, pv, [fv, [due]])` | Implemented as Double with zero FV/due defaults and period validation | N2 | Numerical/result/profile boundary; audited against documented signature | [PPmt](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/ppmt-function) |
| PV | `(rate, nper, pmt, [fv, [due]])` | Implemented as Double with zero FV/due defaults; end/beginning timing | N2 | Numerical/result/profile boundary; audited against documented signature | [PV](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/pv-function) |
| Rate | `(nper, pmt, pv, [fv, [due, [guess]]])` | Implemented with zero FV/due defaults, guess 0.1, and 20 bounded Newton iterations | N2 | Numerical/result/profile boundary; audited against documented signature | [Rate](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/rate-function) |
| SLN | `(cost, salvage, life)` | Implemented as Double with positive-life validation | N2 | Numerical/result/profile boundary; audited against documented signature | [SLN](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/sln-function) |
| SYD | `(cost, salvage, life, period)` | Implemented as Double with period bounds validation | N2 | Numerical/result/profile boundary; audited against documented signature | [SYD](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/syd-function) |
| RGB | `(red, green, blue)` | Implemented as BGR-encoded VBA Long; components are rounded, negatives error 5, values above 255 clamp to 255 | M6A.6/H1 | RGB; VBA Long encoding and range | [RGB](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/rgb-function) |
| QBColor | `(color)` | Implemented for palette indexes 0–15; outside range errors 5 | M6A.6/H1 | No direct counterpart; fixed palette index | [QBColor](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/qbcolor-function) |
| CallByName | `(object, procname, calltype, [args()])` | Implemented as explicit host protocol dispatch; embedding validates object identity and allowlist | M6A.6/H2 | No direct counterpart; member dispatch policy | [CallByName](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/callbyname-function) |
| MsgBox | `(prompt, [buttons, [title, [helpfile, context]]])` | Explicit host service dispatch; provider defines UI policy; default host refuses | M6A.6/H3 | No worksheet analogue; button/result contract | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| InputBox | `(prompt, [title, [default, [xpos, [ypos, [helpfile, context]]]]])` | Explicit host service dispatch; omitted slots preserved; default host refuses | M6A.6/H3 | No worksheet analogue; cancellation/text result | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| DoEvents (statement/function) | `()` | Explicit host service dispatch; scheduler/yield policy belongs to embedding | M6A.6/H3 | No worksheet analogue; yield/cancellation effects | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Command / Command$ | `()` | Explicit host service dispatch; no ambient process arguments | M6A.6/H3 | No worksheet analogue; command-line exposure | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Environ / Environ$ | `(envstring)` | Explicit host service dispatch; requires provider allowlist; no ambient environment reads | M6A.6/H3 | No worksheet analogue; missing variable behavior | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Dir / Dir$ | `([pathname, [attributes]])` | Explicit host service dispatch; provider owns sandboxed namespace and enumeration state | M6A.6/H4 | No worksheet analogue; enumeration state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| CurDir / CurDir$ | `([drive])` | Explicit host service dispatch; provider owns namespace; no ambient current directory | M6A.6/H4 | No worksheet analogue; current-directory state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| EOF | `(filenumber)` | Explicit host service dispatch; file/cursor identity belongs to provider | M6A.6/H4 | No worksheet analogue; file cursor state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FileAttr | `(filenumber, returntype)` | Explicit host service dispatch; provider defines supported file modes | M6A.6/H4 | No worksheet analogue; file mode/platform | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FileDateTime | `(pathname)` | Explicit host service dispatch; provider defines timestamp profile | M6A.6/H4 | No worksheet analogue; timestamp/zone | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FileLen | `(pathname)` | Explicit host service dispatch; provider owns namespace and range behavior | M6A.6/H4 | No worksheet analogue; file length/overflow | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FreeFile | `([rangenumber])` | Explicit host service dispatch; provider owns handle allocation/lifecycle | M6A.6/H4 | No worksheet analogue; allocation/lifecycle | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| GetAttr | `(pathname)` | Explicit host service dispatch; provider defines portable attribute profile | M6A.6/H4 | No worksheet analogue; platform attribute bits | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Input / Input$ | `(number, filenumber)` | Explicit host service dispatch; provider bounds returned text and owns cursor | M6A.6/H4 | No worksheet analogue; bytes/text and cursor | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Loc | `(filenumber)` | Explicit host service dispatch; provider defines mode-specific cursor | M6A.6/H4 | No worksheet analogue; mode-dependent position | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| LOF | `(filenumber)` | Explicit host service dispatch; provider defines logical length | M6A.6/H4 | No worksheet analogue; logical record/file size | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Seek | `(filenumber)` | Explicit host service dispatch; provider defines one-based cursor state | M6A.6/H4 | No worksheet analogue; one-based cursor | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| GetSetting | `(appname, section, key, [default])` | Explicit host service dispatch; provider defines settings scope and missing-key behavior | M6A.6/H5 | No worksheet analogue; registry/service scope | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| GetAllSettings | `(appname, section)` | Explicit host service dispatch; provider returns a bounded array result | M6A.6/H5 | No worksheet analogue; two-dimensional result | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Shell | `(pathname, [windowstyle])` | Absent; prohibited ambient process launch | Explicit refusal H6: no shell authority | No worksheet analogue; side effect prohibited | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| CreateObject | `(classname, [servername])` | Absent; arbitrary COM activation excluded | Explicit refusal H6: no arbitrary COM | No worksheet analogue; activation excluded | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| GetObject | `([pathname], [classname])` | Absent; arbitrary COM activation excluded | Explicit refusal H6: no arbitrary COM | No worksheet analogue; activation excluded | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IMEStatus | `()` | Explicit engine refusal; platform facility unsupported | M6A.6/H6 | No worksheet analogue; platform state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| MacID | `(constant)` | Absent; Macintosh-only | Explicit refusal H6: unavailable target API | No worksheet analogue; platform-specific value | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| MacScript | `(script)` | Absent; external script execution excluded | Explicit refusal H6: script execution prohibited | No worksheet analogue; side effect prohibited | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Spc | `(n)` in Print context | Unsupported syntax; no standalone function identity or output-positioning operation | M6A.6/H3 disposition: deferred syntax profile | No worksheet analogue; output positioning | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Tab | `([n])` in Print context | Unsupported syntax; no standalone function identity or output-positioning operation | M6A.6/H3 disposition: deferred syntax profile | No worksheet analogue; output positioning | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Debug.Print | `Print [arglist]` statement | Implemented as bounded captured UTF-16 output, comma items separated by tab; semicolon and Spc/Tab positioning syntax unsupported by current parser | M6A.6/H3 disposition: documented profile | No worksheet analogue; output effect and pause semantics | [VBA language reference](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/debug-object) |

The `$` forms shown in compound spelling cells are separate public spellings,
not implemented aliases yet. Other host families can also expose typed `$`
forms; H3/H4 packets must reconcile them against the index before implementation.
`Error 5` for invalid compare modes and exact precedence when compare and string
arguments are independently invalid are not asserted here: Microsoft Learn's
StrComp page states that Null compare errors, but does not number that error or
establish precedence. The engine's authored F0 vectors are hypotheses, not
Microsoft observations; keep those cases unresolved until a lawful reference or
profile-tagged observation establishes the number/order.

### StrComp contract established for S1

The public reference declares `StrComp(string1, string2, [compare])`, with two
required valid string expressions and optional comparison mode. Its result is a
Variant whose subtype is Integer: -1, 0, or 1; either string being Null returns
Null. A Null compare argument errors. Omitted compare uses the *calling module's*
Option Compare. `vbUseCompareOption=-1`, `vbBinaryCompare=0`, and
`vbTextCompare=1`; `vbDatabaseCompare=2` is Access-only. Microsoft Learn gives
no number for the Null-compare error and does not settle invalid numeric mode or
competing-error precedence. Binary uses a platform code-page-derived ordering;
Text uses case-insensitive locale ordering. Therefore F4 requires an injected
documented text provider and binary ordering backed by a declared target encoding;
ASCII fold and Unicode scalar ordering are not accepted substitutes.

Reference basis: [Microsoft StrComp](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/strcomp-function),
[MS-VBAL StrComp](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/9541f02a-e173-47a7-8c0f-4d91813c1c88),
[Option Compare](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/option-compare-statement),
[comparison constants](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/comparison-constants).
The semantic binder now resolves `vbUseCompareOption`, `vbBinaryCompare`,
`vbTextCompare`, and `vbDatabaseCompare` as Integer constants after user/project
and reference declarations, so normal shadowing applies. `IntrinsicContext`
provides a per-call configuration and injected comparison-service interface;
Option Compare Text is captured per procedure in IR 8, and relational string
operators use the same injected provider as StrComp. Without a provider,
contextual string comparison fails with a named Unsupported capability. The
test provider is a deterministic test double, not a VBA locale or code-page
implementation and not an Office compatibility observation.

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
| DDB, FV, IPmt, IRR, MIRR, NPer, NPV, Pmt, PPmt, PV, Rate, SLN, SYD | Same-name financial functions | M6A.4 implements and audits VBA signatures/defaults, cash-flow arrays, payment timing, bounded convergence, error behavior and Double return tags; matching names alone do not establish worksheet equivalence. |
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

### S1 implementation: StrComp

Implement `StrComp(string1, string2, [compare])` through normal shared binding,
lowering and execution. The documented result is a Variant containing an Integer
(-1, 0 or 1), or Null when a string argument is Null. Omitted comparison uses
the module's Option Compare; -1 requests that option explicitly, 0 is binary and
1 is text. Null comparison raises an error; value 2 is Access-specific.

The shared implementation returns Variant(Integer), preserves Null for Null
string operands, maps -1 to the caller's module mode, supports 0 and 1 through
the injected provider, and refuses Access mode 2. The no-provider path is an
explicit Unsupported result. Its deterministic tests cover ordering, empty and
unpaired UTF-16 strings, mixed case, argument coercion, Null operands, invalid
and Null compare values, source-order evaluation, qualification/shadowing,
module defaults, cross-module mode capture, error trapping and pause/resume.

The runtime currently maps invalid compare values to error 5 and Null compare
to error 94 by its authored coercion policy. Microsoft references establish
that both error, but do not establish these numbers or competing-error
precedence. These numbers are engine behavior pending profile-tagged Office
evidence; tests do not present them as Microsoft observations. The test provider
also does not establish real locale or code-page collation.

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

## M6A.8 cross-host verification status

The published shared revision and both fork pins exercise `StrComp` in an authored GridCraft worksheet UDF and WordCraft macro. The host and application validation results are recorded in the [work ledger](m6a-work-ledger.md#current-packet); shared tiny-vba CI, GridCraft and WordCraft gates pass. M6A is not accepted: required implementation rows remain partial or absent for `Len` coercion, `LenB`, non-ASCII `LCase`/`UCase`, `StrConv`, code-page `Asc`/`Chr`, and byte-oriented `AscB`, `ChrB`, `LeftB`, `RightB`, and `MidB` (including `$` variants). These are portable implementation gaps, not accepted platform refusals. No fresh Office observation is claimed; those are deferred to M8.5.

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

S2 implements `InStr` and `InStrRev` through the verified IR 8 call plan. `InStr`
defaults its omitted compare argument to the caller's `Option Compare`; `InStrRev`
defaults to Binary, as documented. Explicit `-1`, `0` and `1` select caller,
Binary and Text modes; Access database mode is unavailable. Comparison tests
use a deterministic service double, not a production locale provider. Both
functions return Long values representing the documented Variant(Long) result.
This engine maps invalid starts/compare modes to error 5 and Null start/compare
to error 94; Microsoft documents errors but does not establish those numbers or
their precedence. Long searches have a fixed work cap and return Resource when
it is exceeded. No Office macros were executed and no fresh Office observation
is claimed.

S3 implements Replace with the documented start/count behavior and contextual
compare mode, using the same injected Text provider and bounded search work as
InStr. Its result is the UTF-16 substring beginning at `start`, with replacements
applied to that suffix. Count defaults to -1; an empty find returns the suffix
unchanged, and an empty replacement deletes matches. Engine policy maps invalid
start/count/compare to error 5 and Null input/compare to error 94; Microsoft
documents the errors but not these numbers or validation precedence. The
reference page's count-zero return table and start-sliced return description do
not fully specify their interaction; the engine follows the start-sliced rule.
Text matching has a 1,048,576-unit work budget. No Office observation is claimed.

S4 routes Split through IR 8 for module comparison. It defaults to a space
delimiter, unlimited parts and caller `Option Compare`; it returns a zero-based
String array, preserves empty fields, and treats empty source or limit zero as
an empty array. Join remains a pure String operation over one-dimensional arrays,
uses a space delimiter by default, and iterates elements in array order
independent of the lower bound. Text splitting uses the injected provider and
shared bounded comparison budget. The engine maps invalid limits to error 5 and
Null comparison to 94; exact Office error numbers and competing-error precedence
are not claimed. Tests do not establish locale behavior or an Office observation.

S5 implements Filter through IR 8 for caller comparison context. It requires a
one-dimensional String or Variant array containing only strings, filters by
substring, defaults `include` to True and `compare` to the caller's `Option
Compare`, and returns matching items in a zero-based String array. An empty
match selects every item; no matches produce an empty String array. Text matching
requires the injected provider and shares the bounded comparison budget. Access
database comparison is unsupported. Tests use the deterministic test double and
make no Office locale or error-number claim.

Next task: S6 string edge/case/reverse/repetition functions.
