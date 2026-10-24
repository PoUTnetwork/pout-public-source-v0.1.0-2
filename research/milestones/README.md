# Selected milestone references through M19

Run `python3 -I -B research/milestones/reproduce.py` from the merged repository.
Only Python 3.10+ and its standard library are required. The script reads public
fixtures, performs bounded finite calculations and writes its report to stdout.
It neither builds Rust nor downloads dependencies nor publishes files.

M14 is a compact sixteen-profile posterior/canonical-credit reference adapted
from the documented finite experiment. M18 reproduces the quotient counterexample
and the separate conditioning example. The helper-query accounting is symbolic.
The byte checks reconstruct M19 goldens and the retained M17 Rank input. BLAKE3
computation for the latter is supplied as a separate unexecuted Rust test.

`public_reference_results.json` is the output obtained when preparing this
public package. `recorded_results.json` contains separately labeled historical
engineering summaries. Neither is a general consensus/resource-security proof.
The full native M15/M17 constructions and M16 corpus are not in this export.
