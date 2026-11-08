# M8.5 comparison capture and release readiness

The [eleven-case inventory](../tests/fixtures/m8-office/selection.json) and
[empty template](../tests/fixtures/m8-office/results-template.json) are original
MIT OR Apache-2.0 preparation under ADR 0102. Every Office observation is NOT_RUN.
There is no installed Office environment or fresh comparison bundle. M8 remains
open; the commands below make the remaining gate reviewable without inventing data.

```sh
python3 tools/check_m8_release.py --validate
# Optional: verify recipe bytes and entrypoints in sibling host checkouts too.
python3 tools/check_m8_release.py --validate --siblings
python3 tools/check_m8_release.py --init /tmp/m8-office-results.json
python3 tools/check_m8_release.py
# Later, after capture and provenance review:
python3 tools/check_m8_release.py --bundle /path/to/results.json --report /path/to/readiness.json
```

`--validate` returns 0 for valid preparation and explicitly says execution NOT_RUN.
`--init` writes current selection/runtime identities and null observations, refusing
overwrite. Set `reference_platform` to `windows` or `macos` when an actual environment
is selected. Readiness returns 0 only for PASS, 1 for an unfulfilled or failed gate,
and 2 for INVALID_FIXTURE. Output files also refuse overwrite. Without a bundle,
readiness reports local PASS and eleven NOT_RUN comparisons; that exit 1 is expected.
The ordinary `cargo xtask m8` command still runs the local profile only.

## Capture protocol

1. Choose named Excel and Word builds on one reference platform. Record product,
   version, full build, OS, locale and timezone-bearing observation time. Retain
   independently reviewable run notes/screenshots as appropriate outside the repo.
2. Verify the pinned test/build source bytes against the reviewed M7A inventory.
   Use the recipe's Rust builders and original authored module sources to recreate
   the same native input for Office and its fork. Do not use upstream macros,
   proprietary Office binaries or inherited oracle outputs. Review the authored
   VBA before explicitly invoking it. Capture the actual input and module-source
   hashes; identical hashes in two records require identical bytes, not equivalent
   projects created separately. Hash a deterministic length-delimited module stream
   in sorted module-name order, including names, source bytes and component kinds;
   document its construction in review notes. Preserve the input for review.
3. Follow each case's entrypoints and steps. Record all required labels with actual
   types, values/errors, host effects and ordered callbacks. Host-state observations
   should use tagged arrays/strings with the same coordinate/story labeling in both
   runs. Record `Err.Number` as the observed VBA numeric type. Do not substitute
   Empty, inferred expected values or a test's assertion text for measurements.
4. Compare semantic native source/resource preservation and host effects. Whole
   output ZIP hashes, timestamps, fork trust dialogs, fork undo mechanics and
   signature-removal policies are not prescribed Office equivalence assertions.
   Keep those local acceptance checks in the original host suites. If a prescribed
   semantic operation is unsupported, record UNSUPPORTED; do not skip it to get PASS.
5. Write raw result files and their SHA-256 references beside the result bundle.
   Review actual provenance and applicability before adding the review declaration.
   Preserve failures and raw data. A changed candidate/input requires fresh evidence;
   never rewrite old receipts to describe a new run.

## Packet format

Each non-null `office`/`fork` reference is `{ "path": "relative.json", "sha256":
"<64 lowercase hex digits>" }`. Files must resolve inside the bundle directory,
including symlinks, and be at most 2 MiB. Duplicate JSON keys are invalid.

A raw record has exactly these keys:

```json
{
  "schema": 1,
  "case_id": "gridcraft-context",
  "evidence_basis": "fresh-office",
  "environment": {
    "product": "Microsoft Excel",
    "version": "<actual version>",
    "build": "<actual build>",
    "platform": "windows",
    "os": "<actual OS>",
    "locale": "<actual locale>",
    "observed_at": "<ISO 8601 time with timezone>",
    "runtime_tree_sha256": null
  },
  "source": {
    "recipe_sha256": "<SHA-256 of canonical sorted compact JSON case>",
    "input_sha256": "<SHA-256 of actual unchanged input bytes>",
    "module_source_sha256": "<SHA-256 of documented module source stream>"
  },
  "status": "PASS",
  "observations": {
    "values": {}, "errors": {}, "effects": {}, "attributes": {},
    "package": {}, "event_order": []
  }
}
```

This example is a schema illustration with placeholders, not valid evidence.
`recipe_digest` in the validator implements the canonical recipe hash, including
ASCII JSON escaping. Fork records
use `authored-fork`, the case's fork product and the current runtime tree hash;
their platform can be Linux. All source identities must agree across the pair.

Observation maps contain labeled tagged values. Empty/Null/Missing/Nothing have
only `type`. Boolean/Byte/Integer/Long/CVErr use `value` with exact JSON boolean or
integer payload. LongLong uses a signed canonical `decimal` string; Currency a
signed 64-bit `scaled` decimal string in 1/10,000 units. Decimal uses `negative`,
`coefficient` and `scale` (0–28), normalized by removing trailing decimal zeros.
Single/Double/Date use lowercase `bits` in IEEE big-endian hex notation (8/16
digits); non-finite values are invalid. String uses a `utf16` code-unit array,
including embedded NULs and lone surrogates. Array uses `element_type`, ordered
`bounds` pairs and flat tagged `values`, first dimension varying fastest; preserve
lower bounds. Record object identity through Boolean `Is` results. These are bounded
report encodings, not a runtime serialization ABI.

`event_order` is a string list. All other sections are maps, including errors and
package observations. A completed raw run reports PASS; the comparator computes
PASS or MISMATCH itself. Raw UNSUPPORTED, BLOCKED_BY_POLICY, INVALID_FIXTURE, CRASH
or TIMEOUT propagate. An unrun producer has a null reference. Every selected case
must remain in the bundle even when unrun.

The final `review` object requires `reviewer`, `reviewed_at`,
`method: "permitted-black-box-observation"` and substantive `notes` identifying
the provenance checks and capture procedure. Review time must follow all runs.
Matching raw files without review remain NOT_RUN. Structural validation and a
self-declared review cannot independently prove that Office was used; the reviewer
owns that evidence check. Temporary synthetic packets in the ten maintenance unit
tests establish validator behavior only. No synthetic Office results are shipped.
