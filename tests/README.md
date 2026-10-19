# Test layout

Executable tests are in the crates so Cargo discovers them without an extra root package:

- `crates/pout-cwc-research/src`: unit tests for codecs, actual workers, graph/wave scheduling, serial outcomes and optional controlled completion ordering.
- `crates/pout-cwc-research/tests/correctness.rs`: independent arithmetic oracle, 5,265-case bounded corpus, fixed-seed corpus, conflicts, malformed encodings and finite boundary cases.
- `crates/pout-public-verifier/src/public_verification_tests.rs`: six unit tests retaining parser-private checks, genuine acceptance, independent pins, invalid group encodings, key substitutions, public re-encoding and cancellation.
- `crates/pout-public-verifier/tests/public_verification.rs`: two public-API integration tests for genuine verification/replay identity, changed authority, false tags, malformed wire and local aborts.
- `crates/pout-public-verifier/tests/resource_controls.rs`: 11 new synthetic-only accounting-model tests for freshness, externally pinned identities, replay and key families, bounded entries and fork-alternative views. These have not yet been executed in this published addendum.

Run `cargo test --locked --workspace` for default tests. Run `cargo test --locked -p pout-cwc-research --all-features --all-targets` for the optional scheduling controls. These sets overlap; their totals must not be added as if every invocation were a distinct test. See [Research status](../docs/RESEARCH_STATUS.md) for actual executed results.
