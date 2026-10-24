#!/usr/bin/env python3
"""Public finite-reference calculations for selected M14-M19 results.

Standard library only. No Rust build, no network, no native resource proof.
This is a compact public adaptation, not the complete engineering test corpus.
"""
from __future__ import annotations

import hashlib
import itertools
import json
import struct
from fractions import Fraction
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "tests" / "fixtures" / "milestones"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def m14_score(mode: str, budget: int, prior: tuple, initial: int = 0,
              mapping: tuple[int, int] = (0, 1), required: int = 2) -> int:
    """Exact finite policy search. Choices see posterior replies, not a hidden world."""
    require(mode in ("ordinary", "leaky", "early"), "unsupported mode")
    require(0 <= budget <= 4 and 0 < len(prior) <= 16, "bounded profile")
    envelopes = (0,) if mode == "ordinary" else (0, 1, 2)
    actions = tuple(itertools.product(envelopes, (0, 1), range(4)))
    visited = 0

    @lru_cache(maxsize=None)
    def solve(posterior: tuple, credited: int, remaining: int) -> int:
        nonlocal visited
        visited += 1
        require(visited <= 50000, "state budget exceeded")
        if (credited & ~initial).bit_count() >= required:
            return len(posterior)
        if not remaining or not posterior:
            return 0
        best = 0
        for envelope, label, guess in actions:
            target = mapping[label]
            groups = [[], [], []]  # reject, accept, already-consumed
            for world in posterior:
                if mode == "early" and credited & (1 << target):
                    outcome = 2
                else:
                    correct = world[target] == guess
                    filt = (envelope == 0 or
                            (envelope == 1 and world[1] in (0, 1)) or
                            (envelope == 2 and world[1] in (0, 2)))
                    outcome = int(correct and filt)
                groups[outcome].append(world)
            score = sum(solve(tuple(group), credited | (1 << target) if i == 1 else credited,
                              remaining - 1) for i, group in enumerate(groups) if group)
            best = max(best, score)
        return best

    return solve(prior, initial, budget)


def m14() -> dict:
    worlds = tuple(itertools.product(range(4), repeat=2))
    rows = []
    for mode, expected in (("ordinary", (1, 3, 6)), ("leaky", (1, 3, 7)),
                           ("early", (1, 3, 6))):
        for budget, count in zip((2, 3, 4), expected):
            value = m14_score(mode, budget, worlds)
            require(value == count, "M14 independent-profile mismatch")
            rows.append({"mode": mode, "budget": budget, "successes": value, "worlds": 16})
    warm = tuple(w for w in worlds if w[0] == 0)
    for mode, expected in (("ordinary", 3), ("leaky", 4), ("early", 3)):
        value = m14_score(mode, 3, warm, initial=1, required=1)
        require(value == expected, "M14 warm-profile mismatch")
        rows.append({"mode": "warm_" + mode, "budget": 3, "successes": value, "worlds": 4})
    diagonal = tuple(w for w in worlds if w[0] == w[1])
    for budget, expected in ((2, 1), (3, 2), (4, 3)):
        value = m14_score("ordinary", budget, diagonal)
        require(value == expected, "M14 correlated-profile mismatch")
        rows.append({"mode": "correlated", "budget": budget, "successes": value, "worlds": 4})
    value = m14_score("ordinary", 4, diagonal, mapping=(0, 0))
    require(value == 0, "aliased identity incorrectly creates a second credit")
    rows.append({"mode": "alias", "budget": 4, "successes": value, "worlds": 4})
    return {"classification": "finite public reference; not native work", "profiles": rows}


def direct_bound(q_calls: int, r_calls: int, n: int, alphabet: int,
                 p: Fraction) -> Fraction:
    require(q_calls >= 0 and r_calls >= 0 and n > 0 and alphabet > 0,
            "invalid bound parameters")
    require(0 <= p <= 1, "invalid probability")
    return min(Fraction(1), 1 - (1-p)**r_calls,
               1 - (1-p)**min(r_calls, q_calls//n) + r_calls*p/alphabet)


def m18() -> dict:
    selected = successes = fixed0 = fixed1 = 0
    helper_calls = recipient_calls = rank_calls = 0
    for a, b, rank_pass in itertools.product(range(16), range(16), (0, 1)):
        choice = 0 if a == 0 else 1 if b == 0 else None
        helper_calls += 2
        if choice is not None:
            selected += 1
            recipient_calls += 5
            rank_calls += 1
            successes += rank_pass
        fixed0 += int(a == 0 and rank_pass == 1)
        fixed1 += int(b == 0 and rank_pass == 1)
    require((selected, successes, fixed0, fixed1) == (62, 31, 16, 16),
            "M18 quotient enumeration mismatch")
    wrong = direct_bound(5, 1, 6, 16, Fraction(1, 2))
    charged = direct_bound(7, 1, 6, 16, Fraction(1, 2))
    require(wrong == Fraction(16, 512) and charged == Fraction(256, 512), "bound mismatch")
    require(wrong < Fraction(successes, 512) <= charged, "missing helper counterexample")
    counts = [0, 0]
    zero = passed = 0
    for a, b in itertools.product((0, 1), repeat=2):
        for table in itertools.product((0, 1), repeat=4):
            target = 0 if a == 0 else 1
            label = (a, b)[target]
            counts[target] += 1
            zero += int(label == 0)
            passed += table[2*target + label]
    require((counts, zero, passed) == ([32, 32], 48, 32), "conditioning mismatch")
    return {"classification": "ideal finite reference; symbolic costs, no native hashing",
            "quotient_worlds": 512, "selected": selected, "successes": successes,
            "incorrect_online_only_bound": str(wrong), "global_charged_bound": str(charged),
            "helper_queries": helper_calls, "recipient_queries": recipient_calls,
            "fresh_rank_queries": rank_calls, "complete_tiny_worlds": 64,
            "target_counts": counts, "selected_label_zero": zero, "rank_passes": passed}


def frame(*fields: bytes) -> bytes:
    require(len(fields) <= 16 and all(len(x) <= 4096 for x in fields), "frame bounds")
    return b"".join(struct.pack("<I", len(x)) + x for x in fields)


def parse_frames(data: bytes) -> list[bytes]:
    require(len(data) <= 8192, "input too long")
    result, offset = [], 0
    while offset < len(data):
        require(len(result) < 16 and offset+4 <= len(data), "truncated header")
        size = struct.unpack_from("<I", data, offset)[0]
        offset += 4
        require(size <= 4096 and offset+size <= len(data), "truncated/oversized field")
        result.append(data[offset:offset+size]); offset += size
    return result


def golden_encodings() -> dict:
    node_domain = b"PoUT-research/DDA-EPoR-exact-intent/v2"
    rank_domain = b"PoUT-research/exact-intent-rank/v1"
    vertex_domain = b"PoUT-research/verified-common-vertex/v2"
    target = frame(node_domain, b"epor-node", b"synthetic-m19-intent-0", b"1"*32, b"B"*32)
    zero, one = bytes(32), b"\x80" + bytes(31)
    labels = (one, zero, one, zero, zero, one)
    parents = struct.pack("<I", 2) + b"".join(struct.pack("<I", i) + x for i, x in enumerate(labels[:2]))
    node = target + frame(struct.pack("<I", 2), parents)
    rank = frame(rank_domain, frame(vertex_domain, target, b"".join(labels)))
    require((len(node),len(rank)) == (241,438), "golden lengths")
    require(node == (FIXTURES/"m19-node-index2.frame").read_bytes(), "Node golden mismatch")
    require(rank == (FIXTURES/"m19-rank-vector.frame").read_bytes(), "Rank golden mismatch")
    require(hashlib.sha256(node).hexdigest() == "196aa5cf156f996b576872021ee39432b62c6dc745a2665b3674eee79937843e", "Node golden hash")
    require(hashlib.sha256(rank).hexdigest() == "c7a14424517ab9df2f1a999a98fddc1fe43468cded2992147fbec36137b9e386", "Rank golden hash")
    v = json.loads((FIXTURES/"m17-half-rank.json").read_text())
    t = bytes.fromhex(v["complete_target_hex"])
    common = b"".join(bytes.fromhex(x) for x in v["common6_hex"])
    actual = frame(rank_domain, frame(vertex_domain, t, common))
    require(actual.hex() == v["rank_input_hex"], "M17 full-pair framing mismatch")
    require(actual == (FIXTURES/"m17-half-rank.frame").read_bytes(), "M17 fixture mismatch")
    t_fields = parse_frames(t)
    require(len(t_fields) == 5 and t_fields[:2] == [node_domain, b"epor-node"], "target structure")
    intent_fields = parse_frames(t_fields[2])
    require(len(intent_fields) == 10, "intent field count")
    require(intent_fields[0] == b"PoUT-research/pre-work-intent/v2", "intent domain")
    require(intent_fields[1] == struct.pack("<I",2), "intent version")
    digest = bytes.fromhex(v["rank_output_hex"])
    require(int.from_bytes(digest,"big") < 1 << 255, "retained half-threshold decision")
    # BLAKE3 is verified by the supplied Rust test, not by this stdlib-only script.
    return {"m19_node_bytes":len(node), "m19_rank_bytes":len(rank),
            "m17_rank_bytes":len(actual), "m17_blake3_executed_here":False}


def main() -> None:
    report = {"m14":m14(), "m18":m18(), "public_byte_vectors":golden_encodings(),
              "rust_executed":False, "consensus_eligibility":"NotEstablished"}
    output = json.dumps(report, indent=2, sort_keys=True)
    require(len(output.encode()) <= 65536, "report budget")
    print(output)


if __name__ == "__main__":
    main()
