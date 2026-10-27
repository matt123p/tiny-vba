#!/usr/bin/env python3
"""Offline repository checks. Maintenance tooling only; never imports corpus code."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def contained(base, relative):
    path = (base / relative).resolve()
    if not path.is_relative_to(base.resolve()):
        raise ValueError(f'path escapes checkout: {relative}')
    return path


def layers():
    graph = json.loads((ROOT / 'tools/architecture.json').read_text())
    allowed = graph['allowed_normal_dependencies']
    external = graph.get('allowed_external_dependencies', {})
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], cwd=ROOT, text=True))
    actual = {}
    for package in metadata['packages']:
        if package['license'] != 'MIT':
            raise ValueError(f'{package["name"]}: licence differs from repository LICENSE')
        manifest = tomllib.loads(Path(package['manifest_path']).read_text())
        if manifest.get('lints', {}).get('workspace') is not True:
            raise ValueError(f'{package["name"]}: workspace safety lints not inherited')
        dependencies = package['dependencies']
        actual[package['name']] = sorted(d['name'] for d in dependencies if d['kind'] is None)
        for dependency in dependencies:
            if dependency['name'] in external:
                if dependency['source'] is None or dependency['req'] != external[dependency['name']]:
                    raise ValueError(f'unreviewed external dependency version: {dependency["name"]}')
            elif dependency['name'] not in allowed:
                raise ValueError(f'{package["name"]}: unreviewed dependency {dependency["name"]}')
            if dependency['name'] in {'vba-testkit', 'vba-conformance'} and not (
                package['name'] == 'vba-conformance' or
                (package['name'] == 'vba-cli' and dependency['name'] == 'vba-conformance')
            ):
                if dependency['kind'] != 'dev':
                    raise ValueError('test support leaked into production dependencies')
    if actual != allowed:
        raise ValueError('crate dependency graph changed; update docs/architecture.md and reviewed tools/architecture.json')
    visiting, done = set(), set()

    def visit(name):
        if name in visiting:
            raise ValueError(f'dependency cycle at {name}')
        if name in done:
            return
        visiting.add(name)
        for dependency in actual[name]:
            if dependency in allowed:
                visit(dependency)
        visiting.remove(name)
        done.add(name)

    for name in actual:
        visit(name)
    # UI/platform dependencies may occur only above the toolkit-independent model.
    for root in ['vba-vm', 'vba-debug', 'vba-compiler', 'vba-ide', 'vba-project']:
        pending, seen = [root], set()
        while pending:
            name = pending.pop()
            if name in seen:
                continue
            seen.add(name)
            if name in {'egui', 'eframe', 'vba-ui-egui', 'vba-ide-app', 'image', 'web-sys'}:
                raise ValueError(f'{root}: UI/platform dependency crossed the headless boundary')
            pending.extend(actual.get(name, []))
    from check_ui_provenance import check
    check()
    print(f'layers: {len(actual)} crates, acyclic; engine/IDE model remain headless; UI above model')


def corpora(require_checkouts):
    lock = json.loads((ROOT / 'references/sources.lock.json').read_text())
    inventory = json.loads((ROOT / 'tests/corpora/inventory.json').read_text())
    if lock['schema_version'] != 1 or inventory['schema_version'] != 1:
        raise ValueError('unsupported source lock/inventory version')
    sources, available = {}, set()
    config = subprocess.check_output(
        ['git', 'config', '-f', '.gitmodules', '--get-regexp', r'^submodule\..*\.(path|url)$'],
        cwd=ROOT, text=True)
    module_entries = dict(line.split(' ', 1) for line in config.splitlines())
    for source in lock['sources']:
        identity, path, revision = source['id'], source['path'], source['commit']
        if identity in sources or not re.fullmatch(r'[0-9a-f]{40}', revision):
            raise ValueError(f'invalid/duplicate source pin: {identity}')
        sources[identity] = source
        checkout = contained(ROOT, path)
        prefix = f'submodule.{path}'
        if module_entries.get(prefix + '.path') != path or module_entries.get(prefix + '.url') != source['repository']:
            raise ValueError(f'submodule URL/path mismatch: {path}')
        index = git('ls-files', '--stage', '--', path).split()
        if len(index) != 4 or index[0] != '160000' or index[1] != revision:
            raise ValueError(f'Git submodule pin disagrees with source lock: {path}')
        if not (checkout / '.git').exists():
            if require_checkouts:
                raise ValueError(f'missing submodule: {path}; run git submodule update --init --depth 1')
            print(f'corpora: checkout absent, pin checked only: {path}')
            continue
        if git('-C', str(checkout), 'rev-parse', 'HEAD') != revision:
            raise ValueError(f'checkout revision mismatch: {path}')
        if git('-C', str(checkout), 'status', '--porcelain', '--untracked-files=no'):
            raise ValueError(f'upstream checkout has tracked edits: {path}')
        for notice in source['license_files']:
            data = contained(checkout, notice['path']).read_bytes()
            if hashlib.sha256(data).hexdigest() != notice['sha256']:
                raise ValueError(f'licence notice mismatch: {path}/{notice["path"]}')
        available.add(identity)
    count, seen = 0, set()
    for specimen in inventory['specimens']:
        identity = specimen['source']
        source = sources[identity]
        key = (identity, specimen['path'])
        if key in seen or not re.fullmatch(r'[0-9a-f]{64}', specimen['sha256']):
            raise ValueError(f'invalid/duplicate specimen: {key}')
        seen.add(key)
        fixture = contained(ROOT / source['path'], specimen['path'])
        if identity in available:
            data = fixture.read_bytes()
            if len(data) != specimen['size_bytes'] or hashlib.sha256(data).hexdigest() != specimen['sha256']:
                raise ValueError(f'fixture integrity mismatch: {fixture}')
            count += 1
    print(f'corpora: {len(sources)} pins; {count}/{len(seen)} candidate file hashes verified; no macros executed')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('check', nargs='?', choices=['layers', 'corpora', 'all'], default='all')
    parser.add_argument('--require-checkouts', action='store_true')
    args = parser.parse_args()
    if args.check in {'layers', 'all'}:
        layers()
    if args.check in {'corpora', 'all'}:
        corpora(args.require_checkouts)
        from check_m4_evidence import check
        check(args.require_checkouts)


if __name__ == '__main__':
    main()
