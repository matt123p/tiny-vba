# Original importer seed

`importer.cfb` is an inert, independently authored CFB/MS-OVBA fixture made with
`crates/vba-project/tests/support/mod.rs::fixture().bytes()` on 2026-11-08.
It contains the three original Utilities/Counter/BookCode decoder specimens and
no Office-produced material. License: MIT OR Apache-2.0, as the repository.
It is frozen and hash-pinned by the M8 selection so mutation replay does not
depend on timestamps assigned by the fixture builder. Never execute its source;
the importer fuzz target only decodes it without a VM or host.

`pause-heavy.bas` is an original 140-Stop regression proving that the harness
can cancel at its own slice quota without misclassifying a valid paused program
as a VM failure. It uses the actual default Answer entrypoint.
