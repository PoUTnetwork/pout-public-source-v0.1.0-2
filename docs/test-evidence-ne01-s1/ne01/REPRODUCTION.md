# Reproduction and evidence verification

## Verify the files in this bundle

From this directory, using Python 3.9 or later:

```sh
python3 verify_evidence.py
```

On systems with GNU coreutils, file hashes can also be checked with:

```sh
sha256sum -c SOURCE_MANIFEST.sha256
```

This verifies packaged bytes and internal consistency only. It is not a test
of PoUT, a signature, or a substitute for an external security review.

## Re-run the original Rust experiment

Requires authorized access to the complete matching source workspace, its
Cargo.lock and dependencies, and the appropriate build/admission environment.
These are NOT supplied in this evidence-only bundle. Do not run Cargo in this
bundle and expect the original project to be present.

In the matching source workspace, first confirm the revision:

```sh
git rev-parse HEAD
# Expected: 789e90aa3884904ad504c86fcbe121d08608be3c
```

Commands recorded for the NE-01 checks include:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo clippy --locked -p pout-useful-storage-research --features real-components --all-targets -- -D warnings
cargo test --locked -p pout-useful-storage-research --features real-components
cargo run --locked -q --release -p pout-useful-storage-research --example ne01_skeleton
```

The recorded source environment used rustc/cargo 1.97.0. This bundle does not
assert that any particular alternative toolchain reproduces that run. Timing
values are host-dependent and must not be expected to match byte-for-byte.

The full-workspace check was:

```sh
cargo test --locked --workspace --no-fail-fast
```

That recorded run exited 101 (1,236 passed, 13 failed, 8 ignored). The report
requires separate fixture/supervisor admissions for ingress_cut; do not disable
or relabel failing tests merely to obtain a green result. The full operational
admission procedure and restoration workflow are not part of this bundle.

No tests were newly executed in preparing this publication. Running the above
commands successfully would still not close the resource, delay, beacon,
multi-device, or external-review gates.
