# ADR 0025 — Resolve getter aliases before member dispatch

Accepted 2026-10-29. Owner: vba-semantics; no new IR, protocol or dependency edge.

An imported fixed property getter (`Import.kind = Get`, `member = Some`) is a
value-producing expression. In `current.Member`, resolve the getter first, then
invoke Member on the returned object with its original Get/Call/Let/Set intent,
named arguments and source span. Evaluate it afresh at each source access; an
object assigned to a variable retains the returned identity. This permits hosts
to expose dynamic roots without mutable proxy handles or Office knowledge in
the interpreter.

Fixed method aliases (`kind = Call`) retain ADR 0013/0017's shortcut behavior.
Root-object imports (`member = None`) are unchanged. Hosts still own identities,
consent, lifetime and exactly-once native effects. No ambient object activation.

`vba-vm/tests/embedding.rs::getter_alias_resolves_before_chained_get_let_and_named_call`
checks six exact calls, separate returned-object targets, assignment intent and
named arguments. Existing method/root regressions remain required. This suite
also executes under `cargo xtask wasm-run`.
