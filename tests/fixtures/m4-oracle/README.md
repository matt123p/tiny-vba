# Reviewed ReflexGrid / Microsoft Excel oracle subset

These 153 MIT pure-language cases are data adapted from ReflexGrid revision
`03a338b214df243f1bc68e50512936ddc5566c76`. The exact expressions/function bodies and required
helper bodies are preserved. No upstream interpreter or Python script is imported or executed.
The full upstream MIT notice is in LICENSE.reflex-grid.txt.

selection.json identifies every case, source-file hashes and adaptation hash. selected-expected.json
preserves the raw published TypeName|CStr or E<number> result. Schema 2 fixtures also retain those
results and the oracle environment: Microsoft Excel for Mac 16.114 (build 26101123), macOS 15.3.1,
2026-10-15, run revision 3758c56a39f0c8841fa43eedda0e074414fed3e3. Locale is not recorded per run.
The case/expected files are byte-identical at that run revision and the pinned revision.

This is inherited upstream-reported Microsoft evidence, not a fresh local Excel run. Numeric
expectations originate in published scalar text, not captured Microsoft floating-point bits.
Selected floating values are compared exactly after decoding that published text; no tolerance is
used to turn a mismatch into a pass. Windows-only cases, locale-sensitive functions, non-ASCII
source and the Mac array TypeName quirks are excluded. Unselected cases remain NOT_RUN.

The upstream probe clears Err before each case, evaluates the expression/function under Resume
Next, and reports E<number> if Err remains set. OracleProbe reproduces that behavior before returning
the typed value. This matters for functions that catch their own error but leave Err populated.

`cargo test -p vba-cli --test cli inherited_microsoft_oracle` executes all 153 cases.
`cargo xtask wasm-run` repeats the identical cases under native Rust and WASI without filesystem
access. `python3 tools/check_m4_evidence.py` verifies the bundled selection and, when the checkout
exists, every upstream source hash and source adaptation. Normal Rust tests need neither Office
nor corpus checkouts. See docs/m4-oracle-report.md for acceptance results and limits.
