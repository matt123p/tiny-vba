# M6A acceptance: VBA intrinsic functions

Accepted 2026-11-02 for the declared inventory and explicit platform/service
profile in [VBA intrinsic functions](vba-intrinsic-functions.md). The shared
implementation is published at tiny-vba revision
`0b41fd7bf7c78a62323133b290f2f9646667f9d3`; GridCraft and WordCraft pin it.
The [implementation plan](implementation-plan.md) now starts at M7.

## Verification

I independently ran these checks on the current tiny-vba checkout:

- `cargo xtask ci` — PASS; 19-crate layer check, workspace tests, 646/646
  candidate hashes, M4 evidence verification, and UI interaction suite pass.
- `cargo xtask wasm` — PASS.
- `cargo xtask wasm-run` — PASS; native and WASI smoke pass, including the
  descriptor, contextual runtime, string-intrinsic, verifier, compiler, VM,
  and IDE test suites.
- In GridCraft, `cargo test -p gridcraft-vba --locked` — PASS (76 tests),
  including `worksheet_udf_calls_shared_strcomp_with_binary_vba_result` and
  `workbook_macro_strcomp_uses_the_configured_binary_provider`.
- In WordCraft, `cargo test -p wordcraft-vba --locked` — PASS (29 tests),
  including `authored_word_macro_uses_shared_strcomp_result`.

The [M6A ledger](m6a-work-ledger.md#m6a-8-acceptance) records the implementation
packet results and the earlier full application gates: GridCraft CI, native UI,
VBA-window UI, WASM and JavaScript-WASM tests; WordCraft CI and WASM checks.
Both forks' commits were pushed to their existing feature branches. This review
did not rerun every application-wide gate; the evidence is the recorded gate
result plus the fresh shared and adapter runs above.

## Accepted limits

The accepted matrix includes executable pure intrinsics, explicit bounded host
dispatch for environment/file/prompt/settings operations, and descriptors that
refuse restricted/platform-only facilities. Such a refusal is not VBA behaviour
or a compatibility pass. Default hosts can report an unavailable service; a
host must explicitly provide a service before a call can perform it.

Comparison, locale, calendar and code-page behavior is injected. Tests use
deterministic providers; this acceptance does not claim the providers reproduce
every Office locale, Windows code page or database collation. Engine-selected
error numbers and precedence that public references do not settle are identified
as profile policy in the ledger and function matrix, not as fresh Office results.
The matrix also records source-profile limitations such as `Spc`/`Tab` print
positioning syntax. No fresh Excel or Word observation was made; those remain
deferred to M8.5.

The audit, all packet status and per-function dispositions remain in the
[intrinsic matrix](vba-intrinsic-functions.md) and [ledger](m6a-work-ledger.md).
This acceptance covers that declared profile. It makes no claim of universal VBA
or worksheet-function parity.
