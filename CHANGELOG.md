# Changelog

## Public finite-reference update - October 21, 2026

- Add exact four-bit sampled-pair reference models and a separate list/character-based checker.
- Publish all 65,536 advice encoders, 256 response tables, 64 target rows, twelve fixed-strategy repeated-audit rows, 154 bucket-advice profiles and two small public-information counterexamples.
- Add 128 deterministic request vectors, 32 chunk known answers and sixteen synthetic byte frames with explicit expected acceptance/rejection layers.
- Add 40 new Python unit/mutation tests; the runner also executes the twelve existing Python reference tests (52 unique tests, not multiple-run totals).
- Check the same suite under normal and optimized Python; explicit validation does not disappear under optimization.
- Add clearly labeled README-only publication markers under blockchain/.
- Correct obsolete references to an included Actions workflow. No workflow or Cargo dependency change is included.
- Refresh the complete source manifest for the merged public tree.

## October 20, 2026 — Public byte-observation review export

- Add selected requested-byte verification and atomic, nonrefilling byte-budget primitives.
- Add a deterministic actual-file/local-Unix-IPC example and 23 Rust regression definitions.
- Add an independent Python reference and read-only source-manifest checker.
- Extend the existing resource-controls example with an explicit invalid-relation rejection.
- Update existing architecture, security, research-status, example and test documentation.
- The initial package proposed a public-source CI workflow; that file was removed before upload. Rust validation of this export remains pending; no inherited engineering-suite result is claimed for it.
- Keep CWC/Groth16 libraries, Cargo.toml, Cargo.lock and third-party notices unchanged.


## Research addendum — October 19, 2026 (validation pending)

- Added a standalone public-only synthetic accounting-controls example and 11 targeted integration tests.
- Modeled half-open challenge freshness windows, issuer/scope/provider/resource binding, stable authorized key-family consumption, duplicate presentation and fork-local views.
- Preserved counterexamples where separately admitted families or challenges yield distinct logical entries without proving additional independent resources.
- Updated existing research, security and example documentation. Original v0.1.0 validated results remain historical; new addendum validation is not yet claimed.

## 0.1.0 — Initial public prototype package

- Selected bounded CWC serial and real parallel execution with canonical encodings and adversarial correctness tests.
- Verification-only Groth16 public evidence importer, synthetic proof vectors and executable positive/negative controls.
- Standalone locked Rust workspace, technical limitations and retained dependency licenses.
- Manual upload package; no automated publication integration.
