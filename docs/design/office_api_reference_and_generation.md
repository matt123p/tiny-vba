# Microsoft Office Object Model: Reference Sources and Rust API Generation

**Status:** Proposed reference-ingestion and code-generation specification  
**Version:** 0.1  
**Prepared:** 2026-10-26  
**Companion document:** [`office_automation_architecture.md`](office_automation_architecture.md)  
**Target applications:** [GridCraft](https://github.com/storytold/gridcraft) and [WordCraft](https://github.com/storytold/wordcraft)  
**Implementation language:** Rust  
**Scope:** Sources, extraction, normalization, validation, versioning and code generation for the **language-independent Office compatibility layer**. This document does **not** specify the implementation of a VBA parser, compiler, interpreter, or Python/JavaScript engine.

> **Recommended decision:** Treat [MicrosoftDocs/VBA-Docs](https://github.com/MicrosoftDocs/VBA-Docs) as the version-pinned *primary behavioural reference*, and use published Microsoft Office Primary Interop Assembly (PIA) API signatures and, where lawfully available, COM type-library metadata as *secondary structural references*. Convert these sources into a reviewed, versioned, machine-readable schema, from which the Rust object-dispatch registry, constants, documentation, conformance scaffolding and compatibility reports can be generated. **Do not generate dummy implementations that report success.**

---

## 1. Executive summary

GridCraft and WordCraft already expose useful Rust document-engine APIs. The new layer needs to mimic the **Microsoft Excel and Word VBA object models**, rather than simply wrap application commands. To make that feasible across hundreds of Office objects and thousands of members, keep the object model in a central, declarative catalogue derived from authoritative references.

A good reference catalogue has to answer:

1. **Existence:** Is `Excel.Range.Offset`, `Word.Document.Range` or `Excel.Worksheet.Change` part of the target API?
2. **Shape:** Is it a property, method or event? What arguments does it accept, in what order, and what are its return and parameter types?
3. **Dispatch semantics:** Are arguments optional or named? Is it an indexed/default property? Is the object a collection, and how is it enumerated?
4. **Observable behaviour:** What does the member do? What happens with multi-cell ranges, empty values, missing objects, changes to document structure and errors?
5. **Implementation status:** Is the member implemented, partially implemented, deliberately unsupported or not yet researched in GridCraft/WordCraft?
6. **Provenance:** Which exact source and revision support each assertion? Which properties are inferred and require confirmation?

No single currently identified source answers all six reliably. Therefore **the normalized catalogue is an independently maintained project artifact**, not merely a copy of the Markdown documentation or .NET signatures.

### Priority ordering

| Source | Best use | Authority / limitation |
|---|---|---|
| [MicrosoftDocs/VBA-Docs](https://github.com/MicrosoftDocs/VBA-Docs) | Objects, members, semantics, examples, events, enumerations | Official Microsoft-maintained Markdown; individual descriptions and signatures can be incomplete or inconsistent |
| [Excel VBA object model](https://learn.microsoft.com/en-us/office/vba/api/overview/excel/object-model) | Discover intended Excel API surface | Official browser documentation; primarily navigational |
| [Word VBA object model](https://learn.microsoft.com/en-us/office/vba/api/overview/word/object-model) | Discover intended Word API surface | Official browser documentation; primarily navigational |
| [Excel Interop API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.interop.excel?view=excel-pia) | Typed COM-interface signatures and metadata | Strong structural reference; .NET/COM projection, not a direct neutral or VBA semantic model |
| [Word Interop API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.interop.word?view=word-pia) | Typed Word COM interfaces and metadata | Same qualifications |
| [Microsoft Office Core Interop API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.core?view=office-pia) | Shared Office objects and enums | Required by both Excel and Word; avoid duplicating shared Office definitions |
| [Microsoft Open Specifications](https://learn.microsoft.com/en-us/openspecs/) | File format and VBA/Office protocol definitions | Authoritative within their specified scope; not a complete Excel/Word object-model definition |
| Excel/Word black-box compatibility tests | Resolve ambiguities and behaviour not specified sufficiently in documentation | Requires appropriately licensed Office installations and independent test fixtures; test observations must be version-tagged |

**Do not use Office.js as the authoritative model for VBA.** [Office JavaScript APIs](https://learn.microsoft.com/en-us/office/dev/add-ins/develop/understanding-the-javascript-api-for-office) are useful as supplementary cross-platform comparisons, but they use their own object relationships, asynchronous/batch semantics and requirements sets. This project targets the **classic Office VBA/COM object model**, even when the consuming language is Python or JavaScript.

---

## 2. Primary source: MicrosoftDocs/VBA-Docs

### Repository and local checkout

- Repository: <https://github.com/MicrosoftDocs/VBA-Docs>
- API directory: <https://github.com/MicrosoftDocs/VBA-Docs/tree/main/api>
- Documentation licence: <https://github.com/MicrosoftDocs/VBA-Docs/blob/main/LICENSE> — Creative Commons Attribution 4.0 International (**CC BY 4.0**)
- Code-sample licence: <https://github.com/MicrosoftDocs/VBA-Docs/blob/main/LICENSE-CODE> — **MIT**

```bash
git clone https://github.com/MicrosoftDocs/VBA-Docs.git references/VBA-Docs
cd references/VBA-Docs
git rev-parse HEAD  # record this exact commit in the generator lockfile

# Inventory documents. Do not assume all files follow one case convention.
find api -maxdepth 1 -type f -name '*.md' | sort > ../vba-docs-api-files.txt
```

The repository contains many individual pages under `api/`. Filenames often follow names such as `Excel.Range.Value2.md`, `Word.Document.Range.md` and `Excel.Range(object).md`, but capitalization, suffixes and other details are **not** consistently predictable. Discover files by enumerating the repository and reading YAML front matter; do not construct file paths from member names alone.

### Examples checked against the actual repository

| Page | What it provides | Recommended use |
|---|---|---|
| [`Excel.Range(object).md`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Excel.Range%28object%29.md) | Description; properties/methods; important default-member rules | Discover `Range`, its semantics, and member relationships |
| [`Excel.Range.Value2.md`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Excel.Range.Value2.md) | Read/write `Variant`, and the difference from `Value` regarding Date/Currency variants | Model `.Value2` separately from `.Value` |
| [`Word.Document.Range.md`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Word.Document.Range.md) | `Start` and `End` are optional `Variant` arguments; returns `Word.Range` | Method signature and optional-argument tests |
| [`Word.Document.md`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Word.Document.md) | Object description and lists of events, methods and properties | Generate member inventory |
| [`Excel.Worksheet.Change.md`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Excel.Worksheet.Change.md) | `Worksheet.Change(Target As Range)` event and when it does *not* fire | Event-schema and ordering tests |
| [`Word.Document.Open.md`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Word.Document.Open.md) | Document-open event and special rules for templates | Word event catalogue |
| [`Excel.XlCalculation.md`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Excel.XlCalculation.md) | Numeric values `-4105`, `-4135`, `2` | Generate Excel constants |
| [`Word.WdColorIndex.md`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Word.WdColorIndex.md) | Numeric Word colour indices such as `wdYellow = 7` | Generate Word enums/constants |

### Useful sections to extract

The extraction tool should preserve separate information from:

- YAML front matter: `title`, `api_name`, `ms.assetid`, `ms.date`, `ms.localizationpriority`, etc.
- Page heading and short description.
- `Syntax`, `Parameters`, `Return value`, `Remarks`, `Example` and `See also` sections.
- Object-level `Events`, `Methods` and `Properties` lists.
- Enumerations, including signed numeric values and aliases.
- Hyperlinks to related objects and members.
- Notes on defaults, errors, exceptions, context dependence and mutation behaviour.
- Information about whether a property is read-only, write-only, or read/write.

**Important:** `ms.date` is page metadata, *not* a reliable representation of which Office release implements a feature. Pin a source commit and track Office-version applicability separately.

### Extract once; retain source evidence

Keep both parsed metadata and provenance. Never silently invent missing types or arguments. A field that isn't documented should remain `unknown` or `needs_review`, with a pointer back to its source.

---

## 3. Supplementary source: Office Primary Interop Assemblies

### Microsoft references

- [Microsoft.Office.Interop.Excel API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.interop.excel?view=excel-pia)
- [Microsoft.Office.Interop.Word API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.interop.word?view=word-pia)
- [Microsoft.Office.Core API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.core?view=office-pia)
- [Microsoft: Office Primary Interop Assemblies](https://learn.microsoft.com/en-us/visualstudio/vsto/office-primary-interop-assemblies?view=visualstudio)

Microsoft describes PIAs as managed-code interop representations of the underlying COM Office object models. A PIA or corresponding type library can reveal:

- Interface names and their relationships.
- Methods and properties, including return and argument types.
- Enumeration types and integer values.
- Optional/default arguments and COM metadata where present.
- Parameter directions (`in`, `out`, `retval`), marshaling details, and sometimes member identifiers (`DISPID`).
- Shared Office interfaces reused across host applications.

### Limitations and normalization

A .NET interop signature is not automatically the signature a VBA program observes. Handle:

| COM/.NET representation | Neutral schema treatment |
|---|---|
| `object`, `VARIANT` or `System.Object` | `Variant`/`Any` as documented; keep original source type |
| `SAFEARRAY` / managed array | Array with element kind, rank if known, and per-dimension lower bounds determined at runtime |
| Optional `object` argument with missing sentinel | Distinguish `Missing` from explicit `Empty` and `Null` |
| `ref` / `out` argument | Preserve direction and reference requirement; language-specific `ByRef` handling remains outside the host model |
| `HRESULT` or .NET exception | Map to a neutral host-error taxonomy; convert to VBA error numbers only at the VBA boundary |
| `DISPID` / default member | Record as external compatibility metadata, not necessarily the internal Rust `MemberId` |
| COM `IEnumVARIANT` / special enumeration member | Model explicit collection enumeration behaviour and ordering |
| `get_X` / `set_X` accessors | Normalize to the correct property and read/write accessor semantics |
| COM coclass vs interface | Deduplicate toward a canonical Office object with type/interface relations |
| Separate event interface | Normalize to events, not ordinary methods |

**Don't** generate one public Rust method for every COM interface method without review. COM inheritance/projection may yield duplicate members or signatures that do not match VBA call behaviour.

### Licensing and provenance caution

NuGet packages and copies of Microsoft-generated PIAs are **not automatically permissively licensed**. Some third-party repackagings explicitly disclaim a license (e.g. [MicrosoftOfficeInterop on NuGet](https://www.nuget.org/packages/MicrosoftOfficeInterop)). Do not vendor, redistribute or publish extracted proprietary binaries without determining the permissions for the particular source. Prefer official published reference material and an independently normalized implementation schema. Have distribution/licensing decisions reviewed before releasing generated artifacts.

The project should be able to run its normal code generator **without Office or a PIA present**. Interop inspection is an optional, separately documented reference-import step, and the normalized schema is reviewed into source control.

---

## 4. Additional official references

### 4.1 Enumeration catalogues

- [Excel enumerations](https://learn.microsoft.com/en-us/office/vba/api/excel%28enumerations%29)
- [Word enumerations](https://learn.microsoft.com/en-us/office/vba/api/word%28enumerations%29)
- [Excel `XlCalculation`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Excel.XlCalculation.md)
- [Word `WdColorIndex`](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/api/Word.WdColorIndex.md)

Numeric constants are not cosmetic: existing VBA may compare against literal numeric values, persist them, or use them in parameter assignments. Preserve exact signed values and aliases even where a Rust enum cannot represent every dynamic input.

### 4.2 Worksheet functions called through the Office object model

- [Worksheet functions available to VBA](https://learn.microsoft.com/en-us/office/vba/excel/concepts/events-worksheetfunctions-shapes/list-of-worksheet-functions-available-to-visual-basic)
- [Excel `WorksheetFunction` object](https://learn.microsoft.com/en-us/office/vba/api/excel.worksheetfunction)
- [GridCraft source and formula/function architecture](https://github.com/storytold/gridcraft/blob/main/docs/architecture.md)

`Application.WorksheetFunction` and `Application.Evaluate` should be **Office façade adapters to GridCraft's existing calculation engine**. Their public argument handling and error behaviour must follow Excel's VBA contracts, not necessarily formula-cell semantics. Avoid implementing hundreds of worksheet functions twice.

### 4.3 Office file-format and macro-package contracts

- [MS-OVBA — Office VBA File Format Structure](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/575462ba-bf67-4190-9fac-c275523c75fc)
- [MS-VBAL — VBA Language Specification](https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/d5418146-0bd2-45eb-9c7a-fd9502722c74)
- [MS-OE376 — Office Open XML File Formats](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/)

`MS-OVBA` informs package preservation, extraction and attachment to a document context. `MS-VBAL` is the language-runtime specification and belongs primarily to the separately planned interpreter project; consult it here only where language/host semantics touch. Neither document supplies the complete Excel/Word object model.

**Observed Craft baseline (2026-10-26):**

- [GridCraft file-format parity](https://github.com/storytold/gridcraft/blob/main/docs/file-format-parity.md) describes `.xlsm`/`.xltm` round-trip preservation of the VBA project but no execution. Verify with the current integration tests before changing file IO.
- [WordCraft file-format parity](https://github.com/storytold/wordcraft/blob/main/docs/file-format-parity.md) describes `.docm`/`.dotm` preservation of macros/signatures but no execution. Maintain this behaviour.

### 4.4 Behavioural concepts

- [Excel `Range` object](https://learn.microsoft.com/en-us/office/vba/api/excel.range%28object%29)
- [Excel `Range.Value`](https://learn.microsoft.com/en-us/office/vba/api/excel.range.value)
- [Excel `Range.Value2`](https://learn.microsoft.com/en-us/office/vba/api/excel.range.value2)
- [Word `Range` concepts](https://github.com/MicrosoftDocs/VBA-Docs/blob/main/word/Concepts/Working-with-Word/working-with-range-objects.md)
- [Word `Document.Range` method](https://learn.microsoft.com/en-us/office/vba/api/word.document.range)
- [Excel worksheet Change event](https://learn.microsoft.com/en-us/office/vba/api/excel.worksheet.change)
- [Word Document Open event](https://learn.microsoft.com/en-us/office/vba/api/word.document.open)

These are particularly important for implementing correct **live object semantics**, rather than detached data transfer objects.

---

## 5. The canonical API metadata model

The repository should own one versioned schema containing objects, properties, methods, events, enums and shared type definitions. Metadata is **language-neutral** and **application-neutral** within the Office compatibility surface.

### 5.1 Required top-level entities

1. **Library:** `Excel`, `Word`, `Office` (shared), optionally later `Forms`, `VBIDE` and additional host libraries.
2. **Object type:** `Excel.Application`, `Excel.Workbook`, `Excel.Worksheet`, `Excel.Range`, `Word.Document`, `Word.Range`, etc.
3. **Member:** property, method or event, with ordered arguments and a return type.
4. **Enumeration:** named constants and exact integer values, including aliases.
5. **Type reference:** built-in scalar, Variant, array, enum, Office object/interface, void or unknown.
6. **Metadata evidence:** source URLs, pinned revisions and confidence/verification status.
7. **Host capability mapping:** which GridCraft/WordCraft host operations are needed to implement the member.
8. **Coverage record:** unimplemented, partial, implemented, verified, unsupported-by-host or deferred.

### 5.2 Example schema

```yaml
schema_version: 1
library: Word
object: Word.Document
kind: object
sources:
  - kind: microsoft_vba_docs
    path: api/Word.Document.md
    revision: ${PINNED_VBA_DOCS_COMMIT}

members:
  - name: Range
    kind: method
    return_type: Word.Range
    parameters:
      - name: Start
        type: Variant
        optional: true
        default: missing
        position: 0
      - name: End
        type: Variant
        optional: true
        default: missing
        position: 1
    dispatch:
      named_arguments: true
      case_insensitive: true
    behaviour:
      summary: Returns a live Word.Range within the document.
      review_required: true
    sources:
      - kind: microsoft_vba_docs
        path: api/Word.Document.Range.md
        revision: ${PINNED_VBA_DOCS_COMMIT}
    status: not_implemented
```

`${PINNED_VBA_DOCS_COMMIT}` is a **build-time placeholder** and must become an actual full git SHA in the generated lockfile; it is not valid source metadata when left unresolved.

### 5.3 Property example: Excel `Range.Value2`

```yaml
library: Excel
object: Excel.Range
members:
  - name: Value2
    kind: property
    return_type: Variant
    access: read_write
    index_parameters: []
    behaviour:
      summary: Read or assign values of the range.
      notes:
        - Multi-cell reads may return a two-dimensional array.
        - Unlike Value, Value2 does not use Currency and Date VBA subtypes.
    sources:
      - kind: microsoft_vba_docs
        path: api/Excel.Range.Value2.md
        revision: ${PINNED_VBA_DOCS_COMMIT}
    status: not_implemented
```

The example is deliberately schematic: precise array assignment, single-cell return and error conditions require conformance cases before marking this member `verified`.

### 5.4 Enumeration example: Excel calculation mode

```yaml
library: Excel
enum: Excel.XlCalculation
underlying_type: i32
values:
  xlCalculationAutomatic: -4105
  xlCalculationManual: -4135
  xlCalculationSemiautomatic: 2
sources:
  - kind: microsoft_vba_docs
    path: api/Excel.XlCalculation.md
    revision: ${PINNED_VBA_DOCS_COMMIT}
```

Generate **both** Rust-friendly typed values and the original Office constant-name registry. Scripts refer to `xlCalculationManual`, not `XlCalculation::Manual`.

---

## 6. Semantic features that the schema must represent

### 6.1 Default members

Microsoft documents that an Excel `Range`'s default member forwards a call without parameters to its `Value` property and a call with parameters to `Item`.

```vb
someRange = someOtherRange        ' Value semantics
x = someRange(1, 1)            ' Item semantics
```

The schema must explicitly represent default-member **selection rules**, not just a boolean `is_default` flag. Other collections may have default indexed `Item` members. Documentation inconsistencies should be resolved using reference tests.

### 6.2 Indexed properties, named and omitted arguments

Preserve:

- Indexed `Item` / `Range` and properties that look like method calls in VBA.
- Positional versus named arguments (`Start:=0, End:=10`).
- Optional parameters and an **absent** argument, distinct from explicitly passed `Null` or `Empty`.
- Parameter passing direction and any observable mutation of arguments.
- Case-insensitive lookup for VBA, while allowing language adapters to expose idiomatic aliases.
- Zero-argument calls versus an omitted parameter list.

### 6.3 Collections and enumeration

A collection schema should record `Count`, default `Item`, legal keys, indexing base, enumeration order and mutability. Do not assume every collection is one-based merely because many Office collections are; verify each.

### 6.4 Object identity and live references

The schema must describe *what an object denotes*; runtime object handles and locators remain in the Office compatibility core. Excel `Range` points to a live range; Word `Range` points to a live text span and can be affected by edits. Include invalidation and mutation tests for held references.

### 6.5 Events

Events need parameter definitions, source object, conditions, suppression controls and sequencing rules. Example: Microsoft's `Worksheet.Change` documentation says this event doesn't occur for cells that changed *only because of recalculation*. A correct facade must distinguish direct edits from formula recalculation.

### 6.6 Error behaviour

The catalog should be able to annotate known error types and circumstances. The host emits neutral structured errors; a future VBA adapter maps them into VBA run-time errors. Do not encode VBA numeric errors directly into GridCraft/WordCraft core operations.

### 6.7 Cross-application shared types

Include common Office objects and enums in a shared namespace, rather than creating independent Excel and Word copies. Document each application-specific projection or alias where behaviour differs.

---

## 7. Extraction and normalization pipeline

```text
Pinned MicrosoftDocs/VBA-Docs checkout
           │
           ├── front matter + headings + tables + hyperlinks
           │
           ├──────────────┐
           ▼              │
    Raw source inventory  │    Optional, legally acquired
           │              │    Interop/type-library metadata
           ▼              │              │
    Markdown extractor    │              ▼
           │              │       Structural importer
           └──────────────┴──────────────┘
                          │
                          ▼
                 Evidence records
                          │
                          ▼
                Schema reconciler
                   │           │
              confident      ambiguous
                   │           │
                   │        Review queue
                   │           │
                   └─────┬─────┘
                         ▼
              Reviewed canonical YAML
                         │
          ┌──────────────┼───────────────┐
          ▼              ▼               ▼
    Rust dispatch     Enum/constants   Conformance
      registry         and type map     test scaffolds
          │              │               │
          └──────────────┼───────────────┘
                         ▼
                 Coverage reports
```

### 7.1 Pass A — inventory

Enumerate all files, parse front matter, and classify each document as an object overview, property, method, event, enumeration or unclassified page. Build links from overview pages to member pages. Use filename patterns as hints, **never the sole classifier**.

Output fields: `source_path`, `api_name`, `title`, `inferred_kind`, `classification_confidence`, `source_sha`, `links`.

### 7.2 Pass B — structure extraction

Parse Markdown into a proper AST, rather than with a single large regex. Extract tables, syntax sections, return-value sections and event signatures. Handle mixed casing, absent sections, different table shapes, italic parameter names, smart quotes, Markdown reference links, relative links and incorrectly typed documentation.

Normalize whitespace but preserve original identifiers and their spelling. Record **raw snippets / offsets** internally for reviewer diagnostics, while respecting documentation licensing if distributing derived content.

### 7.3 Pass C — reconciliation

Merge same-name members and resolve conflicting signatures with a deterministic source-precedence rule:

1. Specific Microsoft member page for description/observable behaviour.
2. Published interop/type-library metadata for technical types and directions, if legally sourced.
3. Microsoft object-overview page for membership discovery.
4. Human review and independent Excel/Word conformance observations for disputed details.

Conflicts become review tasks; a source with less information must not overwrite a known more-specific assertion. No implicit conversion from `unknown` to `Variant` just to make generation pass.

### 7.4 Pass D — reviewed overlay

Some members require manual corrections, semantic annotations and host mappings. Store these in a **small, tracked override layer**, not edits to vendored source files.

```text
references/
  vba-docs.lock.toml
  sources/                 # optional checkout outside the packaged product
schema/
  generated/raw/           # ignored or committed according to policy
  reviewed/
    excel/
    word/
    office/
  overrides/
    default-members.yaml
    signature-fixes.yaml
    events.yaml
  review-queue.yaml
```

Review overlays must cite source links or conformance tests, state their rationale, and be detectable after a source update.

### 7.5 Pass E — validation and generation

Validate uniqueness of fully qualified member IDs, parameter ordering, enum ranges, type resolution, object references, event signatures and external source links. Refuse production generation on fatal inconsistencies; allow explicitly tracked unresolved fields for unimplemented members.

Generate Rust code deterministically with stable ordering and formatted output. CI should fail if `cargo xtask api-generate --check` would change committed generated files.

---

## 8. Recommended Rust repository organization

Use a **standalone offline reference tool**, not a runtime dependency on Markdown, GitHub or .NET.

```text
craft-automation/
  docs/
    office_automation_architecture.md
    office_api_reference_and_generation.md   # this file
    office_api_coverage.md                   # generated summary
  references/
    vba-docs.lock.toml
    NOTICE-reference-sources.md
  schema/
    reviewed/
      excel/
      word/
      office/
    overrides/
      default-members.yaml
      semantic-notes.yaml
      type-fixes.yaml
    schema-version.toml
  tools/
    office-api-import/
      src/
        github_markdown.rs
        interop_import.rs    # optional source importer
        reconcile.rs
        validate.rs
        report.rs
    office-api-codegen/
      src/
        dispatch.rs
        enums.rs
        docs.rs
        tests.rs
  crates/
    automation-types/
    automation-dispatch/
    office-excel/
    office-word/
    office-common/
    automation-testkit/
```

The tools may use Rust crates for YAML, Markdown AST and command-line handling. Keep ingestion tooling dev-only; the runtime depends on generated Rust metadata and hand-written behaviour implementations, not on parsers or file downloads.

### Suggested command interface

```bash
cargo xtask api-import --source references/VBA-Docs
cargo xtask api-reconcile
cargo xtask api-validate
cargo xtask api-generate
cargo xtask api-generate --check
cargo xtask api-coverage --format markdown
cargo xtask api-review --unresolved
```

These commands are **proposed**, not commands currently available in either Craft project.

---

## 9. Generated artifacts

### 9.1 Rust member registry

Generated metadata should support fast case-insensitive resolution of `Excel.Worksheet.Range` and other members to **internal** `MemberId`s, with complete argument descriptions.

```rust
// Conceptual generated API; not compiled or committed code.
pub struct MemberSpec {
    pub declaring_type: TypeId,
    pub name: &'static str,
    pub id: MemberId,
    pub kind: MemberKind,
    pub params: &'static [ParameterSpec],
    pub return_type: TypeId,
    pub access: AccessMode,
    pub support: SupportLevel,
}
```

Stable internal member IDs can be derived from a canonical key or explicitly assigned. Do **not** rely on unverified COM `DISPID`s, documentation filenames, or declaration order for persistence. Reserve collisions and schema migration tests.

### 9.2 Rust constant lookup

Generate original symbolic names and numeric values in a lookup table. Optionally generate ergonomic Rust enums where they are safe and complete.

```rust
// Illustrative output.
pub const XL_CALCULATION_AUTOMATIC: i32 = -4105;
pub const XL_CALCULATION_MANUAL: i32 = -4135;
pub const XL_CALCULATION_SEMIAUTOMATIC: i32 = 2;
```

### 9.3 Behaviour stubs and implementation scaffolds

Generating trait interfaces and **explicitly unsupported member registrations** is useful. Generating empty methods returning `Ok(())`, `Null` or a guessed default is dangerous and must be forbidden.

### 9.4 Generated documentation and coverage

Provide per-object matrices listing:

| Member | Type | Source | Status | Tests | Host operation |
|---|---|---|---|---|---|
| `Excel.Range.Value2` | read/write property | MS VBA-Docs | verified/partial/etc. | linked test IDs | range read/write |
| `Excel.Range.Offset` | property returning Range | MS VBA-Docs | ... | ... | range locator |
| `Word.Document.Range` | method returning Range | MS VBA-Docs | ... | ... | range locator |
| `Excel.Worksheet.Change` | event | MS VBA-Docs | ... | ... | engine edit event |

These are illustrative rows, not actual completion claims.

---

## 10. Exact behaviours to prioritize

Do not attempt all Office APIs at once. Build a complete schema inventory, then implement high-use surfaces with focused compatibility tests.

### GridCraft / Excel first

1. `Excel.Application`: active/containing workbook, active worksheet, selection, calculation, event switches.
2. `Excel.Workbooks`, `Excel.Workbook`, `Excel.Worksheets`, `Excel.Worksheet`: collections, indexing, lookup, lifecycle.
3. `Excel.Range`: `Value`, `Value2`, `Formula`, `FormulaR1C1`, `Address`, `Cells`, `Rows`, `Columns`, `Offset`, `Resize`, `Item`, `Count`, enumeration, `ClearContents`, `Copy`, formatting.
4. `Excel.Font`, `Excel.Interior`, `Excel.Borders` and `Excel.Names`.
5. `Excel.WorksheetFunction` and `Application.Evaluate` adapters into GridCraft's function engine.
6. Workbook/worksheet events, especially `Workbook_Open`, `Worksheet_Change`, `Worksheet_Calculate` and save/close lifecycle.

### WordCraft / Word second

1. `Word.Application`, `Word.Documents`, `Word.Document`.
2. `Word.Range`, `Word.Selection`, `Word.Paragraphs`, `Word.Paragraph`.
3. `Word.Font`, `Word.ParagraphFormat`, `Word.Styles`, `Word.Style`.
4. `Word.Tables`, `Word.Table`, `Word.Cell`, `Word.Bookmarks`, `Word.Fields` as subsequent feature sets.
5. `Document_Open`, save/close and selection/document modification events where documented.

### Workload-driven ranking

Rank members by (1) presence in representative `.xlsm`/`.docm` macro corpora, (2) dependencies of already selected members, (3) implementation cost, (4) observable compatibility significance and (5) native host capabilities. Maintain a **source coverage percentage** separately from **behavioural conformance percentage**; they measure different things.

---

## 11. Conformance-testing methodology

A metadata catalogue can be syntactically complete and still behave unlike Microsoft Office. Differential tests are therefore a core deliverable.

### 11.1 Test layers

1. **Source parser tests:** Markdown table/heading/frontmatter oddities and malformed files.
2. **Schema tests:** no unresolved internal type references, duplicate member identities or wrong enum values.
3. **Generated registry tests:** property/method/event classification, named arguments, member lookup and case handling.
4. **Host contract tests:** fake GridCraft/WordCraft model, document ownership, invalid handle, errors and events.
5. **Real host integration tests:** direct Rust calls against actual engine sessions, including undo and round-trip saves.
6. **Office conformance tests:** create the same operations in Microsoft Excel/Word where available; compare outcomes and error cases.

### 11.2 Proposed golden fixtures

| Area | Example | Expected test emphasis |
|---|---|---|
| Excel scalar range | `Range("A1").Value2` | Value type and coercion |
| Excel rectangular range | `Range("A1:B2").Value` | 1-based 2-D array and shape |
| Excel default member | `Range("A1:B2")(1, 1)` | `Item` dispatch versus `Value` |
| Excel omitted argument | `Range("A1").Offset(, 2)` | `Missing` semantics |
| Excel collection | `Worksheets(1)`, `Worksheets("Sales")` | Index base, names, errors |
| Excel calculation | `Calculation = xlCalculationManual` | Interaction with GridCraft calc |
| Excel event | change A1 then recalc formula in B1 | `Change` does not fire for recalc-only changes |
| Word range | `Document.Range(0,10)` | Character-position semantics |
| Word editing | hold a range, insert before it | Live range movement/invalidation rules |
| Word selection | `Selection.TypeText` | Selection replacement, formatting and caret |
| Word opening | `Document_Open` | Event ordering and trust requirements |

Where testing is not available, retain `not_verified`, not `passed`.

### 11.3 Evidence record per case

```yaml
case: excel.range.default_item_2d
source:
  path: api/Excel.Range(object).md
  revision: ${PINNED_VBA_DOCS_COMMIT}
reference_product: Excel
reference_version: reviewed-externally-or-not-yet-measured
host: GridCraft
status: not_run
notes: Default invocation with two indices should use Item semantics.
```

Pin actual Excel/Word product versions for measured outcomes. Do not substitute a website's `ms.date` for that version.

---

## 12. Documentation precedence and resolving conflicts

**Recommended rule:**

1. **Observed target-application compatibility behaviour**, measured against a specific Excel/Word version with reproducible tests, determines the conformance target when official documentation is ambiguous.
2. **Official member-specific VBA documentation** determines intended behaviour in the absence of a contrary reliable observation.
3. **Official COM/PIA structural metadata** fills in type/signature details, subject to VBA-specific projection differences.
4. **Official object overview pages** supply member inventory and relationships.
5. **Office.js descriptions or third-party examples** may provide clues but do not silently override classic VBA definitions.

Every conflict resolution should become a comment/overlay containing (a) the competing claims, (b) the selected interpretation, (c) the evidence, (d) affected tests and (e) the Office target version.

Review all `low_confidence` and `needs_review` members before releasing them as `supported`.

---

## 13. Licensing, attribution and clean-room development

This work must be a **fresh implementation**. In particular:

- Do not copy or adapt code from `craft-vba` or any other excluded VBA implementation.
- Use Microsoft public reference documentation and specifications to understand observable APIs.
- Microsoft's VBA-Docs content has **CC BY 4.0** documentation licensing; any redistributed adapted documentation may require attribution and indication of changes. Its example code is covered by the repository's separate **MIT** code licence; preserve applicable notices.
- Avoid bundling proprietary Microsoft Office binaries, DLLs, type libraries or PIAs into released products unless redistribution rights have been established.
- Generated interface facts (names, numeric enum values, signatures) should be tracked with provenance and should not be treated as a blanket licence to republish long documentation excerpts.
- Maintain a `NOTICE-reference-sources.md` listing source, date, commit and licensing basis.
- Respect [GridCraft](https://github.com/storytold/gridcraft/blob/main/AGENTS.md) and [WordCraft](https://github.com/storytold/wordcraft/blob/main/AGENTS.md) project conventions: Rust-only code, no unsafe, no panic paths for hostile inputs, strict dependency layering and WASM builds.

**This is an engineering policy, not a legal opinion.** Have the intended extraction, adapted-documentation distribution and external-binary handling reviewed if publishing packages to third parties.

---

## 14. Source pinning and maintenance

Create a `references/vba-docs.lock.toml`:

```toml
[reference.vba_docs]
repository = "https://github.com/MicrosoftDocs/VBA-Docs"
commit = "REPLACE_WITH_FULL_40_CHAR_GIT_SHA"
docs_license = "CC-BY-4.0"
code_license = "MIT"
checked_on = "2026-10-26"

[generator]
schema_version = 1
version = "0.1.0"
```

**Required practices:**

- Generate reproducibly from a pinned commit, never a moving `main` branch in release builds.
- Keep the generator and reviewed schema versioned together.
- Re-run ingestion against upstream periodically as an *explicit update action*, not an implicit build step.
- Report added, changed and removed members plus affected reviewed overrides.
- Never auto-delete existing implementation metadata solely because an upstream document disappeared or was renamed.
- Trigger targeted compatibility tests after changes to signatures, defaults, events or enums.
- Record the source update in a changelog and compatibility report.

---

## 15. Delivery plan for the reference generator

### Phase R0 — sources and schema

**Deliverables:** source inventory, reviewed legal/provenance policy, schema version 1, source lockfile, parser fixtures.

**Acceptance:** `Excel.Range.Value2`, `Word.Document.Range`, `Excel.Worksheet.Change` and one enumeration all parse with their source references intact.

### Phase R1 — object and member inventory

**Deliverables:** catalogue of Excel, Word and Office objects; member tables; events; unresolved classification report.

**Acceptance:** member lists are discoverable, duplicates reconciled, missing references reported rather than guessed.

### Phase R2 — signatures and enums

**Deliverables:** optional/default arguments, types, directions, enum constants; optional interop evidence importer.

**Acceptance:** known examples produce exact ordered signatures and correct signed integer enum values.

### Phase R3 — Rust code generation

**Deliverables:** generated member registry, constant lookup, documentation index and conformance test scaffolds.

**Acceptance:** deterministic output; `--check` succeeds; all generated API entries are valid but **default to unsupported** until implemented.

### Phase R4 — high-priority Excel host mapping

**Deliverables:** `Application`, `Workbook`, `Worksheet`, `Range`, formatting and worksheet-function member-to-host mappings.

**Acceptance:** core Excel compatibility tests pass through Rust host traits and the real GridCraft engine without an interpreter.

### Phase R5 — Word host mapping and events

**Deliverables:** `Document`, `Range`, `Selection`, paragraphs, styles, and documented event mappings.

**Acceptance:** live range tests, event tests and `.docm` preservation cases pass through the actual WordCraft engine.

### Phase R6 — corpus-driven expansion

**Deliverables:** multi-file conformance corpus, source inventory report, measured API coverage, compatibility report format.

**Acceptance:** all supported members have passing tests, provenance, known limits and host implementation links.

---

## 16. Completion criteria

This reference/generation layer is ready to underpin the Office compatibility implementation when:

- [ ] A pinned MicrosoftDocs/VBA-Docs revision is recorded and reproducibly imported.
- [ ] Excel, Word and shared Office object inventories have been generated.
- [ ] Method/property/event differences are tracked explicitly.
- [ ] Member argument order, optionality, named arguments and return types are represented.
- [ ] Default-member and collection enumeration behaviour can be expressed in metadata.
- [ ] All generated enum integer values and symbolic names are validated.
- [ ] Ambiguous source assertions remain visible in a review queue.
- [ ] Rust dispatch metadata is generated deterministically with stable member identities.
- [ ] Stub members cannot silently report success.
- [ ] Every supported member links to source evidence, a host implementation and tests.
- [ ] Generated code runs without a VBA, Python or JavaScript interpreter dependency.
- [ ] GridCraft and WordCraft host mappings respect each application's engine invariants.
- [ ] Native/WASM build targets remain valid without Microsoft Office installed.
- [ ] Licensing and attribution records are present, including clear treatment of third-party binary metadata.
- [ ] External source updates produce auditable diffs and regression tests.

---

## Appendix A. Quick-start checklist for the next implementation agent

1. Read [`office_automation_architecture.md`](office_automation_architecture.md) before implementing a new API.
2. Clone [MicrosoftDocs/VBA-Docs](https://github.com/MicrosoftDocs/VBA-Docs) into an offline references directory, record the SHA and confirm licence files.
3. Implement the schema and inventory importer; do not hand-write hundreds of methods yet.
4. Import `Excel.Range`, `Excel.Worksheet`, `Word.Document`, `Word.Range`, `XlCalculation` and `WdColorIndex` first.
5. Add the special semantics for `Range` default dispatch, optional Word range arguments and collection enumeration.
6. Generate one Rust member-registry table and one API coverage report.
7. Write one fake-host Rust test and one real GridCraft integration test using the generated metadata.
8. Keep interpreter implementations, craft-vba code and JSON-RPC internal transport **out of scope**.

## Appendix B. Link index

### Microsoft object models and source repository

- <https://github.com/MicrosoftDocs/VBA-Docs>
- <https://github.com/MicrosoftDocs/VBA-Docs/tree/main/api>
- <https://learn.microsoft.com/en-us/office/vba/api/overview/excel/object-model>
- <https://learn.microsoft.com/en-us/office/vba/api/overview/word/object-model>
- <https://learn.microsoft.com/en-us/office/vba/api/excel%28enumerations%29>
- <https://learn.microsoft.com/en-us/office/vba/api/word%28enumerations%29>

### Microsoft interoperability and specifications

- <https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.interop.excel?view=excel-pia>
- <https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.interop.word?view=word-pia>
- <https://learn.microsoft.com/en-us/dotnet/api/microsoft.office.core?view=office-pia>
- <https://learn.microsoft.com/en-us/visualstudio/vsto/office-primary-interop-assemblies?view=visualstudio>
- <https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-ovba/575462ba-bf67-4190-9fac-c275523c75fc>
- <https://learn.microsoft.com/en-us/openspecs/microsoft_general_purpose_programming_languages/ms-vbal/d5418146-0bd2-45eb-9c7a-fd9502722c74>
- <https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/>

### Application and implementation references

- <https://github.com/storytold/gridcraft/blob/main/docs/architecture.md>
- <https://github.com/storytold/gridcraft/blob/main/docs/file-format-parity.md>
- <https://github.com/storytold/wordcraft/blob/main/docs/architecture.md>
- <https://github.com/storytold/wordcraft/blob/main/docs/file-format-parity.md>
- <https://learn.microsoft.com/en-us/office/dev/add-ins/develop/understanding-the-javascript-api-for-office>

---

**Design invariant:** The official reference data defines the public Office-shaped API. The hand-written Rust compatibility layer supplies behaviour. GridCraft and WordCraft supply document operations. Future VBA/Python/JavaScript interpreters supply syntax and language semantics. **Each responsibility must remain independently testable and replaceable.**
