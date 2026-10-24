# Reproducible public examples

Run from the complete merged repository:

```sh
cargo run --locked -p pout-cwc-research --example demo
cargo run --locked -p pout-public-verifier --example verify_fixtures
cargo run --locked -p pout-public-verifier --example resource_controls
cargo run --locked -p pout-public-verifier --example causal_byte_bridge
```

The first three are inherited demonstrations. The new causal_byte_bridge example
checks two public golden encodings, runs the eight-trace M19 model and prints a
bounded text report. It does not contact a network or create any asset.

Earlier local service-observation instructions remain in
[RESOURCE_OBSERVATION](../docs/RESOURCE_OBSERVATION.md).
The optional CWC scheduling controls remain separate.

The compact finite references need only Python's standard library:

```sh
python3 -I -B research/milestones/reproduce.py
```

These commands are reproduction instructions, not claims that Rust was run while
preparing this update. See [validation](../docs/VALIDATION_M19_PUBLIC.md).
