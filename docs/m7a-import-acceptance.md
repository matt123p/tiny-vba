# M7A.1 / M7A.2 — native UserForm import and attachment acceptance

Date: 2026-11-04. Profile: `native-userform-import-v1` under
[ADR 0040](decisions/0040-native-userform-import-and-attachment.md).
This accepts inventory/contracts and the selected native import/attachment subset;
full M7A, Office semantics, edited native persistence and `.tvform` removal are
separate remaining tasks. Fresh Office comparisons remain NOT_RUN under M8.5.

## Contract and fixture provenance

The [control/property/event/format/host matrix](m7a-control-contracts.md) completes
FC0.2; ADRs 0039/0040 complete FC0.3. [The audited path inventory](m7a-inventory.md)
covers the actual shared and host paths. The profile admits a native UserForm root
with immediate Label/TextBox/CommandButton sites. All other control/resource/behavior
variants are explicitly unsupported pending their native packet.

Original Rust fixture builder `crates/vba-project/tests/support/native_forms.rs`
authors MS-OFORMS f/o/site/control/TextProps records and a lawful CFB envelope.
Its source includes ordinary export attributes, exact CRLF handlers, an inert
Initialize marker and a Button that prints the initial TextBox value then hides
the form. There are no ambient IO/COM/network requirements or vendor assets.
Fork copies keep their tests self-contained and use each repository's formatting.
All are MIT OR Apache-2.0; no Office output, external implementation or downloaded
macro was adopted. The fixture generator emits only this authored data.

## Assertions by stage

| Stage | Evidence | Result |
|---|---|---|
| Inventory/contracts | Current code paths; per-property/event/control/format/host matrix; version/ownership/dependency/limit/rights/persistence contracts | Complete FC0 |
| Native export | `.frm` header + named `.frx`; exact source/attributes/resources, native IDs/name/value/caption/size/order | Authored regression PASS |
| Embedded input | Separate BaseClass/MS-OVBA fixture with distinct MODULESTREAMNAME, VBFrame and f/o streams | Authored regression PASS; design equals export |
| Malformed/unsupported | Every truncated f/o byte boundary; unsupported classes/properties/binding/flags; missing companion, traversal, inconsistent name, length/budget/protection failures | Authored regressions PASS |
| Shared attachment | No output/init on import, unchanged existing project and SourceId on failure; exact source and independent original resources | Authored regressions PASS |
| Runtime/debugger | Explicit modal Show -> Initialize once, Button handler, Stop/continue without replay, return and Reset | Authored regressions PASS |
| Shared rendered UI | Native import -> Project Explorer, Properties, designer, actual runtime Button | egui harness PASS |
| Standalone native IO | `.frm` selection loads bounded validated sibling companion; failed import keeps existing IDE | Shell regression PASS |
| GridCraft XLSM | Native attachment, explicit consent before run, modal handler, Reset, original project bytes retained on save/reopen | Host regression PASS |
| WordCraft DOCM | Independent native attachment, explicit authorization before run, modal handler, Reset, original project bytes retained on save/reopen | Host regression PASS |
| Native edited saving | Requests and modified-form save acknowledgments refuse; no custom-format substitution | Unsupported until M7A.5 |
| Broader controls/lifecycle | Full Office state/default-instance/multi-instance/input/event ordering | M7A.4, not accepted here |

## Validation commands and environment

Environment: Linux x86_64, Rust/Cargo 1.99.1; actual WASI via Node 18.19.1; actual
JavaScript/WASM host tests use the existing Node 22.23.4 toolchain. Browser
validation uses Chromium/Playwright against the original locally built UI.
Optional acceptance fonts remain an external build input and are never copied.

Required validation PASS is recorded in [progress](implementation-progress.md).
The web bundler uses an isolated wasm-bindgen CLI 0.2.126 matching the repository;
the JavaScript/WASM host runner uses the existing 0.2.129 tool. Commands:

```text
cargo xtask ci
cargo xtask wasm
cargo xtask wasm-run
cargo xtask ui
cargo run --locked -p vba-ui-egui --example native_form_snapshots
xvfb-run -a cargo xtask ui-native
cargo xtask web
cargo run -p vba-project --example native_form_fixture -- /tmp/m7a-native-fixture
VBA_NATIVE_FIXTURE_DIR=/tmp/m7a-native-fixture node tools/ui-browser/native-forms.cjs
node tools/ui-browser/smoke.cjs
node tools/ui-browser/forms.cjs
```

Both forks run full `cargo xtask ci` and targeted native/actual JS-WASM
`cargo test -p <host>-vba --test native_forms`, plus optional VBA UI checks.
Historical `python3 -O tools/check_m7.py` must retain all 12 artifact hashes;
new form cases supplement those unchanged historical artifacts. No fresh Office
test or edited binary writeback PASS may be inferred from these commands.

## Completed task definitions

M7A.1: audit import/designer/runtime/persistence and reference contracts; publish
control/property/event/format/host matrix, lawful fixture provenance, platforms,
versions, dependencies and acceptance ledger, keeping UserForms, worksheet Form
Controls and Office-supplied worksheet ActiveX separate.

M7A.2: bounded inert native export and embedded designer import for the reviewed
subset; preserve source, component/code/storage names, resources and native root/
child identities. Attach to shared designer/code/Properties/debugger workflows in
standalone and both forks. Native export-format import and embedded import have
separate tests. Native export writing is M7A.5.

Next: M7A.3 source DrawingML identity/reconciliation and worksheet interaction;
M7A.4 native lifecycle/property/input parity and broader control packets; M7A.5
native encoders and edited-save/signature/cache policy; M7A.6 custom-format removal.

Fixture builder SHA-256 identities at acceptance (formatting differs by repository):

| Repository | SHA-256 |
|---|---|
| tiny-vba | `b69945fd23937cc093ef2a7799ec7732494ad2dbd1a7a61467348227263ac3a0` |
| gridcraft | `70cfc51550677911491e7c011850dea2065b94291cce4a8adf37daca05369e88` |
| wordcraft | `70cfc51550677911491e7c011850dea2065b94291cce4a8adf37daca05369e88` |

Consumer pin verification: shared implementation commit `87a862a9f11a372e112d7c166b9a82d74c8e009d`.
Both forks resolve that exact git revision and pass their native form package test
with `--locked --offline`, after removing temporary working-tree source overrides.
Full host CI/UI/JS-WASM gates above used the same implementation from the working
checkout. The commit was fetched locally into Cargo's git cache; publication of
consumer commits requires publishing the shared revision first. No remote push
was performed by this task.
