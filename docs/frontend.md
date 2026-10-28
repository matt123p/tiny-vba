# M1 source frontend

Implemented 2026-10-26 in `vba-syntax`; no external Rust dependencies. Parsing is
syntax validation, not binding, type checking, execution or an Office compatibility result.

```sh
cargo run -p vba-cli -- parse examples/pure-vba/Arithmetic.bas
cargo run -p vba-cli -- parse tests/syntax/m1/control.bas
cargo run -p vba-cli -- parse tests/syntax/m1/metadata.cls
cargo run -p vba-cli -- parse tests/syntax/m1/invalid.bas
cargo test -p vba-syntax
```

The first command prints `PARSED examples/pure-vba/Arithmetic.bas: 1 procedures
(syntax only; use check or run)` and the procedure's name/range. Invalid input returns
exit 2 and reports `invalid.bas:3:19: VBA0101` at the unexpected `)`.
`check`, `compile` and `disasm` now implement the narrower [M2 compiler subset](compiler.md); `run` executes the narrower [M3 runtime subset](runtime.md).

## API and source ownership

`parse(&SourceFile, ModuleKind)` returns `ParseResult { module, diagnostics }`.
Only `syntax_valid()` with no diagnostics establishes accepted M1 syntax. A recovered AST
is partial and must never be compiled as valid input. Lexical errors retain tokens and skip
AST parsing. The caller supplies source identity and standard/class kind; the CLI chooses
kind from a case-insensitive `.bas`/`.cls` extension and accepts UTF-8 source only.

The module owns a source snapshot and all tokens (including whitespace, comments and
continuations). Successful lexing reconstructs the exact input through token spans.
Identifiers retain spelling and offer an ASCII case-insensitive key. Numbers retain raw
spelling/suffixes; strings decode doubled quotes without converting to runtime values.
All ranges are checked half-open UTF-8 byte spans. Diagnostic coordinates are one-based
Unicode scalar columns, with LF, CRLF and lone CR support.

Statement IDs are deterministic source-local ordinals assigned in source order, including
colon-separated statements, If/ElseIf headers, End If, For/Next and procedure endings.
The first If arm shares its parent statement's site; it is one checkpoint, not two.
Control statement sites cover their headers; bodies/terminators have their own sites.
Else has no executable site. Procedure/declaration/expression nodes retain spans; procedure
headers are not executable statement sites. IDs are snapshot-local, not stable across edits.

## Supported grammar and limits

| Feature | Status / exact boundary |
|---|---|
| Tokens | Implemented: ASCII identifiers/type suffixes, decimal/exponent/hex/octal numbers, doubled-quote strings, operators, apostrophe/Rem comments, explicit continuations, newline/colon separators |
| Metadata | Partial: literal Attribute paths, Option Explicit/Base 0 or 1/Compare Binary or Text/Private Module; class VERSION 1.0 CLASS prefix |
| Procedures | Partial: Public/Private/Friend/default Sub and Function, scalar parameters, ByVal/ByRef, Optional/default expressions, scalar result types |
| Declarations | Partial: module/local Dim and Const, module Public/Private/Friend variables, comma-separated scalar names and As types |
| Expressions | Partial: literals (including True/False/Empty/Null/Nothing), names, parentheses, unary +/−/Not, arithmetic/comparison/logical operators, member access and application |
| Calls | Partial: explicit Call, implicit calls, expression applications, named and omitted argument syntax; preserve parenthesized arguments for later ByRef binding |
| Statements | Partial: Let/Set/implicit assignment, multiline If/ElseIf/Else, single-line If with colon lists, numeric For/To/Step/Next, Exit Sub/Function/For and Stop |
| Later grammar (historical first-slice limits; see extensions below) | For Each, Do/While, Select Case, With, Property procedures, Type/Enum, As New, fixed-length types, conditional compilation, date/bracketed/non-ASCII identifiers, designer blocks, file/DLL/event statements remain unsupported |
| Qualified declared types (new M1.1) | One library qualifier, e.g. `As HostLib.Mode`; original source span retained including whitespace. Binder requires an explicitly registered type; parsing does not imply nominal object support |
| Semantic checks | Not implemented: scope/name/type resolution, argument legality, Option enforcement, matching Next variable, Exit context, constant evaluation, host member validity |

No binding is implied for a parsed type name or a named/omitted/Optional argument.
Numeric widths/coercion and runtime strings belong to later layers. Some unsupported syntax
reports a syntax error when it reaches an unsupported suffix/form; no diagnostics means
accepted syntax only within this documented subset.

Bounds: 1 MiB source, 65,536 tokens including EOF, 32 diagnostics, 64 parser nesting levels
and 64 AST expression depth, 128 bytes per identifier, 256 arguments per call. Parser nesting
counts helper/control/expression recursion, so the practical control nesting limit is smaller
than 64. Budget failures report VBA0102; lexical failures VBA0100; syntax failures VBA0101;
recognized unsupported constructs VBA0001. Recovery advances to a newline/colon and stops
at the diagnostic limit. At a token budget failure the token stream is truncated explicitly.

## Evidence

`crates/vba-syntax/tests/frontend.rs` covers both examples, 20 M0 source bodies, metadata,
precedence, calls, exact error spans, recovery, statement identities, token reconstruction,
Unicode/newlines, malformed literals, expression/control/token/identifier budgets and
truncated/generated malformed input. CLI tests cover source reads, extensions, diagnostics
and failure exit codes. M0 cases remain NOT_RUN for execution.

The [reviewed external grammar report](m1-corpus-report.md) records the separate optional
five-file corpus evaluation. Local tests do not depend on corpus checkouts.

## M4 extension

[The M4 contract](m4-language.md) adds array declarations/indexing, ReDim/Erase, Static local
declarations, labels/GoTo, On Error and Resume syntax. Optional/named syntax now binds and executes.
These additions supersede corresponding M1 historical unsupported lists; other limits still apply.

M4 completion adds whole-array/ParamArray parameter syntax and explicit legality checks. Labels
are recognized at physical line starts; a following colon-separated bare name is parsed as a
call. Option Base is bound explicitly, duplicate directives are rejected and qualified VBA
intrinsic calls use the deterministic registry. See [current acceptance](m4-oracle-report.md).

## New M1 collection syntax

The parser preserves For Each ... In ... Next (optional matching control), including
nested loops and Exit For. Binding requires Variant/Object collection controls,
Variant native-array controls and distinct nested control variables. Other loop
constructs are unchanged. See [M1 implementation](m1-defaults-and-collections.md).
