#!/usr/bin/env python3
"""Generate finite public fixtures in a NEW directory; never overwrite evidence."""
from __future__ import annotations
import argparse
import csv
import hashlib
import importlib.util
import io
import json
from fractions import Fraction
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("public_finite_models", HERE / "models.py")
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


def csv_bytes(fields, rows):
    buf = io.StringIO(newline="")
    w = csv.writer(buf, lineterminator="\n")
    w.writerow(fields)
    w.writerows(rows)
    return buf.getvalue().encode("ascii")


def build() -> dict[str, bytes]:
    files = {}
    files["sample_targets.csv"] = csv_bytes(
        ["source", "challenge", "target_pair"],
        ((x, c, m.target(x, c)) for x in m.STATES for c in m.CHALLENGES))
    files["response_tables.csv"] = csv_bytes(
        ["table", *(f"source_{i}_hits" for i in range(16))],
        ((t, *(m.response_table_score(t, x) for x in m.STATES)) for t in range(256)))
    scores = []
    for encoder in range(65536):
        q0, q1 = m.encoder_scores(encoder)
        m.require(q1 == m.fourier_q1(encoder), "independent Q1 check failed")
        scores.append((encoder, q0, q1, 64))
    files["advice_encoders.csv"] = csv_bytes(
        ["encoder", "q0_numerator", "q1_numerator", "denominator"], scores)
    m.require(max(r[1] for r in scores) == 28, "Q0 optimum changed")
    m.require(max(r[2] for r in scores) == 48, "Q1 optimum changed")
    rows = []
    for rounds in range(1, 13):
        p = m.fixed_book_repetition(rounds)
        m.require(p == m.fixed_book_formula(rounds), "fixed book expression disagrees")
        naive = Fraction(7, 16)**rounds
        rows.append((rounds, p.numerator, p.denominator, naive.numerator, naive.denominator))
    files["same_source_repetition.csv"] = csv_bytes(
        ["rounds", "fixed_strategy_numerator", "fixed_strategy_denominator", "iid_guess_numerator", "iid_guess_denominator"], rows)
    rows = []
    for q in (2, 4, 8, 16, 32):
        for advice in range(1, q+1):
            if q % advice:
                continue
            for probes in range(0, q//advice + 2):
                if probes > q:
                    continue
                hits, worlds = m.bucket_first_hit(q, advice, probes)
                m.require(Fraction(hits, worlds) == min(Fraction(1), Fraction(advice*probes, q)), "bucket formula")
                rows.append((q, advice, probes, hits, worlds))
    files["bucket_first_hit.csv"] = csv_bytes(["alphabet", "advice_states", "probes", "hits", "worlds"], rows)
    files["gf4_affine_worlds.csv"] = csv_bytes(["a", "b", "y0", "y1", "y2", "y3"], m.gf4_worlds())
    files["masked_public_view.csv"] = csv_bytes(["target", "helper", "public_xor"],
        ((u, r, u ^ r) for u in range(2) for r in range(2)))
    files["expected_summary.json"] = (json.dumps({
        "schema":1, "kind":"synthetic-finite-reference-v1",
        "advice_encoders":65536, "source_states":16, "challenges":4,
        "q0_max_numerator":28, "q1_max_numerator":48, "denominator":64,
        "q0_maximizers":sum(r[1] == 28 for r in scores),
        "q1_maximizers":sum(r[2] == 48 for r in scores),
        "repetition_profiles":12, "first_hit_profiles":len(rows),
        "gf4_worlds":16, "masked_view_worlds":4,
        "consensus_eligibility":"NotEstablished", "consensus_weight":None,
        "full_protocol_export":False,
    }, indent=2, sort_keys=True)+"\n").encode()
    return files


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("output", type=Path)
    args = p.parse_args()
    if args.output.exists():
        p.error("output already exists; choose a new directory")
    args.output.mkdir(parents=True)
    try:
        files = build()
        for name, content in files.items():
            with (args.output/name).open("xb") as f:
                f.write(content)
        print(f"Generated {len(files)} deterministic finite fixture files.")
    except Exception:
        print("Incomplete generation; existing output retained for diagnosis.")
        raise

if __name__ == "__main__":
    main()
