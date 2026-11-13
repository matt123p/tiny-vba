# Compiler and verified bytecode

`vba-semantics` binds the source AST, `vba-compiler` lowers it, and `vba-ir`
verifies the resulting bytecode. Only IR constructs opaque immutable
`VerifiedProgram` values. Compilation does not execute VBA or host operations.

## CLI and capability selection

```sh
cargo run -p vba-cli -- check examples/pure-vba/Arithmetic.bas
cargo run -p vba-cli -- compile examples/pure-vba/M2Control.bas --entry Answer
cargo run -p vba-cli -- disasm examples/pure-vba/Arithmetic.bas
```

`check` and `compile` run parse/bind/lower/verify; `disasm` emits diagnostic text
for the verified program. These commands accept UTF-8 standard modules and an
explicit entry name. There is no persisted bytecode file format or binary loader.

`CompileOptions::default()` selects `ir7-structural`. Execution callers must select
`CompileOptions { capabilities: vba_vm::capabilities(), ..Default::default() }`
and use the VM's actual capabilities, currently through IR 14. The VM verifies
compatibility again at session creation. A caller-provided capability set cannot
enable unimplemented instructions. Older structural targets retain their own
feature/version refusal rules; recompile source rather than relabeling a program.

## Binding

Names are case-insensitive. Source identities remain distinct across modules;
module/procedure visibility, declaration shadowing, Option Explicit, component
kind and reference snapshots control resolution. Host metadata must be supplied
explicitly; metadata discovery never activates external objects/libraries.
See [module contexts](vba_module_types_and_execution_contexts.md) and
[name/object resolution](vba_excel_context_and_object_resolution.md).

Bound calls retain nominal types, parameter modes and canonical ByRef locations.
Named/optional/omitted arguments preserve source-order evaluation and map into
declaration order without replay. Omitted argument metadata is distinct from a
runtime Missing/Empty/Null value. Shared runtime coercion handles constants,
defaults and typed values; the binder does not implement another conversion path.

The implemented language profile includes supported scalars/arrays, numeric and
collection loops, handlers, persistent module/Static state, project classes,
properties/default members, events and explicit host imports. Native form support
is supplied by the IDE through ordinary classes and typed references, not Office
names inside the compiler/VM. Unsupported syntax or binding shapes produce
structured diagnostics rather than successful placeholder operations.

## Lowering and verification

Lowering is deterministic and emits explicit frames, stack operations, source
maps and statement checkpoints. Distinct statements on one line retain distinct
identities. Call continuations preserve caller operands and resume without
re-evaluating effects. Hidden support/attribute records do not become authored
breakpoint sites. See [attribute projection](vba-attributes-and-editor-projection.md).

Verification checks instruction operands, indices, stack effects and joins,
branch/checkpoint targets, call signatures, location types, source maps and
capability requirements before execution. Work and input-derived metadata are
bounded. Resource failures are distinct from unsupported features and VBA errors.
The [bytecode specification](bytecode-vm-specification.md) defines the foundational
IR 1 layout; versioned decisions define the extensions.

## Versioned extensions

| IR | Contract | Decision |
|---|---|---|
| 1 | Stack bytecode, verification, checkpoints and explicit continuations | [0004](decisions/0004-bytecode-vm-contract.md), [0006](decisions/0006-m2-binding-and-verification.md) |
| 2 | Arrays, handlers, intrinsics and persistent slots | [0011](decisions/0011-m4-language-extension.md) |
| 3 | Array-layout/parameter metadata and canonical reference captures | [0012](decisions/0012-m4-oracle-acceptance.md) |
| 4 | Host roots and native protocol binding | [0013](decisions/0013-m5-host-contract.md) |
| 5 | Nominal host object types | [0021](decisions/0021-nominal-host-object-types.md) |
| 6 | Default members and collection enumeration | [0022](decisions/0022-default-members-and-enumeration.md) |
| 7 | Project classes, instances and form execution | [0023](decisions/0023-project-classes-and-forms.md), [0024](decisions/0024-standard-form-execution-and-exchange.md) |
| 8 | Descriptor-verified extended intrinsic calls/context | [0034](decisions/0034-intrinsic-implementation-framework.md) |
| 9 | Class enumerator metadata | [0045](decisions/0045-class-and-native-collection-enumerators.md) |
| 10 | Lazy scalar project-class/native-form As New | [0046](decisions/0046-lazy-project-class-variables.md) |
| 11 | Nominal class interfaces and Forms bindings | [0047](decisions/0047-native-forms-nominal-types-and-library-access.md) |
| 12 | Inert runtime class names/TypeName | [0064](decisions/0064-native-runtime-type-names.md) |
| 13 | VarType default-value inspection through ordinary frames/authorized calls | [0067](decisions/0067-vartype-default-value-inspection.md) |
| 14 | Lazy project-class/native-form array elements | [0068](decisions/0068-lazy-project-class-array-elements.md) |

The [runtime](runtime.md), [intrinsic design](intrinsic-design.md) and
[forms](forms.md) guides describe execution and ownership. Tests under
`vba-semantics`, `vba-compiler` and `vba-ir` cover binding, deterministic
lowering/disassembly, malformed programs and legacy capability refusals.
Actual native/WASI execution is separate evidence; see [testing](testing.md).
