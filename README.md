# PoUT Network - Public Research Prototype

Selected research components for Proof of Useful Traffic. The documented
research record extends through M41 and NR05, with the earlier public native
source through M19 retained. This is not a production blockchain, running
mainnet or established useful-resource consensus mechanism.

## This update

The M20-NR05 update contains 27 attributed milestone summaries, two public
mathematical/format notes, nine executable arithmetic/finite-model reference
checks, and actual new run records. Original private native test corpora, full
runtime logs, circuits and proving material are not exported. A summary of a
historical test is not a reproduction of it.

[Chronology](docs/MILESTONES_M20_NR05.md) and
[evidence index](docs/test-evidence-m20-nr05/EVIDENCE_INDEX.md) explain precisely
which results are historical, analytical or newly executable. The new references
are intentionally narrower than the original engineering milestones.

This is an overlay for public baseline
`b5cb49b7fbaf76d4f523dcfe014d57829b438dd4`. Preserve all baseline files not replaced
by this overlay. No Cargo manifest, lockfile, license file, operational source or
native consensus behavior is changed.

## Run the new public checks

Python 3.10 or later; standard library only:

```sh
python3 -I -B research/post-m19/reproduce.py
python3 -I -B -O research/post-m19/reproduce.py
python3 -I -B scripts/check_post_m19_update.py --self-test
```

Normal and optimized execution repeat the same nine references; do not add their
counts. The independent checker self-tests concern file integrity, not research
security. Exact commands, full stdout/stderr, exit codes and hashes are retained
under `docs/test-evidence-m20-nr05/logs/`.

The NR05 checks recompute codec parameters, integer probability certificates,
ideal authentication-error arithmetic and visible-format byte sizes. They do not
prove the finite codec, execute SHA/proof hardware, establish real-time response
bounds or demonstrate resource-derived consensus membership.

## Verify integrity

For the extracted update alone:

```sh
python3 -I -B scripts/check_post_m19_update.py --overlay-only
```

After applying it to the pinned complete public baseline:

```sh
python3 -I -B scripts/check_post_m19_update.py --merged
python3 -I -B scripts/check_source_manifest.py
```

The existing source checker compares the Git tracked-file set when available;
stage the intended new files before that check. The merged source manifest covers
all inherited and new files, excluding itself. The separate update manifest
covers payload files except the two root manifests. Its own hash is included in
the merged source manifest. The original 189-entry baseline manifest is retained
byte-exactly, with its published Git blob ID and SHA-256 checked. Inherited file
hashes were carried from that baseline; all newly supplied payload bytes were
hashed. Full inherited-byte verification requires the merged checkout.

This is a reconstructed update, not a claim of byte identity with an earlier
lost archive. The public update metadata lists the exact payload paths.

## Earlier public references remain available

Run these from the merged repository:

```sh
python3 -I -B research/milestones/reproduce.py
python3 -I -B scripts/check_research_pack.py
cargo test --locked -p pout-public-verifier --test causal_byte_bridge
cargo test --locked -p pout-cwc-research --test rank_public_vector
```

These earlier tools were not newly executed to prepare this reconstruction.
Existing Groth16 material is verification-only with synthetic public fixtures.
No new Rust build/test, rustfmt, Clippy, hosted CI or network campaign is claimed.
Existing Rust minimum/version and platform statements retain their earlier scope.

## Research and security

- [Current status](docs/RESEARCH_STATUS.md)
- [M20-NR05 chronology](docs/MILESTONES_M20_NR05.md)
- [NR05 conditional sampled-batch bound](docs/NR05_SAMPLED_BATCH_BOUND.md)
- [NR05 visible response and costs](docs/NR05_VISIBLE_AUDIT.md)
- [M14-M19 source record](docs/MILESTONES_M14_M19.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Security limitations](docs/SECURITY.md)
- [Dependency notices](THIRD_PARTY_LICENSES.md)

Project source remains offered under MIT OR Apache-2.0. Keep all existing license
and upstream attribution files. Current consensus eligibility is `NotEstablished`
and weight is `None`. No external audit, production readiness, token entitlement,
price or return is established by this research update.
