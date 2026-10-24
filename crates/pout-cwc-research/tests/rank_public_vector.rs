//! Public retained M17 BLAKE3 vector; no resource proof or network execution.

#[test]
fn retained_full_pair_rank_matches_recorded_half_policy_digest() {
    let input = include_bytes!("../../../tests/fixtures/milestones/m17-half-rank.frame");
    let observed = blake3::hash(input);
    assert_eq!(
        observed.to_hex().as_str(),
        "61cee385e783a45f257747620f360b417e62fce2c563fed1dd31ab84c9b0f62b"
    );
    let mut half = [0_u8; 32];
    half[0] = 0x80;
    assert!(observed.as_bytes() < &half);
    // This is a retained-input digest check, not a fairness or scarcity proof.
}
