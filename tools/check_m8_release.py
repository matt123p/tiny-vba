#!/usr/bin/env python3
"""Validate M8.5 observation packets and compute release readiness. Never runs macros."""
import argparse
from datetime import datetime
import hashlib
import json
import math
from pathlib import Path
import re
import struct

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'tests/fixtures/m8-office'
SECTIONS = {'values', 'errors', 'effects', 'attributes', 'package', 'event_order'}
STATUSES = {'PASS', 'MISMATCH', 'UNSUPPORTED', 'BLOCKED_BY_POLICY',
            'INVALID_FIXTURE', 'CRASH', 'TIMEOUT', 'NOT_RUN'}
LOCAL_GATES = {
    'M7A baseline re-verification', 'shared native CI', 'browser WASM compilation',
    'full actual WASI', 'M8 local gate', 'final M8 WASI gate',
    'editor/forms UI and snapshots', 'native captures', 'production browser bundle',
    'current shared candidate in both hosts', 'GridCraft full shipping-pin CI',
    'Grid new workbook suite strict Clippy', 'final VM fuzz strict Clippy',
    'final gate orchestration strict Clippy', 'frozen importer fuzz strict Clippy',
    'asserted release benchmarks', 'Chromium original workflows',
}


class InvalidFixture(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise InvalidFixture(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def hex_digest(value):
    return isinstance(value, str) and re.fullmatch(r'[0-9a-f]{64}', value) is not None


def text(value, limit=512):
    return isinstance(value, str) and bool(value.strip()) and len(value) <= limit


def fields(value, expected, label):
    require(isinstance(value, dict) and set(value) == set(expected), f'{label}: fields')


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f'duplicate JSON key: {key}')
        result[key] = value
    return result


def reject_number(value):
    raise InvalidFixture(f'non-integer JSON number {value}; encode float bits explicitly')


def read_json(path, limit=256 * 1024, expected_sha256=None):
    path = Path(path)
    require(path.is_file() and path.stat().st_size <= limit, f'missing/oversized JSON: {path.name}')
    try:
        with path.open('rb') as source:
            data = source.read(limit + 1)
        require(len(data) <= limit, f'oversized JSON: {path.name}')
        if expected_sha256 is not None:
            require(hashlib.sha256(data).hexdigest() == expected_sha256, 'raw result hash mismatch')
        return json.loads(data.decode('utf-8'), object_pairs_hook=unique_pairs,
                          parse_float=reject_number, parse_constant=reject_number)
    except (UnicodeError, json.JSONDecodeError, RecursionError) as error:
        raise InvalidFixture(f'invalid JSON: {path.name}') from error


def timestamp(value):
    require(text(value, 64), 'timestamp required')
    try:
        parsed = datetime.fromisoformat(value.replace('Z', '+00:00'))
    except ValueError as error:
        raise InvalidFixture('invalid timestamp') from error
    require(parsed.utcoffset() is not None, 'timestamp must include timezone')
    return parsed


def typed_value(value, depth=0):
    require(depth <= 8 and isinstance(value, dict), 'typed value nesting/type')
    kind = value.get('type')
    if kind in {'Empty', 'Null', 'Missing', 'Nothing'}:
        fields(value, {'type'}, kind)
    elif kind == 'Boolean':
        fields(value, {'type', 'value'}, kind)
        require(type(value['value']) is bool, 'Boolean payload')
    elif kind in {'Byte', 'Integer', 'Long', 'CVErr'}:
        fields(value, {'type', 'value'}, kind)
        low, high = {'Byte': (0, 255), 'Integer': (-32768, 32767),
                     'Long': (-2147483648, 2147483647), 'CVErr': (0, 65535)}[kind]
        require(type(value['value']) is int and low <= value['value'] <= high, f'{kind} range')
    elif kind in {'LongLong', 'Currency'}:
        key = 'scaled' if kind == 'Currency' else 'decimal'
        fields(value, {'type', key}, kind)
        number = value[key]
        require(isinstance(number, str) and re.fullmatch(r'0|-?[1-9][0-9]{0,18}', number), f'{kind} decimal encoding')
        require(-(1 << 63) <= int(number) < (1 << 63), f'{kind} range')
    elif kind == 'Decimal':
        fields(value, {'type', 'negative', 'coefficient', 'scale'}, kind)
        coefficient, scale = value['coefficient'], value['scale']
        require(type(value['negative']) is bool and type(scale) is int and 0 <= scale <= 28, 'Decimal sign/scale')
        require(isinstance(coefficient, str) and re.fullmatch(r'0|[1-9][0-9]{0,28}', coefficient), 'Decimal coefficient')
        require(int(coefficient) <= (1 << 96) - 1, 'Decimal magnitude')
        require((coefficient != '0' or (scale == 0 and not value['negative']))
                and (scale == 0 or not coefficient.endswith('0')), 'Decimal must use canonical exact numeric representation')
    elif kind in {'Single', 'Double', 'Date'}:
        fields(value, {'type', 'bits'}, kind)
        bits = value['bits']
        width = 8 if kind == 'Single' else 16
        require(isinstance(bits, str) and re.fullmatch(r'[0-9a-f]{' + str(width) + '}', bits), 'IEEE bit encoding')
        number = struct.unpack('>f' if kind == 'Single' else '>d', bytes.fromhex(bits))[0]
        require(math.isfinite(number), 'non-finite value')
        if kind == 'Date':
            require(-657434 <= number < 2958466, 'Date range')
    elif kind == 'String':
        fields(value, {'type', 'utf16'}, kind)
        units = value['utf16']
        require(isinstance(units, list) and len(units) <= 32768, 'String bound')
        require(all(type(unit) is int and 0 <= unit <= 65535 for unit in units), 'UTF-16 code units')
    elif kind == 'Array':
        fields(value, {'type', 'element_type', 'bounds', 'values'}, kind)
        bounds, values = value['bounds'], value['values']
        scalar = {'Variant', 'Boolean', 'Byte', 'Integer', 'Long', 'LongLong',
                  'Single', 'Double', 'Currency', 'Date', 'String'}
        require(value['element_type'] in scalar, 'array element type')
        require(isinstance(bounds, list) and len(bounds) <= 60 and isinstance(values, list), 'array shape')
        count = 1 if bounds else 0
        for pair in bounds:
            require(isinstance(pair, list) and len(pair) == 2
                    and all(type(n) is int and -(1 << 31) <= n < (1 << 31) for n in pair), 'array bounds')
            low, high = pair
            require(high >= low - 1, 'array reversed bounds')
            count *= high - low + 1
            require(count <= 65536, 'array cell budget')
        require(len(values) == count, 'array cell count')
        for cell in values:
            typed_value(cell, depth + 1)
            require(value['element_type'] == 'Variant' or cell['type'] == value['element_type'], 'array element tag')
    else:
        raise InvalidFixture('unknown value tag; record object identity through Boolean Is observations')


def recipe_digest(case):
    return hashlib.sha256(json.dumps(case, sort_keys=True, separators=(',', ':'),
                                     ensure_ascii=True).encode()).hexdigest()


def check_preparation(directory=DIRECTORY, root=ROOT, siblings=False):
    manifest = read_json(directory / 'selection.json')
    fields(manifest, {'schema', 'profile', 'license', 'evidence_basis', 'baseline_selection_sha256', 'cases'}, 'comparison selection')
    require(manifest['schema'] == 1 and manifest['profile'] == 'm8-office-v1', 'comparison profile')
    require(manifest['license'] == 'MIT OR Apache-2.0'
            and manifest['evidence_basis'] == 'authored-protocol-not-observation', 'selection provenance')
    baseline_path = root / 'tests/fixtures/m7a/selection.json'
    require(manifest['baseline_selection_sha256'] == digest(baseline_path), 'M7A selection identity')
    baseline = read_json(baseline_path)
    approved = {(group['repository'], item['path']): item['sha256']
                for group in [baseline['shared'], *baseline['specimens']]
                for item in group['artifacts']}
    require(isinstance(manifest['cases'], list) and len(manifest['cases']) == 11, 'eleven required comparisons')
    seen, coverage = set(), set()
    for case in manifest['cases']:
        fields(case, {'id', 'product', 'fork', 'category', 'entrypoints', 'sources',
                      'steps', 'required_observations', 'initial_status'}, 'comparison case')
        require(text(case['id'], 96) and case['id'] not in seen, 'duplicate/missing comparison identity')
        seen.add(case['id'])
        require((case['product'], case['fork']) in {('Microsoft Excel', 'GridCraft'), ('Microsoft Word', 'WordCraft')}, 'product/fork pair')
        coverage.add((case['product'], case['category']))
        require(case['initial_status'] == 'NOT_RUN', 'selection is not execution')
        require(isinstance(case['steps'], list) and 1 <= len(case['steps']) <= 20
                and all(text(step, 2048) for step in case['steps']), 'comparison steps')
        require(isinstance(case['entrypoints'], list) and case['entrypoints']
                and all(text(entry, 256) for entry in case['entrypoints']), 'reviewed test entrypoints')
        require(isinstance(case['sources'], list) and 1 <= len(case['sources']) <= 20, 'source selection')
        for source in case['sources']:
            fields(source, {'repository', 'path', 'sha256'}, 'source pin')
            require(approved.get((source['repository'], source['path'])) == source['sha256'], 'unreviewed or changed source recipe')
            if siblings:
                base = root.parent / source['repository']
                path = (base / source['path']).resolve()
                require(path.is_relative_to(base.resolve()) and path.is_file()
                        and digest(path) == source['sha256'], 'changed/missing sibling source')
        if siblings:
            sources = '\n'.join((root.parent / source['repository'] / source['path']).read_text(encoding='utf-8')
                                for source in case['sources'])
            require(all('fn ' + entry + '(' in sources for entry in case['entrypoints']), 'missing recipe entrypoint')
        required = case['required_observations']
        require(isinstance(required, dict) and required and set(required) <= SECTIONS, 'required observation sections')
        for section, labels in required.items():
            require(isinstance(labels, list) and labels and all(text(label, 256) for label in labels)
                    and len(set(labels)) == len(labels), 'required observation labels')
            if section == 'event_order':
                require(labels == ['ordered_callbacks'], 'event sequence requirement')
    required_categories = {'context', 'events', 'attributes', 'package', 'forms'}
    require(coverage == {(product, category) for product in ['Microsoft Excel', 'Microsoft Word']
                         for category in required_categories} | {('Microsoft Excel', 'worksheet-controls')}, 'comparison category coverage')
    template = read_json(directory / 'results-template.json')
    fields(template, {'schema', 'profile', 'selection_sha256', 'local_selection_sha256',
                      'runtime_tree_sha256', 'reference_platform', 'review', 'results'}, 'results template')
    require(template['schema'] == 1 and template['profile'] == manifest['profile']
            and template['selection_sha256'] == digest(directory / 'selection.json'), 'template selection identity')
    require(template['local_selection_sha256'] is None and template['runtime_tree_sha256'] is None
            and template['reference_platform'] is None and template['review'] is None, 'blank template provenance')
    require(isinstance(template['results'], list) and {item['case_id'] for item in template['results']} == seen
            and len(template['results']) == len(seen), 'template inventory')
    for item in template['results']:
        fields(item, {'case_id', 'office', 'fork'}, 'template case')
        require(item['office'] is None and item['fork'] is None, 'template must contain no observations')
    return manifest


def raw_record(reference, base, case, producer, platform, runtime_tree):
    fields(reference, {'path', 'sha256'}, 'raw result reference')
    name = reference['path']
    require(text(name, 512) and '\\' not in name and ':' not in name, 'raw result path')
    relative = Path(name)
    require(not relative.is_absolute() and '..' not in relative.parts and relative.suffix == '.json', 'raw result must be contained JSON')
    try:
        path = (base / relative).resolve()
    except RuntimeError as error:
        raise InvalidFixture('raw result symlink loop') from error
    require(path.is_relative_to(base.resolve()), 'raw result escapes bundle')
    require(hex_digest(reference['sha256']) and path.is_file()
            and path.stat().st_size <= 2 * 1024 * 1024, 'raw result bound/hash')
    raw = read_json(path, 2 * 1024 * 1024, reference['sha256'])
    fields(raw, {'schema', 'case_id', 'evidence_basis', 'environment', 'source', 'status', 'observations'}, 'raw result')
    require(raw['schema'] == 1 and raw['case_id'] == case['id'], 'raw result identity')
    require(raw['evidence_basis'] == ('fresh-office' if producer == 'office' else 'authored-fork'), 'fresh versus authored evidence basis')
    require(raw['status'] in STATUSES - {'MISMATCH', 'NOT_RUN'}, 'raw execution status')
    env = raw['environment']
    fields(env, {'product', 'version', 'build', 'platform', 'os', 'locale', 'observed_at', 'runtime_tree_sha256'}, 'environment')
    require(env['product'] == case['product' if producer == 'office' else 'fork'], 'wrong producing product')
    require(all(text(env[key]) for key in ['version', 'build', 'os', 'locale']), 'named build/OS/locale required')
    require(env['platform'] in {'windows', 'macos', 'linux'}, 'execution platform')
    timestamp(env['observed_at'])
    if producer == 'office':
        require(env['platform'] == platform and env['runtime_tree_sha256'] is None, 'reference platform or evidence mismatch')
    else:
        require(env['runtime_tree_sha256'] == runtime_tree, 'fork ran another shared implementation')
    source = raw['source']
    fields(source, {'recipe_sha256', 'input_sha256', 'module_source_sha256'}, 'observed source')
    require(source['recipe_sha256'] == recipe_digest(case)
            and all(hex_digest(source[key]) for key in ['input_sha256', 'module_source_sha256']), 'exact fixture/source identity required')
    observations = raw['observations']
    fields(observations, SECTIONS, 'observations')
    for section, entries in observations.items():
        if section == 'event_order':
            require(isinstance(entries, list) and len(entries) <= 4096
                    and all(text(event, 512) for event in entries), 'ordered callback trace')
        else:
            require(isinstance(entries, dict) and len(entries) <= 256, 'observation entry budget')
            for label, value in entries.items():
                require(text(label, 256), 'observation label')
                typed_value(value)
    if raw['status'] == 'PASS':
        for section, labels in case['required_observations'].items():
            if section == 'event_order':
                require(bool(observations[section]), 'missing callback trace')
            else:
                require(set(labels) <= set(observations[section]), f'missing {section} observations')
    return raw


def compare_bundle(path, manifest, local_selection_sha256, runtime_tree):
    bundle = read_json(path)
    fields(bundle, {'schema', 'profile', 'selection_sha256', 'local_selection_sha256',
                    'runtime_tree_sha256', 'reference_platform', 'review', 'results'}, 'comparison bundle')
    require(bundle['schema'] == 1 and bundle['profile'] == manifest['profile'], 'bundle profile')
    require(bundle['selection_sha256'] == digest(DIRECTORY / 'selection.json'), 'bundle comparison selection')
    require(bundle['local_selection_sha256'] == local_selection_sha256
            and bundle['runtime_tree_sha256'] == runtime_tree, 'bundle belongs to another local candidate')
    require(bundle['reference_platform'] in {'windows', 'macos', None}, 'one named Office platform per bundle')
    cases = {case['id']: case for case in manifest['cases']}
    require(isinstance(bundle['results'], list) and len(bundle['results']) == len(cases), 'complete comparison inventory')
    require(all(isinstance(item, dict) for item in bundle['results']), 'comparison result objects')
    if bundle['reference_platform'] is None:
        require(bundle['review'] is None
                and all(item.get('office') is None and item.get('fork') is None
                        for item in bundle['results']), 'observed bundle requires a named Office platform')
    results, observed_dates = {}, []
    for item in bundle['results']:
        fields(item, {'case_id', 'office', 'fork'}, 'comparison result')
        ident = item['case_id']
        require(ident in cases and ident not in results, 'missing/duplicate comparison case')
        records = {}
        for producer in ['office', 'fork']:
            if item[producer] is not None:
                records[producer] = raw_record(item[producer], Path(path).parent, cases[ident],
                                               producer, bundle['reference_platform'], runtime_tree)
                observed_dates.append(timestamp(records[producer]['environment']['observed_at']))
        if len(records) != 2:
            results[ident] = 'NOT_RUN'
        elif any(record['status'] != 'PASS' for record in records.values()):
            results[ident] = next(record['status'] for record in records.values() if record['status'] != 'PASS')
        else:
            require(records['office']['source'] == records['fork']['source'], 'different input/source compared')
            results[ident] = 'PASS' if records['office']['observations'] == records['fork']['observations'] else 'MISMATCH'
    review = bundle['review']
    if review is not None:
        fields(review, {'reviewer', 'reviewed_at', 'method', 'notes'}, 'provenance review')
        require(text(review['reviewer']) and text(review['notes'], 4096)
                and review['method'] == 'permitted-black-box-observation', 'reviewed provenance required')
        reviewed_at = timestamp(review['reviewed_at'])
        require(all(reviewed_at >= observed for observed in observed_dates), 'review predates observations')
    failed = next((status for status in results.values() if status not in {'PASS', 'NOT_RUN'}), None)
    status = failed or ('PASS' if all(value == 'PASS' for value in results.values()) and review is not None else 'NOT_RUN')
    return {'profile': 'm8-release-v1', 'status': status, 'office_comparisons': results,
            'review_present': review is not None, 'comparison_bundle_sha256': digest(path)}


def release_report(bundle=None):
    from check_m8 import check
    from check_m7a import runtime_digest
    check()
    manifest = check_preparation()
    local_path = ROOT / 'tests/fixtures/m8/selection.json'
    # Historical local receipts contain fractional durations, not VBA values.
    local = json.loads((ROOT / 'tests/fixtures/m8/execution.json').read_text(encoding='utf-8'),
                       object_pairs_hook=unique_pairs, parse_constant=reject_number)
    if local['local_status'] == 'NOT_RUN':
        require(isinstance(local['gates'], list), 'local gate list')
        return {'profile': 'm8-release-v1', 'status': 'NOT_RUN', 'local_status': 'NOT_RUN',
                'office_comparisons': {case['id']: 'NOT_RUN' for case in manifest['cases']},
                'reason': 'Changed local candidate has not completed all release gates; historical receipts are not reused'}
    require(local['local_status'] == 'PASS' and isinstance(local['gates'], list)
            and LOCAL_GATES <= {gate['gate'] for gate in local['gates']}
            and len({gate['gate'] for gate in local['gates']}) == len(local['gates'])
            and all(gate['status'] == 'PASS' and gate['exit_code'] == 0 for gate in local['gates']), 'local gates incomplete')
    if bundle is None:
        return {'profile': 'm8-release-v1', 'status': 'NOT_RUN', 'local_status': 'PASS',
                'office_comparisons': {case['id']: 'NOT_RUN' for case in manifest['cases']},
                'reason': 'Fresh Excel/Word observations and provenance review are unavailable'}
    report = compare_bundle(bundle, manifest, digest(local_path), runtime_digest())
    report['local_status'] = 'PASS'
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--validate', action='store_true', help='validate preparation only; does not accept release')
    parser.add_argument('--siblings', action='store_true', help='also verify pinned recipe bytes and entrypoints in sibling forks')
    parser.add_argument('--init', type=Path, metavar='OUTPUT', help='write a current-candidate NOT_RUN packet; refuses overwrite')
    parser.add_argument('--bundle', type=Path, help='validate reviewed fresh Office/fork raw results')
    parser.add_argument('--report', type=Path, help='write a computed readiness report; refuses overwrite')
    args = parser.parse_args()
    try:
        require(sum([args.validate, args.init is not None, args.bundle is not None]) <= 1, 'choose one input mode')
        manifest = check_preparation(siblings=args.siblings)
        if args.validate:
            print(f'M8.5 preparation: {len(manifest["cases"])} cases VALID_FIXTURE; execution NOT_RUN')
            return 0
        if args.init:
            from check_m8 import check
            from check_m7a import runtime_digest
            check()
            template = read_json(DIRECTORY / 'results-template.json')
            template.update(local_selection_sha256=digest(ROOT / 'tests/fixtures/m8/selection.json'),
                            runtime_tree_sha256=runtime_digest())
            with args.init.open('x', encoding='utf-8') as output:
                json.dump(template, output, indent=2)
                output.write('\n')
            print(f'M8.5 packet created: {len(manifest["cases"])} comparisons NOT_RUN')
            return 0
        report = release_report(args.bundle)
        if args.report:
            with args.report.open('x', encoding='utf-8') as output:
                json.dump(report, output, indent=2)
                output.write('\n')
        print('RESULT ' + report['status'] + ' m8-release-v1')
        print(json.dumps(report, indent=2))
        return 0 if report['status'] == 'PASS' else 2 if report['status'] == 'INVALID_FIXTURE' else 1
    except (InvalidFixture, KeyError, TypeError, OSError, ValueError) as error:
        print(f'RESULT INVALID_FIXTURE m8-release-v1: {error}')
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
