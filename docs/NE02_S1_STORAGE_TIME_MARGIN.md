# NE-02 S1: frozen storage/time contract and NE-01 attack baseline

**Date:** 7 November 2026  
**Classification:** recorded local release measurement and negative security
result  
**Status:** `NE01_SEAL_NO_STORAGE_TIME_MARGIN`

## Question

Does a successfully audited storage claim show that the claimed replica data
was actually retained, or only that the response can be reconstructed before
the deadline? The initial NE-01 eligibility result alone does not distinguish
these cases.

## Experimental contract

The pinned contract uses a 5-second audit deadline, hardware speed-up
scenarios of 1x, 10x and 100x relative to the fastest measured software,
storage deficits of 10%, 25% and 50%, 5% gap tolerance, eight epochs and a
20-bit target detection bound. The minimum samples per epoch for the three
storage deficits are respectively 34, 8 and 3 under that conditional model.

The recorded run used two 64-KiB claimed replicas (131,072 total claimed
bytes), a 4,096-step profile, and 16 chunks of 4 KiB.

## Reconstructed-response attacks

The native release example tried these strategies, which the public verifier
accepted in the recorded execution:

| Strategy | Cached state exclusive of shared ciphertext | Recorded response time |
| --- | ---: | ---: |
| Honest | 199,840 bytes, for the local specimen | 0.000138 s |
| FinalsParallel | 1,024 bytes | 0.040430 s |
| Spaced/8 | 9,216 bytes | 0.041204 s |
| ResealSerial | 0 bytes | 0.122241 s |

The strategies' reported state sizes refer to label/tree state in the live
example; additional ciphertext requirements and full accounting are separately
recorded in the JSON result. Do not confuse these measurements with a measured
public mainnet security margin.

In a separate single-provider, two-replica intake run, the provider kept one
ciphertext copy (65,536 bytes) plus 2,048 bytes of retained label state and
reproduced audit responses within approximately **0.045 seconds**. That run
still received `Established` and weight 512 *in the NE-01 model*.

**Conclusion:** a positive test-model eligibility outcome did not demonstrate
an enforceable storage scarcity cost. All three hardware scenarios failed the
contract's storage/time margin requirement (maximum gap 500 per mille against
a tolerated 50 per mille). The cheap-response attack is a real, acknowledged
limitation of this construction.

## Follow-up and transparency

A later construction must bind the claimed bytes to sufficiently expensive
regeneration while preserving cheap independent verification. S1 defines the
baseline and attack contract; it does **not** establish that such a construction
already exists. Future steps are not part of this release.

Files in `docs/test-evidence-ne01-s1/ne02-s1/` preserve the exact recorded
release result and a clearly labelled check extraction. Public Python checks
can validate reported arithmetic and integrity; they do not replace the
original native Rust test run, external security review or hosted CI.
