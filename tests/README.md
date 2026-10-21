# Public tests and test-data scope

## New finite-reference pack

```sh
python3 -I -B scripts/check_research_pack.py --json
python3 -I -B -O scripts/check_research_pack.py --json
```

52 unique Python unit/mutation tests run: 40 in `tests/public/test_finite_models.py`
and the 12 previously published byte-reference tests. The second invocation
checks the same suite in optimized mode; it does not double the total.

In addition, the runner recomputes all 65,536 advice-encoder rows, the other finite
tables, 128 request digests, 32 chunk hashes and sixteen frame cases. These records
are not extra test functions. See [fixture schemas](fixtures/README.md).

Negative controls include missing/empty/extra report controls, integer 1 replacing
boolean true, unexpected eligibility or weight, wrong CSV schema, truncation,
ambiguous integers, incorrect frames, stale request echoes and wrong objects.
Checks use explicit exceptions rather than assertions that optimization removes.

## Existing Rust tests

The original v0.1.0 package recorded 41 default workspace passes. Its optional
CWC configuration overlapped those tests; the original combined distinct count
was 46, not the sum of invocations. That historical record does not certify later
addenda. The accounting addendum added 11 integration-test definitions; the byte
observation addendum added 23 Rust test definitions, with their public Rust
execution still pending. This update changes none of that Rust code.

```sh
cargo test --locked --workspace
cargo test --locked -p pout-cwc-research --all-features --all-targets
cargo test --locked -p pout-public-verifier --test resource_controls
cargo test --locked -p pout-public-verifier --test service_observation --test byte_budget
```

No automatic GitHub Actions workflow is included. Run the documented commands in
a compatible build environment and record actual results before claiming a new
Rust validation. A passing finite reference is not an audit or a consensus proof.
