"""Run the permanent M8 completion regressions; no temporary tests or downloaded inputs."""
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[1]

def run():
    failed = False
    for package in ['vba-ide', 'vba-ui-egui']:
        result = subprocess.run(['cargo', 'test', '-p', package, '--locked', 'm8_', '--', '--nocapture'], cwd=ROOT)
        failed |= result.returncode != 0
    print('M8_COMPLETION_AUDIT: ' + ('FAIL' if failed else 'PASS'))
    return int(failed)

if __name__ == '__main__':
    raise SystemExit(run())
