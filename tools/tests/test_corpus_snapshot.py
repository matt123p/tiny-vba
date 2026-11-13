"""Snapshot integrity checks; never execute the retained upstream source."""
import hashlib
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_repository import oracle_snapshot


class OracleSnapshot(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.notice = b'MIT notice\n'
        (self.base / 'LICENSE').write_bytes(self.notice)
        (self.base / 'cases.py').write_bytes(b'raise RuntimeError("never execute corpus")\n')
        self.source = {
            'snapshot_files': {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                               for p in self.base.iterdir()},
            'license_files': [{'path': 'LICENSE', 'sha256': hashlib.sha256(self.notice).hexdigest()}],
        }

    def test_verifies_all_files_without_executing_python(self):
        self.assertEqual(oracle_snapshot(self.source, self.base), {'LICENSE', 'cases.py'})

    def test_changed_or_missing_source_is_rejected(self):
        (self.base / 'cases.py').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'hash mismatch'):
            oracle_snapshot(self.source, self.base)
        (self.base / 'cases.py').unlink()
        with self.assertRaisesRegex(ValueError, 'inventory differs'):
            oracle_snapshot(self.source, self.base)

    def test_unreviewed_extra_file_is_rejected(self):
        (self.base / 'extra.py').write_bytes(b'')
        with self.assertRaisesRegex(ValueError, 'inventory differs'):
            oracle_snapshot(self.source, self.base)

    def test_notice_must_match_reviewed_license(self):
        self.source['license_files'][0]['sha256'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'notice differs'):
            oracle_snapshot(self.source, self.base)


if __name__ == '__main__':
    unittest.main()
