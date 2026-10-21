# Architecture

This workspace has two explicit members and no operational service dependencies.

```text
Synthetic operations -> serial reference / dependency graph + real workers
                     -> accepted state + canonical results + BLAKE3 root

Public synthetic bundle + independently obtained expected pins
                     -> bounded decoding + key/statement binding
                     -> necessary key coherence + Groth16 pairing verification
                     -> finite-relation opportunity identifier
```

These are separate experiments. A successful pairing check does not authorize a CWC operation, allocate consensus weight or settle a payment. The exported package has no bridge that silently turns a proof into a block or reward.

## CWC boundary

The host supplies a bounded existing counter state, typed operations and cancellation flag. Read/write conflicts are derived by the implementation. The serial path supplies the reference candidate order. The parallel path uses immutable per-wave snapshots, joins its actual scoped workers, merges writes canonically and exposes no new state until complete acceptance. Rejection and local execution abort are distinct. Callers must separately bound concurrent execution attempts; this is not a global scheduler or durable state store.

## Verification boundary

The host supplies an exact bounded evidence frame and separately authenticated expected hashes of the entire verifying key and full enrollment/audit statement. The importer rejects malformed, noncanonical, subgroup-invalid or identity encodings and checks necessary beta/delta cross-group consistency. The original Groth16 pairing verifier remains authoritative for the finite public relation.

A caller choosing its own key and statement pins has not acquired external authority. The low-level `verify` API assumes an already trusted prepared key; prefer `public_verify::verify_bundle` at an untrusted evidence boundary. Neither API establishes that setup was honest or that a key corresponds to an independently audited circuit.

## Dependencies and I/O

CWC uses the Rust standard library, BLAKE3 and typed errors. The verifier uses Bellman, BLS12-381, SHA-256 and typed errors. The original verifier and CWC examples use in-memory public constants and synthetic state. The byte-observation addendum additionally uses controlled fixture files and local Unix streams. Verification does not make live network requests or need a filesystem witness. Cargo may fetch public dependencies at build time; dependencies are not vendored implementation code.

## Deliberately outside this package

Encrypted storage/CDN operations, authenticated service enrollment, discovery, operational authorizations and settlement, persistent recovery, publication quotas, wallets, tokens, consensus eligibility, GHOSTDAG ordering and consensus activation are not exported. They must not be inferred from these isolated experiments. No cross-platform runtime or multi-device interoperability claim follows from a Linux build.

## Isolated public accounting-controls addendum

The standalone `pout-public-verifier` example `resource_controls` adds no operational protocol dependency and is not imported into the Groth16 verifier library. It models independent trusted context, a pre-authorized key-family map and a bounded in-memory consumption view. Successful records are conditional on assumed relation validity and external labels. Cloned views are alternate histories. No network, ledger, customer funds, persistent resource measurements or consensus authority is provided.

## Requested-byte and byte-budget addendum

```text
Caller-pinned manifest + exact request
    -> file chunk read -> request-bound frame
    -> receiver framing/context/content checks -> VerifiedResponse or typed failure

Controller-assigned budget
    -> atomic reservation -> metered writes -> completion accounting
    -> unused confirmed capacity released; written/uncertain capacity not refunded
```

These are separate primitives. A `VerifiedResponse` does not authenticate customer demand and a `Lease` does not authenticate a physical link. The standard-library `Write` boundary must be controlled; arbitrary programs can bypass it. The new public example composes these primitives only for an isolated deterministic experiment.

Public adaptation boundary: JSON controllers, random-source preparation, private artifacts and full internal campaign supervisors are not exported. Serialization annotations are removed from the extracted cores. A separate deterministic fixture, public example and independent reference make the reduced scope inspectable without adding dependencies. No new circuit, prover, joint proof, consensus weighting or service payment adapter is included.


## Portable finite-reference layer

The October 21 update adds research/finite-models and synthetic fixtures under
tests/fixtures. These standard-library Python tools are independent of the Rust
consensus/service paths. They test declared information-game assumptions and
public encoding controls; they neither assign economic weight nor extend the
Groth16 statement. README-only markers under blockchain/ describe the
publication boundary and contain no executable implementation.
