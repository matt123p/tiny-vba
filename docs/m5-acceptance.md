# M5 document events and Range-aware UDF acceptance map

Updated 2026-10-30. These are independently authored acceptance scenarios for the
declared tiny-vba/GridCraft profile. They are **not Office oracle cases**. The source
and expectations below describe the intended GridCraft contract; each result records
only the scope actually exercised by authored tests.

The milestone is complete only when M5.1–M5.3 are exercised against real GridCraft
workbook state, event/debugger/lifecycle acceptance passes, and the relevant native
and WASM gates pass. Compilation alone is not an execution result.

## Scenario inventory

| ID | Workflow | Expected result | Status |
|---|---|---|---|
| M5-E01 | Inert document-module import | Import preserves authored source and performs no workbook writes or automatic open callback | PASS |
| M5-E02 | Worksheet event from Design mode | One `Worksheet_Change` receives the changed Range; `Me.Range` addresses the owning sheet even when another sheet is active | PASS |
| M5-E03 | Event ordering and suppression | Queued changes run FIFO exactly once; disabling events suppresses new notifications and discards the queued tail; re-enabling applies only to future changes; draining one chain restores the next chain's budget | PASS |
| M5-E04 | Debugger and callback lifecycle | A breakpoint pauses inside the event handler; inspect/resume completes once; Reset, close and trust revocation cancel queued callbacks | PASS |
| M5-E05 | Range-aware UDF context | Direct Range arguments retain object identity; Caller, Row/Column, Address, Offset/Resize and typed Value/Value2 produce the declared values | PASS |
| M5-E06 | UDF state and volatility | Function Static state persists across recalculation; `Volatile True` recalculates on unrelated edits; a later False removes that volatility | PASS |
| M5-E07 | UDF mutation boundary | Cell/workbook writes and asynchronous host calls return a worksheet-function error and leave workbook state unchanged | PASS |
| M5-E08 | Native edit event sources | Worksheet UI, cell-editor, direct-control and drag-cut edits, AutoFill/Series/directional fill, Flash Fill, sort, remove-duplicates, Text to Columns, clear and paste queue an event for the committed range; format-only edits do not | PASS |
| M5-E09 | Queue budget and partial effects | Native edits remain committed if notification capacity is exhausted; VBA writes refuse before mutation; callback failure cancels later events while prior writes remain undoable | PASS |
| M5-E10 | M2 standard form events in GridCraft | Imported `.tvform` controls run through the embedded IDE and workbook host; TextBox and button callbacks use the declared host surface, and modal completion/cancel leaves no pending wait token | PASS |

## Core authored source probes

The event-owner probe belongs in a document module whose code name matches the
worksheet. Setup places the workbook on a different active sheet, changes `D5`, and
expects the handler to write the target row to the owner sheet's `B1` without
retargeting active selection:

```vb
Private Sub Worksheet_Change(ByVal Target As Range)
    Application.EnableEvents = False
    Me.Range("B1").Value2 = Target.Row
End Sub
```

The import probe checks workbook values and undo history before any explicit host
event call. `Workbook_Open` must remain dormant until the trusted host command queues
it. The event probe is repeated with a breakpoint in the handler; after resume, the
target and callback count must show one delivery.

The UDF probe belongs in an authored standard module. Put the formula in `D4`, put a
number in `B1`, then change an unrelated cell to force recalculation:

```vb
Public Function CallerProbe(ByVal source As Range, ByVal makeVolatile As Boolean) As Long
    Static calls As Long
    calls = calls + 1
    Application.Volatile makeVolatile
    CallerProbe = Application.Caller.Row + source.Row + calls
End Function
```

`=CallerProbe(B1, TRUE)` in `D4` is expected to return `6` initially and `7` after
one unrelated-edit recalculation. Replacing the formula with
`=CallerProbe(B1, FALSE)` is expected to return `8`; another unrelated edit must
leave it at `8`. This is an authored GridCraft profile expectation, not an observation
of Excel. Separate typed-value cases must cover Date/Currency through `Value`, numeric
serial/value through `Value2`, and error side effects. A UDF attempting a cell write
must produce a cell error and leave the destination untouched.

The forms probe imports an independently authored `.tvform` with one TextBox and one
CommandButton, then runs it modally in the embedded GridCraft IDE. Text entry and
button click must reach their existing M2 event procedures through the workbook host;
closing or cancelling the modal form must retire the wait token. `.tvform` remains the
declared exchange format; this does not imply binary Office form-resource writeback.

## Execution record

Current source checks for the in-progress implementation:

- tiny-vba `cargo check -p vba-vm -p vba-ide -p vba-semantics -p vba-compiler --locked`: PASS.
- GridCraft `cargo check` for model, calc, XLSX, VBA and UI crates against the local
  tiny-vba worktree: PASS (command-scoped path patches).
- GridCraft `cargo check -p gridcraft-engine --locked`: PASS after adding the
  committed Text to Columns output range used by `Worksheet_Change` notification.
- GridCraft `cargo fmt --all`: PASS; `git diff --check` in both worktrees: PASS.
- GridCraft `gridcraft-ui-egui --features vba` compile after routing cell-editor,
  direct control and drag-cut edits through the event hook: PASS.
- `cargo xtask layers`: PASS; the reviewed tiny-vba DAG remains acyclic.
- GridCraft authored tests: `cargo test -p gridcraft-vba --tests` with command-scoped
  tiny-vba path patches: PASS (70 integration/unit cases across the package).
- GridCraft `gridcraft-vba --test events`: PASS (8 cases), including breakpoint/resume,
  Reset/close/trust cancellation, queue overflow, and callback failure with an undoable prior write.
- GridCraft `gridcraft-vba --test worksheet range_aware_udf_supports_address_offset_and_resize`:
  PASS; the added test confirms retained Range identity through Address, Offset and Resize.
- GridCraft `gridcraft-vba --test worksheet range_aware_udf_preserves_typed_value_and_value2_distinction`:
  PASS; Date remains typed for `Value`, Currency reads at four-decimal precision, and `Value2`
  retains the numeric serial/precision.
- GridCraft `gridcraft-vba --test worksheet functions_cannot_mutate_and_work_is_bounded`:
  PASS; a worksheet function's attempted `Application.OnTime` call yields `#VALUE!` and leaves
  workbook cells and transaction groups unchanged.
- GridCraft `cargo test -p gridcraft-ui-egui --features vba --test vba_events` with
  command-scoped tiny-vba path patches: PASS (5 cases), including native `cell.set`, direct
  control-channel `cell.set`, direct cell-editor commit and queue exhaustion.
- GridCraft `cargo test -p gridcraft-ui-egui --features vba --test it
  drag_cursor::native_drag_cut_delivers_change_events_for_source_and_destination` with
  command-scoped tiny-vba path patches: PASS; real pointer input moves A2:B2 to D5:E5 and delivers
  one change callback for each committed source/destination range.
- GridCraft `gridcraft-ui-egui --features vba --test vba_events native_bulk_content_commands_notify_their_committed_ranges`:
  PASS for bulk values, range fill, clear/Clear All, AutoFill, Series, directional fills, cut/paste,
  Flash Fill, sort, Text to Columns and remove-duplicates; callbacks received committed target ranges.
- GridCraft `cargo test -p gridcraft-ui-egui --features vba --test vba_forms` with command-scoped
  tiny-vba path patches: PASS (1 case); `.tvform` import, TextBox/button workbook writes, modal
  completion and close all leave no parked wait token.
- GridCraft `cargo test -p gridcraft-calc --lib`: PASS (90 passed, 1 ignored).
- tiny-vba `cargo test -p vba-ide document_module_tests --locked` and
  `cargo test -p vba-vm --test classes --locked`: PASS (1 and 27 cases).
- Native UI acceptance covers worksheet edits, direct-control edits, cell-editor commits and
  drag-cut gestures, plus the reviewed bulk content commands. Queue exhaustion, callback
  partial-effects and `.tvform` callback behavior are covered by GridCraft authored tests.
- Office oracle execution: NOT_RUN; no Office observation is claimed.

The native edit-source list includes the reviewed content mutation commands and explicit UI entry
paths. Runtime evidence covers UI and direct-control `cell.set`, the cell-editor commit hook, an
actual drag-cut gesture with both committed ranges, 15 content commands, format-only suppression
and zero-removal duplicate behavior. M5-E08 is `PASS`.

See [the plan completion record](implementation-plan.md#M5-completion-record--document-modules-events-and-range-aware-udfs),
[the shared runtime contract](runtime.md), [the compatibility matrix](compatibility-matrix.md),
and GridCraft's [VBA integration contract](../../gridcraft/docs/vba.md). Implementation
history is tracked in
[implementation progress](implementation-progress.md#M5-callback-and-udf-implementation-slice--accepted-profile-2026-10-30).
