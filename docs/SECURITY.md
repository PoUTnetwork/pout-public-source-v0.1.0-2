# Security limitations

This is an experimental source package, not an audited security product. Do not use it to secure real assets, authorize production transfers or grant consensus influence.

## Cryptographic verification assumptions

- Groth16 acceptance is conditional on a trusted verifying key, its actual circuit, cryptographic assumptions and sound setup. The circuit/prover are outside this verifier-only export. No independent circuit-to-key or trusted-setup audit is supplied.
- Complete external key and statement pins must be authenticated independently. Public test constants are not a production trust service. Necessary key coherence is not full CRS validity.
- A proof can be publicly re-encoded without new witness work. Proof bytes are not a scarce resource or a safe unit of new eligibility. The stateless verifier does not track consumption or fork history.
- No proof here establishes persistent custody, independent demand, real useful delivery, Sybil resistance, collusion resistance, honest timing, fresh unpredictable selection, bandwidth scarcity or secure consensus weight. `consensus_eligibility = NotEstablished`.
- Synthetic fixture keys, proofs, enrollment identifiers and randomness are intentionally public. No proving parameters, secret scalar, private signing key, raw private witness, certificate or wallet is included.

## Synthetic freshness and consumption controls

The optional `resource_controls` example is an isolated finite-policy model. Every positive `RelationObservation` is explicitly assumed for the model; no pairing verification, witness proof or physical-resource verification occurs in this module. Its accepted records are logical entries, not protocol consensus eligibility.

Challenge issuance, timestamps, identity bindings, resource-anchor provenance and key-to-family admission must come from a separately authenticated authority. An attacker-controlled source tag or family mapping defeats assumptions of uniqueness. Key rotation deduplicates only when both keys were independently admitted to the same family. Distinct admitted families or challenge IDs can create distinct logical records from one synthetic resource. Fork-local consumption views are alternatives and must not be summed as canonical chain weight. No cross-fork consensus, persistence, challenge unpredictability, true service freshness or resource scarcity is proved.

## Execution assumptions

- CWC tests use finite artificial counters; they do not create currency or connect to operational accounting.
- Serial and parallel agreement on the tested corpus is finite evidence, not a proof of all-input correctness. The operation evaluator is shared, with an additional independent arithmetic oracle in tests.
- Handled local cancellation, worker panic or thread-spawn failure is an incomplete local attempt, not evidence that the candidate is invalid. Fatal process/allocator/host failures are outside in-memory recovery guarantees.
- Resource bounds apply per candidate/attempt. There is no process-wide concurrent-attempt admission or hostile-host resource isolation. Small tasks may be slower in parallel; no performance advantage is claimed.
- The optional `research-controls` feature is an isolated scheduling experiment, disabled by default. Its local synchronization budget does not define a consensus deadline.

## Release and dependency boundary

Only selected Rust research components, public synthetic vectors, documentation and license notices are included. No services are deployed and no automated publication workflow is configured. Dependencies come from public crates.io packages fixed by Cargo.lock; normal downstream dependency review still applies.

Validation was performed on Linux x86-64 only. Windows, macOS, Android and iOS runtime behavior is untested. iPhone upload instructions describe transferring files to GitHub, not executing Rust on iOS.

Public source and synthetic fixture fingerprints may permit correlation if other copies become available. Removing private account details does not guarantee anonymity. Required upstream third-party copyright and attribution notices are deliberately preserved.

For a suspected defect, prepare a minimal reproduction using synthetic inputs. Do not attach secrets or real user data to public issue reports. No private reporting channel or response-time commitment is provided in this package.
