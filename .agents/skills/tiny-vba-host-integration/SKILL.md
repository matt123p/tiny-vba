---
name: tiny-vba-host-integration
description: Design or review tiny-vba generic foreign calls, host references, ByRef/wait ownership, source import and application integration contracts. Use when bridging the engine or IDE to external document applications.
---

# Host and import boundaries

Read [architecture ownership](../../../docs/architecture.md),
[scope ADR](../../../docs/decisions/0001-repository-scope.md) and
[UI ownership amendment](../../../docs/decisions/0002-editor-debugger-ui.md).
Inspect real host APIs before proposing an adapter; do not implement imaginary Gridcraft methods.

M0's current owned snapshot/writeback and explicit cancel contract is in
[ADR 0003](../../../docs/decisions/0003-m0-source-scenarios-foreign.md). Inspect vba-foreign and
vba-testkit tests before extending it; final alias/type/error writeback behavior is not frozen.
Session callers must cancel on close/disconnect; Drop cannot access a host. Host/location owner
identities must be unique during their session.

The [IR 1 specification](../../../docs/bytecode-vm-specification.md) requires an exactly-once VM-owned
host continuation and an M3 typed-writeback adapter. The restricted snapshot feature rejects unsupported
foreign aliases/error writebacks; it does not define full VBA ByRef. Close/stop/disconnect must cancel
pending host tokens, and a vanished host must not be reported as successfully cleaned up.

## Keep the boundary generic

vba-foreign carries invocation kind, reference/target, named/positional/omitted arguments, ByRef
obligations and explicit result/error/wait state. Preserve Missing versus Empty, Nothing versus Null,
and property Let versus Set. Prove ownership/error/suspension with a deterministic generic-host double.

Applications own Office names, member IDs/defaults/catalogues, document/object identity, Office-value
conversions, undo/recalc, OOXML packaging and trust/lifecycle decisions. Office-shaped contract tests
belong there. tiny-vba owns VBA values/classes and source decoding, plus the reusable editor/debugger;
Gridcraft embeds those panels instead of maintaining a second editor.

Ordinary host calls use native in-process Rust dispatch. JSON/DAP must not become the hot-path member
ABI. No ambient filesystem/network/shell/COM or native dialogs in the VM. Supply time, random, locale,
prompts and allowed capabilities explicitly. Capability checks cover late binding, callbacks, watches
and user functions, not just the initial run request.

Before pause/yield/callback, release mutable document borrows. Define stale-reference, reentrancy,
close/cancel/disconnect and cleanup outcomes. Preserve host identity; do not add a parallel registry
or document model. Do not impose whole-macro rollback to simplify failure handling; the host's chosen
compatibility policy determines what happens after a successful write followed by a runtime error.

## Decode without executing

vba-project reads bytes/source into module metadata. Preserve source identities, code names, encoding
and attributes. Bound decompression/allocation/nesting; reject unsupported/protected/corrupt inputs
explicitly. Import never executes macros. Project attachment, consent and Office package preservation
are host tasks. P-code, protection bypass and VBA binary project writeback remain outside the initial scope.

Verify generic invocation and pause/resume locally, including omitted/named/ByRef arguments, errors,
suspension and stale handles. Real workbook mutation, recalc/undo and package round trips require
fork integration evidence; do not claim those from a generic double or source decoder alone.

## Implemented M3 contract

Read [runtime](../../../docs/runtime.md) and [ADR 0007](../../../docs/decisions/0007-m3-execution-and-debugger.md).
The m3-core-v1 runtime is intentionally narrower than structural IR. Preserve explicit frame/alias
storage, one-visit checkpoint permits, saved fault PCs, generation checks and nonreplayed continuations.
Use vba-runtime for operations/edits. VM host dispatch must use checked precommit return/writeback
validation and explicit per-site permissions; foreign aliases remain unsupported. Run cargo xtask ci,
wasm and wasm-run (native plus actual WASI execution). Runtime example results do not mark unrun M0
or external scenario assertions PASS. Graphical UI, DAP and VBA error-handler semantics remain later work.

## Implemented M5 contract

Read [ADR 0013](../../../docs/decisions/0013-m5-host-contract.md). The current consumer profile is
`m5-host-v1` over verified IR 1/2/3/4. `vba-foreign` owns protocol version 1 (`API_VERSION`,
`ProtocolVersion`, `check_protocol`); hosts report their version through `Host::protocol` and the
foreign session and VM refuse incompatible majors/minors before any dispatch. Embeddings supply
generic source imports (`vba_foreign::Import`, Call/Get only, validated by `validate_imports`) through
`CompileOptions::imports`; each import lowers to a hidden IR 4 `HostRoot` slot plus a fixed member
call/get and can never be written or captured. `HostBinding` carries case-insensitive named roots
(host identity must match, cap 64); missing or mismatched roots refuse launch, and
`HostBinding::for_program` derives the full site allowlist from a verified program. Reuse
`vba_testkit::run_to_boundary` and `vba_testkit::statement` for bounded bridge drivers;
vba-vm/tests/embedding.rs is the break/edit/resume proof with exactly one dispatch. Office-shaped
names, catalogues, workbook FillCells evidence, undo/recalc and permission gates remain fork-owned
(Gridcraft); do not add them here.

## Implemented M7 contract

Root-object imports (`member: None`) preserve actual property/method dispatch under
[ADR 0017](../../../docs/decisions/0017-imported-root-members.md). Only fixed-member
aliases consume the whole member application. Preserve the exact dispatch regression
in vba-vm/tests/embedding.rs; do not silently discard an imported root's member call.

GridCraft baseline integration is tracked in
[the staged work log](../../../docs/gridcraft-integration.md); fork-owned tests supply
workbook/undo/recalc evidence. Neither fork becomes a tiny-vba dependency.

Read [ADR 0015](../../../docs/decisions/0015-m7-callbacks-and-lifecycle.md). The VM stays
non-reentrant: schedule VBA callbacks with `Session::schedule_callback` (FIFO, `Limits.callbacks`
budget, name/arity validated) and deliver each with `Session::run_next_callback` once the current
program is terminal; a delivery runs to a boundary, shares static state and bindings, applies
breakpoints, and waits through `resume_host` exactly-once. `reset_state` (terminal-only) and
`rebind_roots` cover static reset and the host's own object-lifetime model; a stale root at
delivery surfaces the host's reference error 91. Cancel/disconnect/host loss drain queued
callbacks without executing them and permanently close the lifecycle even after completion.
Reset requires a terminal/open/current session, clears output/static/Err state and prepares the
original entry with original bound arguments; queued callbacks, instance/debugger identities and
lifetime budgets survive. Invalid delivery quotas/host/source/lifecycle preserve the queue and
outcome; argument types are checked before enqueue and queued metadata/payload share the cumulative
session budget. Top-level ByRef arguments are owned locations. Run the callback suite natively and
through cargo xtask wasm-run. Event ordering, `EnableEvents`, document events and UDF policy
remain fork-owned over this queue.

M6 is implemented under [ADR 0014](../../../docs/decisions/0014-m6-project-decoding.md).
Read [the import contract](../../../docs/project-import.md) before changing vba-project.
Keep extraction read-only/inert, project-local ModuleId separate from caller-owned SourceId,
UTF-8 attribute spans and code names distinct from stream/display names. Preserve explicit
corrupt/unsupported/protected/limit refusals and container/decompression work caps. cfb/encoding_rs
are reviewed pinned external dependencies; package preservation and trust UX remain fork-owned.
The 14 controlled importer tests run natively and under WASI through cargo xtask wasm-run.
