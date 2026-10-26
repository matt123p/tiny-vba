# Planned executable examples

`pure-vba/Arithmetic.bas` is the first host-free target: return a VBA Long with value 42.
`foreign-host/FillCells.bas` is the later integration acceptance target from the design:
pause at an assignment, inspect/change `i`, resume, and check the call trace.
The Cells/Value test facade and real Office bridge belong to the application forks.
These files are self-authored fixtures, not supported execution demonstrations yet.
