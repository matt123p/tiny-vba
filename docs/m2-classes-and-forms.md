# M2 implementation record

Started 2026-10-28 on `feat/m2-classes-and-forms`, after fast-forwarding and
pushing tiny-vba main to `2ee181d91411317a294d7ea30a49ac610c203431`.
GridCraft main and its feature branch were not changed by that merge.

Updated 2026-10-29. **M2.x remains in progress.** This record distinguishes the
working runtime foundation from the remaining event-language/designer work.
M3 has not been started; it follows completion of the requested M2 work.

| Stage | State |
|---|---|
| M2.1 class/property syntax and project-local type binding | Implemented for the runtime profile below |
| M2.1 verified instances, methods, fields, lifecycle and debugger execution | Implemented and exercised natively/WASI |
| M2.1 document instance/event contracts and class error modes | Document state, declared events, callbacks and three UI error modes implemented; static member binding remains |
| M2.2 standard-control model and interactive designer | Headless model implemented; visual designer pending |
| M2.3 signed handlers and actual form event execution/debugging | Pending |
| M2.4 source/resources, persistence boundaries and round trips | Pending |
| Class checkpoint native/WASM/native UI/browser acceptance | Passed; full form acceptance pending |

Implement runtime execution before enabling a designer Run action. Preserve
ordinary canonical locations, shared coercion, statement checkpoints, native
host dispatch, bounded work and safe inspection. Document bindings, Office
objects, storage, trust and package persistence stay in the forks. No arbitrary
COM/DLL or third-party ActiveX/OLE execution is introduced.

Public behavior references reviewed for this milestone:

- [Property procedures](https://learn.microsoft.com/office/vba/language/concepts/getting-started/calling-property-procedures)
- [Property Get](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/property-get-statement)
- [Property Let](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/property-let-statement)
- [Property Set](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/property-set-statement)
- [UserForm](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/userform-object)
- [Show](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/show-method)
- [Event declaration](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/event-statement)
- [RaiseEvent ordering and ByRef](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/raiseevent-statement)
- [WithEvents declarations](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/public-statement)
- [Error trapping](https://learn.microsoft.com/office/vba/language/how-to/stop-code-execution)

Original implementation and synthetic tests only. Public documentation is not
a fresh Office oracle observation. Unsupported resources must fail explicitly;
no binary VBA project writeback is implied by form source export.

## Completion audit after feature merges (2026-10-29)

Audited merged main `cadc744`. **M2 is incomplete.** Merging all feature
histories did not supply the missing designer, form runtime or persistence.

| Acceptance requirement | Evidence and outstanding work |
|---|---|
| M2.1 class/document execution, lifecycle, events and error modes | 23 VM class tests and three IDE class/controller tests pass. Static project-member binding/visibility and external field ByRef acceptance remain open. |
| M2.2 interactive form designer | Six headless form-model tests pass. `vba-ui-egui` does not mount `FormDesigner`; Toolbox, Insert UserForm and Format commands remain unavailable. Drag/resize, visual Properties, keyboard/accessibility and rendered form acceptance are missing. |
| M2.3 generated handlers and real form run/debug | Generic instance callbacks pause/edit/resume, but no form control-to-handler generation or runtime/modal lifecycle connects the design model to those callbacks. |
| M2.4 reviewed form persistence and round trips | No form source/resource codec, host persistence interface or saved/reopened form round-trip fixtures exist. |

Fresh verification at the audited revision:

- `cargo test -p vba-ide --test forms --test classes --locked`: PASS, nine tests.
- `cargo test -p vba-vm --test classes --locked`: PASS, 23 tests.
- The preceding merge validation at the same product tree passed `cargo xtask ci`
  (349 workspace tests), `cargo xtask wasm` and `cargo xtask wasm-run`
  (162 actual WASI tests). These gates do not establish missing form acceptance.

M3 has not started. Under the current remaining plan, Excel range/worksheet M3
is GridCraft-owned; tiny-vba only supplies necessary generic mechanisms. The
previous repository-only scope must be resolved before changing GridCraft.
Next implementation task remains mounting the existing shared model in the
reusable egui designer, with real command routing and interaction tests.

## Runtime foundation

IR 7 (`project-classes-v7`) adds checked class/field/method descriptors, native
construction, predeclared/document instances and identity comparison. Project
types use the compiler-owned `VBAProject` reference namespace, separate from
external host references. In class-enabled sessions, host identity zero denotes
VM-owned instances; handles also carry the unique session generation. No second
host registry, application dependency, IO/COM activation or copied implementation
is involved. Legacy consumers refuse the new operations and metadata.

Working behavior includes New, Me, public/private members, Property Get/Let/Set
and Exit Property, indexed/default properties, named/omitted optional parameters,
canonical instance fields and internal ByRef calls, object identity/returns,
array fields and shared constant/Option Base rules. Class_Initialize and normal
last-reference Class_Terminate run as actual VM frames. Cancel/reset/disconnect
releases native state without running user finalizers; reference cycles remain
retained. Terminating instances cannot be retained again. Class fault pausing
preserves the original fault frame and honors handlers within the same class.

Project snapshots preserve fields and reference graphs, but rebind native handles
to the new session before committing restored state. Document modules have lazy,
predeclared script state; parsing/importing never instantiates them. Office base
objects, code-owner routing and actual document events remain fork-owned M5 work.

`schedule_instance_callback` and `start_next_callback` reuse the bounded callback
queue and ordinary execution slices. IDE `dispatch_instance_event` uses the same
controller, watches, breakpoints, local edits and resume path. Safe evaluation
rejects construction and effectful properties. Component kind survives renaming.
This does not create a form designer or enable a form Run action.

The original VM class suite has 23 tests and the IDE class suite has three. They
exercise effects, privacy, identity, state restore, document laziness, callback/
reset/cancel lifetime, arrays, resource refusal, verifier damage, legacy refusal
and pause/edit/resume. `cargo xtask wasm-run` includes both suites.

## Remaining stages before M2 acceptance

1. Complete static project-member binding/visibility and external field ByRef
   behavior against lawful reference evidence. Friend and As New remain explicitly
   unsupported. Complete early-bound signatures before claiming full class compatibility.
2. Build the selected Label/TextBox/CommandButton form model and interactive
   designer, including Properties, transforms, tab order, grouping and zoom.
3. Connect correctly signed source handlers and real run/debug form state,
   including suspension/modal lifecycle. Keep design state independent of Reset.
4. Implement the reviewed form source/resource and host persistence contract,
   supported round trips and explicit unsupported-resource diagnostics.
5. Run native/WASM and actual native/browser UI acceptance on those surfaces.

Do not mark M2 complete or remove it from the remaining plan until these stages
and their gates pass. This is not universal class, Office document or UserForm
compatibility.

## Source event checkpoint (2026-10-29)

Public/default Event declarations, class/document-level scalar WithEvents project
types and RaiseEvent now use checked IR 7 metadata. Event names are not ordinary
methods or runnable macros. Event and handler signatures must match required
parameter kinds, modes, arrays and nominal identities before execution. Standard
modules, unknown events, named/omitted arguments, Optional/ParamArray and invalid
WithEvents declarations fail explicitly. No-argument RaiseEvent omits parentheses.

Connections follow successful assignments to canonical WithEvents fields.
Delivery captures subscribers in connection order, runs synchronously in ordinary
VM frames, preserves shared ByRef locations between sinks and supports nested
events, breakpoints, Stop, local edits and resume. A propagated handler fault
clears remaining deliveries when its publisher transfers to an error handler.
Queues and descriptor scans consume existing callback/work/payload budgets;
pending receivers and values participate in lifetime retention. Project snapshots
preserve connection order while remapping object handles. Nothing disconnects;
Reset/cancel/close release subscriptions without executing user cleanup code.
Foreign ByRef writes to WithEvents storage are explicitly unsupported.

Insert > Class Module now creates a real class. Project Explorer groups standard,
class and document components, and Properties shows the actual component kind.
Editor Options exposes mutually exclusive Break on All Errors, Break in Class
Module and Break on Unhandled Errors. Interaction tests drive those controls;
the IDE regression pauses a real class fault then resumes into its caller handler.
Original light/dark class and dark Options images were rendered and inspected.

Native CI, UI interactions/renders, native app capture, web build, Chromium smoke,
headless WASM and actual WASI execution have passed during this checkpoint; final
rerun counts are recorded before commit. Browser smoke verifies the existing
edit/debug/clipboard/file-transfer workflow, not an unimplemented form designer.

Checkpoint validation: `cargo xtask ci` PASS (343 workspace tests),
`cargo xtask wasm` PASS and `cargo xtask wasm-run` PASS (156 actual WASI tests).
`cargo xtask ui` PASS (99 tests and 17 original renders), `ui-native` PASS,
`web` PASS and Chromium `tools/ui-browser/smoke.cjs` PASS with no page errors.
Native/browser pause images and class light/dark/Options renders were inspected.
Tests ran with Rust 1.99.1 and bundled Node 24.22.1. External candidate corpora
remain unexecuted; no fresh Office oracle is claimed.

## M2.2 headless design model (2026-10-29)

`vba_ide::forms` owns a bounded, toolkit/IO-free Form/Control document and
FormDesigner edit state. Label, TextBox and CommandButton have stable non-recycled
IDs, names, captions/text, points-based finite geometry, visibility/enabled state
and independent paint/tab order. Names are checked with the existing parser.
The selected design profile limits a document to 128 controls and 2 MiB, individual
text to 16 KiB, captions to 1 KiB and history to 128 entries/8 MiB. These are
explicit resource limits, not claims about Office limits.

Edits validate complete candidates before committing identity, document or history.
The model supports grouped selection, delete/Properties replacement, snap/grid,
move/resize, six reference-control alignments, grid alignment, equal spacing,
centering, same size, grouping/ungrouping and front/back. Tab order is independent
of z-order. Zoom (10–400 percent), selection and grid settings are view state.
Dirty state compares saved design content, so undo back to saved content clears
it. Stable IDs survive undo without reuse. Runtime Reset has no ownership of
this design document.

Six original headless workflows cover transforms, tab/paint order, groups,
atomic invalid edits, resource refusal, identity and saved-content history.
There is no visual canvas, Toolbox/Properties wiring, source handler generation,
form Run action or persistence format yet. This module does not complete M2.2.
Next: mount this model in the shared egui designer and connect selection, drag/
resize, Properties, keyboard, Format and tab-order commands before M2.3 runtime.

Public behavior references: [UserForm window](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/userform-window),
[UserForm toolbar](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/userform-command-bar),
[Format menu](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/format-menu),
[Tab Order](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/tab-order-dialog-box),
and [points-based geometry](https://learn.microsoft.com/office/vba/language/reference/user-interface-help/height-width-properties).
No reference artwork or implementation code was copied.

Form-model checkpoint validation: `cargo xtask ci` PASS (349 workspace tests),
`cargo xtask wasm` PASS and `cargo xtask wasm-run` PASS (162 actual WASI tests).
The six new form workflows run natively and under WASI. The previously inspected
UI/native/browser evidence above belongs to the class/editor checkpoint; no form
canvas render or interactive form-runtime acceptance is claimed.
