# Reproducible examples

Run from the workspace root:

```sh
cargo run --locked -p pout-cwc-research --example demo
cargo run --locked -p pout-public-verifier --example verify_fixtures
```

Both commands execute real Rust implementations and fail with a nonzero result if their assertions or verification checks fail. All inputs are synthetic/public and in-memory. No service, private identity, user file or proving key is required.

CWC compares the serial reference with 1, 2 and 4 real workers, prints the accepted state/root, rejects shared-reserve overdraw, and demonstrates cancellation followed by retry. The verifier example performs genuine pairing checks, distinguishes two statements, rejects a false source binding and accepts its truthful alternate.

Exact expected behavior and limitations are in [CWC model](../docs/CWC_MODEL.md) and [Public verification](../docs/PUBLIC_VERIFICATION.md). These examples are correctness demonstrations, not throughput benchmarks or evidence of consensus security.
