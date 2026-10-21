# Exact public-update validation - 21 October 2026

## Executed for this package

Environment: Python 3.13.5 on Linux. No external APIs or private credentials.

| Check | Result |
| --- | --- |
| New unit/mutation tests | 40 passed |
| Existing public Python byte-reference tests | 12 passed |
| Combined unique Python suite | 52 passed; zero failures or errors |
| Same suite under optimized Python | Same 52 passed; not additional unique tests |
| All one-bit advice encoders | 65,536 rows independently checked |
| Response tables / sample targets | 256 / 64 rows checked |
| Fixed-strategy repetition profiles | 12 exact fraction rows checked |
| Bucket-advice first-hit profiles | 154 exact fixed profiles checked |
| Public-information controls | 16 GF4 worlds and four mask worlds checked |
| Chunk known-answer hashes | 32 checked |
| Deterministic request vectors | 128 checked |
| Synthetic frame cases | 16 checked: three Verified, one Unavailable, twelve expected rejections |

The dataset generator and checker have different calculation paths. Regeneration
into a new directory matched all eight mathematical fixture files byte-for-byte.
The source manifest binds released bytes. The archive was extracted into a fresh
directory and the package checker was rerun there. These repeated checks are not
added to the 52-test total.

## Not executed here

No Rust compiler is present in the packaging environment. Rust compilation,
Clippy, tests, native IPC examples, hosted CI, an external audit, operational
networking and cryptographic proof generation were not executed for this update.
No Rust source, dependency or manifest change is included. Earlier public-release
results and the broader private-workspace result cannot substitute for a future
full Rust check of the current public tree.

The new fixtures are synthetic generated test inputs. Their byte hashes and
expected categories are evidence of reproducibility, not real customer traffic,
retained storage, bandwidth independence or consensus eligibility.

## Release checks

The update is based on public commit `2e1893647adb1e7db75bad5efc34b7652ed256c1`.
Available predecessor files matched that release's source manifest before edits.
Unchanged paths retain their manifest digests; new and changed file digests were
computed from the actual package bytes. The baseline manifest's Git blob is
`320dde8c2203e3cc369e6c1c35615f8a2c9b7459`.

All paths in the ZIP are relative and allowlisted. No Git directory, automated
workflow, private engineering notes, credentials or personal runtime artifacts
are included. Required inherited license notices are not removed.

The complete `scripts/check_source_manifest.py` check must be run on the merged
repository. The overlay alone intentionally does not contain all inherited files.
