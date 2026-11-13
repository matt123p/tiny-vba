"""Repository source is UTF-8 even when the host's default text encoding is not."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1]


class SourceEncoding(unittest.TestCase):
    def check_source(self, source):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / 'crates/sample/src/lib.rs'
            path.parent.mkdir(parents=True)
            path.write_text(source, encoding='utf-8')
            script = (
                'import sys; from pathlib import Path; '
                'sys.path.insert(0, sys.argv[1]); import check_repository as check; '
                'check.ROOT = Path(sys.argv[2]); check.native_forms()'
            )
            env = dict(os.environ, LC_ALL='C', LANG='C', PYTHONUTF8='0', PYTHONCOERCECLOCALE='0')
            return subprocess.run(
                [sys.executable, '-X', 'utf8=0', '-c', script, str(TOOLS), str(root)],
                env=env, capture_output=True, text=True, encoding='utf-8', check=False,
            )

    def test_utf8_source_is_read_under_a_non_utf8_locale(self):
        result = self.check_source('// Native text: 漢字\n')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('retired custom codec/examples absent', result.stdout)

    def test_utf8_does_not_hide_a_retired_format(self):
        result = self.check_source('// Native text: 漢字 .tvform\n')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('retired custom form format in product source', result.stderr)


if __name__ == '__main__':
    unittest.main()
