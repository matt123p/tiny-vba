#!/usr/bin/env python3
"""Optional current-checkout host tests. Never changes shipping pins; restores lock bytes."""
import os
from pathlib import Path
import subprocess
import tempfile
from check_m8 import check, ROOT


def run():
    check(siblings=True)
    names = ['vba-compiler', 'vba-diagnostics', 'vba-syntax', 'vba-vm', 'vba-ir', 'vba-foreign',
             'vba-ide', 'vba-project', 'vba-values', 'vba-runtime', 'vba-debug']
    with tempfile.TemporaryDirectory(prefix='tiny-vba-m8-') as directory:
        config = Path(directory) / 'candidate.toml'
        config.write_text('[patch."https://github.com/matt123p/tiny-vba.git"]\n' + ''.join(
            f'{name} = {{ path = "{(ROOT / "crates" / name).as_posix()}" }}\n' for name in names))
        for repository, package, suites in [
            ('gridcraft', 'gridcraft-vba', ['m8_workbooks', 'native_forms', 'package_acceptance', 'form_controls']),
            ('wordcraft', 'wordcraft-vba', ['native_forms', 'package_acceptance', 'word_workflows']),
        ]:
            root = ROOT.parent / repository
            lock = root / 'Cargo.lock'
            original = lock.read_bytes()
            try:
                args = ['cargo', 'test', '--config', str(config), '-p', package]
                for suite in suites:
                    args.extend(['--test', suite])
                print(f'M8 candidate host {repository}: {args}', flush=True)
                subprocess.run(args, cwd=root, check=True)
                wasm = args + ['--target', 'wasm32-unknown-unknown']
                env = dict(os.environ, CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER='wasm-bindgen-test-runner')
                print(f'M8 candidate WASM host {repository}: {wasm}', flush=True)
                subprocess.run(wasm, cwd=root, env=env, check=True)
            finally:
                lock.write_bytes(original)
                print(f'M8 {repository}: original pinned lockfile restored', flush=True)
    check(siblings=True)


if __name__ == '__main__':
    run()
