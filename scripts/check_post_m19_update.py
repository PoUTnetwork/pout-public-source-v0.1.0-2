#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Read-only update/merged integrity checks and bounded checker self-tests."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
import re
import sys
import tempfile
from pathlib import Path, PurePosixPath

BASELINE_DIGEST = '8582fc7ea5215f130ea00f80b34ec6e734bf921a6c3980006d99c2df970bc4e0'
BASELINE_COMMIT = 'b5cb49b7fbaf76d4f523dcfe014d57829b438dd4'
BASELINE_PATH = 'docs/test-evidence-m20-nr05/BASELINE_MANIFEST.sha256'
UPDATE = 'UPDATE_MANIFEST.sha256'
SOURCE = 'SOURCE_MANIFEST.sha256'
ROOT = Path(__file__).resolve().parents[1]

class IntegrityError(ValueError):
    pass

def require(condition: bool, message: str) -> None:
    if not condition:
        raise IntegrityError(message)

def digest_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()

def parse_manifest(text: str, own_name: str) -> dict[str, str]:
    entries: dict[str, str] = {}
    for line in text.splitlines():
        m = re.fullmatch(r'([0-9a-f]{64})  (.+)', line)
        require(m is not None, 'Malformed manifest line')
        digest, name = m.groups()
        p = PurePosixPath(name)
        require(not p.is_absolute() and '..' not in p.parts and '\\' not in name
                and ':' not in name and p.as_posix() == name
                and all(32 < ord(c) < 127 for c in name), 'Unsafe manifest path')
        require(name != own_name and name not in entries, 'Self-reference or duplicate path')
        entries[name] = digest
    require(bool(entries), 'Empty manifest')
    return entries

def verify_files(root: Path, entries: dict[str, str]) -> None:
    for name, expected in entries.items():
        p = root
        for part in PurePosixPath(name).parts:
            p = p / part
            require(not p.is_symlink(), 'Symlink rejected: ' + name)
        require(p.is_file(), 'Missing regular file: ' + name)
        require(digest_file(p) == expected, 'Changed file: ' + name)

def exact_inventory(root: Path) -> set[str]:
    result = set()
    for current, directories, files in os.walk(root, followlinks=False):
        for name in directories + files:
            p = Path(current) / name
            require(not p.is_symlink(), 'Symlink in update inventory')
        for name in files:
            p = Path(current) / name
            require(p.is_file(), 'Nonregular update file')
            result.add(p.relative_to(root).as_posix())
    return result

def self_test() -> dict:
    passed = []
    def reject(label, fn):
        try:
            fn()
        except (IntegrityError, OSError):
            passed.append({'id': label, 'status': 'PASS', 'observed': 'rejected'})
        else:
            raise IntegrityError('Missing expected rejection: ' + label)
    with tempfile.TemporaryDirectory(prefix='pout-integrity-') as tmp:
        root = Path(tmp)
        payload = root / 'sample.txt'
        payload.write_bytes(b'public integrity control\n')
        digest = digest_file(payload)
        line = digest + '  sample.txt\n'
        entries = parse_manifest(line, UPDATE)
        verify_files(root, entries)
        passed.append({'id': 'I01_valid_file', 'status': 'PASS', 'observed': 'accepted'})
        payload.write_bytes(b'changed\n')
        reject('I02_changed_file', lambda: verify_files(root, entries))
        payload.unlink()
        reject('I03_missing_file', lambda: verify_files(root, entries))
        reject('I04_duplicate_path', lambda: parse_manifest(line + line, UPDATE))
        reject('I05_parent_traversal', lambda: parse_manifest(digest + '  ../outside\n', UPDATE))
        reject('I06_self_reference', lambda: parse_manifest(digest + '  ' + UPDATE + '\n', UPDATE))
        reject('I07_noncanonical_path', lambda: parse_manifest(digest + '  a//b\n', UPDATE))
        actual = root / 'actual.txt'
        actual.write_bytes(b'public integrity control\n')
        payload.symlink_to(actual.name)
        reject('I08_symlink', lambda: verify_files(root, entries))
    return {'schema': 'pout-update-checker-selftests-v1', 'tests_passed': len(passed),
            'tests_failed': 0, 'results': passed, 'scope': 'manifest checker controls only'}

def verify(merged: bool, overlay_only: bool) -> dict:
    baseline_bytes = (ROOT / BASELINE_PATH).read_bytes()
    require(hashlib.sha256(baseline_bytes).hexdigest() == BASELINE_DIGEST,
            'Baseline manifest is not the pinned public version')
    baseline = parse_manifest(baseline_bytes.decode('utf-8'), SOURCE)
    require(len(baseline) == 189, 'Unexpected baseline entry count')
    updates = parse_manifest((ROOT / UPDATE).read_text(encoding='utf-8'), UPDATE)
    require(SOURCE not in updates, 'Circular manifest dependency')
    source = parse_manifest((ROOT / SOURCE).read_text(encoding='utf-8'), SOURCE)
    verify_files(ROOT, updates)
    expected = dict(baseline)
    expected.update(updates)
    expected[UPDATE] = digest_file(ROOT / UPDATE)
    require(source == expected, 'Merged source map differs from baseline plus update')
    metadata = json.loads((ROOT / 'PUBLIC_UPDATE.json').read_text(encoding='utf-8'))
    require(metadata['base_commit'] == BASELINE_COMMIT, 'Wrong public baseline commit')
    payload = set(updates) | {SOURCE, UPDATE}
    require(metadata['overlay_files'] == sorted(payload), 'Update inventory mismatch')
    require(metadata['overlay_file_count'] == len(payload), 'Update count mismatch')
    require(metadata['merged_manifest_entries'] == len(source), 'Source count mismatch')
    if overlay_only:
        require(exact_inventory(ROOT) == payload, 'Unexpected or missing update payload file')
    if merged:
        verify_files(ROOT, source)
    return {'schema': 'pout-update-integrity-v1', 'status': 'PASS',
            'update_files': len(payload), 'update_manifest_entries': len(updates),
            'merged_manifest_entries': len(source), 'baseline_manifest_bytes_verified': True,
            'inherited_source_bytes_verified': merged,
            'inventory_check': 'exact update directory' if overlay_only else 'manifest entries only',
            'mode': 'merged' if merged else 'update'}

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--overlay-only', action='store_true', help='Require exact extracted update file inventory')
    mode.add_argument('--merged', action='store_true', help='Also hash all inherited source files')
    mode.add_argument('--self-test', action='store_true', help='Run bounded positive and negative checker controls')
    args = parser.parse_args()
    try:
        result = self_test() if args.self_test else verify(args.merged, args.overlay_only)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(json.dumps({'status': 'FAIL', 'error_type': type(exc).__name__, 'message': str(exc)}, sort_keys=True))
        return 1

if __name__ == '__main__':
    sys.exit(main())
