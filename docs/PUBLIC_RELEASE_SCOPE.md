# Public release scope

As of 9 November 2026, this repository contains selected public research
material, not an operational PoUT blockchain or established production
consensus. The public native-source boundary and later research-documentation
boundary are different.

## Native public source through M19

The earlier public CWC executor, verification-only Groth16 importer, synthetic
fixtures, byte-observation controls and finite models remain in place. Selected
M14-M19 material includes the bounded M19 causal experiment, typed semantic
reference, eight test functions, failure-sticky initializer, golden encodings,
retained M17 Rank input and compact M14/M18 Python references.

The complete M14 DAG corpus, M15 native work relation, M16 multi-model corpus
and M17 resource-witness experiment are not supplied. A retained Rank digest
check does not regenerate or prove a resource witness. The M19 reporting
adaptation uses the existing dependencies without changing native semantics.

## Selected research through M41 and NR05

The October 30 update added 27 attributed summaries for M20-M41 and NR01-NR05,
two NR05 mathematical/format notes, a source register and an evidence index.
It also added nine public executable Python arithmetic/finite-model checks
and three actual execution records. Normal and optimized runs repeat the same
nine references; the eight integrity-checker self-tests are separate.

Historical summaries are not original full runtime logs. These references do
not reproduce unpublished native, circuit or consensus implementations. The
original NR05 research was mathematical; public arithmetic executions do not
turn it into a working resource-proof protocol.

## Research extension through NE-02 S1

The November 9 overlay adds selected NR06-NR22 narrative continuity, an NE-01
first-scaffold test-model result, and an NE-02 S1 storage/time attack baseline.
The test model reported `Established` eligibility and weight 512 after eight
model epochs and 16 verified audit passes; this is **not** production consensus.
S1 then demonstrated that the model's original sealing does not enforce
storage scarcity: cheaper data-regeneration strategies still passed audits.

The included NE-01 evidence is a retained public release of recorded local
results. The NE-02 S1 release-result JSON was copied byte-identically from
the recorded local research execution and has its own reproducible arithmetic
checks. No full private native source, proof generator or working mainnet is
exported. No later NE-02 stage is included.

See [release note](PUBLIC_NE01_NE02_S1_RELEASE.md), [S1 report](NE02_S1_STORAGE_TIME_MARGIN.md),
[continuity note](RESEARCH_NR06_NR22_CONTINUITY.md) and
[current status](RESEARCH_STATUS.md) for classifications and limits.

## Material not exported

No complete proving circuit, setup generator, prover, production networking,
complete blockchain, runtime database, private source archive, wallet,
secret signing key or operator state is included. Hidden-proof extraction,
resource-derived membership and full network safety/liveness remain open.
Blockchain directory notices remain notices, not deployed modules.

## Update and validation boundary

The November package is an overlay for the existing public repository. Preserve
all inherited source, Cargo.toml, Cargo.lock, dependency versions, licenses,
third-party notices and historical execution records. The existing October
`UPDATE_MANIFEST.sha256` stays in use with updated hashes for changed public
files; `NE02_S1_UPDATE_MANIFEST.sha256` records the newly added payload.
The root `SOURCE_MANIFEST.sha256` covers the complete merged source tree.

No fresh Rust build/test, cryptographic setup/proving or hosted workflow is
claimed for this publication. Earlier local execution evidence is attributed
to its original run and checked for integrity, not passed off as a new run.

No Git history, automated push workflow, installer or dependency-fetching
script is distributed in this overlay. Review scripts do not publish or
modify a repository. There is no external security certification, production
readiness, operational token entitlement or demonstrated trading value.
