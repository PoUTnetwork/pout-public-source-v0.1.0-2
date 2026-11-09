"""A public test of evidence consistency, not a production-security test."""
import unittest
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[2]))
from research.post_nr05 import verify_ne02_s1_record as checker

class TestNE02S1RecordedEvidence(unittest.TestCase):
    def test_recorded_result(self):
        self.assertEqual(checker.verify()['status'],'PASS')
    def test_evidence_scope_is_explicit(self):
        result=checker.verify()
        self.assertFalse(result['native_tests_rerun'])
        self.assertFalse(result['production_consensus_validated'])

if __name__=='__main__':
    unittest.main()
