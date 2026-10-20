#!/usr/bin/env python3
"""Independent standard-library reference for the public deterministic byte demo.

This checks encodings and captured bytes, not Rust compilation, physical resource
independence, unpredictable challenges, demand, throughput or consensus.
"""
from __future__ import annotations
import argparse
import hashlib
import struct
import unittest
from pathlib import Path

CHUNK_BYTES = 256 * 1024
CHUNKS = 32
SOURCE_BYTES = CHUNK_BYTES * CHUNKS
HEADER_BYTES = 45
FRAME_BYTES = HEADER_BYTES + CHUNK_BYTES
CHUNK_DOMAIN = b"PoUT/Retrieval/Chunk/v1\0"
OBJECT_DOMAIN = b"PoUT/Retrieval/Object/v1\0"
REQUEST_DOMAIN = b"PoUT/Retrieval/Request/v1\0"
MAGIC = b"POUTRET1"


def chunk(index: int) -> bytes:
    if not 0 <= index < CHUNKS:
        raise ValueError("chunk index")
    period = bytes((offset + index) % 256 for offset in range(256))
    return period * (CHUNK_BYTES // 256)


def chunk_hash(index: int, data: bytes) -> bytes:
    if not 0 <= index < CHUNKS or len(data) != CHUNK_BYTES:
        raise ValueError("chunk bounds")
    return hashlib.sha256(CHUNK_DOMAIN + struct.pack(">I", index) + data).digest()


HASHES = tuple(chunk_hash(i, chunk(i)) for i in range(CHUNKS))
OBJECT_ID = hashlib.sha256(
    OBJECT_DOMAIN + struct.pack(">QII", SOURCE_BYTES, CHUNK_BYTES, CHUNKS) + b"".join(HASHES)
).digest()


def request_id(label: int = 0, interval: int = 0, sample: int = 0,
               index: int = 0, nonce: bytes = b"\x31" * 32,
               object_id: bytes = OBJECT_ID) -> bytes:
    if (label not in (0, 1) or interval not in (0, 1) or sample not in (0, 1)
            or not 0 <= index < CHUNKS or len(nonce) != 32 or len(object_id) != 32):
        raise ValueError("request bounds")
    return hashlib.sha256(
        REQUEST_DOMAIN + object_id + bytes((label, interval, sample, index)) + nonce
    ).digest()


def frame(index: int = 0, **request: object) -> bytes:
    return MAGIC + b"\x00" + request_id(index=index, **request) + struct.pack(">I", CHUNK_BYTES) + chunk(index)


def verify(data: bytes, index: int = 0, **request: object) -> str:
    if request.get("object_id", OBJECT_ID) != OBJECT_ID:
        raise ValueError("ObjectPolicy")
    if len(data) < HEADER_BYTES or data[:8] != MAGIC or data[8] not in (0, 1):
        raise ValueError("Framing")
    if data[9:41] != request_id(index=index, **request):
        raise ValueError("RequestContext")
    length = struct.unpack(">I", data[41:45])[0]
    if data[8] == 1:
        if length != 0 or len(data) != HEADER_BYTES:
            raise ValueError("Framing")
        return "Unavailable"
    if length != CHUNK_BYTES or len(data) != FRAME_BYTES:
        raise ValueError("Framing")
    if chunk_hash(index, data[HEADER_BYTES:]) != HASHES[index]:
        raise ValueError("Integrity")
    return "Verified"


class ReferenceTests(unittest.TestCase):
    def test_sha_known_answer(self):
        self.assertEqual(hashlib.sha256(b"abc").hexdigest(),
                         "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")

    def test_fixture_size_and_frame(self):
        self.assertEqual(CHUNKS * len(chunk(0)), SOURCE_BYTES)
        self.assertEqual(len(frame()), FRAME_BYTES)
        self.assertEqual(verify(frame()), "Verified")

    def test_index_changes_hash(self):
        self.assertNotEqual(chunk_hash(0, chunk(0)), chunk_hash(1, chunk(0)))

    def test_object_order_is_bound(self):
        wrong = hashlib.sha256(OBJECT_DOMAIN + struct.pack(">QII", SOURCE_BYTES, CHUNK_BYTES, CHUNKS)
                               + b"".join(reversed(HASHES))).digest()
        self.assertNotEqual(wrong, OBJECT_ID)

    def test_all_request_fields_are_bound(self):
        base = request_id()
        for change in ({"label": 1}, {"interval": 1}, {"sample": 1}, {"index": 1},
                       {"nonce": b"\x32" * 32}, {"object_id": b"\x00" * 32}):
            self.assertNotEqual(request_id(**change), base)

    def test_corruption(self):
        data = bytearray(frame()); data[100] ^= 1
        with self.assertRaisesRegex(ValueError, "Integrity"):
            verify(bytes(data))

    def test_truncation_and_extra_data(self):
        for data in (frame()[:-1], frame() + b"\x00"):
            with self.assertRaisesRegex(ValueError, "Framing"):
                verify(data)

    def test_stale_nonce(self):
        with self.assertRaisesRegex(ValueError, "RequestContext"):
            verify(frame(), nonce=b"\x32" * 32)

    def test_wrong_object(self):
        with self.assertRaisesRegex(ValueError, "ObjectPolicy"):
            verify(frame(), object_id=b"\x00" * 32)

    def test_cached_bytes_new_valid_request(self):
        self.assertEqual(verify(frame(nonce=b"\x32" * 32), nonce=b"\x32" * 32), "Verified")

    def test_unavailable(self):
        data = MAGIC + b"\x01" + request_id() + struct.pack(">I", 0)
        self.assertEqual(verify(data), "Unavailable")

    def test_invalid_bounds_and_status(self):
        with self.assertRaises(ValueError):
            request_id(index=32)
        data = bytearray(frame()); data[8] = 2
        with self.assertRaisesRegex(ValueError, "Framing"):
            verify(bytes(data))


def check_demo(root: Path) -> None:
    """Verify artifacts from an actual Rust example run; never run it implicitly."""
    for source in ("source-a.bin", "source-b.bin"):
        with (root / source).open("rb") as stream:
            for i in range(CHUNKS):
                if stream.read(CHUNK_BYTES) != chunk(i):
                    raise ValueError(f"{source}: wrong fixture bytes")
            if stream.read(1):
                raise ValueError("extra source bytes")
    expected = {
        "two_files_two_meters": (2, 0, 1), "shared_tight": (1, 1, 1),
        "shared_room": (2, 0, 1), "shared_two_meters": (2, 0, 1),
        "shared_repeat": (1, 3, 2),
    }
    lines = (root / "results.tsv").read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != "case\tverified\tdenied" or len(lines) != 6:
        raise ValueError("report framing")
    actual_report = {}
    for line in lines[1:]:
        name, good, denied = line.split("\t")
        if name in actual_report:
            raise ValueError("duplicate report case")
        actual_report[name] = (int(good), int(denied))
    if set(actual_report) != set(expected):
        raise ValueError("report case set")
    for name, (wanted, denied, waves) in expected.items():
        observed = [0, 0]
        for wave in range(waves):
            for label in range(2):
                data = (root / f"{name}-{wave}-{label}.frame").read_bytes()
                if not data:
                    observed[1] += 1
                    continue
                nonce = bytes([0x31 + label + wave]) + bytes([0x31 + label]) * 31
                if verify(data, index=label, label=label, interval=wave, nonce=nonce) != "Verified":
                    raise ValueError("non-delivery in admitted frame")
                observed[0] += 1
        if tuple(observed) != (wanted, denied) or actual_report[name] != (wanted, denied):
            raise ValueError(f"{name}: wrong outcomes")
    for label in range(2):
        left = (root / f"two_files_two_meters-0-{label}.frame").read_bytes()
        right = (root / f"shared_two_meters-0-{label}.frame").read_bytes()
        if left != right:
            raise ValueError("matched frames differ")
    print("Captured Rust frames independently verified: 5 cases, 8 responses, 4 empty denials.")
    print("This does not establish physical resource independence or independent demand.")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_mutually_exclusive_group(required=True)
    modes.add_argument("--self-test", action="store_true")
    modes.add_argument("--check-demo", type=Path)
    args = parser.parse_args()
    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(ReferenceTests)
        result = unittest.TextTestRunner(verbosity=2).run(suite)
        if not result.wasSuccessful():
            raise SystemExit(1)
    else:
        check_demo(args.check_demo)


if __name__ == "__main__":
    main()
