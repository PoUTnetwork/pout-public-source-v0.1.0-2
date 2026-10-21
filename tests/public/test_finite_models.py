"""Unit and mutation tests for the finite public reference; standard library only."""
from __future__ import annotations
import copy
import importlib.util
import json
import tempfile
import unittest
from fractions import Fraction
from itertools import product
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
def load(name, path):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module
M=load('finite_public_models',ROOT/'research/finite-models/models.py')
V=load('finite_public_check',ROOT/'research/finite-models/verify.py')
O=load('observation_public_reference',ROOT/'scripts/check_observation.py')

class FiniteModels(unittest.TestCase):
    def test_all_sample_target_encodings(self):
        for x,c in product(range(16),range(4)):
            self.assertEqual(M.target(x,c),((x>>(c%2))&1)+2*((x>>(2+c//2))&1))
    def test_invalid_targets_rejected(self):
        for x,c in ((-1,0),(16,0),(0,-1),(0,4),(True,0),(0,False)):
            with self.assertRaises(ValueError):M.target(x,c)
    def test_no_advice_control(self):
        self.assertEqual(M.encoder_scores(0),(16,32))
        self.assertEqual(M.encoder_scores(65535),(16,32))
    def test_complement_encoders_are_equivalent(self):
        for e in (1,3,5,15,0xaaaa,0x1234,0x8000):
            self.assertEqual(M.encoder_scores(e),M.encoder_scores(e^65535))
    def test_one_coordinate_advice(self):
        encoder=sum(1<<x for x in range(16) if x&1)
        self.assertEqual(M.encoder_scores(encoder),(24,48))
    def test_independent_formula(self):
        for e in (0,1,7,256,0xaaaa,0xcccc,0xabcd,65535):
            self.assertEqual(M.encoder_scores(e)[1],M.fourier_q1(e))
            self.assertEqual(M.encoder_scores(e)[0],V.posterior_q0(e))
            self.assertEqual(M.fourier_q1(e),V.character_q1(e))
    def test_encoder_bounds(self):
        for e in (-1,65536,True,0.5,'1'):
            with self.assertRaises(ValueError):M.encoder_scores(e)
    def test_single_response_table(self):
        self.assertEqual(sum(M.response_table_score(0,x) for x in range(16)),16)
        self.assertEqual(M.response_table_score(255,15),4)
    def test_repeated_first_and_second_round(self):
        self.assertEqual(M.fixed_book_repetition(1),Fraction(7,16))
        self.assertEqual(M.fixed_book_repetition(2),Fraction(17,64))
    def test_repetition_is_not_iid(self):
        for t in range(2,13):
            self.assertGreater(M.fixed_book_repetition(t),Fraction(7,16)**t)
    def test_repetition_formula(self):
        for t in range(1,13):self.assertEqual(M.fixed_book_repetition(t),M.fixed_book_formula(t))
    def test_repetition_bounds(self):
        for t in (0,13,-1,True):
            with self.assertRaises(ValueError):M.fixed_book_repetition(t)
        for book in ((),(0,0),(0,1,2)):
            with self.assertRaises(ValueError):M.fixed_book_repetition(1,book)
    def test_guessing_probability(self):
        self.assertEqual(M.guessing([1,1,1,1]),Fraction(1,4))
        self.assertEqual(M.guessing([0,0,4,0]),Fraction(1))
    def test_guessing_rejects_missing_distribution(self):
        for row in ([],[0,0],[-1,2],[True,1]):
            with self.assertRaises(ValueError):M.guessing(row)
    def test_posterior_top_mass(self):
        self.assertEqual(M.top_mass([3,2,1],2),Fraction(5,6))
        self.assertEqual(M.top_mass([3,2,1],0),Fraction(0))
        with self.assertRaises(ValueError):M.top_mass([1,1],3)
    def test_mask_does_not_reduce_marginal_guessing(self):
        worlds=[(u,u^r) for u,r in product(range(2),repeat=2)]
        self.assertEqual(M.conditional_guessing(worlds),Fraction(1,2))
    def test_helper_reveals_masked_source(self):
        worlds=[(u,(u^r,r)) for u,r in product(range(2),repeat=2)]
        self.assertEqual(M.conditional_guessing(worlds),Fraction(1))
    def test_pairwise_independence_is_not_joint_independence(self):
        from collections import Counter
        worlds=M.gf4_worlds()
        self.assertEqual(len({w[2:] for w in worlds}),16)
        for i in range(4):
            for j in range(i+1,4):
                self.assertEqual(Counter((w[2+i],w[2+j]) for w in worlds),Counter(product(range(4),repeat=2)))
        self.assertNotEqual(Fraction(1,16),Fraction(1,4)**4)
    def test_gf4_multiplication(self):
        for a in range(4):
            self.assertEqual(M.gf4_mul(a,0),0)
            self.assertEqual(M.gf4_mul(a,1),a)
        self.assertEqual(M.gf4_mul(2,2),3)
        self.assertEqual(M.gf4_mul(2,3),1)
    def test_first_hit_no_probes(self):self.assertEqual(M.bucket_first_hit(16,4,0),(0,16))
    def test_first_hit_one_bucket(self):self.assertEqual(M.bucket_first_hit(16,1,3),(3,16))
    def test_first_hit_source_advice(self):self.assertEqual(M.bucket_first_hit(16,4,3),(12,16))
    def test_first_hit_saturation(self):self.assertEqual(M.bucket_first_hit(16,4,5),(16,16))
    def test_first_hit_bounds(self):
        for args in ((0,1,1),(64,1,1),(8,3,1),(8,1,9),(8,0,0)):
            with self.assertRaises(ValueError):M.bucket_first_hit(*args)

class ReportMutations(unittest.TestCase):
    def setUp(self):
        self.report={'schema':1,'complete':True,'consensus_eligibility':'NotEstablished','consensus_weight':None,
          'controls':{k:True for k in ('wrong_context_rejected','empty_table_rejected','truncated_table_rejected','duplicate_record_rejected')}}
    def test_complete_report(self):M.validate_control_report(self.report)
    def test_empty_control_dictionary_is_not_success(self):
        self.report['controls']={}
        with self.assertRaises(ValueError):M.validate_control_report(self.report)
    def test_missing_control(self):
        self.report['controls'].pop('wrong_context_rejected')
        with self.assertRaises(ValueError):M.validate_control_report(self.report)
    def test_extra_control(self):
        self.report['controls']['unexpected']=True
        with self.assertRaises(ValueError):M.validate_control_report(self.report)
    def test_truthy_integer_is_not_true(self):
        self.report['controls']['wrong_context_rejected']=1
        with self.assertRaises(ValueError):M.validate_control_report(self.report)
    def test_false_control(self):
        self.report['controls']['empty_table_rejected']=False
        with self.assertRaises(ValueError):M.validate_control_report(self.report)
    def test_eligibility_claim_is_rejected(self):
        self.report['consensus_eligibility']='Established'
        with self.assertRaises(ValueError):M.validate_control_report(self.report)
    def test_weight_is_rejected(self):
        self.report['consensus_weight']=1
        with self.assertRaises(ValueError):M.validate_control_report(self.report)
    def test_wrong_version_and_incomplete(self):
        for key,value in (('schema',True),('schema',999),('complete',False),('complete',1)):
            data=copy.deepcopy(self.report);data[key]=value
            with self.assertRaises(ValueError):M.validate_control_report(data)
    def test_csv_empty_and_truncated(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'cases.csv'
            p.write_text('a,b\n1,2\n',encoding='ascii')
            self.assertEqual(list(V.rows(p,['a','b'],1)),[[1,2]])
            with self.assertRaises(ValueError):list(V.rows(p,['a','b'],2))
            p.write_text('a,b\n',encoding='ascii')
            with self.assertRaises(ValueError):list(V.rows(p,['a','b'],1))
    def test_csv_ambiguous_integer(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'cases.csv'
            for value in ('01','-1','1.0','+1','true',''):
                p.write_text('a\n'+value+'\n',encoding='ascii')
                with self.assertRaises(ValueError):list(V.rows(p,['a'],1))
    def test_csv_header_and_row_width(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/'cases.csv'
            for text in ('wrong\n1\n','a\n1,2\n',''):
                p.write_text(text,encoding='ascii')
                with self.assertRaises(ValueError):list(V.rows(p,['a'],1))

class ServiceFixtures(unittest.TestCase):
    def test_request_parameter_types_and_mutations(self):
        base=O.request_id()
        self.assertNotEqual(base,O.request_id(nonce=b'\x35'*32))
        for key in ('label','interval','sample','index'):
            self.assertNotEqual(base,O.request_id(**{key:1}))
    def test_unavailable_not_service(self):
        import struct
        frame=O.MAGIC+b'\x01'+O.request_id()+struct.pack('>I',0)
        self.assertEqual(O.verify(frame),'Unavailable')
    def test_closed_frame_failure_layers(self):
        data=O.frame()
        for mutated in (data[:-1],data+b'\0',data[:45]):
            with self.assertRaisesRegex(ValueError,'Framing'):O.verify(mutated)
    def test_bytes_remain_valid_for_a_legitimate_new_request(self):
        old=O.frame();new=O.frame(nonce=b'\x44'*32)
        self.assertEqual(old[45:],new[45:])
        self.assertEqual(O.verify(new,nonce=b'\x44'*32),'Verified')
        with self.assertRaisesRegex(ValueError,'RequestContext'):O.verify(old,nonce=b'\x44'*32)

if __name__=='__main__':unittest.main()
