# Public test layout

Existing tests remain in their original crates and Python test directories.
This update adds:

- `crates/pout-public-verifier/tests/causal_byte_bridge.rs`: eight M19 inspection
  and guard functions sharing one failure-sticky finite corpus result.
- `crates/pout-public-verifier/examples/support/causal_byte_bridge.rs`: byte-key
  machine, separately addressed semantic reference and eight bounded traces.
- `crates/pout-public-verifier/examples/support/fixture_once.rs`: retained
  initialization error/panic handling, not process-crash recovery.
- `crates/pout-cwc-research/tests/rank_public_vector.rs`: one retained M17 BLAKE3
  input/output check, not proof generation or a consensus test.
- `tests/fixtures/milestones/`: public synthetic M17 input and two M19 goldens.

Python references are in `research/milestones/reproduce.py`. Sixteen M14 profile
rows, 512/64 M18 worlds and golden byte lengths are reference-model units, not new
Rust tests. The M16 original corpus count is not reproduced by these references.

No new Rust execution result is asserted for this update. Run the commands in
[VALIDATION_M19_PUBLIC](../docs/VALIDATION_M19_PUBLIC.md) before reporting one.
The previous full engineering workspace count is not the public workspace count.
