# fixtures

Small self-authored VBA and scenario manifests. Downloaded workbooks stay in corpora; local/ is ignored.

`m0/` contains 20 independently authored MIT `.vbas` scenarios with typed, reasoned expectations.
See [schema 1](../../docs/scenario-format.md). The conformance and CLI integration tests validate every
file and suite identity; all VBA cases remain NOT_RUN. No external specimen was adopted.
Rust integration tests live under the owning crate’s `tests/` directory and locate these files using
`CARGO_MANIFEST_DIR`. No VBA scenarios execute yet; M1 frontend work is next.
