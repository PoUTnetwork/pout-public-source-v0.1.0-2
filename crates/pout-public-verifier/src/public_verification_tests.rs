use super::*;
use crate::public_fixture;
use bls12_381::{G1Affine, G1Projective, G2Affine, Scalar};

fn fixed<const N: usize>(text: &str) -> [u8; N] {
    assert_eq!(text.len(), N * 2);
    let mut out = [0; N];
    for (byte, pair) in out.iter_mut().zip(text.as_bytes().as_chunks::<2>().0) {
        let nibble = |value| match value {
            b'0'..=b'9' => value - b'0',
            b'a'..=b'f' => value - b'a' + 10,
            _ => panic!("noncanonical trusted test hex"),
        };
        *byte = 16 * nibble(pair[0]) + nibble(pair[1]);
    }
    out
}
fn fixture() -> (public_fixture::PublicFixture, [u8; VK_BYTES], ExpectedPins) {
    let fixture = public_fixture::load().unwrap();
    let key = fixed(public_fixture::VK_HEX);
    let pins = ExpectedPins {
        vk_sha256: fixed(public_fixture::VK_SHA256_HEX),
        statement_sha256: statement_pin(&fixture.enrollment, &fixture.audits[0]),
    };
    (fixture, key, pins)
}
fn wire_error(bytes: &[u8], pins: &ExpectedPins) {
    let error = verify_bundle(bytes, pins, &AtomicBool::new(false)).unwrap_err();
    assert!(matches!(error, PublicVerifyError::Wire), "{error:?}");
    assert!(!error.is_local_abort());
}
fn repin_key(bytes: &[u8; BUNDLE_BYTES], mut pins: ExpectedPins) -> ExpectedPins {
    pins.vk_sha256 = hash(&[&bytes[KEY_START..ENROLLMENT_START]]);
    pins
}
fn repin_statement(bytes: &[u8; BUNDLE_BYTES], mut pins: ExpectedPins) -> ExpectedPins {
    pins.statement_sha256 = encoded_statement_pin(
        &bytes[ENROLLMENT_START..AUDIT_START],
        &bytes[AUDIT_START..TAG_START],
    );
    pins
}
fn replace_key(bytes: &mut [u8; BUNDLE_BYTES], vk: &groth16::VerifyingKey<Bls12>) {
    let mut writer = Cursor::new(&mut bytes[KEY_START..ENROLLMENT_START]);
    vk.write(&mut writer).unwrap();
    assert_eq!(writer.position(), VK_BYTES as u64);
}
#[test]
fn independent_import_accepts_two_genuine_proofs_and_public_reencoding() {
    let (fixture, key, pins) = fixture();
    let cancel = AtomicBool::new(false);
    let vk = decode_key(&key).unwrap();
    // Bellman's independently random bases need not be the canonical G1/G2 pair.
    assert_ne!(
        pairing(&vk.beta_g1, &G2Affine::generator()),
        pairing(&G1Affine::generator(), &vk.beta_g2)
    );
    assert_ne!(
        pairing(&vk.delta_g1, &G2Affine::generator()),
        pairing(&G1Affine::generator(), &vk.delta_g2)
    );
    assert_eq!(
        pairing(&vk.beta_g1, &vk.delta_g2),
        pairing(&vk.delta_g1, &vk.beta_g2)
    );
    key_coherence(&vk, &cancel).unwrap();
    let mut opportunities = Vec::new();
    for index in 0..2 {
        let expected = ExpectedPins {
            statement_sha256: statement_pin(&fixture.enrollment, &fixture.audits[index]),
            ..pins
        };
        let bundle = encode_bundle(
            &key,
            &fixture.enrollment,
            &fixture.audits[index],
            &fixture.responses[index],
        )
        .unwrap();
        let imported = verify_bundle(&bundle, &expected, &cancel).unwrap();
        let original = verify(
            &fixture.verifier,
            &fixture.enrollment,
            &fixture.audits[index],
            &fixture.responses[index],
            &cancel,
        )
        .unwrap();
        assert_eq!(imported.opportunity_id(), original.opportunity_id());
        opportunities.push(imported.opportunity_id());
    }
    assert_ne!(opportunities[0], opportunities[1]);
    let transformed = Response {
        tag: fixture.responses[0].tag,
        proof: adversarial::transform_public(
            &adversarial::public_worker_input(&fixture.delta_g2, &fixture.responses[0]).unwrap(),
            &cancel,
        )
        .unwrap()
        .to_vec(),
    };
    assert_ne!(transformed.proof, fixture.responses[0].proof);
    let bundle =
        encode_bundle(&key, &fixture.enrollment, &fixture.audits[0], &transformed).unwrap();
    assert_eq!(
        verify_bundle(&bundle, &pins, &cancel)
            .unwrap()
            .opportunity_id(),
        opportunities[0]
    );
}
#[test]
fn independent_pins_cover_the_complete_key_enrollment_and_audit() {
    let (fixture, key, pins) = fixture();
    let bundle = encode_bundle(
        &key,
        &fixture.enrollment,
        &fixture.audits[0],
        &fixture.responses[0],
    )
    .unwrap();
    let cancel = AtomicBool::new(false);
    let mut wrong_key = pins;
    wrong_key.vk_sha256[0] ^= 1;
    assert!(matches!(
        verify_bundle(&bundle, &wrong_key, &cancel),
        Err(PublicVerifyError::KeyPin)
    ));
    let mut wrong_statement = pins;
    wrong_statement.statement_sha256[0] ^= 1;
    assert!(matches!(
        verify_bundle(&bundle, &wrong_statement, &cancel),
        Err(PublicVerifyError::StatementPin)
    ));
    // Every component is committed, not only truncated context/challenge.
    for offset in [0, 32, 64, 96, 104, 136, 168, 176, 184, 215] {
        let mut changed = bundle;
        changed[ENROLLMENT_START + offset] ^= 1;
        assert!(matches!(
            verify_bundle(&changed, &pins, &cancel),
            Err(PublicVerifyError::StatementPin)
        ));
        // Deliberately changed TEST authority reaches the original verifier;
        // it is not an attacker being permitted to select its own pin.
        let trusted_changed = repin_statement(&changed, pins);
        let error = verify_bundle(&changed, &trusted_changed, &cancel).unwrap_err();
        if [32, 96, 136, 168].contains(&offset) {
            assert!(matches!(
                error,
                PublicVerifyError::Relation(FileRelationError::Context)
            ));
        } else {
            assert!(matches!(
                error,
                PublicVerifyError::Relation(FileRelationError::Relation)
            ));
        }
        assert!(!error.is_local_abort());
    }
}
#[test]
fn bounded_key_decoder_rejects_counts_framing_identity_and_torsion() {
    let (fixture, key, pins) = fixture();
    let bundle = encode_bundle(
        &key,
        &fixture.enrollment,
        &fixture.audits[0],
        &fixture.responses[0],
    )
    .unwrap();
    for length in [0, 7, 8, 1451, BUNDLE_BYTES - 1] {
        wire_error(&bundle[..length], &pins);
    }
    let mut trailing = bundle.to_vec();
    trailing.push(0);
    wire_error(&trailing, &pins);
    let mut changed = bundle;
    changed[0] ^= 1;
    wire_error(&changed, &pins);
    for count in [0_u32, 5, 7, u32::MAX] {
        let mut changed = bundle;
        changed[KEY_START + IC_COUNT_OFFSET..KEY_START + IC_COUNT_OFFSET + 4]
            .copy_from_slice(&count.to_be_bytes());
        // Even deliberately pinned huge counts fail before allocation.
        wire_error(&changed, &repin_key(&changed, pins));
    }
    // Canonical uncompressed point positions: six headers and six ICs.
    let headers = [
        (0, 96),
        (96, 96),
        (192, 192),
        (384, 192),
        (576, 96),
        (672, 192),
    ];
    for (offset, size) in headers
        .into_iter()
        .chain((0..6).map(|i| (868 + 96 * i, 96)))
    {
        let mut changed = bundle;
        {
            let slot = &mut changed[KEY_START + offset..KEY_START + offset + size];
            if size == 96 {
                slot.copy_from_slice(&G1Affine::identity().to_uncompressed());
            } else {
                slot.copy_from_slice(&G2Affine::identity().to_uncompressed());
            }
        }
        wire_error(&changed, &repin_key(&changed, pins));
        changed[KEY_START + offset..KEY_START + offset + size].fill(0xff);
        wire_error(&changed, &repin_key(&changed, pins));
    }
    // x=0,y=2 is on-curve with order3; checked subgroup decoding rejects it.
    let mut compressed_torsion = [0; 48];
    compressed_torsion[0] = 0x80;
    let torsion =
        Option::<G1Affine>::from(G1Affine::from_compressed_unchecked(&compressed_torsion)).unwrap();
    assert!(bool::from(torsion.is_on_curve()));
    assert!(!bool::from(torsion.is_torsion_free()));
    for offset in [0, 868] {
        let mut changed = bundle;
        changed[KEY_START + offset..KEY_START + offset + 96]
            .copy_from_slice(&torsion.to_uncompressed());
        wire_error(&changed, &repin_key(&changed, pins));
    }
    assert!(matches!(
        encode_bundle(
            &key[..VK_BYTES - 1],
            &fixture.enrollment,
            &fixture.audits[0],
            &fixture.responses[0]
        ),
        Err(PublicVerifyError::Wire)
    ));
}
#[test]
fn key_pins_and_cross_group_checks_reject_unused_header_substitution() {
    let (fixture, key, pins) = fixture();
    let bundle = encode_bundle(
        &key,
        &fixture.enrollment,
        &fixture.audits[0],
        &fixture.responses[0],
    )
    .unwrap();
    let cancel = AtomicBool::new(false);
    for beta in [true, false] {
        let mut vk = decode_key(&key).unwrap();
        if beta {
            vk.beta_g1 = G1Affine::from(G1Projective::from(vk.beta_g1) * Scalar::from(2));
        } else {
            vk.delta_g1 = G1Affine::from(G1Projective::from(vk.delta_g1) * Scalar::from(2));
        }
        let mut changed = bundle;
        replace_key(&mut changed, &vk);
        assert!(matches!(
            verify_bundle(&changed, &pins, &cancel),
            Err(PublicVerifyError::KeyPin)
        ));
        let new_test_authority = repin_key(&changed, pins);
        assert!(matches!(
            verify_bundle(&changed, &new_test_authority, &cancel),
            Err(PublicVerifyError::KeyCoherence)
        ));
        // Controlled demonstration: original pairing does not use these
        // headerG1 fields. This is NOT authority to change an external pin.
        let direct = Verifier {
            prepared: groth16::prepare_verifying_key(&vk),
            hash: new_test_authority.vk_sha256,
            serialized_bytes: VK_BYTES,
        };
        let accepted = verify(
            &direct,
            &fixture.enrollment,
            &fixture.audits[0],
            &fixture.responses[0],
            &cancel,
        )
        .unwrap();
        let original = verify(
            &fixture.verifier,
            &fixture.enrollment,
            &fixture.audits[0],
            &fixture.responses[0],
            &cancel,
        )
        .unwrap();
        assert_ne!(accepted.opportunity_id(), original.opportunity_id());
    }
    let mut vk = decode_key(&key).unwrap();
    vk.alpha_g1 = G1Affine::from(G1Projective::from(vk.alpha_g1) * Scalar::from(2));
    let mut changed = bundle;
    replace_key(&mut changed, &vk);
    let trusted_changed = repin_key(&changed, pins);
    assert!(matches!(
        verify_bundle(&changed, &trusted_changed, &cancel),
        Err(PublicVerifyError::Relation(FileRelationError::Relation))
    ));
}
#[test]
fn proof_framing_subgroups_and_false_tags_reach_the_expected_checks() {
    let (fixture, key, pins) = fixture();
    let bundle = encode_bundle(
        &key,
        &fixture.enrollment,
        &fixture.audits[0],
        &fixture.responses[0],
    )
    .unwrap();
    let mut changed = bundle;
    changed[PROOF_START] ^= 1;
    wire_error(&changed, &pins);
    for (offset, size) in [(8, 48), (56, 96), (152, 48)] {
        let mut changed = bundle;
        if size == 48 {
            changed[PROOF_START + offset..PROOF_START + offset + size]
                .copy_from_slice(&G1Affine::identity().to_compressed());
        } else {
            changed[PROOF_START + offset..PROOF_START + offset + size]
                .copy_from_slice(&G2Affine::identity().to_compressed());
        }
        wire_error(&changed, &pins);
        changed[PROOF_START + offset..PROOF_START + offset + size].fill(0xff);
        wire_error(&changed, &pins);
    }
    let mut changed = bundle;
    changed[PROOF_START + 8..PROOF_START + 56].fill(0);
    changed[PROOF_START + 8] = 0x80; // same order3 torsion as above
    wire_error(&changed, &pins);
    let mut changed = bundle;
    changed[TAG_START] ^= 1;
    assert!(matches!(
        verify_bundle(&changed, &pins, &AtomicBool::new(false)),
        Err(PublicVerifyError::Relation(FileRelationError::Relation))
    ));
    let mut short = fixture.responses[0].clone();
    short.proof.pop();
    assert!(matches!(
        encode_bundle(&key, &fixture.enrollment, &fixture.audits[0], &short),
        Err(PublicVerifyError::Wire)
    ));
}
#[test]
fn cancellation_is_a_local_abort_without_an_invalidity_claim() {
    let (fixture, key, pins) = fixture();
    let bundle = encode_bundle(
        &key,
        &fixture.enrollment,
        &fixture.audits[0],
        &fixture.responses[0],
    )
    .unwrap();
    let cancel = AtomicBool::new(true);
    let error = verify_bundle(&bundle, &pins, &cancel).unwrap_err();
    assert!(matches!(error, PublicVerifyError::Cancelled));
    assert!(error.is_local_abort());
    // Even malformed input under local pre-cancellation is no validity
    // conclusion; cancellation wins before any parser/key authority claim.
    let error = verify_bundle(&[], &pins, &cancel).unwrap_err();
    assert!(matches!(error, PublicVerifyError::Cancelled));
    assert!(error.is_local_abort());
    assert!(PublicVerifyError::Relation(FileRelationError::LocalBackend).is_local_abort());
    assert!(PublicVerifyError::Relation(FileRelationError::Cancelled).is_local_abort());
    assert!(!PublicVerifyError::KeyCoherence.is_local_abort());
    cancel.store(false, Ordering::Release);
    assert!(verify_bundle(&bundle, &pins, &cancel).is_ok());
}

/// Pure algebraic re-encoding of public proof points; no new witness or setup.
mod adversarial {
    use crate::{Response, MAGIC, PROOF_BYTES};
    use bellman::groth16;
    use bls12_381::{Bls12, G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
    use std::{
        io::Cursor,
        sync::atomic::{AtomicBool, Ordering},
    };
    use thiserror::Error;

    const WORKER_INPUT_BYTES: usize = 304;
    const WORKER_MAGIC: &[u8; 8] = b"PTMALL01";
    const DELTA_BYTES: usize = 96;
    #[derive(Debug, Error)]
    pub enum AdversarialError {
        #[error("malformed bounded public-worker input or canonical group encoding")]
        Wire,
        #[error("local cancellation, not invalidity of a public proof")]
        Cancelled,
        #[error("fixed public transformation produced identity; finite experiment abort")]
        DegenerateOutput,
        #[error("public adversarial experiment invariant: {0}")]
        Invariant(&'static str),
    }
    fn live(cancel: &AtomicBool) -> Result<(), AdversarialError> {
        if cancel.load(Ordering::Acquire) {
            Err(AdversarialError::Cancelled)
        } else {
            Ok(())
        }
    }
    fn decode_delta(bytes: &[u8; DELTA_BYTES]) -> Result<G2Affine, AdversarialError> {
        let point = Option::<G2Affine>::from(G2Affine::from_compressed(bytes))
            .ok_or(AdversarialError::Wire)?;
        if bool::from(point.is_identity()) || point.to_compressed() != *bytes {
            return Err(AdversarialError::Wire);
        }
        Ok(point)
    }
    fn encode_proof(proof: &groth16::Proof<Bls12>) -> Result<[u8; PROOF_BYTES], AdversarialError> {
        let mut bytes = [0; PROOF_BYTES];
        bytes[..MAGIC.len()].copy_from_slice(MAGIC);
        proof
            .write(&mut &mut bytes[MAGIC.len()..])
            .map_err(|_| AdversarialError::Wire)?;
        Ok(bytes)
    }
    fn decode_proof(bytes: &[u8]) -> Result<groth16::Proof<Bls12>, AdversarialError> {
        if bytes.len() != PROOF_BYTES || &bytes[..MAGIC.len()] != MAGIC {
            return Err(AdversarialError::Wire);
        }
        let mut reader = Cursor::new(&bytes[MAGIC.len()..]);
        // Bellman invokes subgroup-checked compressed decoding and rejects identity
        // for all three proof points. Re-encoding also checks canonical framing.
        let proof =
            groth16::Proof::<Bls12>::read(&mut reader).map_err(|_| AdversarialError::Wire)?;
        if reader.position() as usize != PROOF_BYTES - MAGIC.len()
            || encode_proof(&proof)?.as_slice() != bytes
        {
            return Err(AdversarialError::Wire);
        }
        Ok(proof)
    }
    /// Encode ONLY public delta_G2 and the existing public proof. The tag, source,
    /// witness, proving parameters, RNG and private preparation are not inputs.
    pub fn public_worker_input(
        delta_g2: &[u8; DELTA_BYTES],
        response: &Response,
    ) -> Result<[u8; WORKER_INPUT_BYTES], AdversarialError> {
        decode_delta(delta_g2)?;
        decode_proof(&response.proof)?;
        let mut bytes = [0; WORKER_INPUT_BYTES];
        bytes[..8].copy_from_slice(WORKER_MAGIC);
        bytes[8..8 + DELTA_BYTES].copy_from_slice(delta_g2);
        bytes[8 + DELTA_BYTES..].copy_from_slice(&response.proof);
        Ok(bytes)
    }
    /// One fixed public algebraic transform, u=2,v=1. No search, randomness,
    /// setup, proving call or file access. Input framing is exact; trailing bytes
    /// are rejected rather than being interpreted as another request.
    pub fn transform_public(
        input: &[u8],
        cancel: &AtomicBool,
    ) -> Result<[u8; PROOF_BYTES], AdversarialError> {
        live(cancel)?;
        if input.len() != WORKER_INPUT_BYTES || &input[..8] != WORKER_MAGIC {
            return Err(AdversarialError::Wire);
        }
        let delta_bytes: [u8; DELTA_BYTES] = input[8..8 + DELTA_BYTES]
            .try_into()
            .map_err(|_| AdversarialError::Wire)?;
        let delta = decode_delta(&delta_bytes)?;
        let proof = decode_proof(&input[8 + DELTA_BYTES..])?;
        let u = Scalar::from(2);
        let inverse = Option::<Scalar>::from(u.invert()).ok_or(AdversarialError::Invariant(
            "fixed_nonzero_scalar_inversion",
        ))?;
        let a = G1Projective::from(proof.a) * u;
        let b = G2Projective::from(proof.b) * inverse + G2Projective::from(delta);
        let c = G1Projective::from(proof.c) + a;
        let result = groth16::Proof::<Bls12> {
            a: G1Affine::from(a),
            b: G2Affine::from(b),
            c: G1Affine::from(c),
        };
        if bool::from(result.a.is_identity())
            || bool::from(result.b.is_identity())
            || bool::from(result.c.is_identity())
        {
            return Err(AdversarialError::DegenerateOutput);
        }
        live(cancel)?;
        encode_proof(&result)
    }
}
