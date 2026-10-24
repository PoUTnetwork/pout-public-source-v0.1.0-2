# Public synthetic milestone fixtures

- `m17-half-rank.json`: selected original synthetic M17 half-policy input fields,
  complete Rank input, recorded digest, threshold and expected decision.
- `m17-half-rank.frame`: exact Rank input reconstructed from those retained fields.
- `m19-node-index2.frame`: 241-byte independently framed Node golden.
- `m19-rank-vector.frame`: 438-byte independently framed full-vector Rank golden.

These are deliberately public synthetic values, not signing keys, credentials,
user content or captures of real customer traffic. M19 fixtures are reproduced by
`research/milestones/reproduce.py` and compared to the original test literals.
M17 fixture reproduction checks bytes only in Python; BLAKE3 is a downstream Rust
check. No file here is a physical-resource certificate or a usable live asset.
