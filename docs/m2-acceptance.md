# M2 class and standard-form acceptance

Updated 2026-10-29. **M2.1–M2.4 are complete for the declared profile below.**
All required final gates pass; executable evidence is mapped below.
[ADR 0024](decisions/0024-standard-form-execution-and-exchange.md) owns execution,
exchange and suspension contracts. The earlier [staged record](m2-classes-and-forms.md)
retains the history of incomplete checkpoints.

## Acceptance map

| Requirement | Implementation and executable evidence |
|---|---|
| M2.1 classes, documents, properties, events and error modes | Existing IR7 instances, lifecycle, document state, Event/WithEvents/RaiseEvent and error trapping; early-bound member visibility/access/signature checks, coherent property accessors and external native-field ByRef in `crates/vba-vm/tests/classes.rs` and `crates/vba-ide/tests/classes.rs` |
| M2.2 shared designer | `vba_ide::forms`, `Ide::edit_form` and shared egui designer; Toolbox, selection/multi-selection, grouped dragging, resize, snap/grid, Properties, tab order, alignment/spacing/size/centering/grouping/z-order, zoom and bounded undo/redo; six headless model tests and real pointer/keyboard/menu/Properties/theme tests in `crates/vba-ui-egui/tests/forms.rs` |
| M2.3 signed source handlers and real run/debug | Idempotent generation/navigation for selected events; ordinary native control classes and source event frames; modal/modeless Show, Load/Hide/Unload, cancellable QueryClose, nested modal input ownership, break/inspect/edit/resume, stale-source refusal and inert Reset/close cleanup in `crates/vba-ide/tests/form_execution.rs` and real UI tests |
| M2.4 source/design exchange and host persistence | Strict `.tvform` v1 codec; exact source bytes, design properties, stable identities, groups, paint/tab order; revision-acknowledged native/browser save/export and inert import; malformed/unknown/resource refusal before project mutation; IDE round trips and `apps/vba-ide-app/tests/source_io.rs` |
| Native/browser/accessibility evidence | Original light/dark/narrow/scaled/host-theme designer and runtime renders; actual native form capture; independent keyboard Tab order and widget labels; Chromium upload, modal breakpoint/resume and exact archive download in `tools/ui-browser/forms.cjs` |

## Running and designing

```sh
cargo run -p vba-ide-app -- --open examples/forms/CustomerForm.tvform
```

Use Insert > UserForm for a new design, or View Object to reopen its designer.
Add Label, TextBox and CommandButton controls from the Toolbox. Properties edits
validate before commit. Shift-click extends selection; grouped controls move
together. Drag a control to move it or its lower-right handle to resize it. A drag
commits one history entry, including movements smaller than the snap increment.
Arrow keys move selected controls; Shift+arrows resize; Delete removes; Ctrl+A
selects all; Tab/Shift+Tab traverse the independent design order. Format exposes
the shared layout operations. Zoom/grid are view preferences, separate from source.

Double-click a control or use View Default Event to generate/navigate its actual
private handler. Form Properties exposes Initialize, Activate, Deactivate,
QueryClose and Terminate handlers. Existing handlers must have the declared
signature; conflicting or unsupported event names fail explicitly. Renaming a
control with existing handlers requires an explicit source migration; the
Properties edit refuses it instead of silently disconnecting authored code.

Forms in this profile execute through their predeclared project instance. Explicit
`New FormType` is refused before constructor effects; general multi-instance
UserForm display is outside this selected profile.

Run Form or F5 from the designer starts the form; Modal selects modal/modeless
execution. TextBox uses Change; Label/CommandButton use Click. Control Caption,
Text/Value, Enabled, Visible and point geometry live in native class state.
Geometry setters reject invalid sizes/bounds with error 380 before changing storage.
TextBox changes from code and interactive typing use the same Change delivery.
Runtime Tab follows TabIndex/TabStop independently of paint order. Breakpoints,
Stop, local edits and F5 continue use the existing debugger. Pausing reveals the
handler source; runtime input is disabled until the invocation is idle again.

Show defaults to modal (1); modeless is 0. `Load FormName` initializes without
showing, Hide retains loaded state, and `Unload FormName` invokes QueryClose with
CloseMode 1. The UI close action supplies CloseMode 0. Nonzero Cancel prevents
closing. Normal unload clears controls and authored fields; subsequent load
initializes again. Repeated Hide/Unload do not repeat deactivation/termination.
The UI Reset Form Execution command cancels pending modal continuations and
releases run state without running user cleanup handlers or changing design data.

## Ownership and execution

`vba-ide` owns the design, authored source, generated support, handler contracts
and runtime orchestration. Generated form/control class support is appended to
compiler input, leaving authored handler offsets intact; it is never substituted
for authored source in save/export. Controls have canonical native fields and
Event/WithEvents connections. No Office application object store, toolkit, COM
activation or ambient IO enters the VM.

`Session::start_suspended_callback` is a separate explicit opt-in API; ordinary
callbacks remain terminal-only. The VM parks the waiting frames, locations,
foreign continuation and error state while an event runs on ordinary frames.
Completion restores the waiting caller without replay. Nested depth is bounded
at eight. Cancel/close cancel every parked host token, and failures abort parked
invocations. Snapshots/reset cannot bypass an outstanding suspended callback.
The IDE polls a form wait only after its form is hidden/closed, not every repaint.
Only the current modal form owns input; unrelated host waits cannot be reentered
through form controls. Embedding wait tokens are wrapped in a separate bounded
namespace, so even equal raw token values resume/cancel the correct continuation. Instruction, payload, location and callback budgets remain
session-wide across the parked caller and its handlers.

Typed project-member validation happens before execution; Object/Variant retain
late-bound runtime checks. IR7 permits named ArgumentGet native-field captures
under `project-classes-v7`; legacy IR/capabilities refuse them. Public native fields
pass canonical locations to ordinary procedures and class methods; parenthesized
arguments remain typed temporaries. Property assignment-value parameters are
ByVal, and accessor names/index parameters/types are checked for coherence.

## Form exchange v1

`.tvform` is a tiny-vba UTF-8 design/source format, not Office `.frm/.frx` binary
compatibility. Header: `TINY-VBA-FORM` + tab + `1`. Subsequent LF-separated records:

| Record | Tab-separated payload |
|---|---|
| F | Hex UTF-8 form name, caption; decimal width, height in points |
| C | Control ID, kind, hex name; left/top/width/height; hex caption/text; enabled/visible/tab-stop Booleans; group ID (0 means none) |
| O | Exact tab-order IDs, independently from C record paint order |
| S | Hex UTF-8 authored source, preserving Unicode, attributes and CR/CRLF/LF exactly |

F occurs first, then C records, O, and S. Duplicate/out-of-order records, unknown
versions/control types/resources, invalid numbers/geometry/names/order and
oversized payloads fail before project replacement. There are no supported binary
resources in this profile; an R record, `.frx` payload, or non-native format is an
explicit refusal. Archive input is bounded at 8 MiB, design content at 2 MiB,
controls at 128, text at 16 KiB and captions at 1 KiB. Field counts and decoding
allocations are bounded; malformed tab-heavy input cannot allocate arbitrary tables.

Save/Export dispatch existing owned `HostRequest` payloads with a `.tvform` name.
The embedding host chooses storage; core code does no IO. The form adapter
reserves the `VbaFormsGate` root and its private handle (object/generation both
`u64::MAX`); embedding-owned objects use different handles. Save carries the combined
source/design revision. `mark_saved` accepts only the current revision, so edits
made while a host is saving remain dirty. New forms require save or explicit
discard. Component rename keeps source/design names and layout history aligned;
component removal checks design dirty state and exports the complete archive. Browser download is explicit and leaves modified state until the host
acknowledges storage; native save acknowledges after a successful write. Native
and browser import routes both use the same validation and never execute macros.

## Evidence and boundaries

All form fixtures and renders are independently authored. Public references are
behavior contracts, not fresh Office oracle observations:

- [MS-VBAL module variables](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/0f9113df-fd9c-485a-9583-fdb0e9d68e1b)
  and [member resolution](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/5ef0cb6d-7cb3-4806-8edf-926a530b4b89): distinct instance variables and variable/member identity.
- [MS-VBAL parameter lists](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/78bcb344-4966-4401-bb55-72729790ebee)
  and [property declarations](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/d19bfeaa-cca6-43d8-9188-27355b8ee025): parameter/receiver identity and property value-parameter rules.
- [Show](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/show-method),
  [Hide](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/hide-method),
  [Initialize](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/initialize-event),
  [QueryClose](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/queryclose-event),
  [Change](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/change-event) and
  [Click](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/click-event): selected lifecycle/control events.

The declared M2 gate does not claim universal VBA class/UserForm compatibility.
Friend, As New, Implements, arbitrary control events/control types, binary form
resources, COM/DLL activation and Office package writeback remain unsupported.
Document bindings/events, Office objects, storage/trust and package persistence
remain fork-owned M5–M7 work. No GridCraft or WordCraft files changed.
M3.1 is the next planned milestone in GridCraft; it has not started here.

| Final gate | Result |
|---|---|
| `cargo xtask ci` | PASS: 374 workspace tests, strict Clippy/format and repository checks |
| `cargo xtask wasm` | PASS: headless libraries compile for wasm32-unknown-unknown |
| `cargo xtask wasm-run` | PASS: smoke plus 180 actual WASI tests |
| `cargo xtask ui` | PASS: 127 tests and 24 original renders, including seven form variants |
| `xvfb-run -a cargo xtask ui-native` | PASS: actual native debugger and modal form captures |
| `cargo xtask web` | PASS: real browser bundle |
| Chromium `tools/ui-browser/smoke.cjs` | PASS: final real Chromium run |
| Chromium `tools/ui-browser/forms.cjs` | PASS: final real Chromium run |

Validation ran on this Linux checkout with Rust 1.99.1, WASI under Node 18.19.1
and Playwright 1.64.1 under Node 24.22.1. Original designer/runtime/native/browser
images were inspected, including narrow layout, injected themes, modal contrast
and actual paused handler source/locals. Reports are generated under ignored
`reports/ui/`; both Rust render harnesses and Chromium scripts reproduce them.
No fresh Windows/macOS or Microsoft Office oracle run is claimed.
