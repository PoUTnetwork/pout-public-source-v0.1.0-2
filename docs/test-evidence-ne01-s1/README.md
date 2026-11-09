# Research evidence: NE-01 through NE-02 S1

`ne01/` retains a previously released, independently checked public evidence
bundle with its own SHA-256 manifest and evidence verifier. Its tests and logs
were recorded before this publication. The PDF is a one-page editorial brief.

`ne02-s1/` contains the byte-identical recorded release output and a separate
public extraction of its recorded local checks. The output explicitly reports
**no storage/time margin**. The original private native test workspace was
not copied into this archive; do not imply that the Python script proves the
original native test execution.

Standalone verification (from the merged public repository):

```
python3 -I -B docs/test-evidence-ne01-s1/ne01/verify_evidence.py
python3 -I -B research/post-nr05/verify_ne02_s1_record.py
python3 -I -B -m unittest discover -s tests/public -p test_ne02_s1_record.py
python3 -I -B scripts/check_ne02_s1_public.py
```

The final public repository must maintain its own complete
`SOURCE_MANIFEST.sha256` and the separate
`NE02_S1_UPDATE_MANIFEST.sha256` for added files. The transfer worker should
stage the intended files, recompute the full source hashes from the combined
repository and run the existing source-manifest checker before publication.
