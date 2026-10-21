# Public finite information models

This is a portable demonstration and regression dataset, not the full resource
protocol, a new cryptographic primitive, a formal external audit or a deployed
consensus rule. All model inputs are synthetic and explicitly finite.

## 1. One coalition, one advice bit

Let X=(A0,A1,B0,B1) be uniform on {0,1}^4. An encoder f maps each of the sixteen
sources to one bit before the challenge. A challenge (i,j) is uniform on {0,1}^2,
and the requested response is (Ai,Bj). A decoder knows the retained advice and
the challenge. Q0 permits no source query. Q1 permits exactly at most one raw
source-bit query, selected from all four coordinates using advice and challenge.
There is no source-dependent public view, earlier response history, external
helper or arbitrary-predicate query in this particular game.

The retained advice belongs to the whole coalition, not separately to each label.
Every encoder is a sixteen-bit truth table. We enumerate all 2^16=65,536 encoders.
For cell S_z={x:f(x)=z}, Q0 computes

```
score0(f) = sum_(z,c) max_y |{x in S_z : target_c(x)=y}|.
Pr[success] = score0(f) / 64.
```

For Q1 the maximization also considers the choice of source coordinate and the
two possible query replies. Exact enumeration gives maxima 28/64 and 48/64.
The separate checker derives Q0 from explicit lists and Q1 from direct parity-
character sums. It does not call the generator's posterior scorer.

These maxima concern this declared finite game only. They cannot be assigned to
real providers, graph hashes or consensus opportunities without a justified
mapping of all information, initialization and permitted work.

## 2. Same-source repetition is not independent-source repetition

The two fixed response tables 0 and 255 always answer (0,0) and (1,1). The source
chooses one table before challenges. For t independent challenges on the SAME
source, that fixed strategy succeeds in all rounds with probability

```
p_fixed(t) = 1/8 + 2^(-t-1) + 2^(-2t-2),  1 <= t <= 12 in this dataset.
```

The table records exact rational values alongside the incorrect IID comparison
(7/16)^t. For example, t=2 gives 17/64 rather than 49/256. This is a concrete
counterexample to multiplying marginal single-round probabilities. The exported
program evaluates this particular strategy; it does not search all adaptive
multi-round policies or establish the full protocol's security.

## 3. Public information and helpers

`masked_public_view.csv` enumerates independent fair bits U,R with V=U XOR R.
Seeing V alone leaves the best guessing probability at 1/2. Seeing both V and R
makes U certain. Marginal independence does not justify ignoring correlated
information in the complete view.

`gf4_affine_worlds.csv` enumerates Yi=A*i+B in GF(4), with independent uniform A,B.
Every pair of coordinates is uniform, but the whole vector has only sixteen
possible values, not 256. Pairwise independence is not joint independence.
These elementary controls are not a claim that the deployed hashes share that
particular algebraic dependence.

## 4. Source-only bucket advice and first hits

A hidden value is uniform in an alphabet of q elements. The source-dependent
advice selects one of A equally sized buckets; a fixed policy tests up to T
values in that bucket. Its hit probability is

```
p_bucket = min(1, A*T/q).
```

The dataset contains 154 fixed profiles for q in {2,4,8,16,32}. It enumerates
hidden source values for this specific bucket strategy. It is not the full
indexed-oracle experiment, does not include oracle-dependent advice, and does
not prove a memory, bandwidth or space-time lower bound.

## 5. Reproducibility and limits

`models.py` is the bounded generator-side model. `generate.py` produces canonical
CSV/JSON in a fresh output directory. `verify.py` separately recalculates every
released row and checks the exact case inventory. File bytes are also covered by
the repository source manifest. Every result is exact integer/rational arithmetic;
there is no Monte Carlo error or invented benchmark clock.

The fixed parser limits exclude arbitrary huge research inputs. Runs do not fetch
web resources, require credentials, create tokens or modify a network. The local
checker runs 52 unique Python unit/mutation tests plus the dataset validation.
Dataset rows and arithmetic visits are not additional unit tests.

`consensus_eligibility = NotEstablished`
`consensus_weight = None`
