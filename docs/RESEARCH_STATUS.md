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

## Public accounting-controls research addendum — October 19, 2026

A new standalone `resource_controls` example and 11 integration tests model externally pinned identity/resource scope, half-open freshness windows, replay consumption, permitted key-family rotation and alternative fork-local accounting. The model also preserves a split-family counterexample: different admitted families may count the same assumed resource twice. A new challenge identifier similarly does not prove a new physical resource.

Run `cargo test --locked -p pout-public-verifier --test resource_controls` and `cargo run --locked -p pout-public-verifier --example resource_controls` after applying this addendum. These are **commands to run**, not executed validation claims. The original 41/0/0 release results in the table above apply only to the unchanged v0.1.0 source package. No aggregate updated result is asserted here.

The new model does not call the cryptographic verifier, prove physical storage/bandwidth, authenticate a clock or source label, establish demand, or enforce global fork consumption. Hosted CI for this addendum is unverified.

## Remaining research and validation gaps

- `consensus_eligibility = NotEstablished`. Neither receipts nor these proof vectors establish independent useful delivery, economic demand, resource scarcity, Sybil resistance or secure consensus weight.
- No consensus algorithm, GHOSTDAG implementation or integrated blockchain is included. CWC is deterministic intra-candidate execution only.
- The public verifier does not include the proving circuit/setup/prover. Key provenance, circuit semantics and honest setup remain explicit trust assumptions; proof creation is not reproducible from this export alone.
- The cryptographic fixture is finite and synthetic. No production-sized cryptographic resource-soundness or service-demand claim follows from verification.
- CWC has no durable storage, process-crash recovery or process-wide resource admission. Its counters are not financial balances.
- Windows, macOS, Android and iOS runtime tests, private multi-device tests and the declared minimum Rust compiler test have not been executed for this package. Manual iPhone upload does not mean iOS Rust support.
- No independent external audit, formal consensus proof or production-readiness assessment is supplied.

Next technical work should expand independent review of the finite relations and key provenance, adversarial resource-soundness models and execution boundaries before any operational consensus integration.

## Public byte-observation export — October 20, 2026

The added public files implement selected retrieval framing/content checks, local request-digest consumption and assigned nonrefilling byte budgets. They include 13 retrieval regression definitions and 10 meter regression definitions, plus a five-scenario Unix IPC example. These counts describe source definitions, NOT executed Rust passes.

Packaging checks actually performed:

- Baseline manifest Git-blob identity matched the pinned public commit.
- All replacement-file baselines available in the prior public update matched its SHA-256 entries.
- 12 Python reference self-tests passed in Python 3 in the packaging environment.
- Python known answers were embedded into the Rust fixture regression.
- Updated/new file hashes, archive integrity, allowed paths and targeted sensitive-content checks completed.

Not executed in the packaging environment:

- Rust compilation, formatting, Clippy or workspace tests for this export.
- The new Rust Unix IPC example or Python verification of its actual output.
- Hosted CI or other-platform runtime validation.

The packaging environment has no Rust toolchain. Existing results for other source versions do not validate this adapted export. The included workflow will attempt an exact-commit build, warning check, default/all-feature workspace tests, examples, captured-byte comparison and source-manifest checks after a matching push or manual dispatch. A workflow file is not evidence of a successful run. Any failed check remains a release blocker until diagnosed.

The deterministic fixture can be reconstructed from a short recipe. A metadata-only example intentionally reports unavailable; that is not a lower bound against arbitrary or delegated adversaries. Byte quota conservation is conditional on trusted assignment/time and complete interception of accepted output.
