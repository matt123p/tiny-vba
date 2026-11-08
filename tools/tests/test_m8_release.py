"""Synthetic validator tests only: these are never Office observations."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import check_m8_release as gate


class ReleaseEvidence(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.manifest = gate.check_preparation()
        self.bundle = gate.read_json(gate.DIRECTORY / 'results-template.json')
        self.bundle.update(local_selection_sha256='a' * 64, runtime_tree_sha256='b' * 64,
                           reference_platform='windows', review={
                               'reviewer': 'synthetic unit test',
                               'reviewed_at': '2026-11-08T12:00:00Z',
                               'method': 'permitted-black-box-observation',
                               'notes': 'Synthetic protocol test. Never release evidence.'})
        for case, item in zip(self.manifest['cases'], self.bundle['results']):
            for producer in ['office', 'fork']:
                observations = {section: [] if section == 'event_order' else {}
                                for section in gate.SECTIONS}
                for section, labels in case['required_observations'].items():
                    if section == 'event_order':
                        observations[section] = ['first', 'second']
                    else:
                        observations[section] = {label: {'type': 'Long', 'value': 1} for label in labels}
                record = dict(schema=1, case_id=case['id'],
                              evidence_basis='fresh-office' if producer == 'office' else 'authored-fork',
                              environment=dict(product=case['product' if producer == 'office' else 'fork'],
                                               version='synthetic', build='synthetic', platform='windows',
                                               os='synthetic', locale='en-US', observed_at='2026-11-08T11:00:00Z',
                                               runtime_tree_sha256=None if producer == 'office' else 'b' * 64),
                              source=dict(recipe_sha256=gate.recipe_digest(case), input_sha256='c' * 64,
                                          module_source_sha256='d' * 64), status='PASS', observations=observations)
                path = self.base / (case['id'] + '-' + producer + '.json')
                path.write_text(json.dumps(record))
                item[producer] = dict(path=path.name, sha256=gate.digest(path))

    def result(self):
        path = self.base / 'bundle.json'
        path.write_text(json.dumps(self.bundle))
        return gate.compare_bundle(path, self.manifest, 'a' * 64, 'b' * 64)

    def mutate(self, change, index=0, producer='office'):
        ref = self.bundle['results'][index][producer]
        path = self.base / ref['path']
        raw = json.loads(path.read_text())
        change(raw)
        path.write_text(json.dumps(raw))
        ref['sha256'] = gate.digest(path)

    def test_complete_synthetic_protocol(self):
        self.assertEqual(self.result()['status'], 'PASS')

    def test_missing_observation_and_review_never_pass(self):
        self.bundle['review'] = None
        self.assertEqual(self.result()['status'], 'NOT_RUN')
        self.bundle['results'][0]['office'] = None
        self.assertEqual(self.result()['office_comparisons'][self.manifest['cases'][0]['id']], 'NOT_RUN')
        self.bundle['reference_platform'] = None
        for item in self.bundle['results']:
            item.update(office=None, fork=None)
        self.assertEqual(self.result()['status'], 'NOT_RUN')

    def test_types_and_event_order_must_match(self):
        self.mutate(lambda raw: raw['observations']['errors'].update({'Err.Number': {'type': 'Integer', 'value': 1}}))
        self.assertEqual(self.result()['status'], 'MISMATCH')
        index = next(i for i, case in enumerate(self.manifest['cases']) if case['category'] == 'events')
        self.mutate(lambda raw: raw['observations'].update(event_order=['second', 'first']), index)
        self.assertEqual(self.result()['office_comparisons'][self.manifest['cases'][index]['id']], 'MISMATCH')

    def test_raw_failure_cannot_be_hidden(self):
        for status in ['TIMEOUT', 'CRASH', 'UNSUPPORTED', 'BLOCKED_BY_POLICY', 'INVALID_FIXTURE']:
            self.mutate(lambda raw: raw.update(status=status))
            self.assertEqual(self.result()['status'], status)
        self.mutate(lambda raw: raw.update(status='MISMATCH'))
        with self.assertRaises(gate.InvalidFixture): self.result()

    def test_reject_stale_candidate_duplicate_and_wrong_inputs(self):
        original = copy.deepcopy(self.bundle)
        self.bundle['runtime_tree_sha256'] = '0' * 64
        with self.assertRaises(gate.InvalidFixture): self.result()
        self.bundle = copy.deepcopy(original)
        self.bundle['results'][1] = self.bundle['results'][0]
        with self.assertRaises(gate.InvalidFixture): self.result()
        self.bundle = copy.deepcopy(original)
        self.bundle.update(reference_platform=None, review=None)
        self.bundle['results'][0] = None
        with self.assertRaises(gate.InvalidFixture): self.result()
        self.bundle = original
        self.mutate(lambda raw: raw['source'].update(input_sha256='0' * 64))
        with self.assertRaises(gate.InvalidFixture): self.result()

    def test_reject_inherited_or_unnamed_or_unreviewed_provenance(self):
        for change in [lambda raw: raw.update(evidence_basis='inherited-oracle'),
                       lambda raw: raw['environment'].update(build=''),
                       lambda raw: raw['environment'].update(platform='linux')]:
            with self.subTest(change=change):
                ref = self.bundle['results'][0]['office']
                before = (self.base / ref['path']).read_text()
                self.mutate(change)
                with self.assertRaises(gate.InvalidFixture): self.result()
                (self.base / ref['path']).write_text(before)
                ref['sha256'] = gate.digest(self.base / ref['path'])
        self.bundle['review']['reviewed_at'] = '2026-11-08T10:00:00Z'
        with self.assertRaises(gate.InvalidFixture): self.result()

    def test_reject_missing_required_observations(self):
        self.mutate(lambda raw: raw['observations'].update(errors={}))
        with self.assertRaises(gate.InvalidFixture): self.result()

    def test_reject_bad_hash_traversal_and_escaping_symlink(self):
        ref = self.bundle['results'][0]['office']
        original = dict(ref)
        ref['sha256'] = '0' * 64
        with self.assertRaises(gate.InvalidFixture): self.result()
        for path in ['../outside.json', '/tmp/outside.json', 'C:\\outside.json']:
            ref.update(original, path=path)
            with self.assertRaises(gate.InvalidFixture): self.result()
        (self.base / 'loop.json').symlink_to('loop.json')
        ref.update(original, path='loop.json')
        with self.assertRaises((gate.InvalidFixture, OSError)): self.result()
        with tempfile.TemporaryDirectory() as outside:
            target = Path(outside) / 'outside.json'
            target.write_text('{}')
            (self.base / 'escape.json').symlink_to(target)
            ref.update(path='escape.json', sha256=gate.digest(target))
            with self.assertRaises(gate.InvalidFixture): self.result()

    def test_reject_ambiguous_json_and_lossy_floats(self):
        path = self.base / 'invalid.json'
        for data in ['{"x":1,"x":2}', '{"x":1.0}', '{"x":NaN}', '{"x":Infinity}']:
            path.write_text(data)
            with self.assertRaises(gate.InvalidFixture): gate.read_json(path)
        path.write_text(' ' * 1025)
        with self.assertRaises(gate.InvalidFixture): gate.read_json(path, limit=1024)

    def test_exact_value_boundaries(self):
        valid = [{'type': 'String', 'utf16': [0, 0xd800, 0xffff]},
                 {'type': 'Currency', 'scaled': '-9223372036854775808'},
                 {'type': 'Double', 'bits': '8000000000000000'},
                 {'type': 'Array', 'element_type': 'Long', 'bounds': [[-1, 0]],
                  'values': [{'type': 'Long', 'value': 1}, {'type': 'Long', 'value': 2}]}]
        for value in valid: gate.typed_value(value)
        invalid = [{'type': 'Long', 'value': True}, {'type': 'Integer', 'value': 32768},
                   {'type': 'Double', 'bits': '7ff8000000000000'},
                   {'type': 'String', 'utf16': [65536]},
                   {'type': 'Currency', 'scaled': '9223372036854775808'},
                   dict(valid[-1], bounds=[[0, 0]]),
                   dict(valid[-1], element_type='Integer')]
        for value in invalid:
            with self.subTest(value=value), self.assertRaises(gate.InvalidFixture): gate.typed_value(value)


if __name__ == '__main__':
    unittest.main()
