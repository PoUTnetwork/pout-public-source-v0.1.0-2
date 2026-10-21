# Reproducible examples

Run from the workspace root:

```sh
cargo run --locked -p pout-cwc-research --example demo
cargo run --locked -p pout-public-verifier --example verify_fixtures
cargo run --locked -p pout-public-verifier --example resource_controls
```

The original CWC and verifier commands execute Rust implementations and fail with a nonzero result if their assertions or verification checks fail. Inputs of these original examples are synthetic/public and in-memory. No service, private identity, user file or proving key is required.

CWC compares the serial reference with 1, 2 and 4 real workers, prints the accepted state/root, rejects shared-reserve overdraw, and demonstrates cancellation followed by retry. The verifier example performs genuine pairing checks, distinguishes two statements, rejects a false source binding and accepts its truthful alternate.

Exact expected behavior and limitations are in [CWC model](../docs/CWC_MODEL.md) and [Public verification](../docs/PUBLIC_VERIFICATION.md). These examples are correctness demonstrations, not throughput benchmarks or evidence of consensus security.

The `resource_controls` example exercises an independent, synthetic in-memory accounting policy. It assumes valid evidence and trusted time/IDs; it does not execute Groth16, query network services or establish physical resource scarcity. Its new command must be run after installing this addendum before reporting a pass.

## Deterministic requested-byte and quota example

```sh
cargo run --locked -p pout-public-verifier --example service_observation -- /tmp/pout-public-observation-new
python3 -I -B scripts/check_observation.py --check-demo /tmp/pout-public-observation-new
```

Use a new output directory under a stable, existing parent. The Rust example refuses to overwrite previous output. It writes two 8 MiB synthetic sources, bounded response artifacts and a TSV report; it does not delete them. This example requires UnixStream support. All bytes are generated from a public deterministic recipe, not from user data.

The five declared scenarios expect 8 verified responses and 4 quota denials in total. These are expected outcomes, not a report of an execution in the packaging environment. The Python checker independently compares captured frames and the matching two-file/shared-source outputs after an actual Rust run.

`python3 -I -B scripts/check_observation.py --self-test` runs only Python reference controls. It does not execute Rust or establish its results. See [resource observation](../docs/RESOURCE_OBSERVATION.md).


## New offline finite-reference pack

```sh
python3 -I -B scripts/check_research_pack.py
```

This command needs only Python 3.10+. It runs fixed unit/mutation tests and
verifies the released synthetic mathematical/byte fixtures. It does not launch
a node, Rust example, API connection or proof generator. See
[finite models](../docs/FINITE_MODELS.md) for assumptions.
