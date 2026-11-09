#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Read-only checks for the historical M20/NR05 and later NE-02 S1 extensions.

The historical 189-file baseline and 47-file update inventory remain pinned.
The new stage has a second manifest; the root source manifest covers both.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
import re
import sys
import tempfile
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
BASELINE = 'docs/test-evidence-m20-nr05/BASELINE_MANIFEST.sha256'
BASELINE_SHA256 = '8582fc7ea5215f130ea00f80b34ec6e734bf921a6c3980006d99c2df970bc4e0'
BASELINE_COMMIT = 'b5cb49b7fbaf76d4f523dcfe014d57829b438dd4'
UPDATE = 'UPDATE_MANIFEST.sha256'
EXTENSION = 'NE02_S1_UPDATE_MANIFEST.sha256'
SOURCE = 'SOURCE_MANIFEST.sha256'

class IntegrityError(ValueError):
    pass

def require(condition: bool, message: str) -> None:
    if not condition:
        raise IntegrityError(message)

def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1048576), b''):
            h.update(block)
    return h.hexdigest()

def parse_manifest(data: str, own_name: str) -> dict[str, str]:
    out: dict[str, str] = {}
    for line in data.splitlines():
        m = re.fullmatch(r'([a-f0-9]{64})  (.+)', line)
        require(m is not None, 'Malformed manifest entry')
        checksum, name = m.groups()
        posix = PurePosixPath(name)
        require(not posix.is_absolute() and '..' not in posix.parts
                and '\\' not in name and ':' not in name
                and posix.as_posix() == name
                and all(32 < ord(ch) < 127 for ch in name),
                'Unsafe manifest path')
        require(name not in {own_name, SOURCE} and name not in out,
                'Self-referential or duplicate manifest entry')
        out[name] = checksum
    require(bool(out), 'Empty manifest')
    return out

def read_manifest(name: str) -> dict[str, str]:
    return parse_manifest((ROOT / name).read_text(encoding='utf-8'), name)

def verify_files(records: dict[str, str]) -> None:
    for name, expected in records.items():
        target = ROOT
        for part in PurePosixPath(name).parts:
            target = target / part
            require(not target.is_symlink(), 'Symlink rejected: ' + name)
        require(target.is_file(), 'Missing regular file: ' + name)
        require(digest(target) == expected, 'Hash mismatch: ' + name)

def inventory() -> set[str]:
    result: set[str] = set()
    for current, directories, files in os.walk(ROOT, followlinks=False):
        for name in directories + files:
            require(not (Path(current) / name).is_symlink(), 'Symlink in inventory')
        for name in files:
            path = Path(current) / name
            require(path.is_file(), 'Nonregular inventory file')
            result.add(path.relative_to(ROOT).as_posix())
    return result

def self_test() -> dict:
    passed = []
    def reject(name, fn):
        try:
            fn()
        except (IntegrityError, OSError):
            passed.append({'id': name, 'status': 'PASS', 'observed': 'rejected'})
        else:
            raise IntegrityError('Expected rejection missing: ' + name)
    with tempfile.TemporaryDirectory(prefix='pout-check-') as tmp:
        root = Path(tmp)
        f = root / 'sample.txt'
        f.write_bytes(b'public integrity control\n')
        chk = digest(f)
        entry = chk + '  sample.txt\n'
        parse_manifest(entry, UPDATE)
        require(digest(f) == chk, 'Good digest rejected')
        passed.append({'id': 'I01_valid_file', 'status': 'PASS', 'observed': 'accepted'})
        f.write_bytes(b'changed\n')
        reject('I02_changed_file', lambda: require(digest(f) == chk, 'Changed file'))
        f.unlink()
        reject('I03_missing_file', lambda: digest(f))
        reject('I04_duplicate_path', lambda: parse_manifest(entry + entry, UPDATE))
        reject('I05_parent_traversal', lambda: parse_manifest(chk+'  ../unsafe\n', UPDATE))
        reject('I06_self_reference', lambda: parse_manifest(chk+'  '+UPDATE+'\n', UPDATE))
        reject('I07_noncanonical_path', lambda: parse_manifest(chk+'  a//b\n', UPDATE))
        actual = root / 'real.txt'
        actual.write_bytes(b'file')
        f.symlink_to(actual.name)
        reject('I08_symlink', lambda: require(not f.is_symlink(), 'Symlink'))
    return {'schema':'pout-update-checker-selftests-v1', 'tests_passed':len(passed),
            'tests_failed':0, 'results':passed, 'scope':'manifest checker controls only'}

def verify(mode: str) -> dict:
    if mode == 'new-only':
        require((ROOT / EXTENSION).is_file(), 'November manifest missing')
        additional = read_manifest(EXTENSION)
        verify_files(additional)
        return {'schema':'pout-update-integrity-v2','status':'PASS',
                'mode':'new-only','new_source_entries':len(additional),
                'inherited_source_bytes_verified':False}
    require(digest(ROOT / BASELINE) == BASELINE_SHA256,
            'Historical 189-entry baseline differs from published source')
    baseline = read_manifest(BASELINE)
    require(len(baseline) == 189, 'Unexpected historical baseline count')
    prior = read_manifest(UPDATE)
    require(SOURCE not in prior and EXTENSION not in prior, 'Circular update manifest')
    historical = dict(baseline)
    historical.update(prior)
    historical[UPDATE] = digest(ROOT / UPDATE)
    require(len(historical) == 230, 'Unexpected October merged source count')
    old_meta = json.loads((ROOT / 'PUBLIC_UPDATE.json').read_text(encoding='utf-8'))
    require(old_meta['base_commit'] == BASELINE_COMMIT, 'Historical base mismatch')
    historical_payload = set(prior) | {SOURCE, UPDATE}
    require(old_meta['overlay_files'] == sorted(historical_payload),
            'October update inventory changed')
    require(old_meta['overlay_file_count'] == len(historical_payload),
            'October update count mismatch')
    require(old_meta['merged_manifest_entries'] == len(historical),
            'Historical merged count mismatch')
    extension_present = (ROOT / EXTENSION).is_file()
    expected = dict(historical)
    additional: dict[str, str] = {}
    if extension_present:
        additional = read_manifest(EXTENSION)
        require(not (set(additional) & set(historical)),
                'November extension overwrites a historical file')
        expected.update(additional)
        expected[EXTENSION] = digest(ROOT / EXTENSION)
    source = read_manifest(SOURCE)
    require(source == expected, 'Full source map differs from stage manifests')
    if mode == 'merged':
        verify_files(expected)
    elif mode == 'overlay-only':
        verify_files(prior)
        if extension_present:
            verify_files(additional)
            require(inventory() == historical_payload | set(additional) | {EXTENSION},
                    'Unexpected update-only files')
        else:
            require(inventory() == historical_payload, 'Unexpected historical update files')
    elif mode == 'new-only':
        require(extension_present, 'November manifest missing')
        verify_files(additional)
    elif mode == 'maps-only':
        require(extension_present, 'November manifest missing')
        verify_files(additional)
        # Does not replace final byte-by-byte verification of inherited files.
    else:
        raise IntegrityError('Unsupported verification mode')
    return {'schema':'pout-update-integrity-v2', 'status':'PASS', 'mode':mode,
            'historical_source_entries':len(historical),
            'new_source_entries':len(additional), 'merged_source_entries':len(expected),
            'inherited_source_bytes_verified':mode == 'merged'}

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    group.add_argument('--merged', action='store_true')
    group.add_argument('--overlay-only', action='store_true')
    group.add_argument('--self-test', action='store_true')
    group.add_argument('--new-only', action='store_true')
    group.add_argument('--maps-only', action='store_true')
    args = parser.parse_args()
    mode = ('merged' if args.merged else 'overlay-only' if args.overlay_only
            else 'new-only' if args.new_only else 'maps-only' if args.maps_only
            else 'update')
    try:
        result = self_test() if args.self_test else verify('maps-only' if mode == 'update' else mode)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    except (OSError, ValueError, TypeError, KeyError) as exc:
        print(json.dumps({'status':'FAIL','error_type':type(exc).__name__,
                          'message':str(exc)},sort_keys=True))
        return 1

if __name__ == '__main__':
    sys.exit(main())
