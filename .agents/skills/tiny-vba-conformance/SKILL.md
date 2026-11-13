---
name: tiny-vba-conformance
description: Create or review tiny-vba test scenarios, external corpus use, oracle evidence, compatibility reports and release claims. Use for fixture adoption, semantic evidence or measuring implemented behaviour.
---

# tiny-vba conformance

Read [testing](../../../docs/testing.md), [scenario format](../../../docs/scenario-format.md)
and, before external material, [corpus policy](../../../tests/corpora/README.md),
[source lock](../../../references/sources.lock.json) and [inventory](../../../tests/corpora/inventory.json).

Separate syntax/binding, language execution, generic host, import/debugger/UI and
real-fork document evidence. Pinning proves integrity, not execution permission or
compatibility. Prefer authored deterministic cases; establish license, provenance,
capabilities, profile and justified expectations before adopting external material.
Never run downloaded scripts/macros or copy another interpreter. The reviewed MIT
allowlist is the bounded execution exception. Observe Office only as a permitted
black box; never inspect proprietary implementation binaries.

Record exact types, values, lower bounds, errors, output/effects, event order,
source hashes and reference build/OS/locale. Keep Mac and Windows profiles separate.
Do not silently rewrite goldens or invent Office observations. Preserve immutable
selections/receipts and their pinned source identities; historical passes do not
certify changed inputs. Two historical scope documents are pinned evidence bytes.

Report PASS/MISMATCH/UNSUPPORTED/BLOCKED_BY_POLICY/INVALID_FIXTURE/CRASH/TIMEOUT/
NOT_RUN accurately, with actual suites/denominators. Fresh Office comparisons are
still NOT_RUN; offline template validation is not release acceptance. Put tests in
the owning crate/app, shared data under root tests/, and generated reports under
ignored reports/. Exercise negative/budget paths, minimized crashes and relevant
native/WASI/UI/host gates. See [release capture](../../../docs/m8-office-comparison.md).
