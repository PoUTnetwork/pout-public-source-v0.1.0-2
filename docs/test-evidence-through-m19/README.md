# PoUT historical test evidence supplement (through M19)

Upload this package as an additional directory or commit to the existing PoUT public research repository, without overwriting source code.

- `EVIDENCE_INDEX.md`: historical record and material limits.
- `CHRONOLOGY_EARLIEST_TO_M19.json`: expanded chronology from earliest separately documented Phase 5, through Round 4–7, CWC, J1–J3, R1–R3, M1–M19.
- `EARLY_STAGES_AND_J1_J3.json`: statuses and relevant original-log fingerprints for early stages.
- `RAW_CARGO_LOG_DERIVED_TOTALS.json`: original Cargo group-sum extracts for Phase 5, Round 4, first CWC, J1 and R1, with original Git-blob SHA-1 values.
- `HISTORICAL_RESULTS_M01_M19.json`: historical milestone results and distinctions.
- `ORIGINAL_RUN_RECEIPT_FIELDS.json`: extracted source hashes, exit codes and original log hashes for M14–M19 (M18 had no new Rust run).
- `VERBATIM_LOG_EXCERPTS.json`: selected lines inspected in original logs, explicitly not complete original logs.
- `logs/`: actual current public Python reference output, both normal and optimized mode, plus earlier corresponding public reference outputs.
- `PUBLIC_REFERENCE_RUNS.json`: actual commands and captured output hashes for current reference runs.
- `FILE_SHA256_MANIFEST.txt`: hashes of all included files (except itself).

**Limitations:** This is not a full archive of original Cargo logs. Original log SHA values support later authentication if those originals are released; they do not independently prove execution merely by being listed here. The source-specific engineering-workspace results are not test results from the adapted public M19 workspace. Failed intermediate runs and omitted or ignored fixtures are disclosed. No external physical-resource security or blockchain consensus result is proven.

To reproduce public Python checks run the commands shown in `EVIDENCE_INDEX.md` from the root of the existing public repository.

## Verification and upload

Upload all ZIP contents to the root of the existing public repository. Keep the
`docs/test-evidence-through-m19/` directory structure and replace the root
`SOURCE_MANIFEST.sha256` with the one supplied here. That root manifest covers the
previously published files plus all new evidence files. The earlier file hashes
are inherited from the published M19 manifest and are not reauthenticated by this
new archive alone. Run the existing repository checker **after** upload:

```sh
python3 -I -B scripts/check_source_manifest.py
```

`FILE_SHA256_MANIFEST.txt` checks file contents relative to this evidence directory,
except itself. `SOURCE_MANIFEST.sha256` covers that file as well as the entire
existing public repository, but excludes its own entry to avoid self-reference.
The exact SHA-256 of this ZIP file is published separately because a ZIP file
cannot contain a checksum of itself without changing its own bytes.

**Authenticity caveat:** original engineering-workspace test logs are generally
not included, only historical source-specific results, verified excerpts and
original-log hashes where available. A hash of a withheld log does not let a
reviewer independently validate its full contents. This release should never
be described as complete raw logs or independent public validation of the
private workspace.
