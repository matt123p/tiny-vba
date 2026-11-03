# README image provenance

Captured 2026-11-03 from tiny-vba's original shared egui renderer with:

```sh
cargo run --locked --offline -p vba-ui-egui --example snapshots
```

| Image | Generated source | Author / license |
|---|---|---|
| `debugger.png` | `reports/ui/dark-paused.png`; actual VM paused at a statement breakpoint with typed Locals | tiny-vba contributors; MIT OR Apache-2.0 |
| `completion.png` | `reports/ui/light-completion.png`; source-based member completion for `Debug.` | tiny-vba contributors; MIT OR Apache-2.0 |

These are unmodified product renders from
[`snapshots.rs`](../../crates/vba-ui-egui/examples/snapshots.rs), not Microsoft
screenshots or reference artwork. The incomplete `Debug.` expression in the
completion image intentionally also shows the live parser diagnostic.
Framework fonts retain their [dependency notices](../provenance/ui-dependencies.json).
Regenerate the reports, inspect them, then copy the two named PNGs here when updating.
