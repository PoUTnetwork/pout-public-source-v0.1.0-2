//! Bounded requested-byte observation, not a SNARK, service receipt or consensus rule.
//!
//! The caller supplies the expected request and object manifest independently.
//! Local paths must be stable and controlled; same-user path races are not isolated.
//! This public extract omits runtime configuration and random-source preparation.
use sha2::{Digest, Sha256};
use std::{
    fmt,
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

pub const SOURCE_BYTES: usize = 8 * 1024 * 1024;
pub const CHUNK_BYTES: usize = 256 * 1024;
pub const CHUNKS: usize = 32;
pub const MAX_REQUESTS: usize = 8;
pub const HEADER_BYTES: usize = 45;
const CHUNK_DOMAIN: &[u8] = b"PoUT/Retrieval/Chunk/v1\0";
const OBJECT_DOMAIN: &[u8] = b"PoUT/Retrieval/Object/v1\0";
const REQUEST_DOMAIN: &[u8] = b"PoUT/Retrieval/Request/v1\0";
const MAGIC: &[u8; 8] = b"POUTRET1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Allocation,
    Bounds,
    Cancelled,
    RequestContext,
    Framing,
    Integrity,
    Io,
    MissingData,
    ObjectPolicy,
    UnsafePath,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

fn check(cancel: &AtomicBool) -> Result<(), Error> {
    if cancel.load(Ordering::Acquire) {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}
fn buffer(n: usize) -> Result<Vec<u8>, Error> {
    if n > CHUNK_BYTES {
        return Err(Error::Bounds);
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(n).map_err(|_| Error::Allocation)?;
    bytes.resize(n, 0);
    Ok(bytes)
}
fn digest(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for part in parts {
        h.update(part);
    }
    h.finalize().into()
}
pub fn chunk_hash(index: u8, bytes: &[u8]) -> Result<[u8; 32], Error> {
    if usize::from(index) >= CHUNKS || bytes.len() != CHUNK_BYTES {
        return Err(Error::Bounds);
    }
    Ok(digest(&[
        CHUNK_DOMAIN,
        &u32::from(index).to_be_bytes(),
        bytes,
    ]))
}
pub fn object_id(chunk_hashes: &[[u8; 32]; CHUNKS]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(OBJECT_DOMAIN);
    h.update((SOURCE_BYTES as u64).to_be_bytes());
    h.update((CHUNK_BYTES as u32).to_be_bytes());
    h.update((CHUNKS as u32).to_be_bytes());
    for item in chunk_hashes {
        h.update(item);
    }
    h.finalize().into()
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub version: u8,
    pub total_bytes: u64,
    pub chunk_bytes: u32,
    pub chunk_hashes: [[u8; 32]; CHUNKS],
    pub object_id: [u8; 32],
}
impl Manifest {
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != 1
            || self.total_bytes != SOURCE_BYTES as u64
            || self.chunk_bytes != CHUNK_BYTES as u32
        {
            return Err(Error::Bounds);
        }
        if self.object_id != object_id(&self.chunk_hashes) {
            return Err(Error::ObjectPolicy);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Request {
    pub object_id: [u8; 32],
    pub label: u8,
    pub interval: u8,
    pub sample: u8,
    pub chunk: u8,
    pub nonce: [u8; 32],
}
impl Request {
    pub fn validate(&self) -> Result<(), Error> {
        if self.label > 1 || self.interval > 1 || self.sample > 1 || self.chunk >= CHUNKS as u8 {
            return Err(Error::Bounds);
        }
        Ok(())
    }
    pub fn digest(&self) -> Result<[u8; 32], Error> {
        self.validate()?;
        Ok(digest(&[
            REQUEST_DOMAIN,
            &self.object_id,
            &[self.label, self.interval, self.sample, self.chunk],
            &self.nonce,
        ]))
    }
}
pub fn validate_requests(requests: &[Request]) -> Result<(), Error> {
    if requests.is_empty() || requests.len() > MAX_REQUESTS {
        return Err(Error::Bounds);
    }
    for request in requests {
        request.validate()?;
    }
    Ok(())
}

// Caller-selected stable paths only. This check is not a sandbox or a complete
// defense against a hostile process changing paths concurrently under the same UID.
fn reject_symlinks(path: &Path) -> Result<(), Error> {
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        let metadata = fs::symlink_metadata(ancestor).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                Error::MissingData
            } else {
                Error::Io
            }
        })?;
        if metadata.file_type().is_symlink() {
            return Err(Error::UnsafePath);
        }
    }
    Ok(())
}
pub fn open_regular(path: &Path) -> Result<File, Error> {
    reject_symlinks(path)?;
    let inspected = fs::symlink_metadata(path).map_err(|_| Error::Io)?;
    if !inspected.is_file() {
        return Err(Error::UnsafePath);
    }
    let file = File::open(path).map_err(|_| Error::Io)?;
    let opened = file.metadata().map_err(|_| Error::Io)?;
    if !opened.is_file() || opened.len() != inspected.len() {
        return Err(Error::UnsafePath);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if opened.dev() != inspected.dev() || opened.ino() != inspected.ino() {
            return Err(Error::UnsafePath);
        }
    }
    reject_symlinks(path)?;
    Ok(file)
}
fn header(request: &Request, available: bool) -> Result<[u8; HEADER_BYTES], Error> {
    let mut header = [0; HEADER_BYTES];
    header[..8].copy_from_slice(MAGIC);
    header[8] = u8::from(!available);
    header[9..41].copy_from_slice(&request.digest()?);
    header[41..].copy_from_slice(&(if available { CHUNK_BYTES as u32 } else { 0 }).to_be_bytes());
    Ok(header)
}
pub fn respond_file(
    requests: &[Request],
    source: &Path,
    output: &mut impl Write,
    cancel: &AtomicBool,
) -> Result<(), Error> {
    check(cancel)?;
    validate_requests(requests)?;
    let mut file = open_regular(source)?;
    if file.metadata().map_err(|_| Error::Io)?.len() != SOURCE_BYTES as u64 {
        return Err(Error::MissingData);
    }
    let mut data = buffer(CHUNK_BYTES)?;
    for request in requests {
        check(cancel)?;
        file.seek(SeekFrom::Start(
            u64::from(request.chunk) * CHUNK_BYTES as u64,
        ))
        .map_err(|_| Error::Io)?;
        file.read_exact(&mut data).map_err(|error| {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::MissingData
            } else {
                Error::Io
            }
        })?;
        check(cancel)?;
        output
            .write_all(&header(request, true)?)
            .map_err(|_| Error::Io)?;
        output.write_all(&data).map_err(|_| Error::Io)?;
    }
    output.flush().map_err(|_| Error::Io)
}
pub fn respond_state_only(
    requests: &[Request],
    output: &mut impl Write,
    cancel: &AtomicBool,
) -> Result<(), Error> {
    check(cancel)?;
    validate_requests(requests)?;
    // Restricted fixture algorithm: not an arbitrary adversary, delegated reader
    // or proof that another process cannot obtain the requested bytes.
    for request in requests {
        check(cancel)?;
        output
            .write_all(&header(request, false)?)
            .map_err(|_| Error::Io)?;
    }
    output.flush().map_err(|_| Error::Io)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedResponse {
    request_id: [u8; 32],
    bytes: u64,
}
impl VerifiedResponse {
    pub fn request_id(&self) -> [u8; 32] {
        self.request_id
    }
    pub fn returned_bytes(&self) -> u64 {
        self.bytes
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckedResponse {
    Verified(VerifiedResponse),
    Unavailable,
}
pub fn verify_one(
    manifest: &Manifest,
    request: &Request,
    response: &mut impl Read,
    cancel: &AtomicBool,
) -> Result<CheckedResponse, Error> {
    check(cancel)?;
    manifest.validate()?;
    request.validate()?;
    if request.object_id != manifest.object_id {
        return Err(Error::ObjectPolicy);
    }
    let mut head = [0; HEADER_BYTES];
    response.read_exact(&mut head).map_err(|_| Error::Framing)?;
    let length = u32::from_be_bytes(head[41..].try_into().map_err(|_| Error::Framing)?);
    if &head[..8] != MAGIC || head[8] > 1 {
        return Err(Error::Framing);
    }
    let expected = request.digest()?;
    if head[9..41] != expected {
        return Err(Error::RequestContext);
    }
    if head[8] == 1 {
        return if length == 0 {
            Ok(CheckedResponse::Unavailable)
        } else {
            Err(Error::Framing)
        };
    }
    if length != CHUNK_BYTES as u32 {
        return Err(Error::Framing);
    }
    let mut data = buffer(CHUNK_BYTES)?;
    response.read_exact(&mut data).map_err(|_| Error::Framing)?;
    check(cancel)?;
    if chunk_hash(request.chunk, &data)? != manifest.chunk_hashes[usize::from(request.chunk)] {
        return Err(Error::Integrity);
    }
    Ok(CheckedResponse::Verified(VerifiedResponse {
        request_id: expected,
        bytes: CHUNK_BYTES as u64,
    }))
}
#[derive(Debug, Eq, PartialEq)]
pub struct Verification {
    /// Complete parsing, including Unavailable, not a scarce-service certificate.
    pub status: &'static str,
    pub verified: usize,
    pub unavailable: usize,
    pub transferred_bytes: u64,
}
pub fn verify_stream(
    manifest: &Manifest,
    requests: &[Request],
    response: &mut impl Read,
    cancel: &AtomicBool,
) -> Result<Verification, Error> {
    check(cancel)?;
    validate_requests(requests)?;
    let mut summary = Verification {
        status: "Complete",
        verified: 0,
        unavailable: 0,
        transferred_bytes: 0,
    };
    for request in requests {
        match verify_one(manifest, request, response, cancel)? {
            CheckedResponse::Verified(value) => {
                summary.verified += 1;
                summary.transferred_bytes += value.returned_bytes();
            }
            CheckedResponse::Unavailable => summary.unavailable += 1,
        }
    }
    check(cancel)?;
    let mut extra = [0];
    if response.read(&mut extra).map_err(|_| Error::Io)? != 0 {
        return Err(Error::Framing);
    }
    Ok(summary)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyError {
    Expired,
    Duplicate,
    Capacity,
    Allocation,
}
pub struct Consumption {
    consumed: Vec<[u8; 32]>,
}
impl Consumption {
    pub fn new() -> Result<Self, PolicyError> {
        let mut consumed = Vec::new();
        consumed
            .try_reserve_exact(MAX_REQUESTS)
            .map_err(|_| PolicyError::Allocation)?;
        Ok(Self { consumed })
    }
    /// Retained local view only. Nonce changes create distinct request digests;
    /// this is not independently authorized-demand deduplication.
    pub fn admit(
        &mut self,
        value: &VerifiedResponse,
        now: u64,
        expires: u64,
    ) -> Result<(), PolicyError> {
        if now >= expires {
            return Err(PolicyError::Expired);
        }
        if self.consumed.contains(&value.request_id) {
            return Err(PolicyError::Duplicate);
        }
        if self.consumed.len() == MAX_REQUESTS {
            return Err(PolicyError::Capacity);
        }
        self.consumed.push(value.request_id);
        Ok(())
    }
    pub fn len(&self) -> usize {
        self.consumed.len()
    }
    pub fn is_empty(&self) -> bool {
        self.consumed.is_empty()
    }
}
