# 0003 — M0 source, scenario and foreign ownership contracts

Date: 2026-10-26. Status: adopted for the M0 prototype; not a frozen fork integration ABI.

## Ownership and source identity

`vba-diagnostics` owns caller-assigned SourceId, per-source StatementId ordinals, SourceFile,
checked half-open UTF-8 byte spans and optional diagnostic attachments. Source IDs must be unique
within a compilation/session; replacing text requires a fresh source identity. Statement identity
is independent of line number, so colon-separated statements can receive different IDs in M1.
SourceFile checks text/name budgets and span order, bounds and UTF-8 boundaries. Display positions
are one-based lines/Unicode scalar columns; CRLF is one newline. Lone CR line handling and
editor UTF-16 coordinate conversion are future frontend work.

`vba-values` owns distinct Empty/Null/Missing/Nothing/CVErr states, exact scalar widths,
Currency scaled by 10,000, serial-day Date, UTF-16 strings (including lone surrogate code units),
host reference tokens and opaque writable locations. Scalar validation is not coercion.
Locations are bounded, owner-scoped, invalidatable and never reused during an owner's lifetime.
Callers assign unique owner identities and retain the store while a call is pending. These are
Variant-like prototype slots, not declared-type checked locals, arrays or property l-values.
Runtime coercion, copy/reference rules and declared slot types arrive with M3/M4.

## Scenarios and frontend dependencies

`vba-conformance` parses schema 1 defined in [scenario format](../scenario-format.md). Its strict,
bounded UTF-8 key/value envelope embeds the exact VBA text and typed expectation. This small format
needs no external parser dependencies. It deliberately accepts only self-authored, MIT,
capability-free, unmeasured pure-language fixtures. External/oracle adoption needs a separately
versioned extension for hashes, exact product/build/locale, setup, effects and array expectations.
VBA text is retained verbatim; parsing a fixture does not parse or run its VBA. The initial 20 cases
are reasoned test proposals, not Microsoft observations. Validation never changes NOT_RUN to PASS.

The development CLI may normally depend on conformance to expose fixture validation. This is an
explicit exception to the test-support dependency rule for the command frontend only. Shipping
engine/debugger/import/UI libraries still cannot normally depend on conformance or testkit.
Conformance now depends directly on values for typed expectations. The reviewed graph and layer
checker enforce both changes. No external dependencies, sibling applications or platform APIs
were added; native and WASM library gates remain applicable.

## Native foreign invocation and waits

`vba-foreign` owns Call/Get/Let/Set kinds, owned positional/named/omitted arguments, ByRef location
obligations, result/error/wait and the single-pending-call Session controller. `vba-testkit` supplies
a bounded deterministic scripted host with owned traces, not an Office-shaped object model.

Ordinary calls use Rust directly. The controller resolves locations to owned value snapshots;
the host receives ByRef flags and returns ordered argument-index writebacks, never VM locations
or borrowed documents. All patches are validated before any VM write. Repeated locations apply
writes in listed order, last write wins. This proves ownership and delayed writeback, not the final
live-alias semantics of VBA/COM parameters. Error replies preserve their exact number/description
and currently carry no VM writebacks. Final alias, type-conversion and partial-ByRef-write-on-error
behavior must be designed/tested in M4 before this becomes a frozen M5 bridge contract.

A wait stores only an owned token, host identity and location obligations. No mutable host/VM
borrow crosses a return. Resume uses the saved token without replaying invoke; another wait may
replace the token. One outstanding call is allowed, with at most 1,024 wait replies. A second
invocation, duplicate resume or a different host is rejected. A wrong-host resume/cancel retains
the pending call. Hosts provide stable, session-unique identity; they own object/generation
validation, member/capability checks and external state. The deterministic double rejects stale
references explicitly. These tokens are not a security boundary against a dishonest host.

Stale locations cancel the pending token before host resumption. Cancellation is explicit and
idempotent; caller must call cancel before stop, close, disconnect or dropping a pending Session.
Automatic Drop cancellation cannot borrow a host and is not promised. The upper VM/debugger must
wire these lifecycle paths at M3/M7. Neither cancellation nor errors imply rollback of successful
host side effects. The prototype has no callbacks, ambient IO or Office catalogue.

## Evidence and next gate

Integration tests exercise source bounds/identity/Unicode/same-line IDs, scalar/location distinctions,
all 20 fixture validations through the CLI, malformed fixtures, omitted/named/ByRef snapshots,
ordered alias patches, malformed/non-ByRef patches, errors, repeated waits, host identity, stale
locations/references, cancel and suspension budgets. `cargo xtask ci` and `cargo xtask wasm` are
required gates. VBA execution and actual WASM execution remain M3 work. Next is M1's lexer and
source-preserving parser for Arithmetic.bas and the reviewed small statement subset.
