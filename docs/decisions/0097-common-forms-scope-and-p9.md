# ADR 0097: Common forms completion and P9 compatibility backlog

Date: 2026-11-08. Status: adopted on explicit owner direction.

The owner requested simplifying M7A.4 to the most common properties and behavior,
completing that scope, and moving uncommon work to P9. This supersedes earlier
instructions that every outstanding native control/property/event/platform case
must block M7A.4. Historical ADRs and progress retain their original evidence.

M7A.4 is complete for [native-forms-common-v1](../m7a4-common-profile.md). That
finite profile covers ordinary native form instances/lifetime, standard controls,
basic properties, text/list/choice/container state, common input and events,
appearance and debugger/trust/budget behavior. The existing implementation and
accepted shared/pinned-host assertions already satisfy it; no product change or
invented Office behavior is needed. Further supported controls remain available.
The revised [checklist](../m7a4-completion-checklist.md) maps the accepted cases
to executable evidence, while [P9](../p9-forms-compatibility.md) preserves the
former outstanding IDs and audit requirements. Unsupported deferred behavior
remains unsupported, not PASS. A bug in included common behavior remains a fix.

P9 follows M8. It owns exhaustive uncommon member/resource/default audits,
unusual reentrant aliases/collections/lifecycle/focus/binding combinations,
platform/IME/repeat/wheel/accelerator details, AutoSize/advanced presentation,
additional image/RefEdit/numeric/container cases and Office-supplied worksheet
ActiveX. Existing M1–M8 identifiers do not change; P9 uses the owner's label.
Third-party controls and arbitrary activation remain excluded.

M7A.5 native persistence, M7A.6 removal of `.tvform`, M7A.7 common-profile native
end-to-end acceptance and M8.5 fresh Office comparisons remain required and
uncompleted. Those gates use the declared common profile; P9 extensions are not
prerequisites. Native edited saving must preserve admitted source/resources and
respect existing package/cache/signature policy; original-byte retention alone
still cannot pass it. No removal, persistence or fresh oracle success is claimed.

No crate boundary, IR/foreign protocol, dependency, authorization, VM limit or
runtime behavior changes. vba-project owns native metadata, vba-ide ordinary
canonical form state/orchestration, vba-ui-egui rendering/input and the forks
Office document/binding/trust/package state. Shared and pinned-host full gates
from ADR 0096 remain valid for the unchanged product. Focused common model/UI
suites and repository/document consistency checks verify the closure change;
[progress](../implementation-progress.md) records their actual results.
