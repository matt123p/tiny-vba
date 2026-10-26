# Source examples

`pure-vba/Arithmetic.bas` is the first host-free target: return a VBA Long with value 42.
`foreign-host/FillCells.bas` is the later integration acceptance target from the design:
pause at an assignment, inspect/change `i`, resume, and check the call trace.
The Cells/Value test facade and real Office bridge belong to the application forks.
These self-authored files parse with M1's frontend:

```sh
cargo run -p vba-cli -- parse examples/pure-vba/Arithmetic.bas
cargo run -p vba-cli -- parse examples/foreign-host/FillCells.bas
```

Parsing does not execute them. No supported execution demonstration exists yet.

M2 adds verified compilation/disassembly:

```sh
cargo run -p vba-cli -- disasm examples/pure-vba/Arithmetic.bas
cargo run -p vba-cli -- compile examples/pure-vba/M2Control.bas --entry Answer
```

M2Control is independently authored and covers scalar If/For/direct calls and exact local ByRef.
These commands compile and verify; they do not run VBA. FillCells remains syntax-only here;
its foreign/global binding needs later host integration. See [compiler support](../docs/compiler.md).
