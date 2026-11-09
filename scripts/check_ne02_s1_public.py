#!/usr/bin/env python3
"""Verify the additive public evidence release; does not mutate or publish."""
from __future__ import annotations
import hashlib
import json
import re
import sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
from research.post_nr05.verify_ne02_s1_record import verify
MANIFEST='NE02_S1_UPDATE_MANIFEST.sha256'
EXCLUDED={'SOURCE_MANIFEST.sha256',MANIFEST}


def main():
    entries={}
    for line in (ROOT/MANIFEST).read_text(encoding='utf8').splitlines():
        match=re.fullmatch(r'([a-f0-9]{64})  ([a-zA-Z0-9._/-]+)',line)
        if match is None: raise AssertionError('Malformed manifest line')
        digest,rel=match.groups()
        assert '..' not in Path(rel).parts and not rel.startswith('/') and rel not in EXCLUDED
        assert rel not in entries
        file=ROOT/rel
        assert file.is_file() and not file.is_symlink()
        assert hashlib.sha256(file.read_bytes()).hexdigest()==digest,(rel,'hash mismatch')
        entries[rel]=digest
    assert len(entries)>=18, 'Truncated update manifest'
    assert 'docs/test-evidence-ne01-s1/ne02-s1/NE02_S1_MARGIN_RELEASE.json' in entries
    verify()
    print(json.dumps({'status':'PASS','new_or_modified_files':len(entries),
                      'recorded_result':'NE01_SEAL_NO_STORAGE_TIME_MARGIN',
                      'native_tests_rerun':False,'production_security_validated':False},indent=2))

if __name__=='__main__':
    # Allow direct execution from any directory, without installed packages.
    sys.path.insert(0,str(ROOT))
    main()
