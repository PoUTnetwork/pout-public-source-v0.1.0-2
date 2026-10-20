# Test layout

Executable tests are in the crates so Cargo discovers them without an extra root package:

- `crates/pout-cwc-research/src`: unit tests for codecs, actual workers, graph/wave scheduling, serial outcomes and optional controlled completion ordering.
- `crates/pout-cwc-research/tests/correctness.rs`: independent arithmetic oracle, 5,265-case bounded corpus, fixed-seed corpus, conflicts, malformed encodings and finite boundary cases.
- `crates/pout-public-verifier/src/public_verification_tests.rs`: six unit tests retaining parser-private checks, genuine acceptance, independent pins, invalid group encodings, key substitutions, public re-encoding and cancellation.
- `crates/pout-public-verifier/tests/public_verification.rs`: two public-API integration tests for genuine verification/replay identity, changed authority, false tags, malformed wire and local aborts.
- `crates/pout-public-verifier/tests/resource_controls.rs`: 11 new synthetic-only accounting-model tests for freshness, externally pinned identities, replay and key families, bounded entries and fork-alternative views. These have not yet been executed in this published addendum.

Run `cargo test --locked --workspace` for default tests. Run `cargo test --locked -p pout-cwc-research --all-features --all-targets` for the optional scheduling controls. These sets overlap; their totals must not be added as if every invocation were a distinct test. See [Research status](../docs/RESEARCH_STATUS.md) for actual executed results.

## Byte-observation regression definitions

`crates/pout-public-verifier/tests/service_observation.rs` contains 13 test definitions for independent known answers, exact response framing, corruption, stale contexts, cached data, object pins, unavailable responses, local consumption, bounds and actual synthetic-file reading.

`crates/pout-public-verifier/tests/byte_budget.rs` contains 10 definitions for atomic contention, spent-byte conservation, partial/drop behavior, cancellation/expiry, late writes, ambiguous errors, panic unwinding, unlocked I/O and explicitly assigned second-meter capacity.

```sh
cargo test --locked -p pout-public-verifier --test service_observation --test byte_budget
python3 -I -B scripts/check_observation.py --self-test
```

The 23 new Rust tests have not been executed in the packaging environment. The 12 Python reference tests have been executed; they are not Rust passes. All totals in earlier sections remain scoped to their recorded source versions. A successful future CI run should be reported with its exact public commit and actual test totals, without adding overlapping default/all-feature invocations.

The included CI workflow checks compilation, warning-free Clippy, tests, examples, byte-reference agreement and manifest integrity. It does not provide an external audit or a physical-resource benchmark. Code-format conformance may additionally be inspected with `cargo fmt --all -- --check`; no formatting pass is claimed here.
