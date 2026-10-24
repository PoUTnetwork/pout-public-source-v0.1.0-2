# Public research architecture

The public workspace retains two members: pout-cwc-research and
pout-public-verifier. No operational services depend on this exported workspace.

CWC is a bounded artificial-state serial reference and parallel executor within
one already ordered candidate. It is not BlockDAG ordering or consensus admission.
The verifier performs genuine Groth16 pairing checks against pinned synthetic
fixtures, but the proving circuit, setup and prover are not exported.

The earlier resource controls and byte-observation examples are separate finite
experiments. Their identities, clocks, policies and installed quotas are assumed
inputs, not an independent source of scarce service or customer demand.

## New M19 path

```text
bounded request bytes -> canonical parser -> byte-key cache and history
                                             |
fixed scripted world -------------------------+
                                             |
separate typed-key reference -> ordered-event comparison
                                             |
                                 bounded research report
```

Caller and category labels are recorded, not used to create new key namespaces.
A query's first Rank classification is immutable. The semantic reference is
separately addressed and does not decode through the byte implementation. This
is a finite differential comparison, not independent native random-oracle law.

The module is included by an example and an integration test using path
attributes. It adds no library or service dependency and changes no existing CWC
execution or verification behavior. Report-only Serde traits are removed for the
public export; bounded text output uses the standard library.

A separate cwc-crate integration test checks a public retained M17 BLAKE3 vector,
using the dependency already present there. The location does not imply any
change to CWC semantics or new consensus integration.

The standard-library Python reference replays selected finite M14/M18 examples
and reconstructs public encodings. It is independent of Rust compilation and
does not substitute for executing the new Rust tests.

No network, wallet, currency, reward, consensus weight or migration operation is
introduced. See [publication scope](PUBLIC_RELEASE_SCOPE.md) for omitted code.
