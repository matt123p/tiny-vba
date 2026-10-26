# Syntax fixtures

`m1/control.bas`, `m1/metadata.cls` and `m1/invalid.bas` are independently authored MIT fixtures
for M1 control flow, source-only class metadata and exact error/recovery assertions.
Rust integration tests live in `crates/vba-syntax/tests/frontend.rs` and also parse the two examples
and all 20 M0 scenario bodies. None executes VBA. Normal tests need no external checkout.

See [frontend](../../docs/frontend.md) for the grammar matrix/bounds and
[the external grammar report](../../docs/m1-corpus-report.md) for the separate opt-in allowlist.
