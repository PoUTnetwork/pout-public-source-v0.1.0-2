"""Separate exact checker for portable finite fixtures; no network or policy claims."""
from __future__ import annotations
import csv
import json
from collections import Counter
from fractions import Fraction
from itertools import product
from pathlib import Path


def check(condition, message):
    if not condition:
        raise ValueError(message)


def rows(path, header, expected_count=None):
    check(path.is_file() and not path.is_symlink(), f"not a regular fixture: {path.name}")
    check(path.stat().st_size <= 8_000_000, "fixture too large")
    with path.open(encoding="ascii", newline="") as stream:
        reader = csv.reader(stream)
        check(next(reader, None) == header, f"header mismatch: {path.name}")
        count = 0
        for row in reader:
            check(len(row) == len(header), f"row width: {path.name}")
            values = []
            for raw in row:
                check(raw.isascii() and raw.isdecimal() and str(int(raw)) == raw, "noncanonical integer")
                values.append(int(raw))
            count += 1
            check(count <= 100_000, "too many rows")
            yield values
        if expected_count is not None:
            check(count == expected_count, f"row count mismatch: {path.name}")


def posterior_q0(encoder):
    # Independent list-based posterior histograms, rather than source bit masks.
    total = 0
    for left, right in product(range(2), range(2, 4)):
        cells = [Counter(), Counter()]
        for x in range(16):
            pair = ((x >> left) & 1, (x >> right) & 1)
            cells[(encoder >> x) & 1][pair] += 1
        total += sum(max(c.values(), default=0) for c in cells)
    return total


def character_q1(encoder):
    # Direct parity-character sums; deliberately no Walsh butterfly code.
    masks = (1, 2, 4, 8, 5, 9, 6, 10)
    coeff = {mask: sum(1 if (((encoder >> x) & 1) ^ ((x & mask).bit_count() & 1)) == 0
                           else -1 for x in range(16)) for mask in masks}
    bonus = sum(max(abs(coeff[1 << a]), abs(coeff[1 << b]), abs(coeff[(1 << a) | (1 << b)]))
                for a in range(2) for b in range(2, 4))
    check(bonus % 2 == 0, "noninteger Q1 character formula")
    return 32 + bonus//2


def verify_fixtures(root: Path) -> dict:
    expected_names = {"sample_targets.csv", "response_tables.csv", "advice_encoders.csv",
                      "same_source_repetition.csv", "bucket_first_hit.csv", "gf4_affine_worlds.csv",
                      "masked_public_view.csv", "expected_summary.json"}
    check({p.name for p in root.iterdir()} == expected_names, "unexpected or missing finite fixture")
    targets = list(rows(root/"sample_targets.csv", ["source", "challenge", "target_pair"], 64))
    for i, (x, c, value) in enumerate(targets):
        check((x, c) == divmod(i, 4), "target order")
        bits = [(x//2**j) % 2 for j in range(4)]
        check(value == bits[c % 2] + 2*bits[2+c//2], "target value")
    tables = list(rows(root/"response_tables.csv", ["table", *(f"source_{i}_hits" for i in range(16))], 256))
    for table, *scores in tables:
        for x, score in enumerate(scores):
            expected = sum((table//4**c) % 4 == targets[x*4+c][2] for c in range(4))
            check(score == expected, "response-table score")
    check([r[0] for r in tables] == list(range(256)), "table order")
    maxima = [0, 0]; maximizers = [0, 0]
    for expected_id, row in enumerate(rows(root/"advice_encoders.csv",
            ["encoder", "q0_numerator", "q1_numerator", "denominator"], 65536)):
        encoder, q0, q1, den = row
        check(encoder == expected_id and den == 64, "encoder order/denominator")
        check(q0 == posterior_q0(encoder), "Q0 posterior mismatch")
        check(q1 == character_q1(encoder), "Q1 character mismatch")
        for i, value in enumerate((q0, q1)):
            if value > maxima[i]:
                maxima[i] = value; maximizers[i] = 1
            elif value == maxima[i]:
                maximizers[i] += 1
    check(maxima == [28, 48], "unexpected finite optimum")
    for t, num, den, iid_num, iid_den in rows(root/"same_source_repetition.csv",
            ["rounds", "fixed_strategy_numerator", "fixed_strategy_denominator", "iid_guess_numerator", "iid_guess_denominator"], 12):
        check(1 <= t <= 12 and den > 0 and iid_den > 0, "repetition bounds")
        # Count the independently recorded score histogram: 2 sources have 4
        # successes; 8 have 2; 4 have 1; 2 have 0. No fresh source between rounds.
        direct = Fraction(2*4**t + 8*2**t + 4, 16*4**t)
        check(Fraction(num, den) == direct, "same-source result")
        check(Fraction(iid_num, iid_den) == Fraction(7, 16)**t, "iid comparison")
        if t > 1:
            check(direct > Fraction(iid_num, iid_den), "missing dependence counterexample")
    first_rows = list(rows(root/"bucket_first_hit.csv", ["alphabet", "advice_states", "probes", "hits", "worlds"]))
    identities = []
    for q in (2,4,8,16,32):
        for advice in range(1,q+1):
            if q % advice == 0:
                identities.extend((q,advice,t) for t in range(q//advice+2) if t<=q)
    check([tuple(r[:3]) for r in first_rows] == identities, "first-hit case set")
    for q, advice, probes, hits, worlds in first_rows:
        check(worlds == q and hits == min(q, advice*probes), "first-hit probability")
    gf = list(rows(root/"gf4_affine_worlds.csv", ["a","b","y0","y1","y2","y3"],16))
    multiplication = ((0,0,0,0),(0,1,2,3),(0,2,3,1),(0,3,1,2))
    check([r[:2] for r in gf] == [list(p) for p in product(range(4),repeat=2)], "GF4 source set")
    for a,b,*outputs in gf:
        check(outputs == [multiplication[a][k]^b for k in range(4)], "GF4 values")
    for i in range(4):
        for j in range(i+1,4):
            check(Counter((r[2+i],r[2+j]) for r in gf) == Counter(product(range(4),repeat=2)), "pairwise law")
    check(len({tuple(r[2:]) for r in gf}) == 16, "joint entropy control")
    mask = list(rows(root/"masked_public_view.csv", ["target","helper","public_xor"],4))
    check(mask == [[u,r,u^r] for u in range(2) for r in range(2)], "masked-view world set")
    for public in range(2):
        check(Counter(u for u,r,v in mask if v==public) == Counter({0:1,1:1}), "uniform target before helper")
    check(all(u == r^v for u,r,v in mask), "helper recovery")
    with (root/"expected_summary.json").open(encoding="utf-8") as f:
        summary = json.load(f)
    expected = {
        "schema":1, "kind":"synthetic-finite-reference-v1", "advice_encoders":65536,
        "source_states":16, "challenges":4, "q0_max_numerator":28, "q1_max_numerator":48,
        "denominator":64, "q0_maximizers":maximizers[0], "q1_maximizers":maximizers[1],
        "repetition_profiles":12, "first_hit_profiles":len(first_rows), "gf4_worlds":16,
        "masked_view_worlds":4, "consensus_eligibility":"NotEstablished",
        "consensus_weight":None, "full_protocol_export":False,
    }
    check(summary == expected, "summary mismatch")
    return {"finite_fixture_files":8, "encoder_rows":65536, "first_hit_rows":len(first_rows),
            "fixed_repetition_rows":12, "q0_numerator":maxima[0], "q1_numerator":maxima[1],
            "denominator":64, "new_rust_execution":False}
