# ADR 0031: scalar Variant For controls

Adopted 2026-11-01 for the reported implicit-counter diagnostic in Food.xlsm.

The semantics layer accepts explicit/implicit scalar Variant controls alongside
Integer/Long for the existing positive integral literal-Step For profile. Arrays
and other declared control kinds remain refused. This is shared VBA behavior;
Office objects, clipboard and sorting remain fork-owned.

The compiler captures Variant bounds once before counter assignment. It validates
numeric convertibility using existing Convert/DupValue/DropValue instructions,
then normalizes numeric strings/Empty with ordinary numeric addition of zero.
Original supported numeric widths remain on the ordinary shared arithmetic path;
no new IR opcode, execution profile or parallel host/UI coercion is introduced.
Conversion failure is an ordinary catchable VBA error, with no initial counter
write. Existing numeric overflow behavior and Step restrictions remain explicit.

The existing loop-init/test/advance checkpoints, aliases and budgets remain the
execution mechanism. Authored native/WASI regressions cover implicit/explicit
nested counters, zero iterations, Exit For, numeric-string bounds, invalid/Null
bounds and exact counter values through Stop/continue. The user workbook is
inspected read-only; its macros are not used as executable test fixtures.
