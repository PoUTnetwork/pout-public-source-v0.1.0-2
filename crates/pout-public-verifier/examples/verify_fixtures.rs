//! Verify retained public synthetic vectors and a cryptographic negative control.
use pout_public_verifier::{
    public_fixture,
    public_verify::{
        encode_bundle, statement_pin, verify_bundle, ExpectedPins, PublicVerifyError, VK_BYTES,
    },
    FileRelationError,
};
use std::{error::Error, sync::atomic::AtomicBool};

fn decode_key(text: &str) -> Result<[u8; VK_BYTES], Box<dyn Error>> {
    if text.len() != 2 * VK_BYTES {
        return Err("unexpected synthetic key length".into());
    }
    let mut key = [0; VK_BYTES];
    for (out, pair) in key.iter_mut().zip(text.as_bytes().as_chunks::<2>().0) {
        let digit = |b| match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            _ => Err("noncanonical synthetic key hex"),
        };
        *out = 16 * digit(pair[0])? + digit(pair[1])?;
    }
    Ok(key)
}
fn main() -> Result<(), Box<dyn Error>> {
    let fixture = public_fixture::load()?;
    let key = decode_key(public_fixture::VK_HEX)?;
    let cancel = AtomicBool::new(false);
    let mut accepted = Vec::new();
    for index in 0..2 {
        let pins = ExpectedPins {
            vk_sha256: fixture.verifier.vk_hash(),
            statement_sha256: statement_pin(&fixture.enrollment, &fixture.audits[index]),
        };
        let bundle = encode_bundle(
            &key,
            &fixture.enrollment,
            &fixture.audits[index],
            &fixture.responses[index],
        )?;
        accepted.push(verify_bundle(&bundle, &pins, &cancel)?.opportunity_id());
    }
    assert_ne!(accepted[0], accepted[1]);
    let pins = ExpectedPins {
        vk_sha256: fixture.verifier.vk_hash(),
        statement_sha256: statement_pin(&fixture.enrollment, &fixture.audits[1]),
    };
    let wrong = encode_bundle(
        &key,
        &fixture.enrollment,
        &fixture.audits[1],
        &fixture.wrong_response,
    )?;
    assert!(matches!(
        verify_bundle(&wrong, &pins, &cancel),
        Err(PublicVerifyError::Relation(FileRelationError::Relation))
    ));
    let alternate_pins = ExpectedPins {
        vk_sha256: fixture.verifier.vk_hash(),
        statement_sha256: statement_pin(&fixture.alternate_enrollment, &fixture.audits[1]),
    };
    let alternate = encode_bundle(
        &key,
        &fixture.alternate_enrollment,
        &fixture.audits[1],
        &fixture.wrong_response,
    )?;
    verify_bundle(&alternate, &alternate_pins, &cancel)?;
    println!("Two original synthetic proofs verified; distinct statements have distinct opportunity identifiers.");
    println!("Wrong-source proof rejected under original statement and verified under truthful alternate statement.");
    println!(
        "Fixture pins are controlled test constants, not independent production authorization."
    );
    println!("Consensus eligibility: NotEstablished. No delivery, scarcity or demand claim.");
    Ok(())
}
