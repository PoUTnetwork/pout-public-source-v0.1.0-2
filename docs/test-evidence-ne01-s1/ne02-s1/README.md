# NE-02 S1 - source-recorded evidence

This folder preserves one byte-identical recorded release-output JSON document
(`NE02_S1_MARGIN_RELEASE.json`) and a clearly labelled public extraction of
locally recorded check results (`NE02_S1_CHECKS_PUBLIC.json`).

The release JSON was compared against the source repository's original Git blob
identifier. It is not a fresh execution, an external audit, or a proof that a
physical resource was stored. The native crate and complete build environment
are not distributed by this update; therefore the public reference verifier
checks the stored result's internal arithmetic and scope, not native execution.

The critical finding is **negative**: the NE-01 seal did not establish a
storage/time scarcity margin. All tested regeneration strategies produced
accepted responses, and a single provider controlled two replicas while
retaining one ciphertext copy plus 2 KiB of per-replica state. The NE-01 test
model still showed `Established` and weight 512. This demonstrates a weakness
in the model, not a production consensus result.

Recorded environment failures are explicitly preserved in the public check
extraction. Root research hosted CI and external assessment remain pending.
