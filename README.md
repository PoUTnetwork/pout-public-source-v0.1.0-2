# PoUT Network - Public Research Prototype

Selected research components for Proof of Useful Traffic, with the publication
boundary extended through M19. This is not a production blockchain, running
service, or verified useful-traffic consensus mechanism.

This is a source update for the existing repository. Earlier public CWC,
verification, byte-observation and finite-model files remain in place. No Cargo
manifest, dependency lockfile, license file or operational service is changed.

## What this update exposes

| Research area | Included public material | Boundary |
| --- | --- | --- |
| M14 distinct opportunities | Executable finite reference for sixteen small profiles | Public adaptation; not the complete engineering optimizer or DAG corpus |
| M15 exact-intent binding | Field-level specification and a retained exact-intent/target vector | Full native resource relation is not included |
| M16 rank/work composition | Conditional bound, separated accounting and reported scope | Full 21,086-case native research corpus is not reproduced by this package |
| M17 Rank/threshold | Retained full-pair input and BLAKE3 digest; Rust digest check | Does not regenerate the resource witness or verify threshold authority |
| M18 helper-selected targets | Executable 512-world quotient and 64-world conditioning references | Defined ideal laws and symbolic costs, not physical-resource measurements |
| M19 causal history | Rust byte machine, semantic reference, eight inspections and golden encodings | Scripted finite replies; no native hash randomness or global observation proof |

[Milestone details](docs/MILESTONES_M14_M19.md) distinguish reported engineering
results from material that can actually be reproduced in this public workspace.
The complete proving circuit, setup generator, prover and operational stack
remain outside the publication boundary. Later milestones are not included.

## Review the selected references

Python 3.10 or later, standard library only:

```sh
python3 -I -B research/milestones/reproduce.py
python3 -I -B -O research/milestones/reproduce.py
```

This recomputes selected M14 and M18 finite probabilities, checks M19 byte goldens,
and reconstructs the retained M17 Rank input. It does not execute Rust or BLAKE3.
The two invocations repeat the same references; do not add their counts.

The existing public finite dataset remains available:

```sh
python3 -I -B scripts/check_research_pack.py
```

## Rust commands for downstream review

Run these from the merged repository, not the update ZIP alone:

```sh
cargo test --locked -p pout-public-verifier --test causal_byte_bridge
cargo run --locked -p pout-public-verifier --example causal_byte_bridge
cargo test --locked -p pout-cwc-research --test rank_public_vector
```

The M19 experiment logic and eight inspection/guard tests come from the M19
source snapshot. Serialization-only adaptations avoid adding dependencies:
Serde derives are omitted and bounded text output replaces JSON output.
[Exact export changes](docs/VALIDATION_M19_PUBLIC.md) are recorded separately.

No Rust build, Rust test suite, rustfmt, Clippy or hosted workflow was run to
prepare this update. The reported 1,065-test engineering result is historical
and is not a validation result for this adapted public workspace.

The existing manifests declare Rust 1.88 or newer. That minimum and other
platforms have not been newly tested for this update. Normal clean builds may
need public dependency downloads specified by Cargo.lock.

## Existing demonstrations

```sh
cargo run --locked -p pout-cwc-research --example demo
cargo run --locked -p pout-public-verifier --example verify_fixtures
cargo run --locked -p pout-public-verifier --example resource_controls
```

Existing service-observation material uses local IPC, not internet-bandwidth
measurement. Existing Groth16 material is verification-only, with synthetic
public fixtures; it does not expose the proving circuit or establish scarce work.
The `blockchain/` directories contain publication-boundary notices, not a deployed
blockchain implementation.

## Integrity

```sh
python3 -I -B scripts/check_source_manifest.py
```

Run on the complete merged repository. The updated manifest covers the inherited
files and this overlay, excluding the manifest itself. Inherited entries were
carried from the published baseline; new/changed files were hashed for this ZIP.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [CWC model](docs/CWC_MODEL.md)
- [Verification format](docs/PUBLIC_VERIFICATION.md)
- [Existing byte observation](docs/RESOURCE_OBSERVATION.md)
- [Earlier finite models](docs/FINITE_MODELS.md)
- [M14-M19 research](docs/MILESTONES_M14_M19.md)
- [M19 public validation boundary](docs/VALIDATION_M19_PUBLIC.md)
- [Research status](docs/RESEARCH_STATUS.md)
- [Security limits](docs/SECURITY.md)
- [Publication scope](docs/PUBLIC_RELEASE_SCOPE.md)
- [Dependency licenses](THIRD_PARTY_LICENSES.md)

Project source is offered under MIT OR Apache-2.0. Existing license and upstream
attribution files must remain. No external audit, production readiness, resource
scarcity, token entitlement, price or return is established by this package.
