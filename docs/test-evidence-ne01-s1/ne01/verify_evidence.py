#!/usr/bin/env python3
"""Check this evidence bundle's bytes and consistency, not PoUT consensus.

Uses only the Python standard library. It does not execute Rust, connect to a
network, authenticate the original experiment, or close any security gate.
"""
from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path, PurePosixPath
from typing import Any

ROOT = Path(__file__).resolve().parent
MANIFEST = "SOURCE_MANIFEST.sha256"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git_blob_sha1(data: bytes) -> str:
    return hashlib.sha1(b"blob " + str(len(data)).encode("ascii") + b"\0" + data).hexdigest()


def safe_file(relative: str) -> Path:
    name = PurePosixPath(relative)
    require(not name.is_absolute() and ".." not in name.parts and "\\" not in relative,
            f"Unsafe path: {relative!r}")
    path = ROOT.joinpath(*name.parts)
    require(path.is_file() and not path.is_symlink(), f"Missing file or symlink: {relative}")
    require(path.resolve().is_relative_to(ROOT), f"Path leaves the bundle: {relative}")
    return path


def load_json(relative: str) -> Any:
    return json.loads(safe_file(relative).read_text(encoding="utf-8"))


def verify() -> int:
    entries: dict[str, str] = {}
    for line in safe_file(MANIFEST).read_text(encoding="utf-8").splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  (.+)", line)
        require(match is not None, f"Malformed manifest line: {line!r}")
        digest, relative = match.groups()
        require(relative not in entries and relative != MANIFEST,
                f"Duplicate or self-referential manifest entry: {relative}")
        entries[relative] = digest
        require(sha256(safe_file(relative).read_bytes()) == digest,
                f"SHA-256 mismatch: {relative}")
    files = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file()}
    require(files == set(entries) | {MANIFEST}, "Manifest coverage differs from package files")
    print(f"[PASS] SHA-256 manifest: {len(entries)} files, exact coverage")

    provenance = load_json("SOURCE_PROVENANCE.json")
    for artifact in provenance["raw_artifacts"]:
        data = safe_file(artifact["bundle_file"]).read_bytes()
        require(len(data) == artifact["bytes"], f"Size mismatch: {artifact['bundle_file']}")
        require(sha256(data) == artifact["sha256"], f"Provenance hash mismatch: {artifact['bundle_file']}")
        require(git_blob_sha1(data) == artifact["source_git_blob_sha1"],
                f"Original-blob mismatch: {artifact['bundle_file']}")
    print("[PASS] Three raw artifacts match recorded original Git blob identifiers")

    report = load_json("evidence/skeleton-report-release.json")
    stdout = load_json("evidence/example_skeleton_release.log")
    require(report == stdout, "Formatted JSON differs from original release stdout")
    checks = load_json("evidence/PUBLIC_CHECKS.json")
    require(checks["source_revision"] == provenance["source_revision"], "Source revision mismatch")
    require(report["consensus_eligibility"] == "Established", "Unexpected model result")
    require(report["scope"] == "NE01_TEST_MODEL_OWNER_RULE_2026_11_07", "Model scope changed")
    require(report["consensus_weight"] == 512 and report["unmet_test_model_criteria"] == 0,
            "Unexpected weight or unmet-model criteria")
    require(sum(report["weight_by_owner_prefix"].values()) == report["consensus_weight"],
            "Per-owner weights do not sum to the model total")
    require(report["epochs"]["audited_epochs"] == 8 and report["epochs"]["passes"] == 16
            and report["epochs"]["failures"] == 0, "Epoch counters changed")
    print("[PASS] Original release output: model Established, 512 units, 8 epochs / 16 passes")

    log = safe_file("evidence/test_real_components.log").read_bytes()
    groups = re.findall(rb"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", log)
    totals = tuple(sum(int(g[i]) for g in groups) for i in range(3))
    require(len(groups) == 4 and totals == (64, 0, 0), f"Unexpected targeted test totals: {totals}")
    require([int(g[0]) for g in groups] == [39, 5, 10, 10], "Unexpected test-group counts")
    require(sha256(log) == checks["runs"]["ne01_real_components"]["source_log_sha256"],
            "Targeted test log differs from the original receipt hash")
    require(sha256(safe_file("evidence/example_skeleton_release.log").read_bytes())
            == checks["runs"]["release_example"]["source_log_sha256"],
            "Release stdout differs from the original receipt hash")
    print("[PASS] Recorded targeted test log: 64 passed / 0 failed / 0 ignored")

    required_gates = ["ExternalSecurityReview", "MultiDeviceTest", "BeaconUnpredictability",
                      "DelayPrimitiveGate", "DepthRobustnessGate", "ResourceTheorem", "HostedCi"]
    require(report["open_production_gates"] == required_gates, "Open production gates altered")
    require(checks["consensus_validation"]["open_production_gates"] == required_gates,
            "Gate list differs across evidence files")
    measured = report["measured_on_this_host"]
    require(measured["asserted_security_margin"] is False
            and measured["finals_cached_regeneration_exceeds_lab_window"] is False,
            "Known resource limitation missing or changed")
    require(measured["regeneration_with_finals_cached_worst_seconds"] < measured["lab_window_seconds"],
            "Timing conclusion is inconsistent")
    require(checks["runs"]["workspace_no_fail_fast"]["failed"] == 13
            and checks["runs"]["workspace_no_fail_fast"]["exit_code"] == 101,
            "Recorded workspace failures removed")
    require(checks["production_security_validated"] is False
            and checks["independent_runtime_reexecution_for_this_bundle"] is False,
            "Evidence review scope changed")
    print("[PASS] Open security gates, regeneration limitation and workspace failures retained")
    print("\nBUNDLE INTEGRITY AND CONSISTENCY CHECKS PASSED.")
    print("No Rust execution, execution authentication or consensus-security validation was performed.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(verify())
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print(f"[FAIL] {exc}", file=sys.stderr)
        sys.exit(1)
