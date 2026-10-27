"""Maintenance-only UI provenance/cache/notice and dependency-pin checks. Never downloads evidence."""
import hashlib
import json
from pathlib import Path
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]

def check():
    sources = json.loads((ROOT / 'docs/provenance/vbe_sources.yaml').read_text())
    for source in sources['sources']:
        if not source['url'].startswith('https://learn.microsoft.com/'):
            raise ValueError('unexpected UI reference publisher')
        cache = (ROOT / source['cache']).resolve()
        if not cache.is_relative_to((ROOT / '.cache/evidence').resolve()):
            raise ValueError('reference cache escaped ignored evidence root')
        if cache.exists() and hashlib.sha256(cache.read_bytes()).hexdigest() != source['sha256']:
            raise ValueError('local UI evidence hash differs from provenance record')
    manifest = json.loads((ROOT / 'docs/provenance/ui-dependencies.json').read_text())
    for notice in manifest['notices']:
        if hashlib.sha256((ROOT / notice['path']).read_bytes()).hexdigest() != notice['sha256']:
            raise ValueError(f'font notice changed: {notice["path"]}')
    tracked = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard'], cwd=ROOT, text=True).splitlines()
    if any(path.startswith('.cache/') for path in tracked):
        raise ValueError('downloaded reference evidence must never enter Git')
    lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    for package in lock['package']:
        name = package['name']
        if name in {'egui', 'eframe', 'egui-wgpu', 'egui-winit', 'egui_glow', 'egui_kittest', 'epaint', 'ecolor', 'emath', 'epaint_default_fonts'}:
            if not package.get('source', '').endswith('#' + manifest['egui_revision']):
                raise ValueError(f'unreviewed egui family source: {name}')
        if name == 'winit' and not package.get('source', '').endswith('#' + manifest['winit_revision']):
            raise ValueError('native event bridge pin differs from UI contract')
    print(f'UI provenance: {len(sources["sources"])} public references, ignored evidence cache, font notices and reviewed GUI pins verified')

if __name__ == '__main__':
    check()
