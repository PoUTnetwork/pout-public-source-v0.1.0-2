# NR05 - visible batch response and accounting

30 October 2026. Public adaptation of source S3 in the evidence register.
Specification and analytical size accounting only: no new verifier, signature,
Merkle proof, accepted response or network benchmark was executed here.

## Separate relation, not the hidden M37 circuit

The proposed relation uses two fixed contexts and one known 1,024-word source,
32 bytes per word. A chain vertex uses its ordered predecessor (none at the
root). Its key hashes the domain, context, vertex index, parent-presence flag
and parent label; its replica label is the source word XOR that key. Source
and replica Merkle trees have distinct leaf/internal domains and canonical
indices, levels and ordering.

An ideal-oracle suite and a concrete-SHA suite need different identifiers and
security statements. Reusing an identifier would not transfer the ideal theorem
to hardware. Immutable correct registration, expected challenge and authorization
policy are independently pinned, not accepted from the responder on demand.
A registration recomputation can check one small known-source instance but cannot
prove the source/context independence or permissionless membership assumptions.

## Named format and costs

Integers are big endian. Digests and identities are 32 bytes. Unsupported versions,
nonzero reserved fields, wrong lengths, inconsistent duplicates and trailing bytes
must reject. Each opening record contains a four-byte index, a 32-byte word and
ten 32-byte sibling hashes: 356 bytes.

| Item | Formula / interpretation | Bytes |
| --- | --- | ---: |
| Immutable registration | Fixed prospective statement | 512 |
| Graph descriptor | 8-byte header + 5-byte root + 1,023 nine-byte nonroots | 9,220 |
| Challenge, k=64 | 128-byte header + 8k target bytes + 64-byte signature | 704 |
| Challenge, k=384 | Same layout | 3,264 |
| Signed response, k=64 | 128-byte header + 6k opening records + 64-byte signature | 136,896 |
| Signed response, k=384 | Same layout | 820,416 |

The six k records are source, target-replica and parent-replica openings for each
of k positions in each of two contexts. A root's absent-parent record is all zero
and must be checked as inactive, not authenticated as a real parent. Repeated
targets are legal and may repeat literal records; there is no assumed multiproof
compression. Target order, exact request and registration identities, network,
epoch, sequence, active indexes and signatures all need the prescribed checks.

The small registration recomputation requires 2,048 key calls, 3,072 leaf calls
and 3,069 internal calls: 8,189 calls before extra context/identity processing.
These are source-derived algorithm counts, not measured execution costs. R09
recomputes the format sizes and this sum; it does not implement these predicates.

## What acceptance would and would not imply

With correct pinned registration and binding, the returned target fields are
already the raw labels required by the batch theorem. This supports the
conditional inclusion

```text
p_accept <= p_raw + epsilon_registration + epsilon_authentication.
```

Any parsing, copying and output overhead must fit the same admitted memory and
deadline. Signature authenticity alone does not establish independently chosen
targets, genuine useful demand, scarce resources or another consensus opportunity.
Invalid input, unavailable data and local execution abort must remain different
outcomes; abort is not acceptance.

The response intentionally reveals source values, labels and authentication paths.
Only approved public/revealed data or appropriate encrypted content should enter
such a format. This is not a promise of confidentiality or access-pattern privacy.
It gives up the hidden-witness objective and creates reusable history.

At k=384 the 820,416-byte literal response is below 1 MiB but far larger than the
12,776-byte joint state cap in the worked theorem. This is the cost of the named
representation, not a universal information lower bound for every compressed
transcript. Keeping an arbitrary lifetime archive does not satisfy the old cap
by definition. A hidden wrapper would reopen the same-budget extraction question.

Correct resource registration, long-run state accounting, real timing, independent
demand and membership remain unresolved. The specification is not a permanent
issuer selection or an operational consensus protocol.
