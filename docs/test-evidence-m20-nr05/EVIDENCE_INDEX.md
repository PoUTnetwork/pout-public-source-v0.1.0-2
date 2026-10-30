# Evidence index - M20 through NR05

This directory distinguishes three different kinds of evidence.

## 1. Source-reported historical work

`milestones/M20.md` through `milestones/M41.md` and `milestones/NR01.md` through
`milestones/NR05.md` give 27 English summaries. `HISTORICAL_RESULTS.json` provides
the same structured record. `SOURCE_REGISTER.json` records the retrieved source
content identifiers and the public baseline.

These summaries are not raw compiler logs, complete original execution receipts,
or a public reproduction of the original Rust/circuit/consensus experiments.
The full original native corpus, cryptographic setup/proving material and raw
runtime evidence are not exported here. Reported counts remain attributed,
non-additive historical results. A documentation-only stage has no invented
native execution, and custody checks are not counted as mathematical tests.

## 2. Newly executed public reference checks

Run from the extracted update or the merged repository with Python 3.10 or later:

```sh
python3 -I -B research/post-m19/reproduce.py
python3 -I -B -O research/post-m19/reproduce.py
```

The script uses only the standard library, fixed parameters and explicit
exceptions rather than assertions that disappear under optimization. It does
not download anything or access a wallet, network, proof backend or private data.

| ID | Recomputed check | What it does not reproduce |
| --- | --- | --- |
| R01 | Four-world target-selection bias and query-order diagnostic | M20's full 14-bit Rust corpus |
| R02 | M30's stated rational bound arithmetic | The complete residual-reduction proof |
| R03 | Exact repeated-draw replay and coupon union-bound arithmetic | Native traffic or storage timing |
| R04 | Exhaustive small-chain closure and balanced two-chain coverage | M38's full graph family or a general lower-bound proof |
| R05 | Four-signer quorum intersection and retained-lock controls | Signatures, network delivery or the M40/M41 implementation |
| R06 | NR05 codec parameters and fresh-target bit counts | The injective codec or a real sampler implementation |
| R07 | Exact k=64 integer probability certificate | Concrete SHA/time or accepted-hidden-proof security |
| R08 | Exact k=384 integer probability certificate | Real repeated-epoch independence |
| R09 | Authentication-error arithmetic and visible-format sizes | An implemented signature/Merkle verifier or actual transfer |

The normal and optimized commands repeat the same nine checks. They do not
constitute eighteen distinct research tests. Fresh output is deterministic;
recorded timestamps and process durations are run-specific.

## 3. Execution and integrity records

`logs/reference_normal.json` and `logs/reference_optimized.json` retain full
stdout/stderr, command arguments, exit codes, script and output digests, and
actual timestamps. `logs/integrity_controls.json` retains the separate executed
manifest-checker self-tests. These are actual new run records, not reconstructed
historical logs. All three records are included in the update hashes.

`BASELINE_MANIFEST.sha256` is a byte-exact copy of the published baseline
manifest: 189 entries. Its Git blob ID and SHA-256 were checked during rebuilding.
Inherited source-file hashes are carried from that manifest, not represented as
freshly rehashed baseline file bytes. New and changed update files are rehashed.

`UPDATE_MANIFEST.sha256` at the repository root covers every update payload file
except itself and `SOURCE_MANIFEST.sha256`. The latter covers the merged source
set, including the update manifest, but excludes itself. This dependency order
avoids circular self-hashes. The update checker independently reconstructs the
merged map from the pinned baseline and this update before accepting it.

```sh
python3 -I -B scripts/check_post_m19_update.py --overlay-only
python3 -I -B scripts/check_post_m19_update.py --self-test
```

After merging into the pinned public baseline, validate all inherited and new
bytes with `python3 -I -B scripts/check_post_m19_update.py --merged`. The existing
`python3 -I -B scripts/check_source_manifest.py` additionally compares against the
Git tracked-file set; stage the intended new files before that check.

This archive is a reconstructed update, not a byte-identical copy of an earlier
lost archive. It changes public research documentation and reference tools, not
operational consensus, Cargo dependencies, license terms or native source logic.
