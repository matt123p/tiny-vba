# M8 revalidation and M10 implementation work log

Started 2026-10-28 on `feat/m10-classic-editor-completion` from `eaafa8c`.
User request: complete M8 and M10. M8's corrected baseline is already implemented;
rerun its regressions rather than relabeling planned M10 work as delivered.

## Stages

| Stage | Scope | State |
|---|---|---|
| M10.1 | Editing/project workflows: block edits, indentation/case, rename/remove/export, bookmarks/history, split/procedure views and view preferences | In progress |
| M10.2 | Metadata-backed constants, Quick/Parameter Info, definition/help navigation, references and Object Browser | Planned |
| M10.3 | Quick Watch, conditional/change watches, array inspection/local editing and error trapping | Planned |
| M10.4 | Verified Set Next Statement and capability-controlled Immediate evaluation | Planned |
| M10.5 | Optional DAP frontend, protocol/transport, scripted lifecycle acceptance | Planned |
| M10.6 | Requirement-to-test mapping, M8 regression rerun, UI/native/browser/WASM/CI and original rendered inspection | Planned |

Ownership: reusable behavior in tiny-vba; no GridCraft edits. Keep native calls,
explicit identities, stale-source/pause rejection and bounded work. No proprietary
Office implementation/assets or unlicensed fixtures. Public behavior references
are recorded with each implemented stage; documentation is not oracle evidence.

Completion requires evidence for all stage requirements, not only compilation.
Update this file with tests, actual limitations and next task at each checkpoint.

## Checkpoint — 2026-10-28

Implemented model/UI work so far: physical-line block indent/outdent/comment,
module rename including VB_Name with stable SourceId, explicit export-before-remove
and discard choice, bookmarks/history; Enter indentation/case formatting; host
reference/constant metadata, scoped definition navigation and parameter selection;
local array inspection/typed element edits, conditional/change watches and error
trapping; safe straight-line checkpoint relocation; optional framed DAP frontend
and `vba-cli/dap` stdio shell. No completion claim yet: remaining view/customization,
Immediate policy and broader UI/runtime gates still need implementation/verification.

Evidence so far: four editing tests, four advanced debugger tests and three DAP
transcript/framing/lifecycle tests pass, alongside existing M8 model/UI suites.
Latest metadata/Enter/UI rerun is logged in ignored `target/m10-current.log`.
Set Next Statement refuses all branch/handler procedures, non-executable targets,
fault/ancestor/stale handles; it does not simulate unrestricted relocation.
Conditional predicates use a new read-only, exact-visit VM inspection API, not
host evaluation. Safe watches do not call properties/functions.

Public behavior references consulted (paraphrased; original implementation/tests):
https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/edit-menu
https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/code-window
https://learn.microsoft.com/en-us/office/vba/language/concepts/forms/add-a-watch-expression
https://microsoft.github.io/debug-adapter-protocol/overview
The DAP schema/transport is implemented independently; no upstream adapter copied.
Pinned serde_json 1.0.154 is an MIT OR Apache-2.0 JSON dependency, resolved from
the local registry cache. It is used only in the optional DAP layer, not native
debugger/host dispatch. Update architecture/provenance contracts and gates before
the stage is committed. Next: finish view/customization and explicit Immediate
evaluation policy, then expand negative/interaction/transport acceptance.
