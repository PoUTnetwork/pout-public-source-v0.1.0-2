#!/usr/bin/env python3
"""Run exact public references, unit/mutation tests and synthetic-byte checks.

No internet, credentials, Cargo build or proof generation. Python 3.10+.
Exit 0 requires every declared fixture and all required tests to pass.
"""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
import platform
import sys
import unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]


def require(ok, message):
    if not ok:raise ValueError(message)


def load(name, path):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module


def read_json(path, maximum=131072):
    require(path.is_file() and not path.is_symlink(),f'missing regular file: {path.name}')
    require(path.stat().st_size<=maximum,'JSON exceeds fixed bound')
    def unique(items):
        out={}
        for key,value in items:
            require(key not in out,'duplicate JSON key');out[key]=value
        return out
    with path.open(encoding='utf-8') as f:return json.load(f,object_pairs_hook=unique)


def verify_service(reference):
    root=ROOT/'tests/fixtures/service-observation'
    metadata=read_json(root/'manifest.json')
    require(metadata['schema']==1 and metadata['unpredictable_source'] is False,'fixture scope')
    require(metadata['source_bytes']==reference.SOURCE_BYTES and metadata['chunk_bytes']==reference.CHUNK_BYTES,'fixture sizing')
    require(metadata['object_id']==reference.OBJECT_ID.hex(),'fixture object')
    require(metadata['chunk_hashes']==[v.hex() for v in reference.HASHES],'chunk known answers')
    vectors=read_json(root/'request_vectors.json')['rows']
    require(len(vectors)==128 and [v['id'] for v in vectors]==list(range(128)),'request vector inventory')
    for vector in vectors:
        kwargs={key:vector[key] for key in ('label','interval','sample','index')}
        kwargs['nonce']=bytes.fromhex(vector['nonce'])
        require(reference.request_id(**kwargs).hex()==vector['request_id'],'request digest')
    case_data=read_json(root/'cases.json')
    require(case_data['schema']==1 and case_data['origin']=='generated-public-reference-not-captured-network-traffic','case provenance')
    cases=case_data['cases']
    expected={'valid-a':'Verified','valid-b':'Verified','cached-new-nonce':'Verified','unavailable':'Unavailable',
      'corrupt-payload':'Integrity','wrong-magic':'Framing','invalid-status':'Framing','truncated':'Framing',
      'extra-byte':'Framing','wrong-length':'Framing','wrong-nonce':'RequestContext','wrong-object':'ObjectPolicy',
      'unavailable-with-payload':'Framing','empty':'Framing','header-only':'Framing','zero-payload':'Integrity'}
    require(len(cases)==len(expected) and {r['case'] for r in cases}==set(expected),'frame case set')
    files={'cases.json','manifest.json','request_vectors.json'}
    for case in cases:
        name=case['file'];require(name==case['case']+'.frame' and '/' not in name and '\\' not in name,'frame path')
        p=root/name;require(p.is_file() and not p.is_symlink() and p.stat().st_size<=reference.FRAME_BYTES+1,'frame bound')
        data=p.read_bytes()
        require(len(data)==case['bytes'] and hashlib.sha256(data).hexdigest()==case['sha256'],'frame hash')
        kwargs=case['expected_request'];require(set(kwargs)<= {'label','interval','sample','index','nonce','object_id'},'request keys')
        kwargs={k:bytes.fromhex(v) if k in ('nonce','object_id') else v for k,v in kwargs.items()}
        try:result=reference.verify(data,**kwargs)
        except ValueError as e:result=str(e)
        require(result==expected[case['case']]==case['expected_result'],'frame result')
        files.add(name)
    require({p.name for p in root.iterdir()}==files,'unexpected or missing frame fixture')
    return {'chunk_known_answers':32,'request_vectors':128,'frame_cases':16,
            'verified_frames':3,'unavailable_frames':1,'expected_rejections':12}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--json',action='store_true',help='print a compact result record after tests')
    args=parser.parse_args()
    ref=load('observation_pack_reference',ROOT/'scripts/check_observation.py')
    module=load('finite_pack_tests',ROOT/'tests/public/test_finite_models.py')
    checker=load('finite_pack_verifier',ROOT/'research/finite-models/verify.py')
    suite=unittest.TestSuite((unittest.defaultTestLoader.loadTestsFromModule(module),
                              unittest.defaultTestLoader.loadTestsFromTestCase(ref.ReferenceTests)))
    result=unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():return 1
    finite=checker.verify_fixtures(ROOT/'tests/fixtures/finite-models')
    service=verify_service(ref)
    report={'schema':1,'status':'PASS','scope':'public-python-reference-and-fixtures',
            'python':platform.python_version(),'optimized':not __debug__,
            'unit_tests':result.testsRun,'unit_failures':len(result.failures),'unit_errors':len(result.errors),
            **finite,**service,'consensus_eligibility':'NotEstablished','consensus_weight':None,
            'rust_workspace_executed':False,'hosted_ci_executed':False,'network_service_executed':False}
    if args.json:print(json.dumps(report,indent=2,sort_keys=True))
    else:
        print(f"PASS: {result.testsRun} Python unit tests; 65,536 exact encoder rows; 16 synthetic frame cases.")
        print('Finite model evidence only. No Rust, network, physical-resource or consensus validation is implied.')
    return 0

if __name__=='__main__':
    try:raise SystemExit(main())
    except (ValueError,KeyError,TypeError,OSError) as exc:
        print(f'Public pack check FAILED: {exc}',file=sys.stderr);raise SystemExit(1)
