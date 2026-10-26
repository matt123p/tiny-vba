# oracle-goldens

Reviewed, versioned type/value/error observations tagged with exact Office profile and build.

This directory reserves the test-data location; it is not automatically discovered by Cargo.
Rust integration tests must live under the owning crate’s `tests/` directory and load these fixtures using `CARGO_MANIFEST_DIR`.
No VBA scenarios execute yet. See [the implementation plan](../../docs/implementation-plan.md).
