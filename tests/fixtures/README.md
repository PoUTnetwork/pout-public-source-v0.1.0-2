# Synthetic public fixture schemas

All content here is generated from public deterministic descriptions. No user
traffic, storage dumps, identities, credentials, live assets or runtime witnesses
are included. In particular, frame data are not claimed to be network captures.

## finite-models/

CSV is ASCII, comma separated, one header, LF endings, canonical nonnegative
base-10 integers, no missing fields or blank rows. Values are exact.

| File | Data rows | Meaning |
| --- | ---: | --- |
| advice_encoders.csv | 65,536 | Encoder ID; best Q0/Q1 numerator; denominator 64 |
| response_tables.csv | 256 | All two-bit response tables, four challenges, per-source hit counts |
| sample_targets.csv | 64 | Sixteen sources by four challenges |
| same_source_repetition.csv | 12 | Fixed strategy and IID comparison, each as numerator/denominator |
| bucket_first_hit.csv | 154 | Alphabet, advice states, probes, hits and source worlds |
| gf4_affine_worlds.csv | 16 | A,B and four field-coordinate values |
| masked_public_view.csv | 4 | Target bit, helper bit, XOR public view |

`expected_summary.json` records scope and exact extrema. The checker requires the
entire exact fixture set, sorted encoder identities and complete rows. A missing
negative test or empty table is never treated as proof of success.

## service-observation/

`manifest.json` describes the unchanged public 8-MiB deterministic fixture and
its 32 indexed SHA-256 chunk hashes. Chunk i consists of `(position+i) mod 256`,
repeated for 256 KiB. This data is highly compressible and proves no scarcity.

`request_vectors.json` gives 128 fixed requests with label, interval, sample,
chunk index and a reproducible 32-byte nonce. Hex strings are lowercase. These
nonces are public test inputs, not secret or unpredictable challenges.

`cases.json` inventories sixteen `.frame` files, their byte length, SHA-256,
independently expected request and exact result category. The declared cases
include three verified frames, one unavailable frame and twelve expected
rejections. The expected layers are Framing, RequestContext, ObjectPolicy or
Integrity. Unavailable is not delivered service.

Frames use the already public R1 format:

```
magic[8] || status[1] || request_digest[32] || payload_length_be32 || payload
```

The expected request is external to the frame. A correct reply to the wrong
request is rejected. New legitimate nonces may authorize repeated cached bytes;
that is not evidence of another independently stored resource.

The package check verifies these frames with the unchanged public Python byte
reference. A Rust interoperability claim requires an actual Rust run and is
not supplied by this Python-only fixture validation.
