# Public research extension: NR06-NR22, NE-01 and NE-02 S1

**Release cut:** 9 November 2026  
**Research content cutoff:** NE-02 S1 (7 November 2026)  
**Distribution:** additive overlay for the existing public repository  
**Not a mainnet, complete node, mining client or production consensus.**

This update supplements the existing published source and documentation
through M41/NR05. It adds a compact later-research chronology, the first NE-01
eligibility-model record, and the NE-02 S1 security baseline. The prior public
Rust source and existing dependency and license files must remain unchanged.

Highlights:

1. NE-01 showed a clean `Established` eligibility evaluation in a *specific
   test model*: eight audited epochs, 16 passes, weight 512, 64 focused tests.
2. NE-02 S1 established a frozen storage/time security contract and **found a
   successful regeneration attack**. The earlier seal did not enforce
   physical storage costs despite passing the test-model checks.
3. The S1 release-output JSON is reproduced byte-for-byte from its recorded
   source blob. It was **not re-executed** while assembling this archive.
4. The retained NE-01 public evidence bundle contains original release
   artifacts, a source excerpt and its own independent integrity checker.
5. Public verification scripts check hashes, recorded data relationships and
   conditional probability arithmetic. They do not recreate a missing native
   implementation or establish production security.

Boundaries: external security review, unpredictable beacon, independent
multiple-device operation, secure resource scarcity, delay primitive,
resource theorem, hosted CI, full network safety/liveness and practical
infrastructure remain open.

The update preserves existing copyright notices and licenses. No new token,
wallet, payment system, deployed network or exchange entitlement is supplied.
