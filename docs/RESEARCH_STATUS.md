# Public research status

**Updated:** 9 November 2026. Public native source through selected M19;
selected documentation and experimental evidence through NE-02 S1.

## Earlier findings (M20-M41 and NR01-NR05)

Content validity, local completion, observed arrival and signatures are
distinct from independently useful demand or physical resource cost. M20-M29
exposed several boundaries. M30-M38 added conditional resource arguments,
reconstruction counterexamples and functional constraint controls while
preserving the missing accepted-proof/resource bridge.

M39-M41 exercised conditional normalization, checkpoint locks and selected
history in a synthetic trust model, but did not establish a deployed finality
network. NR01-NR05 examined proof costs, finite resource admission and
sampled-batch raw-label recovery under explicit model assumptions. These are
historical, source-reported records rather than newly reproduced native runs.

## Continued research (NR06-NR22)

Selected studies investigated accounting and admission constraints, coordinated
endpoint behavior, independent resource boundaries and the limits of
endpoint-attested useful traffic. [Narrative continuity](RESEARCH_NR06_NR22_CONTINUITY.md)
is not a complete native source or an independent security proof.

## NE-01 first-scaffold test model

A bounded, local storage-power research model reported `Established`, model
weight 512, eight audited epochs and 16 passing audits. Its public evidence
contains 64 focused passing tests and reproducible source provenance.
The model's beacon is a fixture. No operational peer network or physical
scarcity guarantee follows from the test-model result.

## NE-02 S1 storage/time attack

S1 fixed a five-second response contract and compared honest storage with
several declared regenerating strategies. All declared shortcuts were accepted
by the recorded NE-01 public verifier. A single holder with one shared
ciphertext copy and 2 KiB of retained label state still received the same
model weight. The S1 verdict is **NO STORAGE/TIME MARGIN**. That negative
result motivates later work, which is not included in this release.

## Validation and open gates

The stored S1 release JSON and attributed NE-01 evidence are available in
`docs/test-evidence-ne01-s1/`. Their public Python checks validate integrity,
accounting relations and declared numeric results, not a fresh Rust execution.

Root / operational consensus eligibility: `NotEstablished`.
Root / operational consensus weight: `None`.
Scoped NE-01 test-model eligibility: `Established` (weight 512).
Research hosted validation: `HOSTED_CI_PENDING`.

Open gates: independently reviewable resource scarcity, externally separated
multi-device tests, unpredictable challenge beacons, delay primitive, depth
robustness, resource-consensus theorem, hosted CI, and complete network
safety/liveness. No security certification, mainnet, native token entitlement,
price or return is established by this release.

## Navigation

- [Release classification](PUBLIC_NE01_NE02_S1_RELEASE.md)
- [NE-01 evidence](NE01_CONSENSUS_TEST_MODEL.md)
- [NE-02 S1](NE02_S1_STORAGE_TIME_MARGIN.md)
- [NR06-NR22 continuity](RESEARCH_NR06_NR22_CONTINUITY.md)
- [M20-NR05 chronology](MILESTONES_M20_NR05.md)
- [Older evidence index](test-evidence-m20-nr05/EVIDENCE_INDEX.md)
