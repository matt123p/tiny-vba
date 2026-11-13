# tiny-vba — instructions for agents

Read the repository-local [development skill](.agents/skills/tiny-vba-development/SKILL.md)
and the focused skills relevant to implementation/design work. Start with
[architecture](docs/architecture.md), [compatibility](docs/compatibility-matrix.md)
and the owning contract in the [documentation index](docs/README.md).
Current user direction and adopted decisions take precedence over historical
milestone wording. Routine choices within the agreed scope need no extra approval.

## Ownership and compatibility

- Implement a fresh Rust VBA engine. Never reuse `craft-vba` or another
  interpreter's implementation. Use public specifications and permitted black-box
  observations; never inspect proprietary Office implementation binaries.
- Mirror applicable Excel/Word behavior, native formats and workflows. Do not add
  custom user-facing language features, file formats or control semantics.
  Unsupported behavior fails explicitly.
- Forks own Office objects/catalogues, document state, trust, OOXML, undo/recalc
  and application integration. `vba-foreign` is a host-neutral native Rust protocol.
  Office-shaped test doubles belong in the forks.
- tiny-vba owns the reusable editor/debugger UI and standalone app. Toolkit/IO
  dependencies stay above the engine and headless IDE model. The VM never
  depends on debug/DAP/UI. Preserve [the reviewed graph](tools/architecture.json);
  `cargo xtask layers` enforces it.

## Correctness and safety

- Product code is Rust; Python is for maintenance checks. Inherit workspace lints.
  No unsafe, unwrap/expect, panic, todo/unimplemented or unchecked input indexing
  in production code. Return structured errors and bound input-derived allocation,
  recursion and execution work.
- Centralize VBA coercion, ByRef locations and error state. Preserve exact tags,
  numeric widths, Currency/Date and array lower bounds. Do not flatten to JSON/f64.
- Use verified bytecode, explicit frames, statement checkpoints and versioned
  capabilities. Resume must not replay effects. Preserve stale-handle checks,
  cancellation and cumulative budgets across pauses/waits.
- Inject time, randomness, locale and prompts. No ambient filesystem/network/shell/
  COM in the VM. Watch inspection is inert; Immediate function evaluation requires
  explicit policy and isolated execution.
- Source extraction/attachment is inert. Ordinary document Save retains imported
  VBA bytes; native form Save/Export and edited XLSM/DOCM copies use explicit,
  revision-checked snapshots. Protected-project bypass, P-code execution and
  cryptographic signature verification/resigning are unsupported.
- Native forms use ordinary canonical VM state and `.frm`/`.frx` resources.
  Keep original resources, authored design/source and runtime state separate.
  Do not restore the retired custom codec/converter/routes. Follow [forms](docs/forms.md).

## Evidence and verification

Read [corpus policy](tests/corpora/README.md) before external corpus work.
Do not edit upstream submodules, run downloaded scripts/macros or adopt unlicensed
samples without an explicit reviewed policy/selection change. The reviewed MIT
allowlist is the bounded execution exception. Pinning alone is not execution
permission or a compatibility result. Retain third-party notices and original assets.

Rust integration tests belong under the owning crate/app; root `tests/` holds
shared data. Verify types, errors, effects, negative paths, budgets and pause/resume.
Crashes require minimized regressions. Preserve immutable selections and receipts;
never invent Office observations. Common forms/local hardening acceptance does
not satisfy fresh Office comparison: M8.5 remains NOT_RUN and release validation
remains open. Historical P9 identifies unsupported uncommon/exhaustive forms scope.

Run `cargo xtask ci`, `cargo xtask wasm` and `cargo xtask wasm-run` for implementation
changes, plus affected UI/browser/host gates. For documentation-only changes,
verify links, referenced commands and repository/evidence integrity. Report exact
checks and unavailable coverage. See [testing](docs/testing.md) and
[development](docs/developer-guide.md) for prerequisites.

Update the owning guide and compatibility claims with behavior changes. Record
material architectural changes in `docs/decisions/`. Keep task plans, progress
logs and session handoffs out of maintained reference docs; use Git history for
past work. Two scope documents remain byte-for-byte historical evidence inputs,
as explained in [testing](docs/testing.md).
