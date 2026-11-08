# ADR 0100: Common native forms acceptance

Date: 2026-11-08. Status: adopted; M7A.7 and M7A COMPLETE for the common profile; all required receipts recorded.

M7A.7 accepts the common scope chosen by the owner in ADR 0097, together with
M7A.3 worksheet controls, ADR 0098 explicit native file/edited package-copy
persistence and ADR 0099 removal of the custom format. It does not reopen P9,
expand the runtime or claim fresh Excel/Word results. The conformance label
`m7a-common-native-v1` describes an acceptance set, not a VBA feature or format.

The reviewed selection records original Rust fixtures/tests, their source hashes,
license/review, separate load/compile/UI/run/edit/save/reopen expectations and
negative safety boundaries. Integrity checks print VALID_FIXTURE and NOT_RUN;
only running the authored Rust assertions establishes execution PASS. Existing M7
context/event/original-byte/signature-preservation regressions stay in the gate.

Both native package suites additionally reject truncated f/o streams, unknown
control classes and hostile site lengths without document mutation. Their edited
copy tests now reopen actual source/resource edits and stop the saved handler
before real workbook/document mutation. Continue runs it once, undo restores
prior host content and Reset clears instances. Invalidated signature removal,
source/design revision checks and original import bytes remain independently
asserted. Renderer/input/trust/cancellation/recalculation behavior remains covered
by the unchanged shared and host UI/runtime suites.

`cargo xtask m7a` is an optional real-sibling acceptance gate. It checks reviewed
hashes/entrypoints and retirement of the custom product format, then runs the
selected native/shared/file/host/worksheet/M7 regressions, failing on any nonzero
status. Ordinary core CI checks local selection integrity only and requires no
sibling checkout. No arbitrary commands, downloaded macros, extra dependencies,
VM/host authority or changed execution/memory limits are introduced. Host tests
remain in their owning forks; source/runtime/IO/package policy ownership is intact.

The final acceptance record distinguishes fresh changed-host tests from retained
M7A.6 full WASM/WASI/UI/browser evidence for unchanged shared production sources.
The selection pins a shared production/dependency tree digest and native/browser
inputs so those receipts cannot silently cover changed implementation. Fresh
Office comparisons remain NOT_RUN under M8.5. A missing gate never becomes PASS.

Acceptance is for the supported common workflows: native file Save/Export and
explicit edited XLSM/DOCM copies. Ordinary document Save still preserves imported
VBA bytes, package copies retain original editor dirty state, and browser download
requests do not acknowledge disk commits. Component insertion/renaming during
embedded project writeback, unsupported resource/group edits, protection bypass,
cryptographic verification/resigning and arbitrary COM/OCX/DLL activation remain
unsupported. Uncommon/exhaustive parity remains P9, after M8.
