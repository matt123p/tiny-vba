# ADR 0062 — Native runtime Controls collections

Status: implemented; CI, WASM and WASI validation pass. M7A.4 remains open.

The native Forms provider in `vba-ide` supplies Controls.Add/Remove/Clear for
Office Forms ProgIDs using ordinary project classes. No COM, registry, DLL/OCX
activation, custom file format or second mutable host control store is added.
Source and imported design/resource bytes remain independent of runtime objects.

A form owns a context object and receives weak WithEvents requests for collection
operations, parent lookup, focus, runtime name/TabIndex changes and option/button
roles. Controls reference context, never their owning form. Immediate child and
flat control collections remain ordinary VBA arrays. Removing a runtime container
detaches its subtree in reverse parent order; imported controls cannot be removed
or renamed by runtime APIs. Clear validates its scope before removing anything.
Reentrant notifications and callbacks use ordinary VM frames and continuations.
UserForm, Frame and indexed MultiPage collection notifications follow their
respective documented signatures. Removing a live control clears its active
control ownership; retained aliases do not authorize subsequent widget input.

The UI and input router derive an inert runtime hierarchy from canonical objects
and stable internal IDs. They do not invoke getters, load forms, alter the native
designer or save runtime additions. The existing opaque u32 widget key transports
negative internal IDs without changing the VM's Long representation. Empty added
Pages inherit available native page geometry; the empty-collection fallback and
complete default/layout/platform behavior still need reference comparisons.

Controls expose ordinary Enter/Exit/BeforeUpdate/AfterUpdate events where
applicable, allowing runtime WithEvents sinks to use existing typed cancellation.
Static designer handlers subscribe through the same event path. The native
provider adapts its generated Exit event metadata after parsing its internal
placeholder; authored syntax is unchanged and no new VBA keyword rule is added.

All existing VM limits remain unchanged, including the 1,024 static permission
entries and executed-call, work, object, location and payload caps. Shared typed
VBA initializer/request helpers and typed object arrays reduce redundant late
binding. The IR verifier validates/indexes nominal metadata once and reuses the
index for event signatures, charging bounded lookup work instead of repeatedly
charging the same table allocation. Generic VM event dispatch skips instances
whose verified class has no WithEvents fields, using derived immutable metadata
also rebuilt on restore. Member dispatch copies only the selected field or
parameter metadata and charges the bounded lookup actually performed, including
failed lookups. These optimizations preserve visibility, connection ordering,
restoration, cancellation and explicit per-site host authorization.

Further M7A.4 bindings, multiple selection, default/property/event/lifecycle and
platform/resource requirements remain in [native forms contract](../forms.md).
Native edited persistence and legacy format removal remain M7A.5/.6. Independently
authored tests are not fresh Office traces; comparisons remain NOT_RUN under M8.5.

Primary contracts:

- [Add](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/add-method-microsoft-forms)
- [Remove](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/remove-method)
- [Clear](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/clear-method-microsoft-forms)
- [AddControl](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/addcontrol-event)
- [RemoveControl](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/removecontrol-event)

Code and fixtures are independently authored Rust under MIT OR Apache-2.0.

The generic VM avoids repeating instance-lifetime scans across scalar-only
checkpoints. Reference-bearing stack/storage changes, frame changes, event
deliveries and enumeration-root removal invalidate the cached scan; every
execution slice starts conservatively invalidated. Normal finalization still
repeats to a fixed point before proceeding. Execution, allocation and host
permission limits remain unchanged.
