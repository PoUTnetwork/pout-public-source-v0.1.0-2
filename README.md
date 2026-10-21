# PoUT Network - Public Research Prototype

Selected, independently inspectable research components for Proof of Useful Traffic.
This repository is not a production blockchain, a running service or a verified
useful-traffic consensus mechanism. The present update adds reproducible finite
models and public test vectors; it does not activate consensus or token migration.

## Included components

| Area | Public contents | Important boundary |
| --- | --- | --- |
| CWC execution | Rust serial reference and bounded parallel candidate execution | Not BlockDAG ordering or consensus admission |
| Cryptographic verification | Groth16 verification over BLS12-381 and public synthetic fixtures | No proving circuit, setup generator or prover in this export |
| Accounting controls | Finite challenge, expiry, replay and key-family models | Assumed authorities are not independent demand |
| Requested bytes and budgets | Rust integrity checks, local Unix-stream example and shared byte-budget controls | Installed quotas do not attest physical links or independent storage |
| Finite information models | Standard-library Python programs, exact tables and independent cross-checks | Four-bit/bucket games are not a resource-hardness proof |
| Blockchain directory | Publication-boundary notices for selected research areas | READMEs only; no executable blockchain code is provided there |

## Reproduce the new public dataset

Python 3.10 or newer; no extra packages, account, credentials or network required:

```sh
python3 -I -B scripts/check_research_pack.py
python3 -I -B -O scripts/check_research_pack.py
```

These commands run 52 Python unit/mutation tests, check all 65,536 one-bit encoder
rows, independently recompute the finite reference tables, and verify 16 public
synthetic byte-frame cases. The two invocations repeat the same tests; their
counts must not be added. Mandatory checks remain active under optimization.

The first-hit dataset is a small fixed bucket-advice model. The repeated-audit
dataset evaluates one explicitly fixed two-table strategy. Neither is a release
of the complete protocol construction or a general attack optimizer.

To regenerate the mathematical fixture bytes in a new directory:

```sh
python3 -I -B research/finite-models/generate.py /tmp/pout-public-finite-new
```

An existing output directory is refused. Inspect [finite model scope](docs/FINITE_MODELS.md),
[data schemas](tests/fixtures/README.md) and [exact validation scope](docs/VALIDATION_2026-10-21.md).

## Rust workspace

The existing manifests declare Rust 1.88 or newer. This update does not modify
Rust source, Cargo manifests or Cargo.lock, and adds no dependencies.
A clean Rust build normally downloads the public packages in Cargo.lock.

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --workspace
cargo test --locked -p pout-cwc-research --all-features --all-targets
cargo run --locked -p pout-cwc-research --example demo
cargo run --locked -p pout-public-verifier --example verify_fixtures
cargo run --locked -p pout-public-verifier --example resource_controls
```

The original v0.1.0 Rust validation remains scoped to that historical release.
The later adapted byte-observation Rust export still needs its own complete Rust
validation. The 968-test engineering-workspace result is not a result for this
public workspace and is not used to certify it. No new Rust execution or hosted
CI success is claimed by this package. There is no included Actions workflow.

## Byte-observation example

The public fixture is deterministic and deliberately compressible. It is not
secret, unpredictable, production-sized storage evidence or real customer data.
The existing example performs local IPC, not an internet throughput benchmark.

```sh
python3 -I -B scripts/check_observation.py --self-test
cargo test --locked -p pout-public-verifier --test service_observation --test byte_budget
cargo run --locked -p pout-public-verifier --example service_observation -- /tmp/pout-public-observation-new
python3 -I -B scripts/check_observation.py --check-demo /tmp/pout-public-observation-new
```

Use a new directory under a controlled parent. The last command requires outputs
from an actual Rust run. The bundled `.frame` files are generated fixtures, not
captures falsely presented as an executed Rust or network experiment.

## Integrity and publication scope

```sh
python3 -I -B scripts/check_source_manifest.py
```

Run this on the complete merged repository, not on the update overlay alone.
The source manifest excludes itself. All inherited license notices must remain.
[Release scope](docs/PUBLIC_RELEASE_SCOPE.md) explains which components are included
and withheld. The `blockchain/` notices are not evidence of deployed modules.

## Further reading

- [Architecture](docs/ARCHITECTURE.md)
- [CWC model](docs/CWC_MODEL.md)
- [Public verification format](docs/PUBLIC_VERIFICATION.md)
- [Byte observation](docs/RESOURCE_OBSERVATION.md)
- [Finite models](docs/FINITE_MODELS.md)
- [Research status](docs/RESEARCH_STATUS.md)
- [Security limits](docs/SECURITY.md)
- [Third-party licenses](THIRD_PARTY_LICENSES.md)

Project source is offered under MIT OR Apache-2.0. Dependencies retain their own
licenses and required notices. No external audit, mainnet readiness, price,
return, independent-resource certification or token entitlement is established.
