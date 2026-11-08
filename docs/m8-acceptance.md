# M8 — Conformance, hardening and release evidence

Updated 2026-11-08. Profile: `m8-local-v1` under [ADR 0101](decisions/0101-m8-conformance-and-host-payload.md).
M7A was reverified before M8 work: 61 artifact hashes and all 130 selected cases
passed. Its accepted common forms/worksheet/native persistence/removal scope is
unchanged. P9 remains deferred.

M8.1–M8.4 are **COMPLETE for the declared finite local profile**. Native CI,
actual WASI, affected UI/native/browser and candidate host gates pass. **M8.5 fresh
Excel/Word comparisons remain NOT_RUN and release validation remains open.** The owner has no Office
installation and requested continuation without it. Neither authored contracts,
public documentation nor inherited observations replace that comparison.

Offline M8.5 preparation is complete under [ADR 0102](decisions/0102-m8-release-evidence-gate.md):
[eleven comparison recipes and capture protocol](m8-office-comparison.md), an empty
results template and a bounded typed-evidence validator. Run
`python3 tools/check_m8_release.py` for release readiness; without fresh observations
it returns NOT_RUN (exit 1). Core CI validates preparation, not Office behavior.

## Evidence matrix

| Task | Implemented scope | Evidence boundary |
|---|---|---|
| M8.1 | 24 additional reviewed MIT library expressions with independently published exact types/values and pinned adaptations | Inherited Excel for Mac 16.114 build 26101123, macOS 15.3.1, observation date 2026-10-15; locale not recorded per run. No Windows or fresh Office claim |
| M8.2 | Parser/importer/verified-VM mutation targets, bounded slices/depth/input/decompression, host scratch reuse and negative regressions, eight asserted native/WASI benchmarks | Finite deterministic campaigns; no long-running coverage-guided claim, no portable wall-clock threshold |
| M8.3 | Native CI, browser WASM compilation, actual Node WASI, affected UI/native/browser checks; supported matrix below | Local Linux and Node/Chromium results only; remote macOS/Windows CI not inferred |
| M8.4 | 20 independent original native XLSM workflows, separate language and host categories, save/reopen/trust/undo checks | Rust-built authored workbooks, not external/Office-produced corpus; no global compatibility percentage |
| M8.5 | Fresh permitted Excel/Word context/event/attribute/package/native-form comparison | NOT_RUN; no installed Office or fresh result bundle |

The [reviewed selection](../tests/fixtures/m8/selection.json) pins shared and optional
host inputs. [Library selection](../tests/fixtures/m8-oracle/selection.json) and
raw results retain the upstream MIT notice and exact environment. The ten excluded
candidates retain their unsupported reason. M4's earlier 153 inherited cases stay
separate. [Execution receipts](../tests/fixtures/m8/execution.json) identify commands,
environment, tree/input hashes and actual results; they are not Office observations.

## Supported versions and profiles

| Surface | Version/profile | Remaining limits |
|---|---|---|
| Shared crates | 0.1.0, Rust edition 2024, pinned Rust 1.99.1; minimum Rust 1.95 | Only the documented compiler/runtime subset; no universal VBA support |
| Bytecode | Verified IR 14 and explicit older capability refusal | Source recompilation required when intrinsic metadata changes; source attachment/import stays inert |
| Language/intrinsics | [Runtime contract](runtime.md), [intrinsic inventory](vba-intrinsic-functions.md), M6A portable/injected-service profiles | No ambient clock/locale/filesystem/network/COM; explicit missing-service refusals |
| Foreign calls | Native protocol 1.1, explicit binding/site permissions; checked immediate/waited results | Host effects already performed survive failures; no rollback or arbitrary activation |
| Native forms | `native-forms-common-v1`, worksheet controls and M7A native file/edited-copy workflows | P9 uncommon behavior unsupported; original bytes separate; ordinary document Save retains imported VBA |
| Package imports | Bounded inert MS-OVBA/native forms; explicit source-only writeback APIs | Fixed embedded identity; unsupported/protected/resource/code-page cases refuse; no cryptographic resigning |
| Editor/debugger | egui/eframe 0.37.0 reviewed fork; shared native/browser renderer | Advanced evaluation and platform boundaries remain those of existing acceptance contracts |
| Host acceptance | GridCraft 0.14.2 and WordCraft 0.13.2 with the shared candidate via temporary dependency overrides | Shipping runtime pins remain unchanged; forks own workbook/document behavior, storage and trust |
| Local execution | Linux x86_64; WASI via Node 22.23.4; browser wasm32/Chromium | No fresh Excel/Word comparison; unexecuted OS/build profiles stay NOT_RUN |

`VarType` returns the enum's Long representation; `QBColor` follows the formal
numeric palette. The 64 MiB payload default is unchanged: host scratch is checked
while live and reused; actual copying/work and call/poll ceilings remain cumulative.

## Reproduce

```sh
cargo xtask ci
cargo xtask wasm
cargo xtask wasm-run
cargo xtask m8
cargo xtask m8-bench
cargo xtask m8-hosts   # optional sibling hosts, candidate local dependencies
cargo xtask ui
xvfb-run -a cargo xtask ui-native
cargo xtask web
```

Use Node 22 for the WASI/host/browser commands. Fuzz seed/count controls and bounded
stdin use are documented in [fuzz](../fuzz/README.md). Benchmarks include VM
construction/execution and report compilation separately; compare runs only under
the recorded toolchain, architecture, sample count and build mode.
[Published benchmark distributions](../tests/fixtures/m8/benchmarks.json) include
measurement conditions; they are descriptive timings, not cross-machine speed claims. Host test
sources/provenance live in GridCraft `tests/fixtures/m8/selection.json`; the host
gate restores its shipping lockfile, so no candidate revision is falsely published.

The next external requirement is an Office environment and a permitted fresh
comparison record containing exact source/build/OS/locale, values/types/errors,
event order, document side effects and raw results for the declared M8.5 profile.

## Final local gate receipts

Native CI passes 1,073 test functions; the UI gate passes 574. Full actual WASI
passes 756 test functions with optimized test builds (`CARGO_PROFILE_TEST_OPT_LEVEL=2`,
`CARGO_BUILD_JOBS=2`), recording exit 0 explicitly. Final test-only fuzz/gate
changes also pass focused strict Clippy and both M8 entrypoints. All 15 original
Chromium flows pass. Both candidate host gates pass: Grid 61 native/61 Node WASM;
Word 65 native/39 Node WASM, including only the three actually ported WASM workflow
tests. The new 20 workbook workflows pass on both targets. Grid's shipping-pin
full CI passes all 10 steps; shipping runtime pins stay unchanged.

An earlier form-launch timeout and a terminated default-profile WASI shell
receipt are preserved separately in execution history. The final unchanged browser
assertions and explicit full WASI completion pass; terminated wrapper status is
never counted as a successful gate. All counts refer to their finite named suites,
not universal Office behavior.
