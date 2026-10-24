# Validation boundary for the M19 public overlay

## What was and was not performed

This is an upload-only preparation. No Rust build, test suite, rustfmt, Clippy,
new proof generation or hosted workflow was executed. No repository was pushed.
The unchanged dependency manifests and lockfile are not regenerated.

The source M19 experiment, eight-test file and failure-sticky helper were checked
byte-for-byte against their retrieved Git blob identities before export adaptation.
The source manifest inherited from the public baseline was also checked against
its Git blob identity. Each new/changed delivered file is hashed after preparation.
The original M19 experiment logic is unchanged; only output support is adapted.

Selected standard-library Python references were executed normally and with -O.
They recompute sixteen finite M14 profiles, the M18 512-world quotient and 64-world
conditioning example, and verify public byte fixtures. These are Python reference
checks only, not Rust passes or reproduction of the full engineering suite.
`research/milestones/public_reference_results.json` records their actual output.

## Exact M19 adaptations

- Move the self-contained experiment and test to the existing public verifier crate.
- Omit the Serde import and Serialize derives. Existing Debug/other derives remain.
- Add a bounded standard-library text renderer that reads every report field.
- In the test, replace serde_json output with that renderer; retain the size check.
- Replace an internal evidence-location comment with the public reference path.
- Retain the original failure-sticky helper, limits, eight test functions, all
  predicates/cases, deterministic tables, typed comparison and golden literals.

No private crate, manifest, runtime fixture, opaque proof authority or secret is
required by these added files. This makes the adapted source inspectable; it does
not establish a new compiled or executed result.

## Downstream commands, not reported executions

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked -p pout-public-verifier --test causal_byte_bridge
cargo run --locked -p pout-public-verifier --example causal_byte_bridge
cargo test --locked -p pout-cwc-research --test rank_public_vector
cargo test --locked --workspace
```

The eight M19 tests share one retained result per test process. The example runs
the model separately. Counts across repeats must not be added. The full workspace
result from the engineering snapshot (1,065/0/8) is not the public test count.

## Integrity, baseline and scope

The overlay is based on public repository revision
`d38086e8b3f10f0f14d71a64994afe15f9f20ae2`. The updated root manifest merges its
published 157 entries with new/changed files, and excludes itself. Unchanged
entries are inherited checksums, not fresh full-content downloads. Use the
existing manifest checker on the merged repository to verify all files there.

Archive paths, encodings, local includes, Python syntax, JSON, known sensitive
markers, literal credential patterns and ZIP metadata were inspected. Such a
check is not a mathematical guarantee of anonymity or exhaustive secret detection.
Required upstream authorship and license material must not be removed. No independent audit or general security certification is
claimed by this export.
