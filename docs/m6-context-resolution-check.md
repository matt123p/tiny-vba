# M6 module-context and name-resolution check

Updated 2026-10-31. This is the M6.5 record required by the [remaining plan](implementation-plan.md):
a per-layer check of the shared binder/runtime and both application forks against the two
supplied specifications, [module types and execution contexts](vba_module_types_and_execution_contexts.md)
and [Excel context and object resolution](vba_excel_context_and_object_resolution.md). It records
implemented, refused and missing behavior per owned layer, and it drove the M6.6/M6.7 work in the
same change. Behaviors marked *implemented (M6.6)* or *implemented (M6.8)* landed with this record;
everything else carries its own evidence reference. No Microsoft oracle observation is claimed:
unverified Excel edge cases stay explicitly NOT_RUN and M7.4 runs the differential matrices as
unchanged-document acceptance.

## Shared language and runtime (tiny-vba)

| Specification area | Status | Evidence / boundary |
|---|---|---|
| Module kinds as immutable metadata | Implemented | `ast::ModuleKind` Standard/Document/Class is fixed at parse time; classes are immutable compile metadata, not a runtime mode flag (`vba-semantics::Class`). Chart-sheet modules are not modeled; the spec assigns other host document types to host extensions |
| Staged identifier resolution | Implemented (M6.6) | Bare names resolve locals/parameters/module declarations/module constants, then project declarations (same-module procedures and standard-module publics), then eligible implicit `Me` members (document modules, host-declared), then host imports, then `Erl`/intrinsics, then the VBA implicit-local/Option Explicit rules. `crates/vba-semantics/tests/binding.rs` |
| Implicit `Me` members | Implemented (M6.6) | Document modules resolve bare names declared by the host (`CompileOptions::document_members`, keyed by `VB_Name`) as `Me` member dispatch before host globals. Declared class members already bind as instance members; undeclared names in class modules still fail binding rather than becoming silent host calls. `crates/vba-vm/tests/with_resolution.rs` proves the dispatch lands on the live owner object |
| Shadowing | Implemented (M6.6) | Local/parameter/module declarations shadow everything; project declarations shadow implicit members and host globals for calls and value reads; a document-module declared procedure shadows a same-named host member declaration. Getter-alias dispatch on an explicit qualified receiver (`Application.Run`) is unchanged — qualification is not shadowing (ADR 0025) |
| Callee module-kind rules | Implemented | `lookup.public` exposes only standard-module public procedures to bare cross-module calls; document/class/form members require their instance, code-name or default-instance receiver (pre-existing, verified 2026-10-31 against `vba-semantics` construction) |
| `With` receivers | Implemented (M6.6) | `With … End With` parses, binds a hidden object slot per block and resolves innermost-only leading-dot chains. The receiver is evaluated once in the enclosing scope; entering `With` never changes `Me` or any active host state; `Exit` leaves cleanly. Hidden slots are invisible to debugger locals and carry bracketed metadata names no VBA identifier can collide with. Native/WASI regressions in `crates/vba-vm/tests/with_resolution.rs` |
| `Me` semantics | Implemented (M5) | `Me` is a reserved read-only receiver local in every class/document procedure; document instances are host-owned (`New` on a document module refuses); owners bind per callback/instance and forward unresolved members through the host permission path (ADR 0026/0027) |
| No implicit current-workbook frame parameter | Implemented (invariant) | VM frames carry only declared parameters, the result, the `Me` receiver, `With` slots and declared locals. Host active state is consulted only through explicit host dispatch; nothing injects a workbook/frame parameter (verified 2026-10-31 against `vba-vm` frame construction) |
| Entry-point context preservation | Implemented (M5/M7 contracts) | Callbacks deliver through fresh top-level invocations with the callee's own module binding; scheduled document callbacks keep owner identity and never inherit the caller's `Me`. `Application.Run`-style re-entry is not implemented (see fork rows) |
| IDE receiver typing for host objects | Implemented (M6.8) | `vba-ide` types receivers from configured host-import result types, call shapes (`Range("A1").`, `Worksheets("Data").`, chained `Workbooks("B").Worksheets("S")`), `Me` and document code names via host-declared instance types, and walks catalogue member result types. Code aids stay read-only and metadata-driven; declaring a member or type never lets them invoke anything. `crates/vba-ide/tests/host_completion.rs` |

### Recorded limitations of the shared layer

- Leading-dot `With` receivers have no statically declared type, so member completion after
  `. ` inside a `With` block is intentionally unavailable; explicit receivers complete.
- Bare value-context implicit members (`Set r = Range` without arguments or a following
  `.member`) still resolve to the host global import; only call-shaped and leading-dot uses
  bind to `Me`. Real VBA resolves collection globals as value reads; closing this needs a
  declared value-get rule per host and is scheduled with the fork catalogue work.
- A `GoTo` into a `With` body can reach a leading-dot use before the receiver assignment runs;
  the slot is `Nothing` and the member dispatch raises VBA error 91, mirroring the undefined-
  receiver hazard rather than inventing a default.
- WithEvents Application-level event routing remains the forks' M6.3/M7 scope; the shared
  Event/WithEvents machinery is implemented and host-neutral (M2).

## GridCraft (Excel host)

| Specification area | Status | Evidence / boundary |
|---|---|---|
| Excel host globals | Implemented (M4/M5 contract) | `Range`, `Cells`, `Rows`, `Columns`, `Worksheets`, `Sheets`, `ActiveWorkbook`, `ActiveSheet`, `ActiveCell`, `Selection`, `Application`, `ThisWorkbook` are configured host imports on hidden IR 4 roots and dispatch at invocation (`crates/vba/src/lib.rs::imports`) |
| Live active-context consultation | Implemented (M4) | Dynamic roots re-resolve against the session's live workbook/sheet/selection per dispatch; `captured_handles_survive_rename_reorder_and_dynamic_roots_follow_selection` proves globals follow activation and selection changes mid-procedure while captured handles stay stable |
| Distinct `ThisWorkbook`/`ActiveWorkbook`/`ActiveSheet`/`Me` identities | Implemented (M4/M5 contract) | `ThisWorkbook` binds the project workbook root; `ActiveWorkbook`/`ActiveSheet` are live application properties; `Me` binds the callback's owner object (M5 evidence). Cross-document host calls refuse with error 1004 (`object belongs to another workbook`) |
| Code-name stability across tab renames | Implemented (M5) | Code names are sheet/document identity, not tab captions; rename/reorder coverage in `crates/vba/tests/identity_values.rs` and the M5 acceptance map |
| Worksheet-module bare `Range`/`Cells`/`Rows`/`Columns` → module's sheet | Implemented (M6.7) | GridCraft declares those names as implicit `Me` members for worksheet document modules; `ThisWorkbook` declares `Worksheets`/`Sheets`. Workbook modules keep the Excel global `Range` (active sheet), matching the unshadowed matrix. Authored fork workflows added with the M6 acceptance update |
| Non-`Range` `Selection` | Not applicable to this host | GridCraft's selection is always a worksheet range; there is no shape/chart selection to misinterpret. `Selection` refuses explicitly when no workbook is open (host error 91/1004 path) |
| Chart-sheet active / no-active-workbook errors | Not applicable / refused | GridCraft has no chart sheets; dispatches with no open workbook return host errors rather than inventing a sheet (adapter error path, M4 evidence) |
| `Application.Run` and add-in `ThisWorkbook` | Refused / NOT_RUN | The catalogue declares no `Application.Run`; synchronous re-entrant dispatch is impossible under the non-reentrant VM contract, and queueing would not return the callee's result. No add-in model exists. Both stay out of the declared profile and are scheduled honestly rather than simulated |

## WordCraft (Word host)

| Specification area | Status | Evidence / boundary |
|---|---|---|
| Document roots under the M6.1 profile | Implemented (M6.1) | Application/Document/Selection/Range with live UTF-16 position mapping; owner-bound `Me` forwarding during callbacks; stale-range refusal. WordCraft's `docs/vba-integration.md` records the exact member set and error behavior |
| Document events against the M2 contract | Implemented (M6.3, declared profile) | Canonical `Document_Close()`, explicit `Document_Open`, explicit WordCraft BeforeClose/BeforeSave extensions with ByRef Cancel; reset/close cleanup and modal waits covered by authored workflows. Standard Application-level event routing and automatic package Open/New delivery remain absent and are recorded as absent |
| Word host globals consulted at invocation | Implemented | Application/ActiveDocument/Selection resolve per dispatch against the live document session; imported projects stay inert until trust |
| Word module-kind binding (`ThisDocument` implicit members) | Implemented (M6.7) | WordCraft declares the document profile's implicit members for `ThisDocument`; ordinary-run owner binding is exercised by the fork's M6 workflows added with this milestone |

## Acceptance mapping

- Unshadowed module-kind binding cases from the specifications run as real fork workflows:
  standard-module bare `Range` → active sheet; worksheet-module bare `Range`/`Cells` → the
  module's own sheet even with another workbook active; `ThisWorkbook` bare `Range` → active
  sheet while `Me.Worksheets` stays on the project workbook (GridCraft acceptance tests).
- Live host-global consultation: activation changes between dispatches are honored per call
  (GridCraft `identity_values.rs` plus the new acceptance workflows).
- Declared host globals and typed/call receivers complete their members in the shared editor
  without host invocation (`crates/vba-ide/tests/host_completion.rs`).
- Remaining spec rows (chart sheets, add-ins, `Application.Run`, differential Excel runs) are
  explicitly NOT_RUN/NOT_APPLICABLE for this profile and feed M7.4 instead of being guessed.
