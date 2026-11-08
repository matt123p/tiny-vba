# M8.5 comparison preparation

Original repository-authored protocol, MIT OR Apache-2.0. No Office observations,
Office-produced binaries or upstream macros are included.

`selection.json` pins eleven recipes to reviewed M7A source identities.
`results-template.json` contains null observations for every case. Initialize a
current-candidate copy with `python3 tools/check_m8_release.py --init OUTPUT`.

`preparation-execution.json` records maintenance validation only. The temporary
synthetic unit packets test the validator and are never Office evidence. Actual
comparison and full M8 release acceptance remain NOT_RUN.

See the [capture protocol](../../../docs/m8-office-comparison.md) and
[ADR 0102](../../../docs/decisions/0102-m8-release-evidence-gate.md).
