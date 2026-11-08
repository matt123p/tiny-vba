# ADR 0102 — Offline preparation and validation of M8 release evidence

Accepted 2026-11-08. Owning layer: repository maintenance and conformance evidence.

M8.1–M8.4 passed the finite local profile in ADR 0101. The owner has no Excel or
Word and requested all remaining work possible without them. This does not waive
M8.5. Authored tests and inherited observations cannot establish fresh comparisons.

Add an offline `m8-office-v1` inventory of eleven comparisons: context, events,
attributes/code views, package handling and native forms for both products, plus
Excel worksheet controls. Recipes pin the reviewed M7A test/build sources. Their
steps and observation labels are preparation, not executable observations or new
language/host support. The original M7A/M8 receipts remain historical and unchanged.

`tools/check_m8_release.py` validates the empty inventory, initializes an empty
current-candidate packet and computes release readiness from paired raw results.
Only fresh, named Office builds and authored fork observations for identical inputs
may be compared. Exact tagged values preserve numeric widths, UTF-16 and array
bounds. JSON floats are forbidden in raw packets. Event traces compare in order.
Missing evidence stays NOT_RUN; malformed evidence is INVALID_FIXTURE; unequal
observations are MISMATCH. Reported execution failures retain their status.

Release PASS additionally requires the existing local gates and an explicit human
provenance review after observation. A JSON validator cannot authenticate the
existence of an Office run. Review must establish that the permitted authored input
was actually run in the named builds and that records describe those runs. Synthetic
unit packets test validation only and are never checked in as comparison evidence.

Ordinary core CI validates preparation and runs adversarial maintenance tests with
no Office, network, upstream macro execution or sibling checkout requirement. No
runtime ABI, product dependency, shipping pin or application workflow changes.
P9 remains after M8. See [capture protocol](../m8-office-comparison.md).
