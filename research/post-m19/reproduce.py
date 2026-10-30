#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Nine bounded public arithmetic/model checks. Not a native protocol test suite.

Python 3.10+, standard library only. No downloads or writes. All acceptance
checks use explicit exceptions and remain active with python -O.
"""
from __future__ import annotations
import hashlib
import itertools
import json
import math
import sys
from decimal import Decimal, localcontext
from fractions import Fraction
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / 'tests/fixtures/post-m19/reference_parameters.json'

class ReferenceFailure(ValueError):
    pass

def require(condition: bool, message: str) -> None:
    if not condition:
        raise ReferenceFailure(message)

def small_decimal(value: Fraction) -> str:
    with localcontext() as ctx:
        ctx.prec = 34
        return str(Decimal(value.numerator) / Decimal(value.denominator))

def pow2(exp: int) -> Fraction:
    return Fraction(1 << exp) if exp >= 0 else Fraction(1, 1 << (-exp))

def codec(c: dict[str, Any], profile: dict[str, int]) -> dict[str, int]:
    n = c['contexts'] * c['vertices_per_context']
    a = c['contexts'] * profile['draws_per_context']
    t, s = profile['counterfactual_copies'], c['cache_vertices']
    require(0 < s < n and t >= 1 and a >= 1, 'invalid codec parameters')
    require(c['max_indegree'] >= 0, 'negative indegree')
    j = n + t * (c['online_queries'] + a)
    ell = j.bit_length()  # ceil(log2(j + 1)), computed without floating point
    h = max(64, ell)
    u = (max(c['max_indegree'], 1) - 1).bit_length()
    saved = c['label_bits'] - 2 * ell - u
    require(saved > 0, 'nonpositive saved bits')
    return {'n': n, 'a': a, 't': t, 's': s, 'J': j, 'ell': ell, 'h': h,
            'u': u, 'B': saved, 'exponent': c['state_bits'] + h - s * saved}

def r01(_: dict[str, Any]) -> dict[str, Any]:
    worlds = list(itertools.product((0, 1), repeat=2))
    targets, selected_zero, wrong_ordinal = [0, 0], 0, 0
    for x, y in worlds:
        target = 0 if x == 0 else 1
        targets[target] += 1
        selected_zero += int((x, y)[target] == 0)
        # A shared ordinal tape is not a consistent assignment to swapped keys.
        first = {'x': x, 'y': y}
        reversed_ordinal = {'y': x, 'x': y}
        wrong_ordinal += int(first != reversed_ordinal)
    require(targets == [2, 2], 'target IDs are not balanced')
    require(selected_zero == 3 and wrong_ordinal == 2, 'diagnostic changed')
    require(Fraction(selected_zero, 4) * 16384 == 12288, 'scaled bias mismatch')
    return {'worlds': 4, 'target_counts': targets, 'selected_zero': '3/4',
            'wrong_ordinal_disagreements': 2,
            'scope': 'elementary diagnostic; not the original 14-bit corpus'}

def r02(_: dict[str, Any]) -> dict[str, Any]:
    public = Fraction(3, 4) ** 2
    replay = Fraction(1, 2) ** 2
    bound = public + pow2(-216)
    require(public == Fraction(9, 16) and replay == Fraction(1, 4), 'rational mismatch')
    require(replay < public < bound < 1, 'bound became trivial or misordered')
    return {'public_assembly': '9/16', 'replay_floor': '1/4',
            'residual_upper_expression': '9/16 + 2^-216',
            'strictly_below_one': True, 'scope': 'stated arithmetic only'}

def r03(p: dict[str, Any]) -> dict[str, Any]:
    q = p['leakage']
    require(q == {'positions_per_context': 1024, 'contexts': 2, 'audits': 10240},
            'unexpected leakage fixture')
    n, audits = q['positions_per_context'], q['audits']
    unseen = Fraction(n - 1, n) ** audits
    replay = (1 - unseen) ** 2
    full_lower = max(Fraction(0), 1 - 2 * n * unseen)
    require(Fraction('0.9999096447470642') < replay < Fraction('0.9999096447470644'),
            'replay probability outside exact enclosure')
    require(Fraction('0.9074741308970459') < full_lower < Fraction('0.9074741308970461'),
            'coverage lower bound outside exact enclosure')
    require(full_lower < replay < 1, 'invalid probability ordering')
    return {'positions_per_context': n, 'audits': audits,
            'replay_probability_decimal': small_decimal(replay),
            'complete_coverage_union_lower_decimal': small_decimal(full_lower),
            'acceptance_comparisons': 'exact rational; decimals are display only',
            'scope': 'independent uniform uncensored draws; paid public history'}

def closure(n: int, cached: set[int], rounds: int) -> set[int]:
    known = set(cached)
    for _ in range(rounds):
        previous = set(known)
        known.update(v for v in range(n) if v == 0 or v - 1 in previous)
    return known

def r04(p: dict[str, Any]) -> dict[str, Any]:
    comparisons = 0
    for n in range(1, 10):
        for rounds in range(4):
            maxima = [0] * (n + 1)
            for mask in range(1 << n):
                s = {v for v in range(n) if (mask >> v) & 1}
                got = len(closure(n, s, rounds))
                bound = min(n, (rounds + 1) * len(s) + rounds)
                require(got <= bound, 'chain coverage exceeded the elementary bound')
                maxima[len(s)] = max(maxima[len(s)], got)
                comparisons += 1
            for size, value in enumerate(maxima):
                require(value == min(n, (rounds + 1) * size + rounds),
                        'finite-chain maximum differs from formula')
    c = p['codec']
    n, total, rounds = c['vertices_per_context'], c['cache_vertices'], c['rounds']
    pairs = [(min(n, (rounds + 1) * left + rounds),
              min(n, (rounds + 1) * (total - left) + rounds))
             for left in range(total + 1)]
    scores = [a * b for a, b in pairs]
    require(max(scores) == 866 ** 2, 'two-chain maximum changed')
    require(scores[288] == max(scores) and pairs[288] == (866, 866), 'balance mismatch')
    require(Fraction(866, 1024) == Fraction(433, 512), 'coverage reduction mismatch')
    return {'small_chain_comparisons': comparisons, 'largest_exhaustive_chain': 9,
            'two_chain_allocation_cases': len(pairs), 'balanced_cache': [288, 288],
            'covered_positions': [866, 866], 'one_context_coverage': '433/512',
            'scope': 'canonical chain closure, not a general concrete-hash theorem'}

def r05(p: dict[str, Any]) -> dict[str, Any]:
    require(p['quorum'] == {'signers': 4, 'quorum': 3, 'max_byzantine': 1},
            'unexpected quorum fixture')
    signers = set(range(4))
    quorums = [set(q) for q in itertools.combinations(signers, 3)]
    require(min(len(a & b) for a in quorums for b in quorums) == 2,
            'intersection size mismatch')
    histories = 0
    for faulty in signers:
        honest = sorted(signers - {faulty})
        for decisions in itertools.product((None, 'A', 'B'), repeat=3):
            a = {faulty} | {v for v, d in zip(honest, decisions) if d == 'A'}
            b = {faulty} | {v for v, d in zip(honest, decisions) if d == 'B'}
            require(not (len(a) >= 3 and len(b) >= 3), 'persistent-lock conflict')
            histories += 1
    # Outside the premise, two faulty signers can sign both decisions.
    outside_a, outside_b = {0, 1, 2}, {0, 1, 3}
    require(len(outside_a) == len(outside_b) == 3 and outside_a & outside_b == {0, 1},
            'two-faulty witness lost')
    return {'quorums': len(quorums), 'minimum_intersection': 2,
            'persistent_lock_assignments': histories,
            'two_faulty_conflict_witness': [sorted(outside_a), sorted(outside_b)],
            'scope': 'finite signer-set model; no signature or network execution'}

def r06(p: dict[str, Any]) -> dict[str, Any]:
    c = p['codec']
    require((c['contexts'], c['vertices_per_context'], c['label_bits'],
             c['state_bits'], c['online_queries'], c['rounds'], c['cache_vertices'])
            == (2, 1024, 256, 102208, 65536, 2, 576), 'unexpected codec model')
    require(c['max_indegree'] == 1, 'unexpected graph indegree')
    require(len(c['profiles']) == 2, 'unexpected profile count')
    results = []
    for profile, expected in zip(c['profiles'], ((64, 512, 33622016, 26, 204, -15232),
                                                (384, 128, 8488960, 24, 208, -17536))):
        z = codec(c, profile)
        got = (profile['draws_per_context'], z['t'], z['J'], z['ell'], z['B'], z['exponent'])
        require(got == expected, 'codec parameter mismatch')
        require(z['J'] == profile['expected_records'] and z['ell'] == profile['expected_locator_bits']
                and z['B'] == profile['expected_saved_bits']
                and z['exponent'] == profile['expected_codec_exponent'], 'fixture certificate mismatch')
        require(math.comb(z['n'], z['s']) < 1 << z['n'], 'binomial bound mismatch')
        results.append({**z, 'draws_per_context': profile['draws_per_context'],
                        'root_codec_exponent': str(Fraction(z['exponent'], z['t'])),
                        'independent_target_bits': z['a'] * 10})
    return {'profiles': results, 'scope': 'codec arithmetic, not the codec injection proof'}

def r07(p: dict[str, Any]) -> dict[str, Any]:
    z = codec(p['codec'], p['codec']['profiles'][0])
    require((z['n'], z['t'], z['exponent']) == (2048, 512, -15232), 'wrong k64 profile')
    certificates = [(433 * 13, 11 * 512), (11 ** 5 * 16, 7 * 13 ** 5),
                    (11 ** 3 * 8, 5 * 13 ** 3), (7 ** 7 * 5, 2 * 8 ** 7),
                    (7 ** 4 * 5, 3 * 8 ** 4)]
    require(all(a < b for a, b in certificates), 'integer certificate failed')
    base_power = Fraction(433, 512) ** 128
    require(base_power < Fraction(3, 125) * pow2(-25), 'power bound failed')
    first = 16 * base_power
    require(first < Fraction(96, 125) * pow2(-26), 'first term bound failed')
    # The t-th-root codec term is 2^(-119/4), not 2^(-15232).
    require(Fraction(z['exponent'], z['t']) == Fraction(-119, 4), 'root was omitted')
    require(z['exponent'] < -29 * z['t'], 'codec term not below 2^-29')
    coarse = (Fraction(96, 125) + Fraction(1, 8)) * pow2(-26)
    require(coarse == Fraction(893, 1000) * pow2(-26) and coarse < pow2(-26),
            'final k64 certificate failed')
    return {'integer_certificates': [{'left': a, 'right': b} for a, b in certificates],
            'root_codec_exponent': '-119/4', 'raw_success_strict_upper': '2^-26',
            'coarse_bound': '(893/1000)*2^-26', 'scope': 'conditional arithmetic certificate'}

def r08(p: dict[str, Any]) -> dict[str, Any]:
    z = codec(p['codec'], p['codec']['profiles'][1])
    require((z['n'], z['t'], z['exponent']) == (2048, 128, -17536), 'wrong k384 profile')
    q128 = Fraction(433, 512) ** 128
    require(q128 < Fraction(3, 125) * pow2(-25) < pow2(-30), 'base certificate failed')
    q768 = Fraction(433, 512) ** 768
    require(q768 == q128 ** 6 and q768 < pow2(-180), 'sixth-power certificate failed')
    require(Fraction(z['exponent'], z['t']) == -137, 'root was omitted')
    first, error = pow2(16) * q768, pow2(-137)
    require(first < pow2(-164) and first + error < pow2(-136), 'final k384 certificate failed')
    return {'root_codec_exponent': '-137', 'first_term_strict_upper': '2^-164',
            'raw_success_strict_upper': '2^-136', 'scope': 'one simultaneous batch, not lifetime amplification'}

def r09(p: dict[str, Any]) -> dict[str, Any]:
    q_auth = 1 << 32
    collision = Fraction(q_auth * (q_auth - 1), 1 << 257)
    require(collision < pow2(-193), 'authentication error bound failed')
    f = p['visible_format']
    require(f == {'header_bytes': 128, 'signature_bytes': 64, 'index_bytes': 4,
                  'word_bytes': 32, 'digest_bytes': 32, 'merkle_depth': 10,
                  'registration_bytes': 512}, 'unexpected format fixture')
    record = f['index_bytes'] + f['word_bytes'] + f['merkle_depth'] * f['digest_bytes']
    require(record == 356, 'opening size mismatch')
    sizes = []
    for k, response_expected, challenge_expected in ((64, 136896, 704), (384, 820416, 3264)):
        response = f['header_bytes'] + f['signature_bytes'] + 6 * k * record
        challenge = f['header_bytes'] + f['signature_bytes'] + 2 * k * f['index_bytes']
        require(response == response_expected and challenge == challenge_expected,
                'wire-size arithmetic mismatch')
        require(p['codec']['state_bits'] % 8 == 0, 'unaligned state budget')
        state_bytes = p['codec']['state_bits'] // 8
        require(state_bytes == 12776 and state_bytes < response < 1 << 20,
                'format envelope mismatch')
        sizes.append({'draws_per_context': k, 'response_bytes': response,
                      'challenge_bytes': challenge, 'state_bytes': state_bytes})
    registration_calls = 2048 + 3072 + 3069
    descriptor = 8 + 5 + 1023 * 9
    require(registration_calls == 8189 and descriptor == 9220, 'registration arithmetic mismatch')
    return {'authentication_query_cap': q_auth, 'collision_error_strict_upper': '2^-193',
            'record_bytes': record, 'formats': sizes, 'registration_hash_calls': registration_calls,
            'graph_descriptor_bytes': descriptor,
            'scope': 'literal specification sizes; no executed verifier or universal compression lower bound'}

CHECKS = [('R01', r01), ('R02', r02), ('R03', r03), ('R04', r04), ('R05', r05),
          ('R06', r06), ('R07', r07), ('R08', r08), ('R09', r09)]

def main() -> int:
    try:
        raw = FIXTURE.read_bytes()
        p = json.loads(raw)
        require(p.get('schema') == 'pout-post-m19-reference-parameters-v1', 'unknown fixture schema')
        results = []
        for name, fn in CHECKS:
            results.append({'id': name, 'status': 'PASS', 'details': fn(p)})
        print(json.dumps({'schema': 'pout-public-reference-results-v1', 'checks_passed': len(results),
              'checks_failed': 0, 'fixture_sha256': hashlib.sha256(raw).hexdigest(), 'results': results,
              'claim': 'Selected exact arithmetic and finite models only; no native cryptographic or consensus certification.'},
              indent=2, sort_keys=True))
        return 0
    except (OSError, ValueError, KeyError, TypeError, OverflowError) as exc:
        print(json.dumps({'status': 'FAIL', 'error_type': type(exc).__name__, 'message': str(exc)}, sort_keys=True))
        return 1

if __name__ == '__main__':
    sys.exit(main())
