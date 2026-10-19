# PoUT Network — Public Research Prototype

PoUT Network is an experimental Rust-based infrastructure and cryptographic research project. This repository provides selected prototype components for technical inspection and reproducible testing. It does not represent a completed or production-ready blockchain.

This v0.1.0 source package includes two original independently runnable research components and a separate public research addendum:

- **CWC execution:** a serial reference and actual bounded, multithreaded execution of artificial counter operations. Tests compare validity, operation results, canonical state bytes and BLAKE3 roots, including conflicting operations and handled worker failures.
- **Public cryptographic verification:** a bounded Groth16 verifier over BLS12-381, retained public synthetic proof vectors, authenticated-input pin checks, malformed-input tests and adversarial public proof transformations. Verification performs actual pairing checks; it is not a simulated acceptance function.
- **Bounded accounting-control research:** a separate synthetic Rust experiment for challenge freshness, explicit issuer/provider/resource identifiers, key-family rotation, replay consumption and alternative fork views. This model does not invoke Groth16 or prove resource scarcity.

No working blockchain, network service, wallet, currency, mining mechanism, operational accounting or secure useful-traffic consensus is included. Resource scarcity, useful delivery, independent customer demand and consensus eligibility remain **NotEstablished**. CWC is intra-candidate execution, not BlockDAG ordering.

## Build and test

Install a stable Rust toolchain with Cargo, rustfmt and Clippy, plus a C compiler for dependencies. The manifests declare Rust 1.88 or newer; release validation used Rust 1.99.0 on Linux x86-64. The minimum compiler and other operating systems have not been tested. A fresh build normally needs internet access to download public crates.io dependencies. The lockfile fixes their versions and checksums. No project account, credential, private data or proving key is needed.

Run these commands from the directory containing this README and Cargo.toml:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --workspace
```

Run the executable demonstrations:

```sh
cargo run --locked -p pout-cwc-research --example demo
cargo run --locked -p pout-public-verifier --example verify_fixtures
cargo run --locked -p pout-public-verifier --example resource_controls
```

Exercise the optional bounded scheduling controls:

```sh
cargo test --locked -p pout-cwc-research --all-features --all-targets
```

Tests include expected, caught worker-panic cases. A successful test result for those cases checks a local abort; it does not declare the candidate invalid. No live network, real assets or file-transfer workload is involved.

## Inspect the implementation

| Location | Contents |
| --- | --- |
| `crates/pout-cwc-research` | Typed model, canonical encoding, serial execution, dependency graph, scoped workers and correctness corpus |
| `crates/pout-public-verifier` | Pairing verifier, bounded bundle importer, public vectors, negative tests, synthetic accounting controls and verification examples |
| [Architecture](docs/ARCHITECTURE.md) | Components, trust boundaries and deliberately excluded systems |
| [CWC model](docs/CWC_MODEL.md) | Operation semantics, conflict rules, bounds and limitations |
| [Verification format](docs/PUBLIC_VERIFICATION.md) | Public evidence framing and authority assumptions |
| [Research status](docs/RESEARCH_STATUS.md) | Release scope and executed validation |
| [Security limitations](docs/SECURITY.md) | What the prototype does and does not establish |
| [Dependency licenses](THIRD_PARTY_LICENSES.md) | Exact locked dependencies and authentic upstream notices |

The verifier-only export lets readers rerun verification and adversarial checks. It does not contain the proving circuit, setup generator or prover, and cannot reproduce proof creation or independently establish which circuit the pinned key represents. That is an explicit review limitation. The public fixture pins are controlled test constants, not an independent production authority or a reviewed trusted setup.

## Public research addendum — October 19, 2026

The additional `resource_controls` example and integration tests are independent finite-policy research. They assume externally authenticated time, issuer, key-family authorization and resource anchors. They perform no real Groth16 verification, network operations or resource measurements. Unlike the v0.1.0 release results, the new tests require separate execution after applying this update; see [research status](docs/RESEARCH_STATUS.md).

## License

Project source is offered under MIT OR Apache-2.0; see [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE). Dependencies retain their own licenses and required third-party notices. No independent security audit or production readiness is claimed.
