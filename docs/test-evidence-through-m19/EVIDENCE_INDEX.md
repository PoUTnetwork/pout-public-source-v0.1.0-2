# PoUT Network — historical execution evidence index through M19

Publication-scope evidence supplement for the public research prototype. Based on existing historical engineering logs and source-gate reports, plus two separately executed public Python reference runs.

**IMPORTANT SCOPE:** This is an evidence **INDEX and selected verified excerpts**, not a republication of every complete raw engineering test log. Raw original logs are not included. Original log SHA-256 values are supplied where available; readers cannot recompute those original hashes from this package alone. No fresh Rust workspace tests were executed for this archive. These limitations must not be described as full independently reproducible historic test evidence.

## Earlier history: before Round 4 and before M1

The **earliest separately documented workspace test** located for this archive is Phase 5, dated 13 October 2026, **100 passed / 0 failed / 0 ignored**. This is earlier than J1 (20 October). It does **not** prove that Phase 5 was the project's first-ever test; any missing Phase 1–4 original artifacts are not invented.

`EARLY_STAGES_AND_J1_J3.json` identifies Phase 5, Round 4–7, early CWC and cryptographic stages, J1–J3, R1–R3 with source-qualified results. `RAW_CARGO_LOG_DERIVED_TOTALS.json` records actual group totals extracted from selected original Cargo logs, and their historical Git-blob fingerprints. `CHRONOLOGY_EARLIEST_TO_M19.json` is the combined chronological index. **These extracted results are not substitutes for publishing full original log bytes.**

Failures are preserved explicitly: Round 6 required a second SIGINT; Round 7B/C sustained acceptance failed; J3 had an earlier permission failure and separate timeout; M13 had a failed partial full run. Passing Rust tests does not erase these operational failure records.

## Historical engineering checkpoint index

| Stage | Historical Rust passed / failed / ignored | Cargo groups | Evidence qualification |
| --- | ---: | ---: | --- |
| Round 4 | 125 / 0 / 0 | 23 | Four concurrent customers, 128 MiB encrypted recovery and once-only mock settlement; historical local pass; original test-log excerpt inspected. |
| Round 5 | 142 / 0 / 0 | 24 | 512 MiB publication with publisher/provider/coordinator restarts; historical local pass. |
| Round 6 | 192 / 0 / 2 | 25 | Resource quotas and mixed load; test suite passes, but an acceptance cleanup required a second SIGINT. Not an unattended shutdown pass. |
| CWC milestone B | 476 / 0 / 7 | — | Hosted Ubuntu completed 476/0/7, plus separate shared platform results. Network representative workload gate remained failed. |
| R1 | 891 / 0 / 7 | 66 | Local checked byte retrieval, not external internet bandwidth or independent demand. |
| R2 | 901 / 0 / 7 | 67 | Shared quota window experiment; no physical link independence. |
| R3 | 909 / 0 / 7 | 68 | Shared source can satisfy multiple encodings. Negative counterexample, not proof of replicated physical storage. |
| M1 | 946 / 0 / 8 | 72 | Reused 946/0/8 Rust suite from identical-source audit; new mathematical reference separately executed, no new Rust suite. |
| M2 | 955 / 0 / 8 | 74 | Nine focused tests included in the 955 total; initial Clippy failure was corrected and retained. |
| M3 | 955 / 0 / 8 | 74 | Rust results reused from M2; Python reference executed 93,855 named visits. |
| M4 | 961 / 0 / 8 | 75 | Two intentionally failing red tests and subsequent fixes; six focused tests and full workspace passed after correction. |
| M5 | 961 / 0 / 8 | 75 | Rust results reused from M4; separate Python analysis, 17,267,811 named events. |
| M6 | 968 / 0 / 8 | 76 | Seven new tests; 20,257,732 logical focused model events. Included in 968 total. |
| M7 | 968 / 0 / 8 | 76 | Rust results reused from M6; Python reference 38,629 events and 14,635 independent checks. |
| M8 | 968 / 0 / 8 | 76 | Rust results reused from M6; 56 finite reference profiles, 283,576 named events, 53,437 callbacks. |
| M9 | 980 / 0 / 8 | 77 | Locked workspace test passed; local CI pending. |
| M10 | 988 / 0 / 8 | 78 | Locked workspace test passed; local CI pending. |
| M11 | 994 / 0 / 8 | 79 | Locked workspace test passed; local CI pending. |
| M12 | 1006 / 0 / 8 | 81 | Locked workspace test passed; local CI pending. |
| M13 repair | 1022 / 0 / 8 | 82 | Prior M13 review run failed: 417 passed, 1 failed, 6 ignored (partial), exit 101. Later corrective source gate passed 1022/0/8. |
| M14 | 1034 / 0 / 8 | 84 | 12 new tests included, plus Python semantic checks; exact-source local gate passed. |
| M15 | 1042 / 0 / 8 | 85 | Eight new tests included; exact-source local gate passed. |
| M16 | 1051 / 0 / 8 | 86 | Nine new tests included; 21,086 finite cases and 41,806 Rank calls in model; exact-source local gate passed. |
| M17 | 1057 / 0 / 8 | 87 | Six new tests included; failures during earlier lint/parser verification preserved in historical record; final local gate passed. |
| M18 | 1057 / 0 / 8 | 87 | ANALYSIS ONLY: 1,057/0/8 reuses M17, not newly executed Rust suite; 31/512 counterexample against applying the bound without helper work. |
| M19 | 1065 / 0 / 8 | 88 | Eight new tests included; 63 trace events / 17,903 modeled request payload bytes; exact-source local gate passed; hosted CI pending. |

**Do not add milestone test totals.** Every subsequent workspace suite includes inherited tests. Focused tests and restored repetitions are not additional unique tests. M1/M3/M5/M7/M8/M18 reused the previously executed matching-source Rust tests; additional mathematical analysis is separate. The earlier Round 7 network acceptance record includes both failures and corrected successes and does not establish production reliability across all target platforms.

## Original machine-generated local execution receipts (M14–M19)

From retained local execution records, the following commands returned exit code 0 for their source-specific snapshots. M18 is intentionally absent because no new Rust workspace suite was run for M18.

| Stage | Source/build SHA-256 | Cargo workspace output log SHA-256 | Recorded log bytes |
| --- | --- | --- | ---: |
| M14 | `22f81ae1592fa151a1c5fb8a2a36f144bcd49a560ae12e7a25f6f77d9bb26cd7` | `8c6acef839dd32998b82c37a071f77bcb70398078017fd72bd29c6a5295cac90` | 111,449 |
| M15 | `7c1e67c1e3a57c8c26093f329077898317402e8e964fcbb2065adce76a6c174a` | `766aabb8e8dc65cad676b880b5f6f88d4a461c8c3de62e1c68fb7dfd0ce39c67` | 112,472 |
| M16 | `c7d4a042e5d3ce24cf28f94c0cd89aa5b5d0377e8c743cc872682f85d8d345e8` | `31a11c1f6ec6d9ba653766e6c594add276da053d94ae65e5f9d901f9aa4959ef` | 113,449 |
| M17 | `802e56ba920281c7cf27f9b22be58fb3495998c45a31115b3b2fc811c4894632` | `93b81a943be8cb6aa361312c82e77011144a0629e99d8303a9230bbd6330dc5e` | 121,031 |
| M19 | `fec8be21db32980f66d1bf9de185bc530bdc2980c3317a161213118b0f973876` | `7908922a8cf866b721bd0f42c24ba34236cbf3ef4c3d84ba982508c62b2d51d4` | 116,601 |

For the individual command lines, exact completion dates, exit codes and warning-free Clippy log hashes consult `ORIGINAL_RUN_RECEIPT_FIELDS.json`. These records are **derived fields** of retained original machine reports. The complete original stdout/stderr files are not included, so these SHA-256 commitments alone are not an independently verified public log archive.

## Selected verbatim lines from original Cargo test logs

The following lines were read verbatim from historical log files. They are short **excerpts**, not full logs. The full log Git blob SHA-1 and number of result-group lines are in `VERBATIM_LOG_EXCERPTS.json`.

**Round 4**
```text
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

**Round 5**
```text
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
```

**Round 6**
```text
test result: ok. 63 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 2.68s
```

**M9**
```text
test result: ok. 106 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 24.96s
```

**M14**
```text
test result: ok. 111 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 33.47s
```

**M15**
```text
test result: ok. 111 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 33.65s
```

**M16**
```text
test result: ok. 111 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 34.34s
```

**M17**
```text
test result: ok. 111 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.73s
```

**M19**
```text
test result: ok. 111 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 18.00s
```

## Current public Python reference checks

The M19 public source repository contains reproducible finite Python models and byte-vector fixtures. This evidence supplement includes two new exact stdout captures in `logs/` and their complete results in `PUBLIC_REFERENCE_RUNS.json`. These checks do not authenticate the original engineering environment and do not execute Rust.

```sh
python3 -I -B research/milestones/reproduce.py
python3 -I -B -O research/milestones/reproduce.py
```

Historical validation is source-specific: the public M19 Rust code has serialization-only adaptations and is not certified by the private 1,065/0/8 result. Independent Rust execution of the public repository is still required to establish its own test result.

## Findings that must not be promoted to consensus claims

- M13 contained a preserved failed full-workspace review and subsequent corrective pass.
- M18 found a counterexample to incorrectly omitting helper queries; the probability theorem was not generally disproven.
- Physical storage/traffic scarcity, independent demand, coalition control, global consumption, and BlockDAG safety/finality remain **NotEstablished**.
- Public reference output, byte-processing counts, and cargo regression tests are not proof of independent scarce useful work.
- Hosted CI status is not silently upgraded from historical pending to passed.

## Integrity and privacy

`FILE_SHA256_MANIFEST.txt` covers this export after sanitization. Source-code author/license notices in the pre-existing public repository are unaffected. No private workspace paths, access tokens, emails or internal repository identifiers are needed to interpret the evidence. Publication of selected excerpts should not be presented as publication of complete raw test output.

