//! Fixed-suite Groth16 verification for a finite SHA-256 research relation.
//!
//! Verification checks cryptographic relation satisfaction under a caller-pinned
//! public key and full statement. It does not establish storage duration, actual
//! delivery, economic demand, resource scarcity or consensus eligibility.
use bellman::{gadgets::multipack, groth16};
use bls12_381::{Bls12, Scalar};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

pub const PROOF_BYTES: usize = 200;
pub const PUBLIC_BYTES: usize = 128;
pub const AUDIT_CONTEXT_BYTES: usize = 80;
const MAGIC: &[u8; 8] = b"PTFREL01";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Enrollment {
    pub source_hash: [u8; 32],
    pub network: [u8; 32],
    pub provider: [u8; 32],
    pub epoch: u64,
    pub replica_root: [u8; 32],
}
impl Enrollment {
    pub fn encode(&self) -> [u8; 136] {
        let mut out = [0; 136];
        out[..32].copy_from_slice(&self.source_hash);
        out[32..64].copy_from_slice(&self.network);
        out[64..96].copy_from_slice(&self.provider);
        out[96..104].copy_from_slice(&self.epoch.to_le_bytes());
        out[104..].copy_from_slice(&self.replica_root);
        out
    }
    pub fn context(&self) -> [u8; 16] {
        hash(&[
            b"PoUT-file-relation/v1/context",
            &self.network,
            &self.provider,
            &self.epoch.to_le_bytes(),
        ])[..16]
            .try_into()
            .unwrap()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    pub tag: [u8; 32],
    pub proof: Vec<u8>,
}
#[derive(Debug, Error)]
pub enum FileRelationError {
    #[error("unsupported or malformed finite file proof")]
    Wire,
    #[error("expected network/epoch does not match enrollment")]
    Context,
    #[error("actual public file relation pairing verification failed")]
    Relation,
    #[error("finite research circuit bound exceeded")]
    Bounds,
    #[error("local cancellation (not proof invalidity)")]
    Cancelled,
    #[error("local proof backend failure (not proof invalidity)")]
    LocalBackend,
}
impl FileRelationError {
    pub fn is_local_abort(&self) -> bool {
        matches!(self, Self::Cancelled | Self::LocalBackend)
    }
}
fn check(cancel: &AtomicBool) -> Result<(), FileRelationError> {
    if cancel.load(Ordering::Acquire) {
        Err(FileRelationError::Cancelled)
    } else {
        Ok(())
    }
}
pub fn hash(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for part in parts {
        h.update(part);
    }
    h.finalize().into()
}
pub fn challenge(
    enrollment: &Enrollment,
    audit: &AuditContext,
) -> Result<[u8; 16], FileRelationError> {
    if enrollment.network != audit.network || enrollment.epoch != audit.epoch {
        return Err(FileRelationError::Context);
    }
    Ok(
        hash(&[b"PoUT-file-relation/v1/challenge", &audit.encode()])[..16]
            .try_into()
            .unwrap(),
    )
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditContext {
    pub network: [u8; 32],
    pub epoch: u64,
    pub round: u64,
    pub randomness: [u8; 32],
}

impl AuditContext {
    pub fn encode(&self) -> [u8; AUDIT_CONTEXT_BYTES] {
        let mut bytes = [0; AUDIT_CONTEXT_BYTES];
        bytes[..32].copy_from_slice(&self.network);
        bytes[32..40].copy_from_slice(&self.epoch.to_le_bytes());
        bytes[40..48].copy_from_slice(&self.round.to_le_bytes());
        bytes[48..].copy_from_slice(&self.randomness);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, FileRelationError> {
        if bytes.len() != AUDIT_CONTEXT_BYTES {
            return Err(FileRelationError::Wire);
        }
        Ok(Self {
            network: bytes[..32].try_into().unwrap(),
            epoch: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            round: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            randomness: bytes[48..].try_into().unwrap(),
        })
    }
}

pub fn public_bytes(
    e: &Enrollment,
    audit: &AuditContext,
    tag: &[u8; 32],
) -> Result<[u8; PUBLIC_BYTES], FileRelationError> {
    let mut out = [0; PUBLIC_BYTES];
    out[..32].copy_from_slice(&e.source_hash);
    out[32..48].copy_from_slice(&e.context());
    out[48..80].copy_from_slice(&e.replica_root);
    out[80..96].copy_from_slice(&challenge(e, audit)?);
    out[96..].copy_from_slice(tag);
    Ok(out)
}

pub struct Verifier {
    prepared: groth16::PreparedVerifyingKey<Bls12>,
    hash: [u8; 32],
    pub serialized_bytes: usize,
}
impl Verifier {
    pub fn vk_hash(&self) -> [u8; 32] {
        self.hash
    }
}
#[derive(Debug)]
pub struct VerifiedFileRelation {
    opportunity: [u8; 32],
}
impl VerifiedFileRelation {
    pub fn opportunity_id(&self) -> [u8; 32] {
        self.opportunity
    }
}
pub fn verify(
    verifier: &Verifier,
    e: &Enrollment,
    audit: &AuditContext,
    response: &Response,
    cancel: &AtomicBool,
) -> Result<VerifiedFileRelation, FileRelationError> {
    check(cancel)?;
    let public = public_bytes(e, audit, &response.tag)?;
    if response.proof.len() != PROOF_BYTES || &response.proof[..8] != MAGIC {
        return Err(FileRelationError::Wire);
    }
    let p =
        groth16::Proof::<Bls12>::read(&response.proof[8..]).map_err(|_| FileRelationError::Wire)?;
    let fields = multipack::compute_multipacking::<Scalar>(&multipack::bytes_to_bits(&public));
    groth16::verify_proof(&verifier.prepared, &p, &fields)
        .map_err(|_| FileRelationError::Relation)?;
    check(cancel)?;
    Ok(VerifiedFileRelation {
        opportunity: hash(&[
            b"PoUT-file-relation/v1/opportunity",
            &verifier.hash,
            &e.encode(),
            &audit.encode(),
        ]),
    })
}

pub mod public_fixture;
pub mod public_verify;
