# External corpus candidates

Six upstream repositories are cloned as Git submodules; Microsoft VBA-Docs is a seventh reference submodule.
Exact revisions and upstream notice hashes: [source lock](../../references/sources.lock.json).
Candidate files and SHA-256/size metadata: [inventory.json](inventory.json).
Whole candidate files remain NOT_RUN. The reviewed M4 exception below executes only selected
pure-language cases, with separate case-level results; other specimens remain pending review.
No downloaded code or macro is automatically executed, linked or copied into interpreter source.

| Checkout | Intended material | Current restriction |
|---|---|---|
| reflex-grid | tests/vba_excel_cases.py, vba_excel_expected.json, vba_object_cases.py, vba_object_expected.json; Mac oracle method | MIT top-level; use data/methodology only; separate Mac profile, bundled third-party rights |
| vba-json | JsonConverter.bas, specs/Specs.bas, specs/VBA-JSON - Specs.xlsm | MIT top-level; identify dictionary/OS dependencies |
| vba-project-starter | src/DemoStudy.xlsm, tests/tests_DemoStudy.xlsm and modules | MIT top-level; cross-workbook dependencies; advanced integration |
| vba-samples | Macro-enabled teaching workbooks | No top-level licence found; pending rights review, IO/forms classification |
| excel-vba-projects | Project-Files/*.xlsm | No top-level licence found; pending rights and external capability review |
| tree-sitter-vba | examples/*.bas/.cls and test/corpus data | MIT top-level; third-party rights independently reviewed; generated C is not a compiler dependency |
| ../../references/VBA-Docs | VBA language and Excel/Word API reference/examples | CC BY 4.0 docs / MIT samples; not the local Office member catalogue |

```sh
git submodule update --init --depth 1
python3 tools/check_repository.py corpora --require-checkouts
```

Normal `cargo test --workspace` works without any checkout. The corpus CI job checks pins/hashes only.
Before adopting a specimen, establish redistribution rights, classify filesystem/network/shell/COM/forms/
external-workbook needs, select a target profile, and author expected observations. A future allowlisted
scenario harness may execute selected cases in the bounded interpreter, never automatically in Office.
No arbitrary upstream test scripts are invoked by repository checks.

## Documented material without a cloned test repository

- Contextures (<https://www.contextures.com/excelfiles.html>) is a download site, not a Git repository.
  No blanket sample download is performed. Select permitted individual specimens, record the original URL,
  retrieval date/licence/hash and expected behavior in a separately reviewed external corpus.
- No Word-specific .docm/.dotm corpus is named in the supplied documents. Word API reference is present
  in VBA-Docs; a real Word document corpus remains a future curation task for Wordcraft integration.
- Oxi/oxivba-core, elixcee, scriptandcompile/vb6, RDCore and LibreOffice are comparative implementation/
  architecture leads, not independent VBA workbook corpora. They are not cloned or adopted as source.
  ReflexGrid is included specifically for its documented case and oracle files.
- ovba is a possible decoder dependency, not required test data. Evaluate its provenance/API separately;
  the design documents cite two different repository owners.

Submodule acquisition is complete for the documented Git-backed corpus sources. Curating executable,
rights-reviewed scenarios and measured oracle goldens remains M0/M9 work, not an accomplished test result.

M1 exception to the pending syntax selection: five directly authored tree-sitter-vba example files
are reviewed for bounded parser ingestion only. See [review and results](../../docs/m1-corpus-report.md).
Their execution remains NOT_RUN; no third-party or realworld subtree selection is included.

## M4 executable language selection

[ADR 0012](../../docs/decisions/0012-m4-oracle-acceptance.md) reviews 153 directly authored MIT
ReflexGrid language-data cases and their published Microsoft Excel Mac expectations. The
[allowlist](../fixtures/m4-oracle/selection.json) pins exact source/expected/report hashes, case
keys, helpers, adaptation hashes and the upstream oracle environment. The selected source/expected
files match the recorded oracle run revision byte-for-byte. Their raw results are redistributed
with the full MIT notice. No upstream implementation, Python oracle script or Office macro runs.

The bounded tiny-vba VM executes the selected source snippets only, using no host capabilities.
Normal Rust tests need no submodule checkout. CI also checks evidence bytes/adaptations when
available. [Results and provenance limits](../../docs/m4-oracle-report.md) distinguish inherited
upstream-reported observations from local native/WASI comparisons. This is the reviewed policy
exception for M4 acceptance; it does not authorize arbitrary corpus execution or whole-file PASS.
