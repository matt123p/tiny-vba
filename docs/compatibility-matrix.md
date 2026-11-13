# Compatibility matrix

Support means the declared finite profile has executable tests. It does not mean
all VBA, all MSForms or arbitrary Excel/Word macros are compatible. Unsupported
syntax, capabilities and host operations must fail explicitly.

| Surface | Implemented profile | Boundary / contract |
|---|---|---|
| Frontend | Source-preserving bounded lexer/parser, module options, attributes and supported declarations/statements | [Frontend](frontend.md); parsing alone does not establish executable support |
| Compiler | Binding, typed arguments, nominal references/classes, deterministic verified bytecode through IR 14 | [Compiler](compiler.md); older capability targets retain explicit refusals |
| Values/runtime | Exact scalar tags, checked arithmetic/coercion, UTF-16 strings, array bounds, ByRef locations, persistent state, handlers and errors | [Runtime](runtime.md), [language profile](m4-language.md) |
| Intrinsics | Portable functions plus explicitly injected environment/locale services | [Inventory](vba-intrinsic-functions.md); missing services refuse, no ambient IO |
| Host calls | Native protocol 1.1, explicit roots/site permissions, typed patches, waits and callbacks | [Host integration](host-integration.md); document effects do not roll back on failure |
| Classes/events | Project-local instances, properties/defaults, interfaces, WithEvents, lazy As New scalars/array elements | [Runtime](runtime.md), [decisions](decisions/README.md); no arbitrary external activation |
| Native forms | `native-forms-common-v1`, inert native import, common runtime/designer and native Save/Export | [Forms](forms.md); finite controls/members/input combinations |
| Worksheet controls | GridCraft `worksheet-form-controls-v1` | Fork-owned identity, engine and persistence; separate from worksheet ActiveX |
| Project/packages | Bounded MS-OVBA decode, original-byte preservation and explicit source-only edited-copy export | [Project import](project-import.md), [forms](forms.md); protected/unsupported inputs refuse |
| Editor | Completion, signature help, diagnostics, navigation, editing, metadata browser and shared native/browser UI | [UI contract](editor-debugger-ui.md), [embedding](ui/embedding.md) |
| Debugger | Statement breakpoints/steps, typed locals/arrays, safe watches, Immediate policy, restricted Set Next Statement, optional DAP | [Debugger](debugger.md); watch inspection never calls getters/functions |
| Platforms | Headless native, browser WASM compilation/UI and actual WASI test execution | Separate gates in [testing](testing.md); no inferred result for untested OS/builds |

## Forms and persistence limits

The common forms profile does not establish exhaustive reentrant lifetime/focus
ordering, full platform/IME parity, every property/resource variant, advanced
binding shape/timing, AutoSize/container scrolling or worksheet Office-supplied
ActiveX support. Selected numeric, TabStrip, Image and RefEdit behavior is retained
at its tested level. See [forms](forms.md) for the precise boundary.

Ordinary document Save preserves imported VBA bytes. Persisting authored edits
requires native form Save/Export or the explicit edited XLSM/DOCM copy workflow.
Signature bytes may be preserved opaquely or removed when invalidated; cryptographic
verification/resigning is unsupported. P-code execution, protected-project bypass,
third-party ActiveX and arbitrary COM/OCX/DLL activation are excluded.

## Evidence and release status

Historical acceptance includes 44 authored language scenarios and 153 reviewed
ReflexGrid cases compared against inherited upstream-reported Excel Mac results.
The expanded local profile adds 24 selected inherited library cases, bounded
parser/importer/VM mutation tests, eight asserted native/WASI benchmarks and 20
original workbook workflows. These are finite suites, not a compatibility percentage.

Common native forms acceptance and local conformance/hardening are recorded in
reviewed selections and execution receipts. **Fresh Excel/Word comparison remains
NOT_RUN; the M8.5 release comparison gate remains open.** Offline recipe/template
validation does not satisfy that requirement. See [testing and provenance](testing.md)
and [release validation](m8-office-comparison.md).
