# Finite reference programs

Run from the repository root:

```sh
python3 -I -B scripts/check_research_pack.py
```

The generator and checker use Python 3.10+ standard libraries only. No environment
configuration is necessary. See ../../docs/FINITE_MODELS.md for exact assumptions.

Files:
- models.py: bounded mathematical operations and strict report controls.
- generate.py: write canonical data into a NEW directory.
- verify.py: separately recompute all released finite rows.

The tests and public data live under tests/public/ and tests/fixtures/. These are
isolated models, not operational blockchain components.
