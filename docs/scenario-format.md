# Scenario format, schema 1

Implemented at M0. Files use UTF-8 and extension `.vbas`. A strict `key = value` header ends with
an exact `---` line; everything following it is embedded VBA source, retained verbatim. LF and
CRLF are accepted. Header blank lines/comments, unknown/duplicate/empty fields and NUL are rejected.
A file is limited to 65,536 bytes and each header line to 4,096 bytes. The CLI limits reads before
parsing. No include paths, imports, external files or downloaded scripts are evaluated.

```text
tiny-vba-scenario = 1
id = arithmetic
module = Module1
entry = Answer
provenance = independently-authored
license = MIT
profile = pure-language-unmeasured
requirements = arithmetic
capabilities = none
evidence = reasoned-not-oracle
expected = Long:42
rationale = Integral multiplication of 6 and 7 yields 42.
---
Option Explicit
Public Function Answer() As Long
    Answer = 6 * 7
End Function
```

All twelve fields are required. Schema, provenance, license, profile, capabilities and evidence
must have the exact values above. IDs are at most 128 ASCII letters/digits/underscores/hyphens,
starting with a letter. Module and entry names additionally reject hyphens. Requirements are
one to 32 comma-separated feature IDs without spaces. Fixture IDs must be unique in the checked
suite and match filenames; the owning Rust suite checks this. Rationale records the independently
reasoned expectation. File path, stable ID and repository revision trace the embedded source.
There are no measured Office observations or source-hash claims in this schema.

Typed expectations are Empty, Null, Missing, Nothing, Boolean:True/False, Byte, Integer, Long,
LongLong, Single, Double, Currency, Date, String, CVErr or Error. All except the four bare states
use `Kind:payload`. Numeric widths are checked; non-finite floats are rejected. Currency payloads
are signed integer ten-thousandths; Date payloads are finite serial-day numbers. CVErr payloads
are unsigned 16-bit codes. Error payloads are positive runtime error numbers, distinct from CVErr.
String payloads are literal UTF-8 text on one header line, converted to exact UTF-16 units; `String:`
expresses an empty string. No escapes, multiline strings, arrays or structured side-effect expectations
are supported in schema 1. Adding them requires a reviewed schema change.

```sh
cargo run -p vba-cli -- conformance validate tests/fixtures/m0/arithmetic.vbas
```

A valid fixture prints `VALID_FIXTURE arithmetic: NOT_RUN (execution unavailable)` and exits 0.
Malformed/unsupported fixture structure or arguments and unreadable/non-UTF-8 files exit 2.
Fixture errors use VBA0010 and include source attachments when parsed. Source infrastructure budget
errors use VBA0003. Filesystem errors are reported by the CLI IO boundary. The validator does not
check VBA grammar, bind names, execute the entrypoint or compare results. It must not report PASS.
`run` executes the [M3 runtime subset](runtime.md); it does not load or compare scenario manifests. M1 separately implements
`parse <source.bas|source.cls>` for its [documented syntax subset](frontend.md); M2 adds
`check`, `compile` and `disasm` for the [compiler subset](compiler.md). These validation/compilation commands do not compare a scenario outcome.

The 20 independently authored [M0 cases](../tests/fixtures/m0/) cover arithmetic/precedence,
integer division/remainder, Boolean/string results, Empty/Null/Nothing/CVErr, branches/loops,
same-line statements, ByRef, optional/missing and named calls, and runtime division-by-zero.
Their MIT licensing follows this repository. Their expectations are proposals for later executable
tests, not a language compatibility result. Missing is also checked directly in protocol/value tests.
