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
| StrComp | `(string1, string2, [compare])` | Absent; Option Compare Text rejected | Required S1; F4 prerequisite | EXACT; ordering and Null result | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/strcomp-function) |
| InStr | `([start,] string1, string2, [compare])` | Partial: binary search only | Required S2 | FIND/SEARCH; no match returns 0, reverse not supported | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| InStrRev | `(stringcheck, stringmatch, [start, [compare]])` | Absent | Required S2 | Reverse composed search; reverse indexing and no-match 0 | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Replace | `(expression, find, replace, [start, [count, [compare]]])` | Partial: binary only; incomplete Null/error precedence | Required S2 | SUBSTITUTE/REPLACE; substring replacement, start/count and compare | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/replace-function) |
| Left / Left$ | `(string, length)` | Left partial; `$` absent | Required S2 | LEFT; `$` preserves String return identity | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Right / Right$ | `(string, length)` | Right partial; `$` absent | Required S2 | RIGHT; `$` preserves String return identity | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Mid / Mid$ | `(string, start, [length])` | Mid partial; `$` absent | Required S2 | MID; UTF-16 indexing and optional length | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Len | `(expression)` | Partial: UTF-16 units for String | Required S2 | LEN; VBA Variant/coercion rules | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| LenB | `(expression)` | Absent | Required S2; byte representation prerequisite | LEN; byte length depends on representation | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| LCase / LCase$ | `(string)` | Partial: ASCII only | Required S2; locale provider | LOWER; locale and Null | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| UCase / UCase$ | `(string)` | Partial: ASCII only | Required S2; locale provider | UPPER; locale and Null | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Trim / Trim$ | `(string)` | Partial: trims U+0020 edges | Required S2 | TRIM; VBA retains internal repeated spaces | [Excel VBA](https://learn.microsoft.com/en-us/office/vba/api/excel.worksheetfunction.trim) |
| LTrim / LTrim$ | `(string)` | Absent | Required S2 | TRIM composed; left edge only | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| RTrim / RTrim$ | `(string)` | Absent | Required S2 | TRIM composed; right edge only | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Space / Space$ | `(number)` | Absent | Required S2 | REPT; creates spaces with count rules | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| String / String$ | `(number, character)` | Absent | Required S2 | REPT; character code or first character semantics | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| StrReverse | `(expression)` | Absent | Required S2 | No direct counterpart; reverse UTF-16 character units | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| StrConv | `(string, conversion, [LCID])` | Absent | Required S2; locale/code-page provider | PROPER/width conversions; conversion flags and LCID | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Asc | `(string)` | Partial: ASCII only | Required S2; code-page provider | CODE is not code-page compatible | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| AscW | `(string)` | Absent | Required S2 | UNICODE; UTF-16 code unit, including surrogate behavior | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| AscB | `(string)` | Absent | Required S2; byte representation prerequisite | No equivalent; byte extraction | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Chr / Chr$ | `(charcode)` | Partial: ASCII only | Required S2; code-page provider | CHAR differs by code page and range | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| ChrW / ChrW$ | `(charcode)` | Absent | Required S2 | UNICHAR differs at UTF-16 boundaries | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| ChrB / ChrB$ | `(charcode)` | Absent | Required S2; byte representation prerequisite | No equivalent; byte construction | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| LeftB / LeftB$ | `(string, length)` | Absent | Required S2; byte representation prerequisite | LEFT; byte rather than character slicing | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| RightB / RightB$ | `(string, length)` | Absent | Required S2; byte representation prerequisite | RIGHT; byte rather than character slicing | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| MidB / MidB$ | `(string, start, [length])` | Absent | Required S2; byte representation prerequisite | MID; byte rather than character slicing | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| CBool | `(expression)` | Partial: shared numeric/string conversion | Required C1 | VALUE/coercion; Boolean rules | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CByte | `(expression)` | Partial: conversion exists | Required C1 | VALUE; Byte rounding and range | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CCur | `(expression)` | Partial: Currency conversion exists | Required C1 | VALUE; fixed-point scale/rounding | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CDate | `(expression)` | Partial: numeric serial only | Required C1; locale/calendar | DATEVALUE; parsing and serial epoch | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CDbl | `(expression)` | Partial: conversion exists | Required C1 | VALUE; locale/coercion | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CDec | `(expression)` | Absent: Decimal value kind missing | Required C1; Decimal foundation | NUMBERVALUE; exact decimal precision | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CInt | `(expression)` | Partial: conversion exists | Required C1 | INT/TRUNC not equivalent; ties-even and width | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CLng | `(expression)` | Partial: conversion exists | Required C1 | INT/TRUNC not equivalent; Long width | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CLngLng | `(expression)` | Partial: conversion exists | Required C1; platform/version | INT/TRUNC; 64-bit result | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CLngPtr | `(expression)` | Absent: target width missing | Required C1; target-width foundation | No equivalent; pointer-width result | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CSng | `(expression)` | Partial: conversion exists | Required C1 | VALUE; Single precision | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CStr | `(expression)` | Partial: locale Date unsupported | Required C1; locale | TEXT; type and formatting rules | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CVar | `(expression)` | Partial: identity wrapper | Required C1 | VALUE; preserve Variant subtype/state | [Conversion](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/type-conversion-functions) |
| CVErr | `(errornumber)` | Partial: constructs CVErr | Required C1 | ERROR.TYPE classifies; does not construct Variant Error | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Val | `(string)` | Absent | Required C1 | VALUE/NUMBERVALUE; numeric-prefix grammar | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Str / Str$ | `(number)` | Absent | Required C1 | TEXT; leading sign-space and formatting | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Hex / Hex$ | `(number)` | Absent | Required C1 | DEC2HEX/BASE; signed widths and limits | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Oct / Oct$ | `(number)` | Absent | Required C1 | DEC2OCT/BASE; signed widths and limits | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Format / Format$ | `(expression, [format, [firstdayofweek, [firstweekofyear]]])` | Absent | Required C2; locale/calendar | TEXT; VBA format grammar and locale | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FormatCurrency | `(expression, [numdigitsafterdecimal, [includeleadingdigit, [useparensfornegative, [groupdigits]]]])` | Absent | Required C2; locale | DOLLAR; currency locale/rounding | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FormatDateTime | `(date, [namedformat])` | Absent | Required C2; locale/calendar | TEXT; named format and regional output | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FormatNumber | `(expression, [numdigitsafterdecimal, [includeleadingdigit, [useparensfornegative, [groupdigits]]]])` | Absent | Required C2; locale | FIXED; locale and grouping | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FormatPercent | `(expression, [numdigitsafterdecimal, [includeleadingdigit, [useparensfornegative, [groupdigits]]]])` | Absent | Required C2; locale | TEXT; percent scaling/locale | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsArray | `(varname)` | Partial: Array tag | Required C1 | ISBLANK/array constants; VBA Variant tags | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsDate | `(expression)` | Absent | Required C1; locale/calendar | DATEVALUE; accepted text and serial range | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsEmpty | `(expression)` | Partial: Empty tag | Required C1 | ISBLANK; Empty Variant state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsError | `(expression)` | Partial: CVErr tag | Required C1 | ISERROR; Variant error values | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsMissing | `(argumentname)` | Partial: detects Missing value | Required C1; omitted argument binder | ISBLANK; only optional Variant procedure arguments | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsNull | `(expression)` | Partial: Null tag | Required C1 | ISBLANK; Null state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsNumeric | `(expression)` | Absent | Required C1 | ISNUMBER; numeric strings and Null | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IsObject | `(expression)` | Partial: Object/Nothing tags | Required C1 | IS; object identity | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| TypeName | `(varname)` | Partial: core tags | Required C1 | No direct counterpart; exact Variant/type name | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| VarType | `(varname)` | Partial: core tags | Required C1 | No direct counterpart; exact subtype flags | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Error / Error$ | `([errornumber])` | Absent | Required C1 | ERROR.TYPE; returns localized description | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Array | `([arglist])` | Partial: Variant array construction | Required A1; lower-bound behavior | Array constants/ROWS; explicit bounds and subtype | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| LBound | `(arrayname, [dimension])` | Partial: array bounds | Required A1 | ROWS/COLUMNS do not report VBA subscripts | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| UBound | `(arrayname, [dimension])` | Partial: array bounds | Required A1 | ROWS/COLUMNS do not report VBA subscripts | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Split | `(expression, [delimiter, [limit, [compare]]])` | Partial: binary split and array result | Required A2; comparison context | TEXTSPLIT; limit/lower bound/empty parts | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Join | `(sourcearray, [delimiter])` | Partial: one-dimensional arrays | Required A2 | TEXTJOIN; array coercion and bounds | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Filter | `(sourcearray, match, [include, [compare]])` | Absent | Required A2; comparison context | FILTER; substring filter and String array result | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Choose | `(index, choice-1, [choice-2, ...])` | Absent | Required A2; conditional argument evaluation | CHOOSE; VBA index coercion/evaluation | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IIf | `(expr, truepart, falsepart)` | Partial: selection; eager values supplied | Required A2; preserve eager side effects | IF; both branches are evaluated | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/iif-function) |
| Switch | `(expr-1, value-1, [expr-2, value-2, ...])` | Absent | Required A2; conditional evaluation contract | IFS/SWITCH differ in match form and evaluation | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Partition | `(number, start, stop, interval)` | Absent | Required A2 | No direct counterpart; range string contract | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Abs | `(number)` | Partial: scalar numeric | Required N1 | ABS; VBA coercion/Null and result tag | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Atn | `(number)` | Absent | Required N1 | ATAN; type/domain rules | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Cos | `(number)` | Absent | Required N1 | COS; VBA coercion/result | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Exp | `(number)` | Absent | Required N1 | EXP; overflow contract | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Int | `(number)` | Partial: floor | Required N1 | INT; negative input behavior | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Fix | `(number)` | Partial: truncation | Required N1 | TRUNC; negative input behavior | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Log | `(number)` | Absent | Required N1 | LN/LOG; natural log and domain | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/log-function) |
| Rnd | `([number])` | Absent | Required R1; seeded VM state | RAND; seed/state/repeat semantics | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Randomize (statement) | `[number]` | Absent | Required R1; seeded VM state | RAND; explicit state mutation | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Sgn | `(number)` | Partial: scalar numeric | Required N1 | SIGN; VBA result tag/coercion | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Sin | `(number)` | Absent | Required N1 | SIN; coercion/result | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Sqr | `(number)` | Absent | Required N1 | SQRT; negative-domain error | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Tan | `(number)` | Absent | Required N1 | TAN; overflow/precision | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Round | `(expression, [numdecimalplaces])` | Partial: ties-even, Double result | Required N1 | ROUND; tie direction/precision/type | [VBA](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/round-function) |
| Date / Date$ | `()` | Absent | Required D1; injected clock | TODAY; now versus constructor | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Time / Time$ | `()` | Absent | Required D1; injected clock | MOD(NOW(),1); Date tag and clock | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Now | `()` | Absent | Required D1; injected clock | NOW; serial/tag/timezone | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Timer | `()` | Absent | Required D1; injected clock | No direct counterpart; elapsed seconds | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| DateAdd | `(interval, number, date)` | Absent | Required D2; calendar service | EDATE/date arithmetic; interval boundaries | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| DateDiff | `(interval, date1, date2, [firstdayofweek, [firstweekofyear]])` | Absent | Required D2; calendar service | DATEDIF/date arithmetic; boundary rules | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| DatePart | `(interval, date, [firstdayofweek, [firstweekofyear]])` | Absent | Required D2; calendar service | date extraction; week rules | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| DateSerial | `(year, month, day)` | Absent | Required D2; calendar service | DATE; normalization/range | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| DateValue | `(date)` | Absent | Required D2; locale/calendar | DATEVALUE; accepted forms and epoch | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| TimeSerial | `(hour, minute, second)` | Absent | Required D2; calendar service | TIME; normalization and Date tag | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| TimeValue | `(time)` | Absent | Required D2; locale/calendar | TIMEVALUE; accepted forms and Date tag | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Day | `(date)` | Absent | Required D2; calendar service | DAY; serial/locale | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Month | `(date)` | Absent | Required D2; calendar service | MONTH; serial/locale | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Year | `(date)` | Absent | Required D2; calendar service | YEAR; serial/locale | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Hour | `(time)` | Absent | Required D2; calendar service | HOUR; serial/locale | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Minute | `(time)` | Absent | Required D2; calendar service | MINUTE; serial/locale | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Second | `(time)` | Absent | Required D2; calendar service | SECOND; serial/locale | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Weekday | `(date, [firstdayofweek])` | Absent | Required D2; calendar service | WEEKDAY; week-start numbering | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| MonthName | `(month, [abbreviate])` | Absent | Required D2; locale service | TEXT date format; index and locale names | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| WeekdayName | `(weekday, [abbreviate, [firstdayofweek]])` | Absent | Required D2; locale service | TEXT date format; index/week-start/locale | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| DDB | `(cost, salvage, life, period, [factor])` | Absent | Required N2; finance contracts | DDB; independently audit arrays/errors | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FV | `(rate, nper, pmt, [pv, [due]])` | Absent | Required N2 | FV; payment timing and argument names | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IPmt | `(rate, per, nper, pv, [fv, [due]])` | Absent | Required N2 | IPMT; period/range/error contract | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IRR | `(values, [guess])` | Absent | Required N2; bounded convergence | IRR; guess/convergence/array input | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| MIRR | `(values, finance_rate, reinvest_rate)` | Absent | Required N2; bounded convergence | MIRR; array and rate contracts | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| NPer | `(rate, pmt, pv, [fv, [due]])` | Absent | Required N2 | NPER; timing and zero-rate behavior | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| NPV | `(rate, values())` | Absent | Required N2 | NPV; first-period timing and arrays | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Pmt | `(rate, nper, pv, [fv, [due]])` | Absent | Required N2 | PMT; timing and argument names | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| PPmt | `(rate, per, nper, pv, [fv, [due]])` | Absent | Required N2 | PPMT; period/range/error contract | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| PV | `(rate, nper, pmt, [fv, [due]])` | Absent | Required N2 | PV; timing and argument names | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Rate | `(nper, pmt, pv, [fv, [due, [guess]]])` | Absent | Required N2; bounded convergence | RATE; guess and convergence | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| SLN | `(cost, salvage, life)` | Absent | Required N2 | SLN; argument/range errors | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| SYD | `(cost, salvage, life, period)` | Absent | Required N2 | SYD; period/range errors | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| RGB | `(red, green, blue)` | Absent | Required H1; host color representation | RGB; VBA Long encoding and range | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| QBColor | `(color)` | Absent | Required H1; host color representation | No direct counterpart; fixed palette index | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| CallByName | `(object, procname, calltype, [args()])` | Absent; explicit host protocol only | Required H2; bounded host member service | No direct counterpart; member dispatch policy | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| MsgBox | `(prompt, [buttons, [title, [helpfile, context]]])` | Absent; prompt service absent | Required H3; injected prompt | No worksheet analogue; button/result contract | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| InputBox | `(prompt, [title, [default, [xpos, [ypos, [helpfile, context]]]]])` | Absent; prompt service absent | Required H3; injected prompt | No worksheet analogue; cancellation/text result | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| DoEvents (statement/function) | `()` | Absent; no scheduler yield intrinsic | Required H3; existing host wait protocol | No worksheet analogue; yield/cancellation effects | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Command / Command$ | `()` | Absent; no process environment service | Required H3; explicit service | No worksheet analogue; command-line exposure | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Environ / Environ$ | `(envstring)` | Absent; no environment service | Required H3; explicit service and allowlist | No worksheet analogue; missing variable behavior | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Dir / Dir$ | `([pathname, [attributes]])` | Absent; no filesystem service | Required H4; bounded host service | No worksheet analogue; enumeration state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| CurDir / CurDir$ | `([drive])` | Absent; no filesystem service | Required H4; bounded host service | No worksheet analogue; current-directory state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| EOF | `(filenumber)` | Absent; no file service | Required H4; bounded host service | No worksheet analogue; file cursor state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FileAttr | `(filenumber, returntype)` | Absent; no file service | Required H4; bounded host service | No worksheet analogue; file mode/platform | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FileDateTime | `(pathname)` | Absent; no filesystem service | Required H4; bounded host service | No worksheet analogue; timestamp/zone | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FileLen | `(pathname)` | Absent; no filesystem service | Required H4; bounded host service | No worksheet analogue; file length/overflow | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| FreeFile | `([rangenumber])` | Absent; no file service | Required H4; bounded host service | No worksheet analogue; allocation/lifecycle | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| GetAttr | `(pathname)` | Absent; no filesystem service | Required H4; bounded host service | No worksheet analogue; platform attribute bits | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Input / Input$ | `(number, filenumber)` | Absent; no file service | Required H4; bounded host service | No worksheet analogue; bytes/text and cursor | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Loc | `(filenumber)` | Absent; no file service | Required H4; bounded host service | No worksheet analogue; mode-dependent position | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| LOF | `(filenumber)` | Absent; no file service | Required H4; bounded host service | No worksheet analogue; logical record/file size | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Seek | `(filenumber)` | Absent; no file service | Required H4; bounded host service | No worksheet analogue; one-based cursor | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| GetSetting | `(appname, section, key, [default])` | Absent; no settings service | Required H5; explicit settings service | No worksheet analogue; registry/service scope | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| GetAllSettings | `(appname, section)` | Absent; no settings service | Required H5; explicit settings service | No worksheet analogue; two-dimensional result | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Shell | `(pathname, [windowstyle])` | Absent; prohibited ambient process launch | Explicit refusal H6: no shell authority | No worksheet analogue; side effect prohibited | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| CreateObject | `(classname, [servername])` | Absent; arbitrary COM activation excluded | Explicit refusal H6: no arbitrary COM | No worksheet analogue; activation excluded | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| GetObject | `([pathname], [classname])` | Absent; arbitrary COM activation excluded | Explicit refusal H6: no arbitrary COM | No worksheet analogue; activation excluded | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| IMEStatus | `()` | Absent; platform facility | Explicit refusal H6 unless a reviewed host service is adopted | No worksheet analogue; platform state | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| MacID | `(constant)` | Absent; Macintosh-only | Explicit refusal H6: unavailable target API | No worksheet analogue; platform-specific value | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| MacScript | `(script)` | Absent; external script execution excluded | Explicit refusal H6: script execution prohibited | No worksheet analogue; side effect prohibited | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Spc | `(n)` in Print context | No standalone function; statement syntax only | Required H3; output-context syntax audit | No worksheet analogue; output positioning | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Tab | `([n])` in Print context | No standalone function; statement syntax only | Required H3; output-context syntax audit | No worksheet analogue; output positioning | [VBA index](https://learn.microsoft.com/en-us/office/vba/language/reference/functions-visual-basic-for-applications) |
| Debug.Print | `Print [arglist]` statement | Partial: dedicated lowering exists | Required H3; output/context behavior audit | No worksheet analogue; output effect and pause semantics | [VBA language reference](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/debug-object) |

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
