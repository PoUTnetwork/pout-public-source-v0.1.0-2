# CWC execution model

CWC explores deterministic parallel execution of independent operations within one already ordered candidate. It is a bounded, in-memory research model using artificial counters. It does not implement block production, BlockDAG ordering, consensus eligibility, a currency or a production ledger.

## State and operations

Version 1 supports at most **1,024 existing state keys**. Each key is a literal 32-byte string and each value is an unsigned 64-bit integer. State entries have a canonical unsigned lexicographic key order. Missing keys are distinct from existing keys whose value is zero. A candidate contains at most **256 operations**.

| Operation | Reads | Writes | Behavior |
| --- | --- | --- | --- |
| `AssertEquals(key, value)` | key | none | Requires the existing value to match and records the observation. |
| `Set(key, value)` | none | key | Replaces an existing value. It does not create a key. |
| `Add(key, amount)` | key | key | Adds with checked unsigned arithmetic. |
| `Move(source, destination, amount)` | both keys | both keys | Requires distinct existing keys and performs a checked debit and credit. |

The counters are synthetic. `Set` does not authorize issuance or modify real assets. Missing keys, same-key moves, arithmetic overflow or underflow and failed assertions reject the entire candidate.

The serial reference runs operations strictly in candidate order. It does not call the dependency scheduler. Its operation evaluator is shared with the parallel implementation; the tests additionally use a separate wide-integer arithmetic oracle to reduce the risk of a shared semantic mistake.

## Dependencies and real workers

For every pair of operations `i < j`, the executor derives their read/write sets from typed operations. It creates a dependency `i -> j` whenever a write of `i` overlaps a read or write of `j`, or a read of `i` overlaps a write of `j`. This includes read-after-write, write-after-write and write-after-read conflicts. A caller cannot provide a replacement graph or declare an incomplete conflict set.

A wave contains operations at the same dependency depth. They read one immutable candidate-state snapshot, compute isolated results on actual scoped worker threads, and finish before the wave commits writes in operation-index order. Canonical results are sorted by the original candidate index, including when a wave contains noncontiguous indices. The candidate state remains private until every wave succeeds.

Only **1, 2 or 4 configured workers** are accepted. There are at most four simultaneous workers per execution attempt and at most 256 total thread spawns per candidate. Each worker requests a **256 KiB stack**; platform overhead is additional. The fixed predecessor bitset occupies **8,192 bytes**, with at most **32,640 dependency edges** and 256 waves. Hosts must separately bound the number of simultaneous execution attempts. This library is not a global admission scheduler.

## Outcomes and cancellation

- `Accepted` contains a complete final state, canonical operation results and state root.
- `Rejected` contains no new state or partially successful prefix.
- `Aborted` reports a local incomplete attempt, including cancellation, unsupported local worker count, worker panic, thread-spawn failure or a detected allocation failure. It does not establish that a candidate is invalid.

The input state is borrowed and remains unchanged. All successfully spawned workers are joined on handled return paths, including worker failures. A cancellation flag must remain set until its attempt returns; a new attempt may use a fresh flag and the original input.

The library does not commit to a database or offer durable process-crash recovery. A process-wide allocator abort, forced process termination or host failure is outside its in-memory recovery boundary. Some bounded internal allocations are infallible Rust allocations and cannot guarantee recoverable out-of-memory handling.

## Canonical encoding

Encoding version is `u16 = 1`. Integers have fixed-width little-endian representations; keys are literal bytes. Unknown tags or versions, duplicate or unsorted state keys, noncanonical order, truncation and trailing input are rejected. No clock, locale, host path, network call or floating-point operation affects execution output.

| Record | Input byte limit |
| --- | ---: |
| State | 41,000 |
| Candidate block | 65,536 |
| Results | 21,300 |
| Complete execution | 65,536 |

The largest constructible state, block, results and execution encodings in this model are respectively **40,979**, **18,707**, **21,269** and **62,309 bytes**. Decoders check byte/count bounds before input-driven allocation. Callers constructing a `Vec` themselves own that initial allocation.

The state root is BLAKE3 over the literal domain `POUT/CWC/STATE_ROOT` with a terminating zero byte, followed by the little-endian version, canonical state length and canonical state bytes. This commitment is a research-domain state checksum; it is not a block-eligibility proof. Decoding a claimed execution checks syntax, state membership and root consistency. Without an initial state and candidate it does not establish that the claimed execution is correct.

## Demonstration

Run:

```sh
cargo run --locked -p pout-cwc-research --example demo
```

The first example starts with A=10 and B=20, adds 5 and 7 independently, and then moves 3 from A to B. Serial and parallel executions with 1, 2 and 4 workers must agree on A=12, B=30, all operation results and the state root.

A five-operation example adds a third counter and checks multiple dependent waves, ending at A=12, B=19, C=36. The example also rejects two moves of 80 from a reserve of 100, checks that rejection leaves the original state unchanged, and demonstrates cancellation followed by a successful retry.

## Test coverage

The CWC crate contains 21 default unit tests and 12 integration tests. The optional `research-controls` feature adds five unit tests. Executed release results are listed in [Research status](RESEARCH_STATUS.md).

The tests include:

- An independent wide-integer arithmetic oracle comparing validity, canonical results, state bytes and state root against the serial and 1/2/4-worker executors.
- A fixed exhaustive corpus of 5,265 candidates over nine two-key states and an eight-operation alphabet at lengths zero through three, plus a fixed-seed corpus and hand-calculated three-key cases. This finite enumeration is not a proof for every possible input.
- Read/write aliasing, assertion ordering, checked arithmetic, missing keys and shared-reserve overdraw.
- Actual overlap on distinct worker threads, controlled reverse completion, bounded three-operation completion orders, cancellation, worker panic and simulated spawn failure. Expected panic diagnostics belong to tests that require a handled local abort.
- A deliberately omitted dependency as a negative control, demonstrating that the test oracle detects the resulting disagreement.
- Literal canonical encoding/root vectors, malformed or truncated bytes, excessive lengths, trailing bytes and model boundary values.

The optional `research-controls` feature exposes a bounded first-wave completion-order controller for isolated tests and examples. It requires a first wave exactly `[0, 1]`, workers 2 or 4 and a fixed five-second local synchronization budget. It uses the real graph and evaluator and cannot alter operation dependencies. A local controller timeout aborts the attempt rather than rejecting a valid candidate. This feature is disabled by default.

## Security and research limits

Matching finite executions do not establish useful-service scarcity, economic demand, Sybil or collusion resistance, finality, secure GHOSTDAG adaptation or production correctness. No serial/parallel speedup is claimed. Thread scheduling and graph overhead may make these small operations slower in parallel. No user files, network service, private identity, wallet or payment system participates in this model.
