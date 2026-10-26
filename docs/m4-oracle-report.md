# M4 acceptance report

Measured locally on 2026-10-26. M4's original growing external-subset gate is satisfied: **153 of
153 reviewed ReflexGrid pure-language cases PASS**, with exact decoded scalar tags/values or error
numbers. All **44 independently authored cases** also PASS. This is 197 asserted scenarios, not a
claim about the whole VBA language or unselected corpus.

## Oracle provenance and license

The selected data comes from ReflexGrid commit `03a338b214df243f1bc68e50512936ddc5566c76`, under
its MIT notice, redistributed [with the fixtures](../tests/fixtures/m4-oracle/LICENSE.reflex-grid.txt).
The pinned [upstream report](https://github.com/milochen0418/reflex_grid/blob/03a338b214df243f1bc68e50512936ddc5566c76/docs/excel-oracle.json)
reports Microsoft Excel for Mac **16.114 (build 26101123)** on **macOS 15.3.1**, run **2026-10-15**
at revision `3758c56a39f0c8841fa43eedda0e074414fed3e3`. We fetched that revision read-only and
verified the selected case and expected-result files are unchanged between the run and pinned
revisions. No upstream interpreter implementation or Python script was executed or imported.

This is **inherited upstream-reported Microsoft evidence**, not a fresh local Office measurement.
The per-run locale is not recorded; documentation describes US-English conventions, but we do not
invent a measured locale. Selection excludes locale-dependent functions, Windows-only/non-ASCII
source and Mac array TypeName quirks. Numeric expectations originate in published TypeName|CStr
text, not captured Microsoft binary floats. Selected results compare exactly to decoded published
expectations, without adding a tolerance or claiming access to unpublished payload bits.

The pinned report describes 442 compared/matched language cases, two Mac naming quirks not compared,
and 13 Windows-only cases not run. We selected 153 of those comparable cases; the remaining 304
of the original 457 candidates retain NOT_RUN locally. Whole upstream files are not marked PASS.

## Reproducibility

[selection.json](../tests/fixtures/m4-oracle/selection.json) records case keys, adaptation hashes,
source/expected/report hashes and environment. [selected-expected.json](../tests/fixtures/m4-oracle/selected-expected.json)
retains raw oracle results. Schema 2 requires that raw results agree with typed fixture expectations.
`tools/check_m4_evidence.py` verifies the bundled inventory and, when present, every pinned source
hash and adaptation. It only parses reviewed literal data; it never executes upstream Python/VBA.

The upstream harness checks Err after each expression/function call and reports E<number> when
set, even when an inner function used Resume Next. The fixture's OracleProbe reproduces that
behavior before returning the typed value; expression/function bodies and selected helpers remain
verbatim. This preserves error-state observations instead of treating them as ordinary returns.

```sh
cargo test -p vba-cli --test cli inherited_microsoft_oracle
python3 tools/check_m4_evidence.py
cargo xtask ci
cargo xtask wasm
cargo xtask wasm-run
cargo test --workspace --release --locked
```

Normal Rust tests need no Office installation or submodule checkout. The same 153 bundled cases
run inside both native Rust and WASI without filesystem access. A missing case, invalid metadata,
MISMATCH, UNSUPPORTED, CRASH or timeout fails the acceptance test; unsupported exit code zero is
not accepted as PASS.

## Final validation and fixes

All repository gates pass: **133 integration tests** in debug and release, strict Clippy,
formatting, the 16-crate DAG, seven pins/646 specimen hashes and the new evidence check.
All libraries compile for wasm32-unknown-unknown. Native and WASI agree on all 153 oracle cases,
plus the previous M3 host/debugger/cancellation/isolation proofs and M4 language smoke cases.
No remote cross-platform or separate MSRV run is claimed.

Oracle comparisons corrected Byte arithmetic widths/overflow, Boolean arithmetic, Currency
addition, Single division, scalar number text, empty allocated-array bounds, 0/0 and power overflow,
Empty/string comparison, eager IIf, and physical-line label versus colon-separated call parsing.
Additional tests cover whole-array parameters and typed copies, ParamArray, Option Base/VBA.Array,
named reference capture, invalidated aliases, debugger/host type integrity and malformed IR.
IR 3 carries the new layouts/capture instructions with explicit feature/version gates; IR 1/2
capability targets retain their previous limits. The execution profile is m4-conformance-v1.

| Published expected tag | Selected cases | PASS |
|---|---:|---:|
| Boolean | 12 | 12 |
| Byte | 2 | 2 |
| Currency | 2 | 2 |
| Double | 14 | 14 |
| Error | 23 | 23 |
| Integer | 22 | 22 |
| Long | 22 | 22 |
| Null | 7 | 7 |
| Single | 2 | 2 |
| String | 47 | 47 |

## Selected case results

| ID | Upstream case key | Published result | Local result |
|---|---|---|---|
| rfx-m4-001 | `CStr(1/3)` | `String\|0.333333333333333` | PASS |
| rfx-m4-002 | `CStr(2/3)` | `String\|0.666666666666667` | PASS |
| rfx-m4-003 | `CStr(0.0001)` | `String\|0.0001` | PASS |
| rfx-m4-004 | `CStr(0.00001)` | `String\|1E-05` | PASS |
| rfx-m4-005 | `CStr(CSng(1/3))` | `String\|0.3333333` | PASS |
| rfx-m4-006 | `CStr(CCur(1.5))` | `String\|1.5` | PASS |
| rfx-m4-007 | `CStr(CCur(1/3))` | `String\|0.3333` | PASS |
| rfx-m4-008 | `CStr(True)` | `String\|True` | PASS |
| rfx-m4-009 | `CStr(Empty)` | `String\|` | PASS |
| rfx-m4-010 | `CInt(2.5)` | `Integer\|2` | PASS |
| rfx-m4-011 | `CInt(3.5)` | `Integer\|4` | PASS |
| rfx-m4-012 | `CInt(-2.5)` | `Integer\|-2` | PASS |
| rfx-m4-013 | `CInt("12")` | `Integer\|12` | PASS |
| rfx-m4-014 | `CInt(" 12 ")` | `Integer\|12` | PASS |
| rfx-m4-015 | `CInt("1e3")` | `Integer\|1000` | PASS |
| rfx-m4-016 | `CInt(True)` | `Integer\|-1` | PASS |
| rfx-m4-017 | `CInt("True")` | `E13` | PASS |
| rfx-m4-018 | `CInt("abc")` | `E13` | PASS |
| rfx-m4-019 | `CInt(40000)` | `E6` | PASS |
| rfx-m4-020 | `CLng(2147483647.4)` | `Long\|2147483647` | PASS |
| rfx-m4-021 | `CLng("2147483648")` | `E6` | PASS |
| rfx-m4-022 | `CBool("True")` | `Boolean\|True` | PASS |
| rfx-m4-023 | `CBool("yes")` | `E13` | PASS |
| rfx-m4-024 | `CBool(2)` | `Boolean\|True` | PASS |
| rfx-m4-025 | `CBool("0")` | `Boolean\|False` | PASS |
| rfx-m4-026 | `CBool(0.0)` | `Boolean\|False` | PASS |
| rfx-m4-027 | `CCur("0.1") * 3` | `Currency\|0.3` | PASS |
| rfx-m4-028 | `CCur(1) + 1.5` | `Currency\|2.5` | PASS |
| rfx-m4-029 | `CCur(10) / 4` | `Double\|2.5` | PASS |
| rfx-m4-030 | `CByte(255)` | `Byte\|255` | PASS |
| rfx-m4-031 | `CByte(256)` | `E6` | PASS |
| rfx-m4-032 | `CByte(-1)` | `E6` | PASS |
| rfx-m4-033 | `CSng("1.5")` | `Single\|1.5` | PASS |
| rfx-m4-034 | `CVErr(2042)` | `Error\|Error 2042` | PASS |
| rfx-m4-035 | `CStr(CVErr(2042))` | `String\|Error 2042` | PASS |
| rfx-m4-036 | `Int(-2.5)` | `Double\|-3` | PASS |
| rfx-m4-037 | `Fix(-2.5)` | `Double\|-2` | PASS |
| rfx-m4-038 | `Int(2.5)` | `Double\|2` | PASS |
| rfx-m4-039 | `Round(2.5)` | `Double\|2` | PASS |
| rfx-m4-040 | `Round(3.5)` | `Double\|4` | PASS |
| rfx-m4-041 | `Round(2.675, 2)` | `Double\|2.68` | PASS |
| rfx-m4-042 | `Round(0.125, 2)` | `Double\|0.12` | PASS |
| rfx-m4-043 | `Round(-2.5)` | `Double\|-2` | PASS |
| rfx-m4-044 | `Round(1.005, 2)` | `Double\|1` | PASS |
| rfx-m4-045 | `200 * 200` | `E6` | PASS |
| rfx-m4-046 | `32767 + 1` | `E6` | PASS |
| rfx-m4-047 | `CInt(32767) + CLng(1)` | `Long\|32768` | PASS |
| rfx-m4-048 | `1 + 1` | `Integer\|2` | PASS |
| rfx-m4-049 | `1 + 1.5` | `Double\|2.5` | PASS |
| rfx-m4-050 | `CByte(1) + CByte(2)` | `Byte\|3` | PASS |
| rfx-m4-051 | `CByte(200) + CByte(100)` | `E6` | PASS |
| rfx-m4-052 | `True + True` | `Integer\|-2` | PASS |
| rfx-m4-053 | `True * 2` | `Integer\|-2` | PASS |
| rfx-m4-054 | `Empty + Empty` | `Integer\|0` | PASS |
| rfx-m4-055 | `Empty + 1` | `Integer\|1` | PASS |
| rfx-m4-056 | `Empty & "x"` | `String\|x` | PASS |
| rfx-m4-057 | `"1" + 2` | `Double\|3` | PASS |
| rfx-m4-058 | `"1" + "2"` | `String\|12` | PASS |
| rfx-m4-059 | `"a" + 1` | `E13` | PASS |
| rfx-m4-060 | `1 / 0` | `E11` | PASS |
| rfx-m4-061 | `0 / 0` | `E6` | PASS |
| rfx-m4-062 | `5 \ 2` | `Integer\|2` | PASS |
| rfx-m4-063 | `-5 \ 2` | `Integer\|-2` | PASS |
| rfx-m4-064 | `5.5 \ 2` | `Long\|3` | PASS |
| rfx-m4-065 | `5 Mod 3` | `Integer\|2` | PASS |
| rfx-m4-066 | `-5 Mod 3` | `Integer\|-2` | PASS |
| rfx-m4-067 | `5.5 Mod 2` | `Long\|0` | PASS |
| rfx-m4-068 | `6.5 Mod 2` | `Long\|0` | PASS |
| rfx-m4-069 | `0 ^ -1` | `E5` | PASS |
| rfx-m4-070 | `2 ^ 1024` | `E6` | PASS |
| rfx-m4-071 | `Null + 1` | `Null\|` | PASS |
| rfx-m4-072 | `Null & "x"` | `String\|x` | PASS |
| rfx-m4-073 | `Null & Null` | `Null\|` | PASS |
| rfx-m4-074 | `Null = Null` | `Null\|` | PASS |
| rfx-m4-075 | `TypeName(Null And False)` | `String\|Boolean` | PASS |
| rfx-m4-076 | `Null Or True` | `Boolean\|True` | PASS |
| rfx-m4-077 | `Not Null` | `Null\|` | PASS |
| rfx-m4-078 | `Not 5` | `Integer\|-6` | PASS |
| rfx-m4-079 | `5 And 3` | `Integer\|1` | PASS |
| rfx-m4-080 | `True And 5` | `Integer\|5` | PASS |
| rfx-m4-081 | `True Or False` | `Boolean\|True` | PASS |
| rfx-m4-082 | `-32768` | `Long\|-32768` | PASS |
| rfx-m4-083 | `1E300 * 1E10` | `E6` | PASS |
| rfx-m4-084 | `CSng(1) + 1` | `Single\|2` | PASS |
| rfx-m4-085 | `CSng(1) + 100000` | `Double\|100001` | PASS |
| rfx-m4-086 | `CLng(1) / 2` | `Double\|0.5` | PASS |
| rfx-m4-087 | `10 \ 4.6` | `Long\|2` | PASS |
| rfx-m4-088 | `CCur(5) \ 2` | `Long\|2` | PASS |
| rfx-m4-089 | `"abc" < "abd"` | `Boolean\|True` | PASS |
| rfx-m4-090 | `"a" = "A"` | `Boolean\|False` | PASS |
| rfx-m4-091 | `Empty = 0` | `Boolean\|True` | PASS |
| rfx-m4-092 | `Empty = ""` | `Boolean\|True` | PASS |
| rfx-m4-093 | `Null = 1` | `Null\|` | PASS |
| rfx-m4-094 | `Left("hello", 2)` | `String\|he` | PASS |
| rfx-m4-095 | `Right("hello", 10)` | `String\|hello` | PASS |
| rfx-m4-096 | `Mid("hello", 2)` | `String\|ello` | PASS |
| rfx-m4-097 | `Mid("hello", 10, 2)` | `String\|` | PASS |
| rfx-m4-098 | `Mid("hello", 0, 1)` | `E5` | PASS |
| rfx-m4-099 | `Len(Null)` | `Null\|` | PASS |
| rfx-m4-100 | `Trim("  a b  ")` | `String\|a b` | PASS |
| rfx-m4-101 | `InStr("hello", "l")` | `Long\|3` | PASS |
| rfx-m4-102 | `InStr(3, "hello", "l")` | `Long\|3` | PASS |
| rfx-m4-103 | `InStr("hello", "")` | `Long\|1` | PASS |
| rfx-m4-104 | `InStr(10, "hello", "l")` | `Long\|0` | PASS |
| rfx-m4-105 | `InStr(Null, "x")` | `Null\|` | PASS |
| rfx-m4-106 | `Replace("abcabc", "a", "x", 3)` | `String\|cxbc` | PASS |
| rfx-m4-107 | `Replace("aaa", "a", "b", 1, 2)` | `String\|bba` | PASS |
| rfx-m4-108 | `UBound(Split("a,b,,c", ","))` | `Long\|3` | PASS |
| rfx-m4-109 | `UBound(Split(""))` | `Long\|-1` | PASS |
| rfx-m4-110 | `UBound(Split("a b  c"))` | `Long\|3` | PASS |
| rfx-m4-111 | `Join(Split("a-b-c", "-", 2), "\|")` | `String\|a\|b-c` | PASS |
| rfx-m4-112 | `Chr(65)` | `String\|A` | PASS |
| rfx-m4-113 | `Asc("A")` | `Integer\|65` | PASS |
| rfx-m4-114 | `IsEmpty(Empty)` | `Boolean\|True` | PASS |
| rfx-m4-115 | `IsNull(Null)` | `Boolean\|True` | PASS |
| rfx-m4-116 | `TypeName(Nothing)` | `String\|Nothing` | PASS |
| rfx-m4-117 | `LBound(Array(1, 2))` | `Long\|0` | PASS |
| rfx-m4-118 | `UBound(Array())` | `Long\|-1` | PASS |
| rfx-m4-119 | `IIf(1 > 2, "a", "b")` | `String\|b` | PASS |
| rfx-m4-120 | `Sgn(-3.5)` | `Integer\|-1` | PASS |
| rfx-m4-121 | `Abs(CInt(-5))` | `Integer\|5` | PASS |
| rfx-m4-122 | `TypeName(1 + 1)` | `String\|Integer` | PASS |
| rfx-m4-123 | `TypeName(1 + 100000)` | `String\|Long` | PASS |
| rfx-m4-124 | `TypeName(30000 + 30000)` | `E6` | PASS |
| rfx-m4-125 | `TypeName(CInt(1) * 1.5)` | `String\|Double` | PASS |
| rfx-m4-126 | `TypeName(1 / 1)` | `String\|Double` | PASS |
| rfx-m4-127 | `TypeName(CSng(1) / 2)` | `String\|Single` | PASS |
| rfx-m4-128 | `TypeName(10 \ 3)` | `String\|Integer` | PASS |
| rfx-m4-129 | `TypeName(10& \ 3)` | `String\|Long` | PASS |
| rfx-m4-130 | `TypeName("1" + 1)` | `String\|Double` | PASS |
| rfx-m4-131 | `TypeName(True And True)` | `String\|Boolean` | PASS |
| rfx-m4-132 | `TypeName(5 And 3)` | `String\|Integer` | PASS |
| rfx-m4-133 | `TypeName(CByte(5) And CByte(3))` | `String\|Byte` | PASS |
| rfx-m4-134 | `ForLongLimit()` | `Long\|13` | PASS |
| rfx-m4-135 | `ResumeNextIf()` | `E11` | PASS |
| rfx-m4-136 | `ErrInHandler()` | `String\|1000 src custom` | PASS |
| rfx-m4-137 | `ErrClearedAfterResume()` | `String\|after 0` | PASS |
| rfx-m4-138 | `RaiseDefaultDescription()` | `E11` | PASS |
| rfx-m4-139 | `ReDimPreserve()` | `String\|470` | PASS |
| rfx-m4-140 | `ReDimPreserveBad()` | `E9` | PASS |
| rfx-m4-141 | `ArrayCopy()` | `String\|19` | PASS |
| rfx-m4-142 | `ByRefDefault()` | `Long\|2` | PASS |
| rfx-m4-143 | `ByValParen()` | `Long\|1` | PASS |
| rfx-m4-144 | `CallByRef()` | `Long\|2` | PASS |
| rfx-m4-145 | `StaticCounter()` | `Long\|2` | PASS |
| rfx-m4-146 | `OptionalMissing()` | `String\|mp` | PASS |
| rfx-m4-147 | `ParamArrayCount()` | `String\|3/00/0` | PASS |
| rfx-m4-148 | `NullCondition()` | `String\|f` | PASS |
| rfx-m4-149 | `EraseFixed()` | `String\|20` | PASS |
| rfx-m4-150 | `EraseDynamic()` | `E9` | PASS |
| rfx-m4-151 | `NestedFunctionReturnArray()` | `String\|220` | PASS |
| rfx-m4-152 | `SplitLimit()` | `String\|a\|b\|c,d` | PASS |
| rfx-m4-153 | `ResumeLabel()` | `String\|there 0` | PASS |
