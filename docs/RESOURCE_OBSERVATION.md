# Public requested-byte and budget observation

Research-only selected export, October 20, 2026. No consensus, reward, migration,
production network or new Groth16 proof is implemented here. The Rust export
requires its own compilation/test gate; source presence is not validation.

## Exact scope

The extracted retrieval core checks bounded response framing, request-context
binding and SHA-256 content integrity. The extracted meter reserves a complete
frame atomically against a nonrefilling controller-assigned allowance. Neither
primitive imports the Groth16 verifier or authorizes settlement.

The public adaptation removes JSON configuration and serialization requirements,
uses the existing SHA-256 dependency, and supplies a separate deterministic
fixture and local Unix-stream example. It is not a byte-identical release of
an entire engineering experiment. No internal logs or experiment inputs are
bundled. The existing CWC and verifier libraries remain unchanged.

## Data and wire format

The fixture has 32 chunks of 262144 bytes, for 8388608 total source bytes.
For chunk index `i`, its byte at offset `j` is `(j + i) mod 256`.
This is deliberately compressible public data, not evidence of random entropy
or storage scarcity. The two provisioned files contain the same fixture.

```text
chunk_hash(i, data) = SHA256(D_chunk || BE32(i) || data)
object_id = SHA256(D_object || BE64(source_size) || BE32(chunk_size)
                   || BE32(chunk_count) || chunk_hash(0) || ... || chunk_hash(31))
request_id = SHA256(D_request || object_id || label8 || interval8
                    || sample8 || chunk8 || nonce32)
```

The domains are `PoUT/Retrieval/Chunk/v1\0`, `PoUT/Retrieval/Object/v1\0`
and `PoUT/Retrieval/Request/v1\0`; `\0` denotes one zero byte.

```text
response = magic8 || status8 || request_id32 || BE32(payload_length) || payload
magic8   = "POUTRET1"
status8  = 0 (data) or 1 (unavailable)
header   = 45 bytes
data response = 262189 bytes including 262144 payload bytes
```

For an unavailable response, payload length is zero. Exact EOF is checked after
the declared request sequence. Extra bytes, truncation, altered contexts and
wrong content are distinct failure cases. The caller supplies the expected
request and manifest independently; responder-provided expectations are not
independently authenticated facts.

## Assigned quota invariant

For each installed meter:

```text
spent + reserved <= capacity
uncertain_spent <= spent
```

A non-Clone lease first reserves up to one frame. Successful writes transfer
reserved bytes into spent bytes. Dropping a lease releases only confirmed
unwritten capacity. An ambiguous or panicking write conservatively charges the
offered bound as uncertain; that is not a claim those bytes reached a receiver.
A write that finishes late remains spent but fails timely completion.

The meter holds no accounting mutex across user-supplied I/O. Admission checks
time/cancellation before and after lock acquisition. The controller supplies
monotonic time; it is not an authenticated decentralized clock. Labels and nonces
do not allocate or reset meters. Newly assigned meters are new allowances, not
newly demonstrated physical resources.

## Example scenarios and expected results

| Scenario | Arrangement | Expected verified / denied |
| --- | --- | ---: |
| two_files_two_meters | Two fixture files; two one-frame meters | 2 / 0 |
| shared_tight | One shared source; one one-frame meter | 1 / 1 |
| shared_room | One shared source; one two-frame meter | 2 / 0 |
| shared_two_meters | One shared source; two assigned one-frame meters | 2 / 0 |
| shared_repeat | One spent one-frame meter; two request waves with fresh nonces | 1 / 3 |

The example uses scoped local threads and Unix streams, not separate remote nodes.
A bounded readiness/release gate starts competing producers. Winning identity under a tight budget is
not prescribed. Empty output after denial is not an integrity failure or a verified
unavailable-response frame. The Python checker compares every nonempty response,
its expected request, fixture bytes and the TSV report.

Matched responses in `two_files_two_meters` and `shared_two_meters` should be
byte-identical. This is an intentional counterexample: the observed data cannot
by itself certify how many physical copies, links or independent operators exist.

## Reproduction

From a complete checkout after merging the update:

```sh
python3 -I -B scripts/check_source_manifest.py
python3 -I -B scripts/check_observation.py --self-test
cargo build --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo test --locked --workspace --all-features
cargo run --locked -p pout-public-verifier --example service_observation -- /tmp/pout-public-observation-new
python3 -I -B scripts/check_observation.py --check-demo /tmp/pout-public-observation-new
```

The example refuses an existing output directory. Use only a stable parent under
your control. It writes synthetic source files and response artifacts, makes no
internet request and leaves outputs for inspection. Socket calls have five-second
timeouts; use a process-level wall limit when running an unattended experiment.
The included Linux CI places the example under a 120-second outer limit.

## Evidence boundary

The Python reference's 12 self-tests have run. This packaging environment has no
Rust compiler, so the new Rust tests and example have not run here. No engineering
workspace test total is substituted for validation of this export. The read-only
public CI workflow is an execution plan until an actual run succeeds.

A response that passes integrity checks is not necessarily useful independent
demand. Cached data can legitimately satisfy multiple fresh requests. Local
request-digest consumption includes the nonce and does not identify a stable
external demand slot. The extract does not solve retry/new-demand attribution,
collusion, global history, bandwidth scarcity, storage retention or consensus.

`consensus_eligibility = NotEstablished`; no consensus weight is assigned.
