#!/usr/bin/env python3
"""Optional sibling integration-fixture integrity check; never executes VBA or upstream code."""
import hashlib
import json
from pathlib import Path


def require(condition, message):
    if not condition:
        raise SystemExit(f"M7 INVALID_FIXTURE: {message}")


root = Path(__file__).resolve().parents[1]
manifest = json.loads((root / "tests/fixtures/m7/selection.json").read_text(encoding='utf-8'))
require(manifest["schema"] == 1, "unsupported selection schema")
require(manifest["profile"] == "m7-unchanged-v1", "unsupported profile")
require(manifest["evidence_basis"] == "authored-contract", "unsupported evidence basis")
require(manifest["office_observations"] == "NOT_RUN", "selection must not claim Office execution")
seen = set()
files = 0
for specimen in manifest["specimens"]:
    require(specimen["id"] not in seen, "duplicate specimen")
    seen.add(specimen["id"])
    require(specimen["license"] == "MIT OR Apache-2.0", "unreviewed license")
    require(specimen["execution_review"] == "original-authored-bounded-host-workflow", "unreviewed execution")
    require(specimen["initial_execution_status"] == "NOT_RUN", "integrity is not execution")
    require(specimen["requirements"] and specimen["expected"], "missing capabilities or expectations")
    require(specimen["repository"] in {"gridcraft", "wordcraft"}, "unreviewed repository")
    repo = root.parent / specimen["repository"]
    for artifact in specimen["artifacts"]:
        path = (repo / artifact["path"]).resolve()
        require(path.is_relative_to(repo.resolve()), "artifact escaped its repository")
        require(path.stat().st_size <= 1024 * 1024, "artifact exceeds reviewed size limit")
        data = path.read_bytes()
        require(hashlib.sha256(data).hexdigest() == artifact["sha256"], f"review changed artifact: {path}")
        files += 1
require(len(seen) == 2, "expected exactly the two reviewed specimens")
execution_path = root / "tests/fixtures/m7/execution.json"
if execution_path.exists():
    execution = json.loads(execution_path.read_text(encoding='utf-8'))
    require(execution["schema"] == 1 and execution["profile"] == manifest["profile"], "execution record profile mismatch")
    selection_hash = hashlib.sha256((root / "tests/fixtures/m7/selection.json").read_bytes()).hexdigest()
    require(execution["selection_sha256"] == selection_hash, "execution record belongs to a different reviewed selection")
print(f"M7 selection: {len(seen)} authored specimens, {files} artifact hashes verified; VALID_FIXTURE; execution NOT_RUN by this check")
