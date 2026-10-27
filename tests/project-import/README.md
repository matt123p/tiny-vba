# Project import fixtures

M6 uses independently authored in-memory CFB/MS-OVBA projects in
[the owning crate's test support](../../crates/vba-project/tests/support/mod.rs).
[Fourteen integration tests](../../crates/vba-project/tests/import.rs) cover read-only extraction,
identities/code names, references/attributes, encoding, malformed/protected/unsupported projects,
cycles, truncations and limits. The same suite executes natively and on WASI.

No proprietary binary fixtures or third-party workbooks are needed. Source with Auto_Open and
an unknown host call remains inert after decoding. No VBA executes during these importer tests.
See [the import contract](../../docs/project-import.md).
