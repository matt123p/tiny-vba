# Editor and debugger UI contract

The reusable VBA editor/debugger and standalone shell belong in tiny-vba.
`vba-ide` owns toolkit-independent source/project/session state and commands;
`vba-ui-egui` renders panels; `vba-ide-app` owns native/browser IO and preferences.
The host owns document integration. See [embedding](ui/embedding.md) for APIs.

## Editing and navigation

The shared UI provides Project Explorer, Properties, module/procedure selectors,
procedure-only and split views, bookmarks and navigation history. Editing includes
undo/redo, clipboard, block indent/outdent/comment, automatic indentation/case,
and literal scoped Find/Replace with case/whole-word options.

Code assistance uses source and immutable host metadata for scoped word/member
completion, signature/parameter help, Quick Info, List Constants, definition
navigation and Object Browser. It must not execute getters to discover members.
Syntax colors, line numbers, live parser diagnostics, breakpoint markers and
current-statement highlights track the correct source revision. Hidden attributes
and original line endings survive [code projection](vba-attributes-and-editor-projection.md).

## Execution and inspection

Run/Continue, Break, Reset, Step Into/Over/Out, typed Locals/arrays, Call Stack,
Watches, Quick Watch, Immediate and Output use native IDE/debugger commands.
Watch expressions are bounded and read-only. Immediate function calls require
explicit isolated-evaluation policy; host effects remain real. Set Next Statement
accepts verified straight-line targets in the current frame only.
See [debugger](debugger.md) for exact refusal and lifecycle rules.

| Shortcut | Action |
|---|---|
| F5 / F9 | Run or Continue / breakpoint |
| F8 / Shift+F8 / Ctrl+Shift+F8 | Step Into / Over / Out |
| Shift+F9 | Quick Watch |
| Ctrl+G / Ctrl+L | Immediate / Call Stack |
| Ctrl+R / F4 / F2 | Project Explorer / Properties / Object Browser |
| Ctrl+Space / Ctrl+J | Completion |

Forms add Toolbox, Properties, drag/resize, alignment, tab order and event-handler
navigation. Runtime input executes ordinary callbacks through existing trust and
debugger continuations. [Forms](forms.md) defines the designer/runtime/save profile.

## Embedding and accessibility

Mount panels in caller-owned Ui/context with independent widget namespaces.
Use injected theme tokens, fonts and scaling; do not replace global host styles,
fonts or window policy. Align the reviewed egui/eframe family through consuming-root
Cargo patches. Host menus route stable action IDs/labels/shortcuts/enabled states.
Bound execution/output work so repaint, pause and cancellation remain responsive.

Give controls accessible labels and keyboard navigation. State needs glyphs as
well as color. Text contrast is at least 4.5:1, functional glyphs at least 3:1.
Use original/licensed assets with notices. Validate light/dark, narrow/scaled/focus
and injected host themes, plus actual native/browser workflows.

File IO, pickers and prompts stay behind shell/host interfaces. Headless commands
never open dialogs. A save acknowledgement must match the exact source/design
revision after persistence; a browser download request is not a commit receipt.
Reset clears runtime state without undoing prior document effects or designer edits.
See [testing](testing.md) for verification and [asset provenance](images/README.md).
