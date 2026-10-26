# M4 independently authored scenarios

These 24 MIT fixtures use schema 1, unique m4-prefixed identities and
`requirements = m4-language-v1`. Expected values are independently reasoned, never represented
as observed Microsoft results. `cargo test -p vba-cli` executes and asserts every fixture.

The directory adds scalar, state, string/intrinsic, array and handler witnesses to the 20 M0
scenarios. More detailed numeric/error/side-effect/lifecycle and malformed-IR assertions live in
the owning crate integration tests. External corpus candidates remain NOT_RUN.
