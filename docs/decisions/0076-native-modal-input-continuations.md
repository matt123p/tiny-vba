# 0076 — Native modal input continuations

Date: 2026-11-06. Status: implemented; validation receipts recorded in progress.
L02 and M7A.4 remain PARTIAL. Fresh Office observations remain M8.5 NOT_RUN.

## Owning layer and existing contract

The reusable egui adapter owns only bounded raw input, target identities,
immutable geometry and pointer anchors. Generated VBA objects still own control
state and completion receipts. The VM/IDE already support parked ordinary modal
callers; every child command must pass the existing executable-input and host
preparation path. Inert retained-modal order from ADR 0075 is presentation
metadata, never authority to execute another callback.

The previous single pending mouse slot prevented a child modal form from
receiving mouse callbacks while its parent's callback waited for Show to return.
The adapter now parks that caller's raw input scope and services the top retained
modal window. Closing a child restores the caller's input scope in modal wait
order. A removed or hidden target is still discarded by the existing identity
and visibility checks. Reset/source invalidation discards every scope.

There are at most eight parked scopes, bounded by the existing eight-modal-wait
contract. The raw queue remains capped at 64 events across all scopes: a child
receives only the remaining shared allowance, rather than another 64-event
budget. Geometry remains a single bounded projection transferred between scopes;
parked scopes keep no duplicate mutable controls or geometry registry. Existing
runtime, memory, static permission and continuation limits are unchanged.

A physical release for a parked capture is queued to its original owner. It
cannot become a child release, execute the parent while its modal child owns
execution, or leave the parent's capture held indefinitely. Numeric repeat ends
on that release. Later completion/defaults still read canonical state and use
ordinary authorized commands; hiding/unloading the parent suppresses its stale
queued defaults.

The keyboard barrier reports its newest pending callback with a visible owner to the mouse adapter.
Dispatch checks count pending callbacks for the current owner, so a parked
parent cannot bypass its child's mouse barrier. Conversely, the parent's pending
keyboard callback cannot block its active child's mouse input. These are adapter
continuation fixes; no VBA syntax, event signatures, VM instructions, native
format, Office object or platform service is added.

The renderer explicitly promotes modal areas in retained Show order, and promotes
an open ComboBox popup above its owning form. A held parent pointer cannot keep
its area above a newly displayed child. Popup promotion remains local to its
form so it cannot cover a later modal child. This adds no mutable window model.

## Evidence and remaining work

Actual egui input regressions cover stopped child mouse handlers, three nested
modal levels, parent unload, Reset followed by a fresh run, keyboard-to-mouse
nesting, a running child key handler above a parked parent key, and physical
release after the child has appeared, plus refusal before callbacks when nested input
exhausts the shared queue. They assert exact one-shot output and no
unauthorized competing input command. The native browser fixture additionally
holds the parent press through child display, releases it there, stops the child
handler and checks child/parent Click order after Continue and unchanged source.
These are independently authored expectations, not measured Office traces.

This completes those selected L02 cases. Full Activate/Deactivate, modeless and
document focus, further reentrant focus/binding/collection and platform input
ordering remain in the unchanged checklist. L02 is PARTIAL, not a blanket
lifecycle sign-off. Native edited persistence and .tvform removal remain M7A.5/.6.
