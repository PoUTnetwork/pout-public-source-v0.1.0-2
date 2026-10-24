# Scope and limits of the historical test evidence

This supplementary directory covers the **reported historical test chronology from the earliest separately documented Phase 5 checkpoint** through research milestone M19. This includes the early Joint/J1–J3 stages, R1–R3, Round 4–7 reliability work, CWC and the M1–M19 research sequence. The complete provenance of every possible test since project inception has **not** been established. Earlier Phase 1–4 original test-log availability remains unverified.

## What is actually included

- Structured, source-specific milestone results extracted from maintained historical technical reports.
- A selection of independently inspected actual original Cargo test group totals, with original Git blob SHA-1 fingerprints and byte counts.
- Selected verbatim lines from private historical Cargo logs; clearly **excerpts only**, not full log copies.
- Historical supervisor-receipt fields and original log SHA-256 commitments for M14–M19 excluding M18 (which reused M17 Rust validation).
- Actual executable Python reference output from the public M19 package (normal and optimized mode).
- Explicit failure qualifications and ignored/subprocess test cases.

## What is NOT included

- Full original stdout/stderr files for all historical executions. Private originals may carry operator-identifying paths, execution metadata or sensitive operational context. No unchanged raw originals are republished in this archive.
- Test binaries, runtime captures, full private source history or signing/custody receipts.
- A fresh Rust test run of the adapted public M19 source. Historical engineering workspace results cannot certify that adapted public code.
- Hosted CI proof where a source explicitly says CI pending; no physical useful-service scarcity or blockchain consensus security.

This archive is a **transparent historical evidence index and selected evidence extracts**, not a claim to have published every original test log from the start of the project. The accompanying hashes of withheld originals can only be independently checked against those originals if separately made available later. Excerpts retain substantive failures; they are not doctored pass lines or recreated program output.
