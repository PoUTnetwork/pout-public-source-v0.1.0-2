# PoUT Network - Public Research Prototype

Selected research components for Proof of Useful Traffic. The public record now
includes selected research continuity from NR06 through NR22, the NE-01
storage-power test-model evidence, and the NE-02 S1 adversarial storage/time
baseline. The earlier native public source remains bounded by M19; this is not
a production blockchain, a live mainnet, or an independently secured consensus
mechanism.

## Research extension - NE-01 and NE-02 S1 (9 November 2026)

The [NE-01 / NE-02 S1 public release](docs/PUBLIC_NE01_NE02_S1_RELEASE.md)
provides separately scoped research records and supporting public evidence:

- NE-01: controlled eligibility evaluation reported `Established` with model
  weight 512 across eight audited epochs, 16 passing audits and 64 passing
  focused tests. This applies only to the pinned research test model.
- NE-02 S1: a successful storage-regeneration attack showed that the initial
  seal does **not** enforce a physical storage/time scarcity margin. The
  test-model's positive eligibility result is therefore not production security.
- Selected NR06-NR22 chronology documents research direction, limitations and
  negative findings; the underlying private Rust corpus is not reproduced.
- The S1 release-result JSON preserves a previously recorded source execution.
  Public verification scripts validate hashes, internal accounting and recorded
  claims; they are **not** a fresh native or independently hosted execution.

A complete public-source SHA-256 manifest includes this extension. Prior M20-NR05
historical evidence, the public Rust/Cargo workspace, dependency licenses and
original fixtures remain unchanged except the public documentation and manifest
checker updated for the new research stage.

## Run public research checks

Python 3.10+ (standard library only):

```sh
python3 -I -B research/post-m19/reproduce.py
python3 -I -B -O research/post-m19/reproduce.py
python3 -I -B scripts/check_post_m19_update.py --self-test
python3 -I -B scripts/check_ne02_s1_public.py
python3 -I -B docs/test-evidence-ne01-s1/ne01/verify_evidence.py
python3 -I -B research/post-nr05/verify_ne02_s1_record.py
python3 -I -B -m unittest discover -s tests/public -p test_ne02_s1_record.py
```

Normal and optimized runs of the older public model repeat the same nine
reference checks, not eighteen distinct experiments. The M20-NR05 record is
historical and contains synthetic finite-model arithmetic; it does not
reproduce the unavailable private native prover or network.

## Verify the merged checkout

After all files are applied at their repository-relative paths, and **after the
new files are staged in Git**, run:

```sh
python3 -I -B scripts/check_post_m19_update.py --merged
python3 -I -B scripts/check_source_manifest.py
python3 -I -B scripts/check_ne02_s1_public.py
```

`SOURCE_MANIFEST.sha256` covers every tracked public file except itself.
`UPDATE_MANIFEST.sha256` still records the revised October extension, while
`NE02_S1_UPDATE_MANIFEST.sha256` covers the new November addition. The historical
189-entry baseline is retained verbatim. All source bytes not shipped in this
November extension keep their published October hashes, so final verification
must be run against the full merged public repository, not the ZIP alone.

## Earlier public references remain available

```sh
python3 -I -B research/milestones/reproduce.py
python3 -I -B scripts/check_research_pack.py
cargo test --locked -p pout-public-verifier --test causal_byte_bridge
cargo test --locked -p pout-cwc-research --test rank_public_vector
```

These inherited references were not rerun while preparing this package.
Verification-only Groth16 synthetic material does not imply a public proving
circuit, production consensus, or secure network admission.

## Research and security

- [Current research status](docs/RESEARCH_STATUS.md)
- [NE-01 / NE-02 S1 public release](docs/PUBLIC_NE01_NE02_S1_RELEASE.md)
- [NE-01 consensus test model](docs/NE01_CONSENSUS_TEST_MODEL.md)
- [NE-02 S1: storage/time baseline](docs/NE02_S1_STORAGE_TIME_MARGIN.md)
- [NR06-NR22 chronology](docs/RESEARCH_NR06_NR22_CONTINUITY.md)
- [M20-NR05 research chronology](docs/MILESTONES_M20_NR05.md)
- [Historical evidence index](docs/test-evidence-m20-nr05/EVIDENCE_INDEX.md)
- [NR05 conditional sampled-batch bound](docs/NR05_SAMPLED_BATCH_BOUND.md)
- [NR05 visible response accounting](docs/NR05_VISIBLE_AUDIT.md)
- [M14-M19 public source record](docs/MILESTONES_M14_M19.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Security limitations](docs/SECURITY.md)
- [Third-party attribution](THIRD_PARTY_LICENSES.md)

Project source remains available under MIT OR Apache-2.0. Keep all existing
license and upstream attribution files. Root consensus eligibility remains
`NotEstablished` with weight `None` outside the scoped NE-01 test model. No
external security audit, production readiness, native token entitlement, price
or return is established by this update.
