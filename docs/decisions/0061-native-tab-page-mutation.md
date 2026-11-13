# ADR 0061 — Native Tabs/Pages mutation and stable member identity

Status: adopted; shared and application validation PASS. M7A.4 remains in progress.

`vba-ide` supplies ordinary VBA classes for native `Tabs.Add/Remove/Clear`,
`Pages.Add/Remove/Clear`, writable Integer `Index`, and runtime-created member
`Name`. Add supports Office's optional Name, Caption and Index arguments and
returns the actual member. Existing imported objects, aliases and immediate Page
child collections retain their identities across insertion and reordering.
Pages and Tabs remain distinct nominal collections; Pages delegates common storage
to an internal Tabs implementation containing actual Page objects. No COM/OCX,
custom format, host-side mutable registry or native designer mutation is added.

A collection owns members in stable reusable slots and an ordinary ordering state.
Each member owns its state and a position cell containing only scalar index, slot,
name and membership metadata. State owns ordered position cells and the selected
cell, never member/control references. This avoids owning member/collection cycles
under the existing reference-count lifetime contract. `WithEvents` forwards state
notifications and supplies the live MultiPage owner to Page.Parent without a strong
back-reference. Removed cells are detached; reused slots receive new cells.

All validation precedes logical replacement. Structural operations commit slots,
order, membership and selection before authored Change handlers. Reentrant
Remove/Clear/Add and Stop/resume use ordinary VM continuations; no host action
replays. Selected identity is preserved on insert/reorder; removing the selected
member chooses the next member at that position, or the previous last member.
An empty collection has Value=-1; its first addition selects zero. A TabStrip's
explicit unselected state stays unselected when adding to a nonempty collection.
Change is emitted when selected identity or numeric index changes; programmatic
structural operations do not synthesize physical Click. These detailed selection,
detachment and error expectations are implementation inferences, not new Office
observations; exact build/platform comparison remains NOT_RUN under M8.5.

The renderer and generic collection enumerator read canonical ordering through
bounded position/slot validation, without executing getters or loading forms.
Native page descriptors remain immutable; reordered descendants follow the live
Page index and removed descendants cannot receive physical input. Runtime-added
Pages are empty containers. Generic negative runtime IDs and ordinary lookup allow
navigation without mutating imported resource identities. Page ordering walks the
canonical ordered collection directly; existing child TabIndex ordering remains
scoped. Budgets remain 128 members per collection, 128 UTF-16 name units and
16,384 caption units, within existing VM work/location/payload ceilings. Names
use the existing ASCII case-insensitive collection lookup profile; broader locale
and identifier edge cases remain open.

`vba-syntax` now permits contextual Name identifiers, including formal parameters
and local assignment, as in Microsoft's Word event example. Other reserved
identifiers and the unimplemented file-renaming statement remain refused.

`vba-vm` derives reference-bearing field/local location lists from verified types
when creating/restoring instances and creating frames. Primitive scalar/array
slots cannot retain objects and need not be scanned at every statement checkpoint.
ByRef frame slots use the caller location's canonical type. Object/Variant fields,
arrays, persistent roots, stacks, callbacks and parked frames retain the existing
lifetime behavior. These lists are derived metadata, never another value store;
work and memory caps are unchanged. This generic optimization lets ordinary
native form mutation/navigation execute under the existing budgets.

This increment does not close Controls.Add/Remove, list multiple selection and
bindings, broader input/property/event/lifecycle/platform/resource parity, native
edited persistence (M7A.5), or legacy format removal (M7A.6). The complete remaining
M7A.4 ledger stays in [the runtime record](../forms.md).

Primary contracts:

- [Microsoft Forms Add](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/add-method-microsoft-forms)
- [Remove](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/remove-method)
- [Clear](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/clear-method-microsoft-forms)
- [Index example](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/index-property-example)
- [Tab.Index](https://learn.microsoft.com/en-us/office/vba/api/outlook.tab.index)
- [Page.Index](https://learn.microsoft.com/en-us/office/vba/api/outlook.page.index)
- [Microsoft Forms Name](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/name-propertye-microsoft-forms)
- [Tab.Name](https://learn.microsoft.com/en-us/office/vba/api/outlook.tab.name)
- [Word Name parameter example](https://learn.microsoft.com/en-us/office/vba/word/concepts/objects-properties-methods/bind-controls-to-a-specific-row-in-the-data-store-when-inserting-a-building-bloc)

Fixtures and regressions are independently authored Rust under MIT OR Apache-2.0.
No vendor binary, artwork, sample macro or interpreter implementation was adopted.

Validation: shared CI/WASM/WASI/UI/native/web and Chromium text input pass,
including 84 form-instance, 31 generic class and 20 form widget cases. Ten dynamic
tab/page renders were inspected in light/dark, narrow, scaled and host themes.
Both application forks pin shared product `422c5c23`; thirteen native and thirteen
actual Node/WASM package cases per fork, full CI (10/8 steps), strict optional
Clippy/web and existing form/VBA UI checks pass. This is authored integration
acceptance for the increment; it does not close the broader M7A.4 ledger.
