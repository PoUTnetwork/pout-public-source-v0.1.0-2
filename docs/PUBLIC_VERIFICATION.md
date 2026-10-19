# Public verification format

The `pout-public-verifier` crate retains an actual Bellman Groth16 pairing verifier over BLS12-381. Its fixture suite was constructed for a finite SHA-256 relation using a 64-byte synthetic source. This release is verification-only: public proof and key encodings are included, but the source witness, proving circuit, setup generator, proving key and setup randomness are excluded. A reader can reproduce pairing acceptance and rejection, not proof generation or an independent circuit-to-key audit.

## Statements and framing

All integers in the enrollment and audit records are explicitly little-endian fixed-width u64 values. Byte arrays are literal octets. There are no paths, timestamps from the host clock, locale dependencies or native-sized integers in the wire statement.

| Record | Exact bytes | Fields |
| --- | ---: | --- |
| Enrollment | 136 | Source hash 32, synthetic network 32, synthetic provider 32, epoch 8, replica root 32 |
| Audit | 80 | Network 32, epoch 8, round 8, randomness 32 |
| Public input | 128 | Source hash 32, derived context 16, replica root 32, derived challenge 16, response tag 32 |
| Framed proof | 200 | `PTFREL01` followed by checked compressed Groth16 A/B/C points |
| Verifying key | 1,444 | Fixed Bellman key encoding with exactly six input-coefficient points |
| Bundle | 1,900 | `PTPUBV01`, full key, enrollment, audit, tag and framed proof |

The input-coefficient count uses Bellman's big-endian u32 convention and is preflighted before count-driven key allocation. Other group serialization follows the upstream checked encoding. The key and proof must re-encode byte-for-byte; truncation, trailing bytes, subgroup-invalid encodings and identity points are rejected.

The context is the first 16 bytes of SHA-256 over `PoUT-file-relation/v1/context`, the network, provider and epoch. The challenge is the first 16 bytes of SHA-256 over `PoUT-file-relation/v1/challenge` and the full audit. Network and epoch must agree between enrollment and audit. Public inputs are bit-packed into field elements by Bellman's specified multipacking implementation.

The circuit is not present in this package. The source/replica/tag relation attributed to the fixture must therefore be treated as a key-provenance assumption, not something a verifier can recover from arbitrary key bytes. Names such as `VerifiedFileRelation` express the implementation's finite relation result under that assumption; they are not a proof of physical custody or delivery.

## Independent authority and key consistency

`ExpectedPins` must come from an authenticated source outside the untrusted bundle. The key pin hashes **all** 1,444 key bytes. The statement pin hashes the complete enrollment and audit with domain `PoUT-public-file-evidence/v1/full-statement`. Self-reported hashes inside an adversary's data do not establish authority. The example's public pins are test constants only.

The importer checks `e(beta_G1, delta_G2) == e(delta_G1, beta_G2)`. This is necessary consistency under independently sampled group bases. Comparing both groups against canonical generators would impose a different condition. The cross-pairing equality is not a complete CRS-validity proof and cannot establish honest setup, erased randomness or a unique circuit. Complete independent key pinning remains necessary.

## Proof identity and adversarial tests

The opportunity identifier hashes domain `PoUT-file-relation/v1/opportunity`, the full key hash, enrollment and audit. It intentionally does not depend on the proof's byte representation. Tests publicly transform valid proof group elements without a witness; the transformed proof still verifies for the same statement and yields the same identifier. A changed representation is not new resource work. Replaying a proof returns the same identifier; the verifier is stateless and does not enforce canonical consumption, payments, fork handling or consensus weight.

The executable example verifies two genuine public proofs, rejects a wrong-source proof under the original statement, and accepts that proof under its truthful alternate statement. Tests also mutate authority pins, full statements, key headers, point encodings, lengths and tags, and distinguish cancellation from invalid evidence.

Cancellation is checked at finite stage boundaries. It is not preemption inside a pairing. Hosts must limit simultaneous verification calls; a bounded input is not a global CPU/memory admission policy.
