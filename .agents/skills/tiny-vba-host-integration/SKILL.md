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
