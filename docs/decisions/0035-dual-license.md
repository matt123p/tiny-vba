# 0035 — Align licensing with the ArtCraft applications

Date: 2026-11-03. Status: adopted on explicit maintainer direction.

## Decision

The maintainer requested that tiny-vba's license match GridCraft and WordCraft.
Both sibling applications declare `MIT OR Apache-2.0`. Adopt the same choice for
tiny-vba, superseding ADR 0001's item 6 and its open licensing decision.

Preserve the existing MIT text and copyright notice verbatim in `LICENSE-MIT`.
Add the standard Apache 2.0 text as `LICENSE-APACHE`, matching the sibling apps,
and make `LICENSE` explain the choice. Set workspace Cargo metadata and the
repository license gate to `MIT OR Apache-2.0`; publish the choice in the README.

Dependencies, fonts, external fixtures and reference submodules retain their own
licenses, copyright notices and provenance. This change does not relicense them
or add any runtime or Office compatibility capability.
