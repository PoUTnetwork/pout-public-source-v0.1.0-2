# NE-01: first storage-power eligibility result

**Date:** 7 November 2026  
**Classification:** recorded local research-model execution  
**Scope:** `NE01_TEST_MODEL_OWNER_RULE_2026_11_07`  
**Operational network:** not established

## Implementation and observed result

The isolated storage-power research model introduced sector commitments,
checkpointed ratchet sealing, beacon-derived audit challenges, verification of
sampled openings, conserved funded resource accounting, and a bounded
eligibility evaluation. Traffic receipts cannot independently issue resource
weight. The acceptance logic requires a clean sequence of eight audited epochs,
distinct beacon values and live, audit-issued units.

The recorded release exercise reported `Established` with **512 live model
units** (two model providers at 256 units each), **8 audited epochs**,
**16 passes**, **0 failures**, and no unmet criteria in this test model.
The recorded focused crate test execution contains **64 passed, 0 failed**.
Those counts are not added to overlapping workspace test totals.

## Evidence and remaining limits

The supporting public evidence folder contains an original release JSON,
original release logs, the 64-test log, a selected Rust eligibility excerpt,
a source-provenance register and a self-checking SHA-256 manifest. Read its
own README and run its included `verify_evidence.py` from that folder.

The original NE-01 sealing construction is **not a proof of expensive physical
storage**. The subsequent S1 research demonstrates that a participant can
reconstruct sampled values cheaply without retaining the counted replicas.
A predictable fixture beacon does not certify external randomness.

Production gates remain open: external security review; independent device
and network testing; unpredictable beacon; delay-primitive validation;
storage/time robustness; a resource-consensus theorem; hosted CI.

The historical workspace test run was **not fully green**: unchanged research
binaries reported typed environment-admission failures. The focused NE-01
checks were recorded as successful; these different scopes must not be merged.

**Status:** first successful eligibility evaluation *inside a controlled
research model*, not a running blockchain or an independently verified mainnet.
