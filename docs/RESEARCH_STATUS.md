# Public research status - 21 October 2026

This file records public-release evidence, not the entire engineering workspace.

## Current public export

The new finite pack adds Python models, separate exact reference checking,
synthetic request/frame fixtures and mutation tests. The existing Rust source,
Cargo manifests, dependency lockfile and license files are unchanged.

The executed package checks and known gaps are recorded in
[VALIDATION_2026-10-21.md](VALIDATION_2026-10-21.md). A reader can reproduce the new
checks without network access, a project account or any additional packages.

## Historical public validation

The original v0.1.0 release recorded successful fmt, locked all-target Clippy,
workspace tests and build on Linux x86-64. Its default workspace had 41 passing
Rust tests, zero failures. The optional CWC run had 38 passes with overlap;
46 distinct tests existed across those configurations. Those totals belong to
that original source only. The retained record reports Rust 1.99.0; the declared
minimum 1.88 and other-platform runtime were not tested in that release record.

The October 19 accounting addendum added eleven Rust test definitions but did
not claim a separately executed public result. The October 20 byte-observation
adaptation added 23 Rust test definitions and twelve executed Python reference
tests. Compilation, Clippy, actual Rust IPC output and full Rust regression for
that adapted source were not executed in its packaging environment.

The workflow mentioned by an earlier package was removed before public upload.
No workflow is included now. Hosted CI remains unverified. This update corrects
those obsolete workflow descriptions instead of promising an automatic run.

## Broader project progress, not a public validation claim

Internal research has advanced through joint statement binding, byte-observation
experiments, sampled-pair and repeated-audit models, complete-public-view
conditioning and first-hit controls. The latest reported engineering workspace
has 968 Rust passes, zero failures and eight parent-invoked helper fixtures.
That broader workspace is not included here. This report does not transfer its
result to the smaller adapted public export or imply an external audit.

This release shares selected elementary finite controls and public synthetic
fixtures, not the full protocol transcript, proof construction or consensus
integration. These portable models were separately generated and checked for
this release; they are not verbatim copies of the internal execution logs.

## Still not established

- Independent useful-service demand or physical resource scarcity.
- A global coalition-cost bound for the actual protocol and accumulated history.
- Safe permissionless eligibility, consensus weight, finality or liveness.
- A released native chain, migration bridge, mainnet or operational validator.
- Production networking reliability or mobile/full cross-platform runtime.
- A new joint Groth16 proof, trusted-setup review or independently audited circuit.

`consensus_eligibility = NotEstablished`
`consensus_weight = None`
