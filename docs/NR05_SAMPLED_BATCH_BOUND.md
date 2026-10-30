# NR05 - sampled-batch bound: public mathematical note

30 October 2026. Adapted from the retrieved NR05 engineering report identified
as S2 in `test-evidence-m20-nr05/SOURCE_REGISTER.json`. This is a conditional
mathematical argument, not an implemented cryptographic system or external audit.
The public script checks its worked arithmetic, not the full codec proof.

## Model and success event

Use two fixed, honestly initialized chains of 1,024 vertices, a fixed known
source, 256-bit labels and one static classical ideal random function. Contexts,
source and graph must meet the stated oracle-independence conditions. Query
addresses are typed and injectively encoded. All correlated public/private
preparation, chosen algorithms, archives, helpers and proof state share one
M=102,208-bit budget. Independent source/code/parameter storage and access still
have physical costs; they are not free because they are independent of the oracle.

One real challenge asks for k independent uniform positions with replacement in
each context, all in one response window. Success means recovering every requested
raw label. There are Q=65,536 online oracle queries in total and R=2 synchronous
rounds in total, not per identity or per target. Queries in a round must be fixed
before that round's replies are available. Finite execution cutoffs apply even
to malformed states.

For a prechallenge cache-index set S, define C_0(S)=S and let C_(r+1) contain
C_r plus every vertex whose parents are in C_r. Let gamma_s be the maximum,
over sets of at most s vertices, probability that all requested targets are
inside C_R(S). This coverage relaxation grants extra computation inside the
proof; it is not a physical work or sustained-space measurement.

## Sampled-counterfactual composition

Let n be total vertices, d maximum indegree, b label bits, a requested positions
and t >= 1 a proof parameter. Define

```text
J_t   = n + t*(Q+a)
ell_t = ceil(log2(J_t+1))
h_t   = max(64, ell_t)
u     = ceil(log2(max(d,1)))
B_t   = b - 2*ell_t - u
```

Under the original injective finite-oracle-table codec lemma, for 1 <= s < n
and B_t > 0, one real batch's raw success probability p satisfies

```text
p^t <= min(1, binom(n,s)*gamma_s^t + 2^(M+h_t-s*B_t))
p   <= min(1, binom(n,s)^(1/t)*gamma_s + 2^((M+h_t-s*B_t)/t)).
```

The t-th root applies to the error term too. Leaving that error at its pre-root
exponent would materially overstate the bound.

### Argument outline and dependency

Condition on one oracle world and preparation. Start t independent hypothetical
challenge/randomness tapes from the same joint prepared state in that same oracle
world, with no state transfer between copies. If p_O is conditional success,
Jensen gives p^t <= E[p_O^t]. No independence of real successive audit outcomes
is assumed.

Collect labels whose first faithful parent/output occurrence precedes their
canonical query answer. If there are at most s such labels, all successful copies
fit one common s-index cache. That cache may depend on the transcripts, so take
a union bound over all binom(n,s) fixed sets; independently sampled target tapes
give the gamma_s^t factor only after each set is fixed.

If there are more than s fresh labels, the original codec retains the M-bit
state, framing and occurrence locators while omitting s canonical b-bit answers.
Including the final topological generator accounts for the n records in J_t.
The original global freshness/lockstep decoder is required to restore the missing
answers without circular dependence. Its injectivity is the substantive lemma,
not something proved merely by computing the parameters in the public script.
Counting gives the second branch. Sum the branches and take the root.

## Worked point A: 64 targets per context

Set n=2048, a=128, s=576 and t=512. A chain with s_j cached labels covers at most
min(1024,3*s_j+2) positions in two rounds. Balanced allocation 288/288 maximizes
the two-chain product and covers 866 positions per context. Therefore

```text
gamma_576 <= (433/512)^128
J_t = 33,622,016; ell_t = 26; h_t = 64; u = 0; B_t = 204
M+h_t-s*B_t = -15,232
p <= 16*(433/512)^128 + 2^(-29.75) < 2^-26.
```

Use binom(2048,576) < 2^2048. An exact certificate gives
(433/512)^128 < (3/125)*2^-25. The first term is below
(96/125)*2^-26 and the second below (1/8)*2^-26. Their sum is below
(893/1000)*2^-26. R07 checks the supporting integer inequalities and rational
powers without floating-point acceptance decisions.

## Worked point B: 384 targets per context

Keep the same graph, M, Q, R and s, but use a=768 and t=128.

```text
J_t = 8,488,960; ell_t = 24; h_t = 64; u = 0; B_t = 208
M+h_t-s*B_t = -17,536
p <= 2^16*(433/512)^768 + 2^-137 < 2^-136.
```

The preceding 128th power is below 2^-30, so its sixth power is below 2^-180.
The first term is below 2^-164; adding 2^-137 stays below 2^-136. R08 verifies
these comparisons exactly. This is a changed target/success law, not an upgrade
of NR01's original single-pair bound to 2^-136.

## Sampling, authentication and remaining gaps

The stated target law uses 1,280 fresh position bits for k=64 and 7,680 for k=384.
A short deterministic nonce expansion is not automatically this law. A different
sampler requires a matching coverage/security argument and anti-selection premises.
Counterfactual proof copies are not actual runs, extra state allowances or
independent real epochs.

For the separately specified visible response, faithful accepted fields can be
projected to raw labels without new label queries, conditional on correct pinned
registration and binding. Parsing, serialization, memory and deadline slack
must still be charged. Authentication error is added once outside the root.
An ideal 256-bit authentication domain with at most q_auth cumulative relevant
inputs gives collision error at most q_auth*(q_auth-1)/2^257. At q_auth <= 2^32
this is below 2^-193. This cap includes setup, preparation, history and verification;
it must not be replaced by the smaller online Q count. A separately justified
honest-context coupling allowance likewise belongs outside the root.

Neither the arithmetic nor the codec argument establishes concrete SHA delay,
a real two-round-to-seconds conversion, unrestricted lifetime history, malicious
registration security, hidden-proof extraction, independently useful demand or
resource-derived permissionless membership. Long-lived public disclosure remains
charged; the visible response can exceed the entire old state cap. No full
Groth16 setup/proving was executed for this public update. Eligibility remains
`NotEstablished` and weight `None`.
