# Public research status - through M19

Publication date: 24 October 2026. This release is a selected-source overlay,
not the complete development workspace and not a mainnet release.

## Available in this repository

The previously published CWC serial/parallel model, verification-only Groth16
component, resource-accounting controls, local byte-observation example and finite
Python dataset remain available. This overlay adds the M19 causal byte/history
experiment, its eight tests, a retained M17 native Rank vector and a compact
Python reference for selected M14/M18 results and byte encodings.

## Engineering results versus this public export

| Checkpoint | Recorded full engineering workspace | Public verification boundary |
| --- | --- | --- |
| M14 | 1,034 passed; 0 failed; 8 ignored | Compact sixteen-profile Python reference; no DAG corpus export |
| M15 | 1,042 passed; 0 failed; 8 ignored | Exact-intent specification and retained context; native relation not exported |
| M16 | 1,051 passed; 0 failed; 8 ignored | Bound/accounting specification; full finite corpus not exported |
| M17 | 1,057 passed; 0 failed; 8 ignored | Retained full-pair digest vector and downstream Rust check |
| M18 | No new Rust execution; M17 results reused | Two finite selection/conditioning references |
| M19 | 1,065 passed; 0 failed; 8 ignored | Causal byte adapter, semantic reference and eight public test functions |

These are reported historical engineering counts, not independently reproduced
public totals. Do not sum checkpoints, repeats or model worlds as unique tests.
Hosted CI for these engineering checkpoints remained pending at their source
freezes. Historical M17 validation includes a corrected test-iterator lint and a
restored-log parser reconciliation; neither failure is evidence that this public
package was built. M19 also retained pre-corpus lint/formatting corrections.

## Checks made when assembling this overlay

No Rust compiler, Cargo build/test, rustfmt, Clippy or hosted workflow was run.
Source transfer, public path/dependency review, golden-byte reconstruction and
archive/integrity/privacy checks are separate from runtime validation. The
selected standard-library Python references were run normally and under -O;
the report identifies their limited scope. Downstream Rust results are pending.

All original Rust test predicates are retained in the exported M19 adapter;
serialization-only adaptation means this is nevertheless a new public source
composition requiring its own runtime checks before a new pass claim.

## What remains open

Physical resource scarcity, independent useful demand, resource ownership,
coalition resistance, global canonical consumption, honest-production and network
delay assumptions, and distributed consensus safety/finality remain unestablished.
A local test result or a valid cryptographic proof does not fill those gaps.
No production asset, wallet, bridge, token migration or service deployment is
included. Existing prototype platform limitations remain.

See [milestone detail](MILESTONES_M14_M19.md),
[validation scope](VALIDATION_M19_PUBLIC.md),
[publication scope](PUBLIC_RELEASE_SCOPE.md) and [security](SECURITY.md).
