# Research checkpoints M14-M19

This note explains the selected public export and the corresponding engineering
research. Milestone labels are a research chronology, not consensus activation.
A statement about the larger engineering workspace is not a result for this
smaller public repository.

## M14: distinct opportunities and replay feedback

In the independent two-target/four-label toy game, the objective is to obtain
credits for two distinct canonical targets. With four attempts, the ordinary
relation succeeds in 6/16 worlds. A hidden filter reachable through replays of
already credited targets permits 7/16. Returning a fixed consumed response before
that hidden predicate restores 6/16 in the corresponding finite model.

`research/milestones/reproduce.py` implements a compact public dynamic-programming
reference for all sixteen declared small profiles: nine independent, three warm,
three correlated and one aliased-target profile. It retains finite support and
canonical credits. It is a public reference adaptation, not the original Rust
optimizer, its full replay transcripts or the separate DAG experiment.

The original M14 research also reproduced order sensitivity when duplicate
candidates were removed only after DAG processing. This public update does not
include that DAG implementation; it does not claim that the Python reference
reproduces a block-ordering result.

## M15: exact-intent binding

The native research intent commits to the versioned domain and fixed layout,
network, epoch, challenge, policy, producer label, sorted unique parents and
complete canonical body before work. Native Node inputs include that intent and
the source context. Tests of changed parents/body reached recomputation rather
than being rejected only by a stale outer tag. Equivalent copied candidates were
normalized before the bounded DAG model.

The retained M17 fixture below carries this version-two exact-intent encoding.
Its outer target has five framed fields: Node domain, `epor-node`, intent,
32-byte source-context digest and 32-byte shard root. Intent fields use the
published frame convention; the reference checks structure and literal bytes.

Field binding alone does not authenticate an independent issuer, prove a fresh
physical resource, or prevent fresh work on the same underlying source. The full
native relation and its DAG/CWC integration are not exported here. A vector is
not a substitute for that missing implementation.

## M16: separate rank/work probability models

The engineering checkpoint evaluated 21,086 finite scenario/world cases in
separate joint-information and direct-query laws. There were 41,806 modeled Rank
calls: 40,010 fresh and 1,796 repeats. These totals are not independent native hash
samples, packet traffic, throughput, or a single unified security experiment.

The direct-query bound used by the later selection counterexample, under its
ideal-law and full-history assumptions, is:

```text
B(Q,R,n,q,p) = min(1,
                    1 - (1-p)^R,
                    1 - (1-p)^min(R, floor(Q/n)) + R*p/q)
```

Q and R are the charged Node and Rank query budgets, n is the common path length,
q the label alphabet size, and p the fixed Rank-pass probability. Helper,
preprocessing, verification and other relevant access must not silently vanish
from the charged history. This equation is not a theorem about arbitrary native
BLAKE3 computations or real private resource expenditure.

The public script evaluates this conditional expression with exact fractions for
the M18 parameters. It does not rerun the full M16 corpus, its G/H codec study or
its all-profile event-mask checks. The original aggregate counts are recorded in
`research/milestones/recorded_results.json`, explicitly as reported results.

## M17: full-pair Rank and expected thresholds

With frame(x) = LE32(len(x)) || x, the native research uses:

```text
V = frame("PoUT-research/verified-common-vertex/v2")
    || frame(T) || frame(common6)
RankInput = frame("PoUT-research/exact-intent-rank/v1") || frame(V)
r = BLAKE3-256(RankInput), interpreted as unsigned big endian
qualified = r < tau
```

The expected canonical policy is supplied independently of the producer and
checked before source/Node work and before Rank. A producer-chosen all-pass policy
cannot authorize itself. Zero, half and all-pass were separate diagnostic policy
views, not producer options. Because policy is in the intent, changing it also
changes the target and digest; cross-policy monotonicity is not asserted.

This export includes the original synthetic half-policy Rank input, target,
common-six labels, output `61cee385...c9b0f62b`, and threshold. The new public Rust
digest test invokes the existing BLAKE3 dependency on that input. It is supplied
for downstream execution and was not run during packaging. The stdlib Python
reference verifies the framing, not the BLAKE3 digest computation.

A measured full native procedure using eight Nodes is not an eight-Node minimum
per Rank attempt: the public Rank can be tried after the six unmasked prefix
Nodes, with the tail deferred until a pass. Correct bytes do not establish
independent randomness, necessary resource cost, ownership or honest share.

## M18: choosing a target can transmit information

For two targets with independent first labels in an alphabet of sixteen, a helper
reads both roots and selects the first with label zero. The recipient learns
that selected label and requests the remaining five labels, then one fresh Rank.
The quotient reference enumerates 512 equally weighted worlds: 62 select a
target and 31 satisfy the success event. Ignoring helper queries and inserting
Q=5, R=1, n=6, q=16, p=1/2 gives the invalid application 16/512. Including the
helper's queries gives Q<=7 and the conservative bound 256/512. The correctly
scoped theorem is not contradicted.

The five tail labels are integrated out; this is not a complete enumeration of
six-label random-function tables. Costs are symbolic oracle-game queries, not
measured native operations. A separate complete 64-world example selects each
target 32 times, yet the selected root is zero 48 times. Uniform target labels do
not imply an unbiased selected path.

Independently fixing the complete exact target before helper observation removes
this particular selection channel. Fixing only a file hash is insufficient if
other target fields remain selectable. This does not solve delegation, shared
backing, subsequent helpers or self-generated demand.

## M19: executable causal byte/history comparison

The exported Rust adapter parses actual bounded canonical request bytes and
compares every event to a separately addressed semantic reference. Both use a
prescribed deterministic toy world, not a measured independent random-oracle law.
Every event preserves the key, reply, freshness, pre-answer classification,
immutable first-Rank classification, actor/category and charged work.

The eight traces cover:

1. Six sequential Nodes followed by Rank.
2. Preparation by a helper using generic labels.
3. Helper-selected targets: seven global Node requests versus five by the recipient.
4. A guessed Rank later completed: the repeat remains cached with its original class.
5. Off-proposed-path preparation, which does not establish the proposed path.
6. Distinct full Rank vectors for one target.
7. Storage/Delivery aliases: fourteen category keys versus seven global keys.
8. Nine malformed, out-of-catalog or unsupported requests rejected before sampling.

Recorded totals are 63 events, 44 Node calls (38 fresh, 6 cached), 10 Rank calls
(8 fresh, 2 cached), 9 rejections and 17,903 request-payload bytes. These aggregate
different scripted views; they are not one physical traffic history or payouts.
A repeated request may cost a modeled call without creating a fresh sample.

The byte machine, semantic reference, guards, eight test functions and golden
vectors are included. Public adaptations remove serialization dependencies and
change report formatting only; see VALIDATION_M19_PUBLIC.md. No parser rule,
experimental predicate, counter, fixture case or scientific limitation was
removed from the Rust experiment.

## Shared limitations

All modeled calls are visible to the experiment by assumption. A real adversary
can compute privately. Rejecting a byte string at a local API does not prevent
private hashing. Neither category names nor an identity label establish physical
independence. Copied histories and fork-safe consumption require separate design.
The result remains consensus_eligibility=NotEstablished and consensus_weight=None.
No production blockchain, live traffic service, currency or migration is enabled.
