# Research status — v0.1.0

## Implemented and exercised

| Component | Executed coverage |
| --- | --- |
| CWC serial reference and actual scoped parallel workers | Canonical encodings, dependency conflicts, finite independent arithmetic corpus, cancellation and worker failures |
| Public Groth16 verifier | Real pairings on genuine public synthetic proofs, complete external pins, bounded canonical decoding and malicious-input rejection |
| Public proof re-encoding | Same-statement transformed proof verifies but preserves the same opportunity identifier; it is not new scarce work |
| Demonstrations | Both `demo` and `verify_fixtures` execute and check their documented outcomes |

## Release validation

The standalone exported workspace was tested on Linux x86-64, Rust 1.99.0 and Cargo 1.99.0, using two build jobs and an isolated public dependency cache. Formatting, warning-free Clippy, default workspace tests and workspace build succeeded. The release archive was extracted to an independent directory and the same four checks were repeated against that extracted source. This is local validation, not a hosted CI result or independent security audit.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Passed, no warnings |
| `cargo test --locked --workspace` | 41 passed, 0 failed, 0 ignored |
| `cargo build --locked --workspace` | Passed |
| `cargo test --locked -p pout-cwc-research --all-features --all-targets` | 38 passed, 0 failed, 0 ignored |
| `cargo run --locked -p pout-cwc-research --example demo` | Passed |
| `cargo run --locked -p pout-public-verifier --example verify_fixtures` | Passed |

The default set is 21 CWC unit tests, 12 CWC integration tests, 6 verifier unit tests and 2 verifier integration tests. The optional CWC run repeats its 33 default tests and adds five scheduling-control tests: there are **46 distinct test functions** across the configurations, not 79. Documentation test targets and the CWC example test target contain zero tests; both executable examples were separately run.

The bounded CWC enumeration covers 5,265 candidates: 1,458 accepted and 3,807 rejected across its finite state/operation corpus. This is finite model evidence. It is not an all-input correctness proof or a benchmark establishing speedup.

## Remaining research and validation gaps

- `consensus_eligibility = NotEstablished`. Neither receipts nor these proof vectors establish independent useful delivery, economic demand, resource scarcity, Sybil resistance or secure consensus weight.
- No consensus algorithm, GHOSTDAG implementation or integrated blockchain is included. CWC is deterministic intra-candidate execution only.
- The public verifier does not include the proving circuit/setup/prover. Key provenance, circuit semantics and honest setup remain explicit trust assumptions; proof creation is not reproducible from this export alone.
- The cryptographic fixture is finite and synthetic. No production-sized cryptographic resource-soundness or service-demand claim follows from verification.
- CWC has no durable storage, process-crash recovery or process-wide resource admission. Its counters are not financial balances.
- Windows, macOS, Android and iOS runtime tests, private multi-device tests and the declared minimum Rust compiler test have not been executed for this package. Manual iPhone upload does not mean iOS Rust support.
- No independent external audit, formal consensus proof or production-readiness assessment is supplied.

Next technical work should expand independent review of the finite relations and key provenance, adversarial resource-soundness models and execution boundaries before any operational consensus integration.
