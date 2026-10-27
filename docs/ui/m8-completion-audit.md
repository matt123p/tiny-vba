# M8 completion audit — open defects

Date: 2026-10-27. Audited implementation: `f4629598b7ce475f5fef4b478c5b6d93338877b6`.
**Result: M8 is implemented, but full completion is not verified. Three delivered-scope defects remain.**
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
