//! Bounded PUBLIC evidence import with independently supplied trust pins.
//!
//! This fixed research suite accepts the original file relation, not a proof of
//! storage duration, delivery, resource scarcity or consensus eligibility. Key
//! coherence checks are necessary public consistency checks, not evidence that
//! setup randomness was honest, erased or independently generated.
use super::{
    groth16, hash, verify, Enrollment, FileRelationError, Response, VerifiedFileRelation, Verifier,
    MAGIC as PROOF_MAGIC, PROOF_BYTES,
};
use crate::AuditContext;
use bls12_381::{pairing, Bls12};
use std::{
    io::Cursor,
    sync::atomic::{AtomicBool, Ordering},
};
use thiserror::Error;

pub const BUNDLE_BYTES: usize = 1900;
pub const VK_BYTES: usize = 1444;
const BUNDLE_MAGIC: &[u8; 8] = b"PTPUBV01";
const KEY_START: usize = 8;
const ENROLLMENT_START: usize = KEY_START + VK_BYTES;
const AUDIT_START: usize = ENROLLMENT_START + 136;
const TAG_START: usize = AUDIT_START + 80;
const PROOF_START: usize = TAG_START + 32;
const IC_COUNT_OFFSET: usize = 864;

/// Trust inputs MUST come from an authenticated source outside the untrusted
/// bundle. A digest claimed by that same bundle does not establish authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedPins {
    pub vk_sha256: [u8; 32],
    pub statement_sha256: [u8; 32],
}

#[derive(Debug, Error)]
pub enum PublicVerifyError {
    #[error("malformed or noncanonical bounded public evidence")]
    Wire,
    #[error("public verifying key differs from the independent expected pin")]
    KeyPin,
    #[error("full enrollment/audit differs from the independent expected pin")]
    StatementPin,
    #[error("public beta/delta key elements are not cross-group coherent")]
    KeyCoherence,
    #[error("local cancellation, not proof invalidity")]
    Cancelled,
    #[error("original file relation: {0}")]
    Relation(#[from] FileRelationError),
}
impl PublicVerifyError {
    pub fn is_local_abort(&self) -> bool {
        matches!(self, Self::Cancelled)
            || matches!(self, Self::Relation(error) if error.is_local_abort())
    }
}
fn live(cancel: &AtomicBool) -> Result<(), PublicVerifyError> {
    if cancel.load(Ordering::Acquire) {
        Err(PublicVerifyError::Cancelled)
    } else {
        Ok(())
    }
}
fn encoded_statement_pin(enrollment: &[u8], audit: &[u8]) -> [u8; 32] {
    hash(&[
        b"PoUT-public-file-evidence/v1/full-statement",
        enrollment,
        audit,
    ])
}
/// Commit to ALL 136 enrollment and 80 audit bytes, including the complete
/// provider identity and randomness. This is not the truncated circuit context.
pub fn statement_pin(enrollment: &Enrollment, audit: &AuditContext) -> [u8; 32] {
    encoded_statement_pin(&enrollment.encode(), &audit.encode())
}

/// Framing ONLY: this neither validates a key/proof nor creates trusted pins.
/// The verifier below must still receive pins from an independent authority.
pub fn encode_bundle(
    vk: &[u8],
    enrollment: &Enrollment,
    audit: &AuditContext,
    response: &Response,
) -> Result<[u8; BUNDLE_BYTES], PublicVerifyError> {
    if vk.len() != VK_BYTES || response.proof.len() != PROOF_BYTES {
        return Err(PublicVerifyError::Wire);
    }
    let mut bytes = [0; BUNDLE_BYTES];
    bytes[..KEY_START].copy_from_slice(BUNDLE_MAGIC);
    bytes[KEY_START..ENROLLMENT_START].copy_from_slice(vk);
    bytes[ENROLLMENT_START..AUDIT_START].copy_from_slice(&enrollment.encode());
    bytes[AUDIT_START..TAG_START].copy_from_slice(&audit.encode());
    bytes[TAG_START..PROOF_START].copy_from_slice(&response.tag);
    bytes[PROOF_START..].copy_from_slice(&response.proof);
    Ok(bytes)
}

fn decode_key(bytes: &[u8]) -> Result<groth16::VerifyingKey<Bls12>, PublicVerifyError> {
    // The exact length and count check PRECEDE Bellman's count-driven allocation.
    if bytes.len() != VK_BYTES || bytes[IC_COUNT_OFFSET..IC_COUNT_OFFSET + 4] != 6_u32.to_be_bytes()
    {
        return Err(PublicVerifyError::Wire);
    }
    let mut cursor = Cursor::new(bytes);
    let vk =
        groth16::VerifyingKey::<Bls12>::read(&mut cursor).map_err(|_| PublicVerifyError::Wire)?;
    // Bellman's checked decoder rejects off-curve/subgroup-invalid encodings,
    // but allows identity header elements. Reject those explicitly as well.
    if cursor.position() != VK_BYTES as u64
        || vk.ic.len() != 6
        || bool::from(vk.alpha_g1.is_identity())
        || bool::from(vk.beta_g1.is_identity())
        || bool::from(vk.beta_g2.is_identity())
        || bool::from(vk.gamma_g2.is_identity())
        || bool::from(vk.delta_g1.is_identity())
        || bool::from(vk.delta_g2.is_identity())
        || vk.ic.iter().any(|point| bool::from(point.is_identity()))
    {
        return Err(PublicVerifyError::Wire);
    }
    let mut canonical = [0; VK_BYTES];
    let mut writer = Cursor::new(&mut canonical[..]);
    vk.write(&mut writer).map_err(|_| PublicVerifyError::Wire)?;
    if writer.position() != VK_BYTES as u64 || canonical.as_slice() != bytes {
        return Err(PublicVerifyError::Wire);
    }
    Ok(vk)
}
fn key_coherence(
    vk: &groth16::VerifyingKey<Bls12>,
    cancel: &AtomicBool,
) -> Result<(), PublicVerifyError> {
    // Bellman samples independent bases g1=a*G1 and g2=b*G2. Comparing beta1
    // or delta1 to canonical generators would incorrectly require a=b. The
    // existing VK instead permits this basis-independent necessary check:
    // e(a*beta*G1,b*delta*G2) = e(a*delta*G1,b*beta*G2).
    // Two finite raw pairings; boundary checks are not pairing preemption.
    // This does NOT establish full CRS validity: joint changes to headers may
    // preserve this equality. The independent complete-key pin remains binding.
    live(cancel)?;
    let left = pairing(&vk.beta_g1, &vk.delta_g2);
    live(cancel)?;
    let right = pairing(&vk.delta_g1, &vk.beta_g2);
    live(cancel)?;
    if left != right {
        return Err(PublicVerifyError::KeyCoherence);
    }
    Ok(())
}
fn check_proof(bytes: &[u8]) -> Result<(), PublicVerifyError> {
    if bytes.len() != PROOF_BYTES || &bytes[..8] != PROOF_MAGIC {
        return Err(PublicVerifyError::Wire);
    }
    let mut cursor = Cursor::new(&bytes[8..]);
    // Checked compressed decoding rejects identity for all three proof points.
    let proof = groth16::Proof::<Bls12>::read(&mut cursor).map_err(|_| PublicVerifyError::Wire)?;
    let mut canonical = [0; PROOF_BYTES];
    canonical[..8].copy_from_slice(PROOF_MAGIC);
    let mut writer = Cursor::new(&mut canonical[8..]);
    proof
        .write(&mut writer)
        .map_err(|_| PublicVerifyError::Wire)?;
    if cursor.position() != (PROOF_BYTES - 8) as u64
        || writer.position() != (PROOF_BYTES - 8) as u64
        || canonical.as_slice() != bytes
    {
        return Err(PublicVerifyError::Wire);
    }
    Ok(())
}

/// Verify one exact public bundle against independently obtained expected pins.
/// No source, proving key, setup, private witness or live network access is used.
/// Success is ONLY the original finite relation; it does not establish scarcity.
pub fn verify_bundle(
    bytes: &[u8],
    expected: &ExpectedPins,
    cancel: &AtomicBool,
) -> Result<VerifiedFileRelation, PublicVerifyError> {
    live(cancel)?;
    if bytes.len() != BUNDLE_BYTES || &bytes[..KEY_START] != BUNDLE_MAGIC {
        return Err(PublicVerifyError::Wire);
    }
    let key = &bytes[KEY_START..ENROLLMENT_START];
    if key[IC_COUNT_OFFSET..IC_COUNT_OFFSET + 4] != 6_u32.to_be_bytes() {
        return Err(PublicVerifyError::Wire);
    }
    let key_hash = hash(&[key]);
    if key_hash != expected.vk_sha256 {
        return Err(PublicVerifyError::KeyPin);
    }
    let enrollment_bytes = &bytes[ENROLLMENT_START..AUDIT_START];
    let audit_bytes = &bytes[AUDIT_START..TAG_START];
    if encoded_statement_pin(enrollment_bytes, audit_bytes) != expected.statement_sha256 {
        return Err(PublicVerifyError::StatementPin);
    }
    live(cancel)?;
    let vk = decode_key(key)?;
    key_coherence(&vk, cancel)?;
    check_proof(&bytes[PROOF_START..])?;
    live(cancel)?;
    let enrollment = Enrollment {
        source_hash: enrollment_bytes[..32].try_into().unwrap(),
        network: enrollment_bytes[32..64].try_into().unwrap(),
        provider: enrollment_bytes[64..96].try_into().unwrap(),
        epoch: u64::from_le_bytes(enrollment_bytes[96..104].try_into().unwrap()),
        replica_root: enrollment_bytes[104..].try_into().unwrap(),
    };
    let audit = AuditContext::decode(audit_bytes).map_err(|_| PublicVerifyError::Wire)?;
    let response = Response {
        tag: bytes[TAG_START..PROOF_START].try_into().unwrap(),
        proof: bytes[PROOF_START..].to_vec(),
    };
    let verifier = Verifier {
        prepared: groth16::prepare_verifying_key(&vk),
        hash: key_hash,
        serialized_bytes: VK_BYTES,
    };
    live(cancel)?;
    // Keep the existing original pairing verifier authoritative and unchanged.
    Ok(verify(&verifier, &enrollment, &audit, &response, cancel)?)
}

#[cfg(test)]
#[path = "public_verification_tests.rs"]
mod tests;
