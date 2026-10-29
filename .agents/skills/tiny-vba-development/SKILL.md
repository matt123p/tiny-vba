---
name: tiny-vba-development
description: Implement or review tiny-vba changes using the project design contracts. Use as the entry point for repository development and route to the focused architecture, semantics, debugger, UI, host or conformance skill.
---

# tiny-vba development

Read [AGENTS.md](../../../AGENTS.md), [progress](../../../docs/implementation-progress.md)
and the relevant [implementation gate](../../../docs/implementation-plan.md). Inspect actual code
before choosing a change; planned crates and APIs are not implemented capabilities.

## Select the relevant design skill

Load only the skills needed for the task; a change crossing boundaries may need more than one.

| Task | Skill |
|---|---|
| Crate ownership, dependencies, public contracts, design proposals | [architecture](../tiny-vba-architecture/SKILL.md) |
| Parser/binder, values, bytecode, runtime and language behaviour | [language semantics](../tiny-vba-language-semantics/SKILL.md) |
| Pause/resume, stepping, watch evaluation, debugger/DAP lifecycle | [debugger](../tiny-vba-debugger/SKILL.md) |
| Editor panels, theme, interaction, standalone/embedded IDE | [editor UI](../tiny-vba-editor-ui/SKILL.md) |
| Foreign calls, host lifetime/capabilities, project import | [host integration](../tiny-vba-host-integration/SKILL.md) |
| Fixtures, oracle evidence, corpora, compatibility reports | [conformance](../tiny-vba-conformance/SKILL.md) |

For each material design choice, identify its owning layer and applicable contract. Follow current
user direction and adopted decisions over historical supplied design snapshots. Resolve routine
implementation choices within that scope; record material contract changes in docs/decisions and
update affected architecture, plan, compatibility and progress documents. Explain any tradeoff
against an existing invariant rather than silently changing it.

Product code is Rust. Follow the root safety/layering rules and keep failures explicit. Preserve
headless native/WASM use and the required editor/debugger UI owned by this repository.

Run the relevant checks plus the repository gates `cargo xtask ci` and `cargo xtask wasm`.
Report exact verification, unavailable checks and remaining gaps; compilation is not proof of
VBA compatibility. Leave the next smallest implementable task recorded in progress.

[ADR 0010](../../../docs/decisions/0010-m4-profile-and-verification-hardening.md) publishes
m4-scalars-v1; do not silently expand m3-core-v1. Keep failure classifications and nonzero
CRASH/TIMEOUT/MISMATCH exits intact, compare exact payloads, and check narrowing bounds before casts.
Confirm xtask's logged workspace matches the intended checkout; shared binaries may originate elsewhere.

Current M4 support supersedes earlier first-slice limits: read [M4 language/IR 2](../../../docs/m4-language.md)
and [ADR 0011](../../../docs/decisions/0011-m4-language-extension.md). Compile against m4-language-v1
VM capabilities. Preserve the IR 1 structural target and explicit IR 2 feature/version gates.
The authored inventory is 20 M0 plus 24 M4 PASS fixtures; no Microsoft oracle observation is claimed.

M4 external acceptance is now implemented under [ADR 0012](../../../docs/decisions/0012-m4-oracle-acceptance.md).
Read [the report](../../../docs/m4-oracle-report.md) and current language contract: 44 authored
plus 153 inherited upstream-reported Excel Mac scenarios PASS. Schema 2 and the reviewed allowlist
preserve raw results/environment/hashes; no fresh Office run is claimed. Use m5-host-v1
and explicit IR 3/4 capabilities, retaining IR 1/2 refusal rules. Unselected external cases remain NOT_RUN.

M6 is implemented under [ADR 0014](../../../docs/decisions/0014-m6-project-decoding.md).
Read [the import contract](../../../docs/project-import.md) before changing vba-project.
Keep extraction read-only/inert, project-local ModuleId separate from caller-owned SourceId,
UTF-8 attribute spans and code names distinct from stream/display names. Preserve explicit
corrupt/unsupported/protected/limit refusals and container/decompression work caps. cfb/encoding_rs
are reviewed pinned external dependencies; package preservation and trust UX remain fork-owned.
The 14 controlled importer tests run natively and under WASI through cargo xtask wasm-run.

## Implemented M8 editor/debugger

Read [ADR 0016](../../../docs/decisions/0016-m8-embeddable-editor.md), the
[embedding/harness guide](../../../docs/ui/embedding.md) and
[acceptance matrix](../../../docs/ui/m8-acceptance.md). vba-ide is toolkit/IO-free; vba-ui-egui
mounts in caller Ui with injected tokens, independent per-view IDs and explicit commands;
vba-ide-app owns native/web IO and preferences. Run cargo xtask ui, ui-native and web plus ordinary
CI/WASM gates for affected UI changes. Keep accessibility checks enabled; render/inspect original
light/dark/narrow/scaled/host-theme/focus output. No downloaded reference evidence enters Git:
.cache/evidence/ is ignored and provenance/notices/pins are verified. Do not claim full C0/C1,
Office event/workbook, Set Next Statement or unsafe evaluation support from standalone UI evidence.

M10's implemented classic tools, advanced debugging and optional DAP are defined by
[ADR 0018](../../../docs/decisions/0018-m10-classic-tools-and-debugger-policy.md) and the
[acceptance map](../../../docs/ui/m10-implementation.md). Preserve shared bounded source history,
literal scoped search, view selection/focus and explicit browser transfer. Set Next Statement
supports verified straight-line checkpoints only; watch evaluation stays read-only; Immediate
calls require explicit isolated-evaluation policy. Metadata does not imply library/type binding.
The [remaining plan](../../../docs/implementation-plan.md) now uses new M1–M8
numbering. Milestone IDs in the implemented contracts above are historical; see
the [mapping](../../../docs/implementation-progress.md#plan-renumbering--2026-10-28).
New M1 owns host type binding, M2 shared classes/standard Office forms, M5 Excel
events/UDFs, and M6/M7 remaining host/package acceptance. Third-party ActiveX/OLE
controls not supplied with Office are out of scope. Native extensions/translation
and AI assistance are removed from planned scope; do not schedule them from the
earlier supplied specification. Preserve fork ownership and existing UI evidence.

The 2026-10-27 user correction includes working scoped word/member completion, source signature help
and live red/underlined parser diagnostics in historical M8 (UI specification section 16).
Declarative reference metadata/List Constants belong to the historical M10 implementation;
executable external type/reference binding is new M1. Preserve original CR/CRLF source
bytes through the editor view mapping and Static state across compatible same-host reruns until Reset.

M2's selected class/standard-form profile is implemented under ADR 0023/0024.
Read docs/m2-acceptance.md for current scope, workflow, exchange format and gates;
docs/m2-classes-and-forms.md retains historical incomplete checkpoints. Preserve
IR7 class/event identity, canonical field ByRef, static visibility/signature checks,
property value ByVal semantics, inert safe inspection and all three error modes.

vba-ide owns the bounded shared design, authored source, .tvform codec and form
orchestration; vba-ui-egui renders the same designer/runtime in native/browser
hosts. Generated native control classes use ordinary Event/WithEvents frames.
Explicit suspended callbacks park a modal caller; existing callbacks remain
terminal-only. Retain parked locations, shared budgets, input ownership and host
tokens through pause/resume. Cancel every token on Reset/close; do not execute
user cleanup during abnormal shutdown. Design/history must survive VM Reset.

Form import is inert and validates before project mutation. Save/export preserve
exact authored source and design; revision acknowledgments must reject intervening
source/design edits. Unknown formats, controls and binary resources fail explicitly.
The selected Label/TextBox/CommandButton profile does not authorize COM/DLL or
third-party controls, binary Office form/resource compatibility or package writeback.
Office document bindings/events, storage, trust and package persistence stay in forks.
Run CI/WASM/WASI plus UI/native/web and both Chromium smoke/form acceptance gates
when changing these surfaces. M3 range/worksheet work is GridCraft-owned.
