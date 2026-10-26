#!/usr/bin/env python3
"""Verify the reviewed M4 oracle data; parse literals, never execute upstream code."""
import ast
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'tests/fixtures/m4-oracle'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def fields(path):
    header, body = path.read_text().split('---\n', 1)
    result = {}
    for line in header.splitlines():
        key, value = line.split(' = ', 1)
        if key in result:
            raise ValueError(f'duplicate evidence field: {path}')
        result[key] = value
    return result, body


def check(require_checkouts=False):
    selection = json.loads((DIRECTORY / 'selection.json').read_text())
    expected_bytes = (DIRECTORY / 'selected-expected.json').read_bytes()
    if sha(expected_bytes) != selection['selected_expected_sha256']:
        raise ValueError('selected oracle expected data changed without review')
    expected = json.loads(expected_bytes)
    ids, keys = set(), set()
    sources = ROOT / 'tests/corpora/reflex-grid'
    available = (sources / 'tests/vba_excel_cases.py').is_file()
    if require_checkouts and not available:
        raise ValueError('ReflexGrid checkout required')
    source_data = {}
    if available:
        for name, digest in selection['files'].items():
            path = (sources / name).resolve()
            if not path.is_relative_to(sources.resolve()) or sha(path.read_bytes()) != digest:
                raise ValueError(f'upstream evidence bytes changed: {name}')
        # Only data literals in three reviewed fixture tables are consumed.
        tree = ast.parse((sources / 'tests/vba_excel_cases.py').read_text())
        for node in tree.body:
            if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name) and node.target.id in {'EXPRESSIONS', 'FUNCTIONS'}:
                source_data[node.target.id] = ast.literal_eval(node.value)
            if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'HELPERS' for t in node.targets):
                source_data['HELPERS'] = ast.literal_eval(node.value)
        source_expected = json.loads((sources / 'tests/vba_excel_expected.json').read_text())
        report = json.loads((sources / 'docs/excel-oracle.json').read_text())['checks']['vba-language']
        for key, value in selection['oracle'].items():
            if key != 'locale' and report[key] != value:
                raise ValueError(f'upstream oracle environment differs: {key}')
    for case in selection['selection']:
        ident, key = case['id'], case['case_key']
        if ident in ids or key in keys:
            raise ValueError('duplicate selected identity')
        ids.add(ident)
        keys.add(key)
        fixture, body = fields(DIRECTORY / f'{ident}.vbas')
        if sha(body.encode()) != case['vba_sha256'] or fixture['case-key'] != key:
            raise ValueError(f'selected VBA changed: {ident}')
        if fixture['id'] != ident or fixture['oracle-result'] != expected[key]:
            raise ValueError(f'expected oracle result changed: {ident}')
        if fixture['source-revision'] != selection['source_revision'] or fixture['oracle-run-revision'] != selection['oracle_run_revision']:
            raise ValueError(f'evidence revision differs: {ident}')
        if fixture['source-repository'] != selection['source_repository'] or fixture['source-path'] != 'tests/vba_excel_cases.py':
            raise ValueError(f'evidence source differs: {ident}')
        for header, name in [('source-sha256', 'tests/vba_excel_cases.py'), ('expected-sha256', 'tests/vba_excel_expected.json'), ('report-sha256', 'docs/excel-oracle.json')]:
            if fixture[header] != selection['files'][name]:
                raise ValueError(f'evidence hash differs: {ident}')
        for header, name in [('oracle-excel', 'excel'), ('oracle-os', 'os'), ('oracle-date', 'date'), ('oracle-locale', 'locale')]:
            if fixture[header] != selection['oracle'][name]:
                raise ValueError(f'evidence environment differs: {ident}')
        if available:
            if source_expected[key] != expected[key]:
                raise ValueError(f'published result differs: {ident}')
            entry = 'CaseValue' if case['shape'] == 'expression' else key[:-2]
            if case['shape'] == 'expression':
                if key not in source_data['EXPRESSIONS']:
                    raise ValueError('expression not in upstream fixture table')
                core = f'CaseValue = {key}'
            else:
                core = dict(source_data['FUNCTIONS'])[entry]
            # This is the upstream oracle's error probe, preserving its observable Err state.
            probe = f'Function OracleProbe() As Variant\nOn Error Resume Next\nErr.Clear\nDim v As Variant\nv = {entry}()\nIf Err.Number <> 0 Then\nDim failure As Long\nfailure = Err.Number\nOn Error GoTo 0\nErr.Raise failure\nEnd If\nOracleProbe = v\nEnd Function\n'
            actual = probe + f'\nFunction {entry}() As Variant\n{core}\nEnd Function\n'
            for helper in case['helpers']:
                match = re.search(r'(?im)^(?:Sub|Function) ' + re.escape(helper) + r'\([^\n]*\n.*?^End (?:Sub|Function)', source_data['HELPERS'], re.S | re.M)
                if not match:
                    raise ValueError('unknown helper')
                actual += '\n' + match.group(0) + '\n'
            if actual != body:
                raise ValueError(f'fixture is not the reviewed source adaptation: {ident}')
    bundle=(ROOT / 'apps/vba-cli/examples/support/m4_oracle_fixtures.rs').read_text()
    bundled=re.findall(r'include_str!\(\s*"[^"]*/(rfx-m4-[0-9]{3})\.vbas"\s*\)',bundle)
    if len(bundled)!=153 or set(bundled)!=ids:
        raise ValueError('native/WASI oracle fixture bundle differs')
    if len(ids) != 153 or keys != set(expected) or {p.stem for p in DIRECTORY.glob('*.vbas')} != ids:
        raise ValueError('oracle selection inventory differs')
    if sha((DIRECTORY / 'LICENSE.reflex-grid.txt').read_bytes()) != selection['files']['LICENSE']:
        raise ValueError('upstream MIT notice differs')
    print(f'M4 evidence: {len(ids)} reviewed fixtures; upstream byte verification {"PASS" if available else "unavailable (bundled evidence checked)"}; no upstream code executed')


if __name__ == '__main__':
    check()
