# 0103 — CI portability and offline oracle source

Date: 2026-11-13. Status: adopted for the CI repair.

Native CI runs all three supported OS jobs independently. Linux uses the tested
Ubuntu 24.04 image; native/UI tests run serially to avoid concurrent graphics-driver
creation/teardown crashes seen on the moving Ubuntu 26.04 image. Linux installs software
Vulkan/EGL before running the workspace tests; rendered tests are not skipped.
CFB paths emitted by the external library are normalized to `/` before becoming
resource/storage identities. CFB names cannot contain backslashes, so the mapping
is lossless. Import/export, embedded resources and storage metadata share it.
Runtime source digests also use `/`, preserving the existing Linux hash algorithm
while making identical Windows checkouts comparable. Maintenance readers explicitly
use UTF-8 rather than the host locale, including Windows CP1252 environments.

The ReflexGrid upstream is unavailable publicly. Replace its submodule with only
the four MIT oracle-data/notice files already pinned by the M4 and M8 selections.
Retain exact bytes, source revision, original URL and license. Verify every file
hash and reject missing/extra files; parse the selected literals/JSON without
executing upstream Python. Unretained historical specimens remain explicitly
unavailable/NOT_RUN. Other corpus checkouts retain strict pin/license/hash checks.
No interpreter implementation or unreviewed workbook is redistributed.

Historical M8 selection/execution bytes remain unchanged under
`tests/fixtures/m8/baseline-2026-11-08/`. The current selection identifies the
changed tree and maintenance inputs. New results cover only actually rerun gates;
remote platform, host and fresh Office acceptance are never inherited from the
historical receipts. An explicitly NOT_RUN local candidate cannot satisfy release
readiness, even if an Office comparison bundle is supplied.
