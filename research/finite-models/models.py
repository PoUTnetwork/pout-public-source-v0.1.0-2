"""Small exact public information games. No network, cryptographic or consensus API.

The four-bit game is intentionally finite. A one-bit encoder is chosen before a
uniform challenge and all advice belongs to one coalition. Fractions are exact.
"""
from __future__ import annotations

from collections import Counter
from fractions import Fraction
from itertools import product

STATES = tuple(range(16))
CHALLENGES = tuple(range(4))
FULL = (1 << 16) - 1


def require(condition: bool, message: str) -> None:
    """Unlike a language assertion, this check also runs under python -O."""
    if not condition:
        raise ValueError(message)


def integer(value: object, lower: int, upper: int, name: str) -> int:
    require(type(value) is int and lower <= value <= upper, f"invalid {name}")
    return value


def target(source: int, challenge: int) -> int:
    integer(source, 0, 15, "source")
    integer(challenge, 0, 3, "challenge")
    left, right = challenge & 1, 2 + (challenge >> 1)
    return ((source >> left) & 1) | (((source >> right) & 1) << 1)


PAIR_MASKS = tuple(tuple(sum(1 << x for x in STATES if target(x, c) == pair)
                              for pair in range(4)) for c in CHALLENGES)
QUERY_MASKS = tuple(tuple(sum(1 << x for x in STATES if (x >> k) & 1 == bit)
                               for bit in range(2)) for k in range(4))


def encoder_scores(encoder: int) -> tuple[int, int]:
    """Posterior-count optimization. Q1 checks all four possible bit queries.

    Returns success numerators with denominator 64. Advice is encoder[x].
    For each advice/challenge the decoder may choose its query and response.
    No free public dependent data or retained earlier replies exist in this game.
    """
    integer(encoder, 0, FULL, "encoder")
    q0 = q1 = 0
    for cell in (encoder, encoder ^ FULL):
        for masks in PAIR_MASKS:
            q0 += max((cell & mask).bit_count() for mask in masks)
            best = 0
            for zero, one in QUERY_MASKS:
                trial = max((cell & zero & mask).bit_count() for mask in masks)
                trial += max((cell & one & mask).bit_count() for mask in masks)
                best = max(best, trial)
            q1 += best
    return q0, q1


def fourier_q1(encoder: int) -> int:
    """Independent algebraic calculation; does not call posterior scoring."""
    integer(encoder, 0, FULL, "encoder")
    coefficients = [1 if not (encoder >> x) & 1 else -1 for x in STATES]
    width = 1
    while width < 16:
        for base in range(0, 16, width * 2):
            for k in range(width):
                a, b = coefficients[base + k], coefficients[base + k + width]
                coefficients[base + k] = a + b
                coefficients[base + k + width] = a - b
        width *= 2
    bonus = sum(max(abs(coefficients[1 << (c & 1)]),
                    abs(coefficients[1 << (2 + (c >> 1))]),
                    abs(coefficients[(1 << (c & 1)) | (1 << (2 + (c >> 1)))]))
                for c in CHALLENGES)
    require(bonus % 2 == 0, "noninteger Fourier numerator")
    return 32 + bonus // 2


def response_table_score(table: int, source: int) -> int:
    integer(table, 0, 255, "response table")
    integer(source, 0, 15, "source")
    return sum(((table >> (2*c)) & 3) == target(source, c) for c in CHALLENGES)


def fixed_book_repetition(rounds: int, book: tuple[int, ...] = (0, 255)) -> Fraction:
    """Success of this fixed two-table strategy, NOT an optimizer over all policies.

    One table is selected using the source before challenges. Independent
    challenges use the same unchanged source. No feedback or new queries.
    """
    integer(rounds, 1, 12, "rounds")
    require(1 <= len(book) <= 2 and len(set(book)) == len(book), "invalid book")
    for table in book:
        integer(table, 0, 255, "table")
    numerator = sum(max(response_table_score(table, x) for table in book) ** rounds
                    for x in STATES)
    return Fraction(numerator, 16 * 4**rounds)


def fixed_book_formula(rounds: int) -> Fraction:
    integer(rounds, 1, 12, "rounds")
    return Fraction(1, 8) + Fraction(1, 2**(rounds+1)) + Fraction(1, 2**(2*rounds+2))


def guessing(counts: list[int]) -> Fraction:
    require(bool(counts) and all(type(n) is int and n >= 0 for n in counts), "bad counts")
    total = sum(counts)
    require(total > 0, "empty distribution")
    return Fraction(max(counts), total)


def top_mass(counts: list[int], limit: int) -> Fraction:
    guessing(counts)
    integer(limit, 0, len(counts), "output alphabet size")
    return Fraction(sum(sorted(counts, reverse=True)[:limit]), sum(counts))


def conditional_guessing(worlds: list[tuple[object, object]]) -> Fraction:
    """Average best guess for a uniform list of (target, public-view) worlds."""
    require(bool(worlds), "no worlds")
    cells: dict[object, Counter] = {}
    for target_value, view in worlds:
        cells.setdefault(view, Counter())[target_value] += 1
    return Fraction(sum(max(cell.values()) for cell in cells.values()), len(worlds))


def gf4_mul(a: int, b: int) -> int:
    integer(a, 0, 3, "GF4 element")
    integer(b, 0, 3, "GF4 element")
    result = 0
    while b:
        if b & 1:
            result ^= a
        b >>= 1
        a <<= 1
        if a & 4:
            a ^= 0b111
    return result


def gf4_worlds() -> list[tuple[int, ...]]:
    return [(a, b, *(gf4_mul(a, i) ^ b for i in range(4)))
            for a, b in product(range(4), repeat=2)]


def bucket_first_hit(q: int, advice_states: int, probes: int) -> tuple[int, int]:
    """Fixed source-only bucket advice and increasing guesses, not a random oracle.

    The advice identifies an equal bucket. All probes are charged globally.
    This enumerates q possible hidden targets for this specific strategy only.
    """
    integer(q, 2, 32, "alphabet size")
    integer(advice_states, 1, q, "advice states")
    integer(probes, 0, q, "probes")
    require(q % advice_states == 0, "unequal buckets not specified")
    width = q // advice_states
    hit = 0
    for x in range(q):
        start = (x // width) * width
        hit += x in range(start, start + min(width, probes))
    return hit, q


def validate_control_report(report: object) -> None:
    """Require every control, including negatives. Empty all([]) is not success."""
    require(type(report) is dict, "report must be an object")
    required = {"schema", "complete", "controls", "consensus_eligibility", "consensus_weight"}
    require(set(report) == required, "unexpected or missing report fields")
    require(type(report["schema"]) is int and report["schema"] == 1, "bad report schema")
    require(report["complete"] is True, "incomplete report")
    require(report["consensus_eligibility"] == "NotEstablished" and report["consensus_weight"] is None,
            "invalid eligibility claim")
    controls = report["controls"]
    require(type(controls) is dict, "controls must be an object")
    require(set(controls) == {"wrong_context_rejected", "empty_table_rejected",
                              "truncated_table_rejected", "duplicate_record_rejected"},
            "control set mismatch")
    require(all(value is True for value in controls.values()), "control did not pass")
