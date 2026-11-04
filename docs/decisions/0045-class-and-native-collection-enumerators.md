# ADR 0045 — class enumerators and native form collection iteration

Date: 2026-11-04. Status: adopted for M7A.4 collection enumeration.
M7A.4 remains in progress with its full remaining scope unchanged.

The language/compiler/IR/VM own generic class enumeration. Following Microsoft's
[For Each guidance](https://learn.microsoft.com/en-us/office/vba/language/concepts/getting-started/using-for-eachnext-statements),
a class member marked `VB_UserMemId = -4` supplies an object enumerator. This
implementation admits public zero-argument Function/Property Get members returning
Object, including nominal object results declared by an explicit reference
snapshot (such as stdole.IUnknown). These use the existing host type resolver;
no implicit library load is added. An array-valued getter is not a VBA enumerator.

IR 9 adds optional Class.enumerator metadata and `class-enumerators-v9`.
Verification checks the version/capability, member ownership/accessibility,
function result storage and zero-argument Object signature. IR 1–8 consumers
refuse the new metadata. Programs without it retain their existing IR versions.
No opcode, external dependency or crate edge is added.

For Each invokes the declared getter as an ordinary receiver frame. On return,
it forwards the opaque enumerator object through the original verified site and
its normal invocation permissions, bounded snapshot, error and wait/cancel path.
Getter checkpoints/Stop and host waits do not replay completed getters. Forwarding
holds at most 64 native receiver roots until the snapshot is installed or the
statement leaves its checkpoint. Cycles exhaust this bound or existing VM work
budgets. Active and parked loop snapshots retain all referenced native instances;
loop exit/error/Reset releases that ownership without abnormal cleanup code.

The IDE owns native Controls/UserForms collection behavior. Their generated
ordinary classes provide an internal VbaNewEnum getter with the standard member
ID. It returns a native wrapper holding an ordinary strong collection reference;
the wrapper supplies the opaque host interface. Controls stores its canonical
child references in a public internal Variant containing an Object array (VBA
forbids Public array declarations on classes). The form host uses the existing
transient read-only ProjectObjectView to snapshot those references or loaded forms
in creation order. No getter runs during inspection, no default form loads, and
there is no duplicate mutable collection/instance registry. The snapshot retains
object identities and field changes remain visible through them.

The form host reserves object IDs MAX-65536 through MAX-1 for opaque collection
interfaces, alongside the existing MAX form gate. An interface encodes the native
wrapper ID and exact session generation. Enumeration revalidates wrapper/collection
class and liveness; stale interfaces refuse. Other members refuse. These internal
bridge handles are not COM activation or a new user-facing form format. Retained
native wrappers own their collection after form unload; no special VM
registry is needed. Direct IEnumVARIANT cursor/Next/Reset interfaces are not exposed.
Collection structural-mutation behavior beyond the existing bounded snapshot
profile remains unverified and is not claimed as fresh Office parity.

Evidence is independently authored: VM getter/function/property, Stop, wait and
legacy/malformed metadata cases; native nested/empty collections, object identity,
unload and paused resume. Office build/OS/locale comparisons remain NOT_RUN under
M8.5. Expanded native families, focus/typed events, nominal bindings and platform
control packets remain required for M7A.4.
