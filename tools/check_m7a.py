#!/usr/bin/env python3
"""Review/hash check for authored M7A acceptance; never executes VBA or sibling code."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise SystemExit(f'M7A INVALID_FIXTURE: {message}')


def runtime_digest():
    paths = [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / 'tools/architecture.json']
    for directory in ['crates', 'apps']:
        paths.extend((ROOT / directory).glob('*/src/**/*.rs'))
        paths.extend((ROOT / directory).glob('*/Cargo.toml'))
    digest = hashlib.sha256()
    for path in sorted(paths, key=lambda item: item.relative_to(ROOT).as_posix()):
        digest.update(path.relative_to(ROOT).as_posix().encode() + b'\0')
        digest.update(hashlib.sha256(path.read_bytes()).digest())
    return digest.hexdigest()


def check(local=False):
    path = ROOT / 'tests/fixtures/m7a/selection.json'
    require(path.stat().st_size <= 128 * 1024, 'selection size limit')
    selection = json.loads(path.read_text(encoding='utf-8'))
    require(selection['schema'] == 1, 'selection schema')
    require(selection['profile'] == 'm7a-common-native-v1', 'acceptance profile')
    require(selection['runtime_profile'] == 'native-forms-common-v1', 'runtime scope')
    require(selection['evidence_basis'] == 'authored-contract', 'evidence basis')
    require(selection['office_observations'] == 'NOT_RUN', 'no fresh Office claim allowed')
    require(selection['deferred'] == ['P9', 'M8.5'], 'deferred boundaries')
    current_tree = runtime_digest()
    if selection['runtime_tree_sha256'] != current_tree:
        # Keep accepted historical receipts immutable. A later milestone records
        # its changed tree explicitly and reruns gates rather than reusing them.
        m8_path = ROOT / 'tests/fixtures/m8/selection.json'
        require(m8_path.is_file(), 'review changed shared runtime/renderer/dependency sources')
        m8 = json.loads(m8_path.read_text(encoding='utf-8'))
        require(m8['m7a_baseline_tree_sha256'] == selection['runtime_tree_sha256'], 'M8 baseline identity')
        require(m8['runtime_tree_sha256'] == current_tree, 'review changed M8 runtime sources')
        print('M7A receipts are historical for the baseline tree; changed M8 implementation requires fresh execution, no retained gate reuse')
    require({s['repository'] for s in selection['specimens']} == {'gridcraft', 'wordcraft'}, 'both native hosts required')
    seen = set()
    count = 0
    for group in [selection['shared'], *selection['specimens']]:
        repo = group['repository']
        require(repo in {'tiny-vba', 'gridcraft', 'wordcraft'}, 'unreviewed repository')
        require(group['id'] not in seen, 'duplicate selection')
        seen.add(group['id'])
        require(group['license'] == 'MIT OR Apache-2.0', 'unreviewed license')
        require(group['execution_review'] == 'original-authored-bounded-host-workflow', 'execution review')
        require(group['initial_execution_status'] == 'NOT_RUN', 'selection integrity is not execution')
        require(group['requirements'] and group['expected'], 'independent expectations required')
        if local and repo != 'tiny-vba':
            continue
        base = ROOT if repo == 'tiny-vba' else ROOT.parent / repo
        require(base.is_dir(), f'missing optional sibling: {repo}')
        if repo != 'tiny-vba':
            for source in (base / 'crates').glob('*/src/**/*.rs'):
                require(not any(token in source.read_text(encoding='utf-8') for token in ['.tvform', 'TINY-VBA-FORM', 'form_archive']), f'retired format in host product: {repo}/{source.relative_to(base)}')

        for artifact in group['artifacts']:
            file = (base / artifact['path']).resolve()
            require(file.is_relative_to(base.resolve()) and file.is_file(), 'missing or escaped artifact')
            require(file.stat().st_size <= 2 * 1024 * 1024, 'artifact size limit')
            data = file.read_bytes()
            require(hashlib.sha256(data).hexdigest() == artifact['sha256'], f'review changed artifact: {repo}/{artifact["path"]}')
            for entry in artifact.get('entrypoints', []):
                require(f'fn {entry}(' in data.decode(), f'missing reviewed entrypoint: {entry}')
            count += 1
    execution = ROOT / 'tests/fixtures/m7a/execution.json'
    if execution.exists():
        record = json.loads(execution.read_text(encoding='utf-8'))
        require(record['schema'] == 1 and record['profile'] == selection['profile'], 'execution profile')
        require(record['selection_sha256'] == hashlib.sha256(path.read_bytes()).hexdigest(), 'execution belongs to another selection')
        require(record['office_observations'] == 'NOT_RUN', 'execution must distinguish Office evidence')
        require(record['runtime_tree_sha256'] == selection['runtime_tree_sha256'], 'retained receipts belong to another shared implementation')
    print(f'M7A selection: {count} artifact hashes verified; VALID_FIXTURE; execution NOT_RUN by this check' + ('; local-only, sibling artifacts not checked' if local else ''))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--local', action='store_true', help='validate only shared artifacts; ordinary CI needs no siblings')
    check(parser.parse_args().local)
