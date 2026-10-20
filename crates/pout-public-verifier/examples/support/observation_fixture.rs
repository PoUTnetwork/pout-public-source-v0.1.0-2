//! Public deterministic, deliberately compressible fixture. NOT entropy evidence.
use super::retrieval::{self, Manifest, Request, CHUNKS, CHUNK_BYTES, SOURCE_BYTES};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

pub fn chunk(index: u8) -> Vec<u8> {
    assert!(usize::from(index) < CHUNKS);
    (0..CHUNK_BYTES)
        .map(|i| (i as u8).wrapping_add(index))
        .collect()
}
pub fn manifest() -> Manifest {
    let hashes = std::array::from_fn(|i| {
        retrieval::chunk_hash(i as u8, &chunk(i as u8)).expect("bounded public fixture")
    });
    Manifest {
        version: 1,
        total_bytes: SOURCE_BYTES as u64,
        chunk_bytes: CHUNK_BYTES as u32,
        object_id: retrieval::object_id(&hashes),
        chunk_hashes: hashes,
    }
}
pub fn request(manifest: &Manifest, label: u8) -> Request {
    Request {
        object_id: manifest.object_id,
        label,
        interval: 0,
        sample: 0,
        chunk: label,
        nonce: [0x31 + label; 32],
    }
}
/// The caller must own a stable parent directory. Never overwrite an existing file.
pub fn write_source(path: &Path) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    for i in 0..CHUNKS {
        file.write_all(&chunk(i as u8))?;
    }
    file.flush()
}
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(DIGITS[usize::from(b >> 4)] as char);
        out.push(DIGITS[usize::from(b & 15)] as char);
    }
    out
}
