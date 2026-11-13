# Shared test data

Rust integration tests live in the owning crate/app's `tests/` directory.
This root tree holds shared inputs, expected results and immutable evidence;
Cargo does not discover tests here automatically. Use `CARGO_MANIFEST_DIR` to
locate fixtures reliably.

| Location | Contents |
|---|---|
| [fixtures](fixtures/README.md) | Authored scenarios, native resources and reviewed milestone selections/receipts |
| [syntax](syntax/README.md) | Source parser fixtures and recovery cases |
| `semantics/m2/answer.disasm` | Reviewed deterministic bytecode disassembly |
| [corpora](corpora/README.md) | External specimen policy, inventory and rights |
| [project-import](project-import/README.md) | Project decoder fixture conventions |

See [testing and evidence](../docs/testing.md) for commands, result classifications,
provenance and the distinction between local tests and fresh Office observations.
