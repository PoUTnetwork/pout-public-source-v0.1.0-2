//! Public-API acceptance and rejection checks for the distributed vectors.
use pout_public_verifier::{
    public_fixture,
    public_verify::{
        encode_bundle, statement_pin, verify_bundle, ExpectedPins, PublicVerifyError, BUNDLE_BYTES,
        VK_BYTES,
    },
    FileRelationError,
};
use std::sync::atomic::AtomicBool;

fn bundle(index: usize) -> ([u8; BUNDLE_BYTES], ExpectedPins) {
    let fixture = public_fixture::load().unwrap();
    let mut key = [0; VK_BYTES];
    assert_eq!(public_fixture::VK_HEX.len(), 2 * VK_BYTES);
    for (out, pair) in key
        .iter_mut()
        .zip(public_fixture::VK_HEX.as_bytes().as_chunks::<2>().0)
    {
        let digit = |b| match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            _ => panic!("noncanonical public fixture hex"),
        };
        *out = 16 * digit(pair[0]) + digit(pair[1]);
    }
    let pins = ExpectedPins {
        vk_sha256: fixture.verifier.vk_hash(),
        statement_sha256: statement_pin(&fixture.enrollment, &fixture.audits[index]),
    };
    (
        encode_bundle(
            &key,
            &fixture.enrollment,
            &fixture.audits[index],
            &fixture.responses[index],
        )
        .unwrap(),
        pins,
    )
}

#[test]
fn public_api_accepts_genuine_proofs_and_replay_preserves_identifier() {
    let (first, first_pins) = bundle(0);
    let (second, second_pins) = bundle(1);
    let cancel = AtomicBool::new(false);
    let first_id = verify_bundle(&first, &first_pins, &cancel)
        .unwrap()
        .opportunity_id();
    assert_eq!(
        verify_bundle(&first, &first_pins, &cancel)
            .unwrap()
            .opportunity_id(),
        first_id
    );
    assert_ne!(
        verify_bundle(&second, &second_pins, &cancel)
            .unwrap()
            .opportunity_id(),
        first_id
    );
}

#[test]
fn public_api_rejects_wrong_authority_false_tag_and_malformed_wire() {
    let (mut bytes, pins) = bundle(0);
    let cancel = AtomicBool::new(false);
    let mut wrong_key = pins;
    wrong_key.vk_sha256[0] ^= 1;
    assert!(matches!(
        verify_bundle(&bytes, &wrong_key, &cancel),
        Err(PublicVerifyError::KeyPin)
    ));
    let mut wrong_statement = pins;
    wrong_statement.statement_sha256[0] ^= 1;
    assert!(matches!(
        verify_bundle(&bytes, &wrong_statement, &cancel),
        Err(PublicVerifyError::StatementPin)
    ));
    assert!(matches!(
        verify_bundle(&bytes[..BUNDLE_BYTES - 1], &pins, &cancel),
        Err(PublicVerifyError::Wire)
    ));
    // The public tag is the 32 bytes immediately before the 200-byte proof.
    bytes[BUNDLE_BYTES - 200 - 32] ^= 1;
    assert!(matches!(
        verify_bundle(&bytes, &pins, &cancel),
        Err(PublicVerifyError::Relation(FileRelationError::Relation))
    ));
    let error = verify_bundle(&bytes, &pins, &AtomicBool::new(true)).unwrap_err();
    assert!(matches!(error, PublicVerifyError::Cancelled));
    assert!(error.is_local_abort());
}
