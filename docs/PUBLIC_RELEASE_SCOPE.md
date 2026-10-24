# Public release scope

This is a selected research export through M19, not a complete release of the
native PoUT architecture. Earlier public files are retained. New public files
include the bounded M19 causal experiment and semantic reference, its eight test
functions, failure-sticky initialization helper, golden encodings, a retained
M17 Rank-input fixture and a compact M14/M18 Python reference.

M14's separate DAG controls, the complete M15 native work relation, the full M16
multi-model corpus and the complete M17 native experiment are not supplied here.
Their specified behavior and historical counts are described, with that limitation
explicit. The retained Rank input is sufficient for a digest check, not for
regenerating or proving its resource witness. The compact Python reference is a
public adaptation, not a claim of byte-identical export of the original optimizer.

The original verification-only scope remains: no proving circuit, setup generator,
prover, production networking, complete blockchain, runtime databases, source
archives, wallets, private signing keys or operator state are exported. Later
milestones are excluded. The blockchain/ notices remain notices, not modules.

This ZIP is an overlay for the existing public repository, not a replacement for
its complete contents. Cargo.toml, Cargo.lock, dependency versions and all license
files are unchanged. The M19 source adaptation removes Serde-only reporting and
uses bounded text output with the existing dependencies. The full experiment
logic and test cases remain.

There is no .git history, automated push workflow, installer or dependency-fetching
script in the overlay. The included review scripts do not publish or modify any
repository. Review of this source does not establish an external security audit.
