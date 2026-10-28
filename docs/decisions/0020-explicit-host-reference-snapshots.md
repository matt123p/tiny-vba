# ADR 0020 — Explicit host reference snapshots

Date: 2026-10-28. Status: adopted for the first new M1.1 increment.

## Decision

`vba-foreign` owns an immutable validated `ReferenceSet` built from caller-supplied
library names, versions, availability, type descriptors and typed constant values.
This is native metadata, not a loader, COM registry or Office object model.
`CompileOptions.references` passes the snapshot to `bind_with_references`.
Existing entry points retain empty-reference defaults.

Reference enum types bind to VBA Long using shared coercion, parameter/ByRef,
array and return rules. Constants bind at compilation into existing typed IR
constants, including constant expressions, Optional defaults and array bounds.
No IR version or host invocation ABI changes: execution uses the verified program,
not a live metadata lookup. Changing references requires a fresh compilation.

Names are case-insensitive. Ordered references choose the first unqualified
symbol; library qualification selects explicitly. Constants additionally support
module and library/module qualification. Source variables/constants and imports
retain precedence; constants cannot be assigned or called. Duplicate library/type
and constant identities within one library are rejected rather than guessed.

Qualified declared-type syntax retains exact source spans. Missing references
report VBA0131; undefined declared types report VBA0132; malformed metadata reports
VBA0130. Declared nominal Object descriptors are explicitly unsupported, never
silently mapped to generic Object. Nominal object identity/assignment validation
and the unchanged `As Worksheet`/`As Excel.Range` proof remain unfinished M1.1 work.
Default-member and collection execution remain M1.2/M1.3.

## Bounds and lifecycle

Limit snapshots to 64 libraries, 4,096 total type/constant descriptors and 1 MiB
declared payload. Identifiers are ASCII, alphabetic-first, at most 128 bytes;
versions are nonempty, at most 128 bytes and contain no controls. Constants reject
arrays, object handles, Nothing, Missing and invalid/nonfinite values. Lookup
paths are length-bounded and indexed; snapshots cannot be mutated after validation.

The IDE accepts bindings only without a session, derives discovery rows from the
same snapshot and passes it to ordinary and opted-in isolated Immediate compilation.
Declarative `configure_metadata` remains discovery-only; it grants no execution.
There is no ambient filesystem, network, DLL/COM activation or macro execution.

## Evidence

Independently authored native tests cover validation/budgets/priority/qualification,
source spans, actual enum/constant execution, scoping, invalid writes/calls,
missing references, IDE reset guards and isolated compilation. VM and IDE reference
tests run under WASI. No new external corpus or Office oracle observation is claimed.
See [the implementation log](../m1-host-references.md) for gates and continuation.
