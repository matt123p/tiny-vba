# M8 completion audit — findings and corrections

Historical audit. Its milestone IDs and future allocations predate the
[remaining plan](../implementation-plan.md); use that plan for new work.

Date: 2026-10-27. Audited implementation: `f4629598b7ce475f5fef4b478c5b6d93338877b6`.
**Original result: three delivered-scope defects found. The correction re-audit below records fixes and current verification.**
The M10–M14 feature allocation is unchanged; these are M8 corrections, not new deferred features.

The working tree was already committed. `git push origin main` pushed `f462959`, and
`git ls-remote origin refs/heads/main` confirmed that exact SHA. This record documents the subsequent
audit; any commit containing this report additionally records the audit findings.

## M8-A1 — Normal reruns silently discard Static state

[Ide::launch](../../crates/vba-ide/src/lib.rs#L507) always creates a fresh `Session` without transferring
its completed predecessor's `project_state`. Run this authored module twice through the same Ide,
without source edits or UI Reset:

```vb
Sub Main()
    Static counter As Long
    counter = counter + 1
    Debug.Print counter
End Sub
```

Expected output: `1`, then `2`. Actual output: `1`, then `1`. The IDE loses project state on ordinary
completion/relaunch, although the runtime provides `project_state`/`restore_project` and the UI's
explicit Reset owns state loss. Correct the invocation lifecycle and add reset/source-revision/host
binding regressions before closing this finding. Core VM state transfer already has separate tests;
those tests do not prove the GUI/model launch path uses it.

## M8-A2 — Find Next remains on the current occurrence

[The find bar](../../crates/vba-ui-egui/src/lib.rs#L247) searches from `m.caret` inclusively. Open Find
with Ctrl+F, enter `counter` in the authored demo and click Find Next twice. Both actions leave the
caret at byte **73**, the first occurrence. Expected: the second action advances to the next match,
with a documented wrap behaviour after the final match. This is a real egui pointer/keyboard probe,
not a model-only search assertion. Preserve UTF-8 boundaries and query-change handling in the fix.

## M8-A3 — Lone-CR source has inconsistent debugger line coordinates

[Module::line_at/line_start](../../crates/vba-ide/src/lib.rs#L52) count only LF; the frontend and
`SourceFile::position` recognise LF, CRLF and lone CR. Import this valid source using `\r` separators:

```text
Sub Main()\rDim x As Long\rx = 1\rEnd Sub\r
```

Setting a breakpoint at `x = 1` is rejected as nonexecutable because the IDE treats that location as
line 1 while compiled checkpoints use line 3. Navigation, gutter numbering and diagnostics must
share the compiler's line-ending convention. Add LF/CRLF/lone-CR and Unicode coordinate regressions.

## Reproduction and evidence

```sh
python3 tools/audit_m8_completion.py
```

This maintenance runner reuses the existing independently authored Rust fixtures and adds three
temporary integration probes. It executes the actual model and egui harness, returns exit **1** while
any probe fails, and removes temporary `.rs` files afterwards. All **three probes failed** against
the audited implementation. They are not silently skipped or counted as CI passes. The runner is
separate from default CI so the passing baseline and newly discovered failures remain distinguishable.
When corrected, promote the cases into permanent owning-crate regression tests and update this report.

Positive local checks rerun during this audit:

- `cargo xtask ci`: **219 existing tests pass**, formatting, strict Clippy, layer/provenance/corpus checks.
- `cargo xtask ui`: model/UI/shell tests pass; seven real paused-state snapshot variants render.
- `cargo xtask wasm` and `cargo xtask wasm-run`: compile/execution gates pass.
- `cargo xtask ui-native`: real native window capture succeeds under Xvfb.
- `cargo xtask web` and the Chromium smoke runner: real F8/F5 pause/completion and output checks.

Passing existing checks does not discharge the three additional failing probes. Native/browser
rendering output remains ignored under `reports/ui/`; no reference evidence was downloaded for this
audit and the original public-source/provenance restrictions remain in force. Remote GitHub Actions
status is separate from these local results and should be checked on the relevant pushed commit.

## Remote CI gate — Windows checkout-location regression

The [push-triggered run](https://github.com/matt123p/tiny-vba/actions/runs/40545722437) completed
with UI, corpus, WASM, Linux-native and macOS-native jobs successful. The
[Windows-native job](https://github.com/matt123p/tiny-vba/actions/runs/40545722437/job/122335678396)
failed in `cached_binary_uses_the_invoking_checkout_instead_of_its_build_path`, at
[xtask/tests/checkout.rs](../../xtask/tests/checkout.rs#L23). Its generated Python check fails:

```python
assert Path.cwd() == Path(__file__).resolve().parent.parent
```

The reported temporary directory uses a Windows short-name component (`RUNNER~1`). The observed
failure is the checkout-root path comparison; canonical path/alias handling needs investigation
and verification on Windows. This is a separate CI/tooling gate failure, not an additional claimed
UI feature defect. The three M8 product probes above fail independently on Linux.

Decoded logs were retrieved using the connected GitHub workflow-job-logs tool and retained only in
ignored `.cache/evidence/m8-audit/windows-ci-job.log` (SHA-256 `5c7257eb31754b2732b89152b40f98dd5aec1c305d0541c93a11145dcbdad136`). They are not checked in.
The local 219-test pass must not be represented as a fully green cross-platform workflow.

## Completion condition

M8 can regain a verified-complete label after M8-A1–A3 are fixed, their permanent regression tests
pass, the required native/WASM/UI gates are rerun, the Windows CI regression is resolved and
verified remotely, and the resulting commit is pushed. No M10–M14
feature is required to close these three M8 findings.

## Correction re-audit — 2026-10-27

The failures above describe the original audited commit. All three product defects are now fixed:

| Finding | Correction | Permanent regression evidence |
|---|---|---|
| M8-A1 | Transfer terminal VM ProjectState into compatible same-host invocations; Reset/source/host changes prevent reuse | `m8_static_reruns_reset_and_source_revision`, `m8_host_identity_change_does_not_restore_previous_project_state` |
| M8-A2 | Advance beyond the previously found occurrence and wrap; remember query/source/caret | `m8_find_next_advances_and_wraps` (real Find field and pointer buttons) |
| M8-A3 | Shared CR/CRLF/LF physical line starts and checked source/display projection; untouched mixed endings preserved | `m8_line_endings_and_unicode_share_compiler_coordinates`, `m8_editor_diff_preserves_mixed_line_endings_and_unicode_offsets`, `m8_cr_editor_typing_keeps_source_endings_and_gutter` |
| Windows checkout assertion | Compare filesystem identity with `Path.samefile`, accepting Windows short/long aliases | Existing invoking-checkout regression; remote Windows confirmation is required below |

The user additionally made functional IntelliSense/syntax feedback M8 requirements. Design-mode
scoped symbols/parameters, simple typed receivers with supported host metadata, project signature
help and an editor-anchored keyboard/pointer completion list replace the original dialog-only path.
Live parser messages and red/underlined source lines now update/clear on edits without Compile or
moving the typing caret. Tests inspect actual rendered underline sections in both themes. EOF and
lexical errors, out-of-scope/private symbols, unknown receivers, correction, CR editing, Escape focus,
fast typing after acceptance and normal editor commands are covered. Edit menu selection/clipboard/
undo/redo commands preserve selection/history across focus changes. Browser Paste uses its standard
keyboard route; the standalone web menu explains that boundary.

`tools/audit_m8_completion.py` now runs the permanent `m8_` regressions directly; temporary failing
probes have been removed. Public examples/references and intentional nonmodal/live-check policy are
in UI specification section 16. All ten reference-page downloads and the prior Windows log remain
ignored cache evidence; only provenance/observations are committed.

Local gates: `cargo xtask ci` (236 workspace tests, format, strict Clippy and repository checks),
`cargo xtask ui` (33 IDE/embedding, 21 UI interactions, shell tests and eleven original render variants),
`cargo xtask wasm`, `cargo xtask wasm-run` (including 33 headless IDE tests), native Xvfb capture,
`cargo xtask web` and Chromium keyboard/clipboard/debugger/syntax correction smoke. Generated product
images remain in ignored `reports/ui/`; reference-cache content is never checked in.

Remote CI: all six jobs passed on [run 40552813785](https://github.com/matt123p/tiny-vba/actions/runs/40552813785) for pushed implementation commit `e24d07a3b86fb2550455957e51c67d99ca17a716`: Windows, Linux, macOS, WASM, UI and corpus verification. The Windows invoking-checkout regression now passes. M8 is complete for the defined baseline. This documentation closure changes no product/test code. No fresh Office or real-fork integration
result is inferred from these checks; M10–M14 boundaries remain explicit in the roadmap.

Windows confirmation: job [122360798367](https://github.com/matt123p/tiny-vba/actions/runs/40552813785/job/122360798367)
passed, including `cached_binary_uses_the_invoking_checkout_instead_of_its_build_path`.
Its raw log is retained only as ignored `.cache/evidence/m8-audit/windows-ci-corrected-job.log`
(SHA-256 `028dd93d9bfbf48d55ce290bca28b244bdfed78d6289c254eb4abc8ecc2ad371`).
