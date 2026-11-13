#!/usr/bin/env python3
"""M8 selection integrity only; never execute upstream scripts or infer Office results."""
import hashlib
import json
from pathlib import Path
from check_m7a import runtime_digest
from check_m4_evidence import check as oracle_check

ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise ValueError(f'M8 INVALID_FIXTURE: {message}')


def check(siblings=False):
    path = ROOT / 'tests/fixtures/m8/selection.json'
    require(path.stat().st_size <= 128 * 1024, 'selection size')
    selection = json.loads(path.read_text(encoding='utf-8'))
    require(selection['schema'] == 1 and selection['profile'] == 'm8-local-v1', 'profile')
    require(selection['office_observations'] == 'NOT_RUN', 'fresh Office evidence unavailable')
    require(selection['runtime_tree_sha256'] == runtime_digest(), 'review changed runtime tree')
    baseline = json.loads((ROOT / 'tests/fixtures/m7a/selection.json').read_text(encoding='utf-8'))
    require(selection['m7a_baseline_tree_sha256'] == baseline['runtime_tree_sha256'], 'M7A baseline')
    count = 0
    for artifact in selection['artifacts']:
        require(artifact['repository'] in {'tiny-vba', 'gridcraft'}, 'repository')
        if artifact['repository'] != 'tiny-vba' and not siblings:
            continue
        base = ROOT if artifact['repository'] == 'tiny-vba' else ROOT.parent / artifact['repository']
        file = (base / artifact['path']).resolve()
        require(file.is_relative_to(base.resolve()) and file.is_file(), 'missing/escaped artifact')
        require(file.stat().st_size <= 2 * 1024 * 1024, 'artifact size')
        require(hashlib.sha256(file.read_bytes()).hexdigest() == artifact['sha256'], f'changed artifact: {file}')
        count += 1
    oracle_check(directory=ROOT/'tests/fixtures/m8-oracle', count=24, prefix='rfx-m8', bundle_path='apps/vba-cli/examples/support/m8_oracle_fixtures.rs')
    previous = json.loads((ROOT / 'tests/fixtures/m4-oracle/selection.json').read_text(encoding='utf-8'))
    additional = json.loads((ROOT / 'tests/fixtures/m8-oracle/selection.json').read_text(encoding='utf-8'))
    require({case['case_key'] for case in previous['selection']}.isdisjoint(
        case['case_key'] for case in additional['selection']), 'M8 library cases must be additional to M4')
    if siblings:
        base = ROOT.parent / 'gridcraft'
        workbooks = json.loads((base / 'tests/fixtures/m8/selection.json').read_text(encoding='utf-8'))
        require(workbooks['profile'] == 'm8-authored-workbooks-v1', 'workbook profile')
        require(workbooks['license'] == 'MIT OR Apache-2.0' and workbooks['office_observations'] == 'NOT_RUN', 'workbook provenance')
        require(len(workbooks['cases']) == len({case['id'] for case in workbooks['cases']}) == 20, '20 independent workflow identities')
        for case in workbooks['cases']:
            require(case['execution_status'] == 'NOT_RUN' and case['review'] and case['expected'], 'selection is not execution')
            file = (base / case['path']).resolve()
            require(file.is_relative_to(base.resolve()) and file.stat().st_size <= 65536, 'bounded workbook source')
            require(hashlib.sha256(file.read_bytes()).hexdigest() == case['sha256'], 'changed workbook source')
    execution = path.with_name('execution.json')
    if execution.exists():
        record = json.loads(execution.read_text(encoding='utf-8'))
        require(record['selection_sha256'] == hashlib.sha256(path.read_bytes()).hexdigest(), 'execution selection identity')
        require(record['runtime_tree_sha256'] == selection['runtime_tree_sha256'], 'execution runtime identity')
        require(record['office_observations'] == 'NOT_RUN', 'no fresh Office claim')
    print(f'M8 selection: {count} artifacts VALID_FIXTURE; execution NOT_RUN by integrity check; siblings {"checked" if siblings else "not checked"}')


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--siblings', action='store_true')
    check(parser.parse_args().siblings)
