#!/usr/bin/env python3
"""Verify tracked source bytes against SOURCE_MANIFEST.sha256. No writes."""
from __future__ import annotations
import hashlib
import re
import subprocess
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = "SOURCE_MANIFEST.sha256"


def main() -> None:
    entries = {}
    for line in (ROOT / MANIFEST).read_text(encoding="utf-8").splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  (.+)", line)
        if not match:
            raise SystemExit("Malformed manifest line")
        digest, name = match.groups()
        path = PurePosixPath(name)
        if path.is_absolute() or ".." in path.parts or "\\" in name or name == MANIFEST or name in entries:
            raise SystemExit("Unsafe, duplicate or self-referential manifest entry")
        target = ROOT / name
        if any(parent.is_symlink() for parent in [target, *target.parents]):
            raise SystemExit(f"Symlink is not a source file: {name}")
        if not target.is_file() or hashlib.sha256(target.read_bytes()).hexdigest() != digest:
            raise SystemExit(f"Missing or changed source: {name}")
        entries[name] = digest
    result = subprocess.run(["git", "-C", str(ROOT), "ls-files", "-z"],
                            capture_output=True, check=False)
    if result.returncode == 0:
        tracked = {p.decode("utf-8") for p in result.stdout.split(b"\0") if p}
        tracked.discard(MANIFEST)
        if tracked != set(entries):
            raise SystemExit("Manifest and tracked file set differ")
    print(f"Verified {len(entries)} source files; manifest excludes itself.")


if __name__ == "__main__":
    main()
