# 0001 — Standalone interpreter and external application compatibility

Date: 2026-10-26. Status: adopted for setup, with open implementation decisions below.

## Adopted choices

1. **tiny-vba is the standalone language repository.** Current user direction overrides the combined
   overview's separate craft-automation repository proposal. Excel and Word compatibility layers and bridges
   live in the Gridcraft/Wordcraft forks. Neither fork is a build dependency here.
2. **Focused Rust Cargo workspace.** Edition 2024; MSRV 1.95 matches Gridcraft; development/CI toolchain
   pinned to the installed 1.99.1 version. A root src/ would blur compiler/runtime boundaries, so each crate has src/.
3. **Generic foreign protocol only.** Reserve vba-foreign and a generic deterministic testkit. No Office
   registry, fake Office facade, AutomationValue duplicate or document engine in tiny-vba.
4. **Bytecode and debugger together.** Explicit frames, source statement IDs, budget checks and inspection
   hooks in the first executable VM. DAP is a later frontend without a VM/debugger dependency cycle.
5. **Pinned test-data submodules.** Six documented corpus repositories plus Microsoft VBA-Docs are gitlinks,
   locked again with provenance/notices and per-candidate SHA-256. No runtime/build dependencies on them.
   Source snapshots and corpus metadata are committed; downloaded binary contents stay in upstream repos.
6. **Original licence preserved.** Existing LICENSE is MIT. Cargo metadata remains MIT. The design's
   MIT OR Apache-2.0 target is not silently treated as permission to relicense existing material.
7. **Truthful scaffold.** Unfinished libraries expose no fake implementation. CLI execution errors explicitly.
   Plans are committed under docs/ rather than Gridcraft's ignored plan/ so this checkout is self-contained.
8. **Adapted agent guidance.** Gridcraft has AGENTS.md/CLAUDE.md but no tracked SKILL.md files. Those
   entry points were reimplemented for this repo, with one local SKILL.md to guide interpreter work.
   Missing craftrules is recorded rather than required from a nonexistent sibling path.

## Open decisions and recommended defaults

| Decision | Default / impact | When needed |
|---|---|---|
| MIT versus MIT OR Apache-2.0 | Retain MIT until maintainer explicitly chooses dual licensing | Before publishing packages |
| Shared Office contract ownership across forks | One canonical versioned neutral contract; no duplicated incompatible catalogues | Before fork bridge work |
| Exact foreign value/reference/ByRef/wait ABI | Prove ownership with a small compiling generic-host example; typed native calls | M0 contract work |
| Parser implementation | Pure Rust lexer plus Pratt expressions/recursive-descent statements; no generated C dependency | M1 |
| Compatibility profile | Explicit Excel365Windows64 intent; exact Office build and locale still unmeasured; keep Mac results separate | Before adopting oracle results |
| Unlicensed workbook specimens | Keep candidates external and NOT_RUN; adopt only after rights/capability review | Before fixture adoption |
| Read-only MS-OVBA decoder — resolved | Independent bounded MS-OVBA reader with pinned cfb/encoding_rs; ovba not adopted; [ADR 0014](0014-m6-project-decoding.md) | M6 complete |
| Missing Office architecture document | Obtain or author it in the fork-owned compatibility design | Before cross-repo contract freeze |
| Word-specific corpus | None is identified in supplied docs; curate licensed .docm/.dotm later rather than inventing a source | Word integration |

These open choices do not block source frontend work. Protocol ownership is the first decision that needs
cross-fork agreement; language semantics and tests can proceed independently.

## Scope amendment

[ADR 0002](0002-editor-debugger-ui.md), adopted 2026-10-26 on explicit user direction,
extends tiny-vba to own the reusable editor/debugger UI and standalone IDE. The headless
engine remains independent; Office compatibility and document integration remain fork-owned.
