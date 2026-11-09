# NE-01 source excerpts

Source revision: `789e90aa3884904ad504c86fcbe121d08608be3c`
Source file: `crates/pout-useful-storage-research/src/eligibility.rs`
Original complete-file Git blob SHA-1: `dc5e6e99734b4003191942cad9327c586b733db7`

`eligibility_excerpts.rs` contains three exact blocks from that file:
`audited_stock`, `evaluate`, and `OPEN_PRODUCTION_GATES`, in that presentation
order. The original source places the gate constant before the functions.
No function logic has been rewritten. Comments and surrounding definitions are
omitted. This file is **not a self-contained, compilable Rust crate**.

`established = unmet.is_empty()` concerns the model's acceptance criteria.
The seven production gates are returned separately; they do not block this
model-scoped status. Consequently this source does not mean production
consensus security has been established.

Private repository metadata and the complete source workspace are not bundled.
The opaque revision and blob identifiers preserve provenance without publishing
repository-owner identities. Public readers cannot reproduce the original
Rust run from these excerpts alone.
