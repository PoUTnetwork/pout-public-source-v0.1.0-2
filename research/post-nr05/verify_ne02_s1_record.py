#!/usr/bin/env python3
"""Verify recorded NE02 S1 values; not a native Rust execution or proof."""
from __future__ import annotations
import hashlib
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / 'docs/test-evidence-ne01-s1/ne02-s1/NE02_S1_MARGIN_RELEASE.json'
CHECKS = ROOT / 'docs/test-evidence-ne01-s1/ne02-s1/NE02_S1_CHECKS_PUBLIC.json'
PIN = '24e64820735281622da4355863f476a578654867c113b119518705df033113c3'


def verify() -> dict:
    raw = EVIDENCE.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == PIN, 'Recorded release bytes changed'
    result = json.loads(raw)
    checks = json.loads(CHECKS.read_bytes())
    assert checks['result_sha256'] == PIN
    assert checks['source_result_blob_sha1'] == '3699d4cea8d679cf2c29aef54bff1e87648caf4c'
    assert result['schema'] == 'NE02_S1_MARGIN_BASELINE_V1'
    assert result['verdict'] == 'NE01_SEAL_NO_STORAGE_TIME_MARGIN'
    assert result['build'] == 'release' and result['real_components'] is True
    contract = result['contract']
    assert contract['id'] == 'f13b440bdce264b0e2377daa86600e4855ed4eb0534f3e21586303b4cc2eb609'
    assert contract['audit_window_seconds'] == 5
    assert contract['hardware_speedups'] == [1,10,100]
    assert contract['gap_tolerance_permille'] == 50
    assert contract['deficits_permille'] == [500,250,100]
    assert contract['acceptance_epochs'] == 8 and contract['security_bits'] == 20
    expected=[]
    for deficit in contract['deficits_permille']:
        available = 1.0 - (deficit - contract['gap_tolerance_permille']) / 1000.0
        minimum = math.ceil(contract['security_bits'] * math.log(2) /
                            (contract['acceptance_epochs'] * -math.log(available)))
        assert available ** (minimum * contract['acceptance_epochs']) <= 2 ** -20
        assert available ** ((minimum - 1) * contract['acceptance_epochs']) > 2 ** -20
        expected.append(minimum)
    assert expected == [3,8,34]
    assert [r['samples_per_epoch'] for r in contract['required_samples']] == expected
    setup = result['setup']
    assert setup['chunks'] * setup['chunk_bytes'] * setup['replicas'] == setup['claimed_bytes'] == 131072
    runs = result['live_runs']
    assert {r['strategy'] for r in runs} == {'Honest','FinalsParallel','Spaced/8','ResealSerial'}
    assert all(r['public_verifier']=='accepted' and r['within_window_on_this_host'] for r in runs)
    assert max(r['seconds'] for r in runs) < contract['audit_window_seconds']
    shared = result['single_holder_intake_run']
    assert shared['epochs']==8 and shared['replicas_held']==2
    assert shared['ciphertext_copies_kept']==1 and shared['label_state_bytes_kept']==2048
    assert shared['slowest_answer_seconds'] < 0.05
    assert shared['consensus_eligibility']=='Established' and shared['consensus_weight']==512
    assert shared['scope'].startswith('NE01 test model')
    assert [x['hardware_speedup'] for x in result['scenarios']]==[1,10,100]
    assert all(x['margin_met'] is False and x['max_gap_permille']==500 for x in result['scenarios'])
    assert all(len(x['strategies'])==4 for x in result['scenarios'])
    assert all(v==0 for v in checks['recorded_check_exits'].values())
    assert checks['workspace_test']['exit_code'] != 0 and checks['workspace_test']['failure_count'] == 13
    assert checks['production_security_established'] is False
    return {'status':'PASS','checks':14,'scope':'recorded data and conditional arithmetic only',
            'native_tests_rerun':False,'production_consensus_validated':False}

if __name__=='__main__':
    print(json.dumps(verify(),indent=2,sort_keys=True))
