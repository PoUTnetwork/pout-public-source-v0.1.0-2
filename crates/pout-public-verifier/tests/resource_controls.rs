//! Isolated synthetic accounting regressions; NOT Groth16 or service validation.
#[path = "../examples/support/resource_controls.rs"]
mod research;

use research::{
    authorized_fixture, claim, fixture, Decision, KeyFamilies, Reject, RelationObservation, View,
};

#[test]
fn freshness_window_is_half_open_and_bounded() {
    let c = fixture(3);
    let keys = authorized_fixture();
    let p = claim(&c, 10);
    let mut view = View::default();
    assert_eq!(view.observe(&c, &p, &keys, 99), Err(Reject::NotYetIssued));
    assert_eq!(view.observe(&c, &p, &keys, 100), Ok(Decision::FirstModelConsumption));
    assert_eq!(view.observe(&c, &p, &keys, 199), Ok(Decision::AlreadyConsumed));
    assert_eq!(view.observe(&c, &p, &keys, 200), Err(Reject::Expired));
    assert_eq!(view.consumption_count(), 1);
}

#[test]
fn invalid_windows_fail_without_consumption() {
    let mut c = fixture(3);
    c.expires_at = c.issued_at;
    let mut view = View::default();
    assert_eq!(view.observe(&c, &claim(&c, 10), &authorized_fixture(), 100), Err(Reject::InvalidWindow));
    assert_eq!(view.consumption_count(), 0);
}

#[test]
fn authorized_key_rotation_does_not_reset_consumption() {
    let c = fixture(3);
    let keys = authorized_fixture();
    let mut view = View::default();
    assert_eq!(view.observe(&c, &claim(&c, 10), &keys, 150), Ok(Decision::FirstModelConsumption));
    assert_eq!(view.observe(&c, &claim(&c, 11), &keys, 150), Ok(Decision::AlreadyConsumed));
    assert_eq!(view.consumption_count(), 1);
}

#[test]
fn split_families_can_double_count_same_synthetic_resource() {
    // Counterexample: an operator mapping the same resource to different families
    // creates distinct model consumption domains. There is no physical independence.
    let c = fixture(3);
    let keys = authorized_fixture();
    let mut view = View::default();
    assert_eq!(view.observe(&c, &claim(&c, 10), &keys, 150), Ok(Decision::FirstModelConsumption));
    assert_eq!(view.observe(&c, &claim(&c, 12), &keys, 150), Ok(Decision::FirstModelConsumption));
    assert_eq!(view.consumption_count(), 2);
}

#[test]
fn renamed_unbound_resource_cannot_replace_expected_anchor() {
    let c = fixture(3);
    let mut p = claim(&c, 10);
    p.claimed_resource = [42; 32];
    let mut view = View::default();
    assert_eq!(view.observe(&c, &p, &authorized_fixture(), 150), Err(Reject::WrongResource));
    assert_eq!(view.consumption_count(), 0);
}

#[test]
fn issuer_scope_challenge_and_provider_are_separately_bound() {
    let c = fixture(3);
    let keys = authorized_fixture();
    let cases = [
        (Reject::WrongIssuer, 0),
        (Reject::WrongScope, 1),
        (Reject::WrongChallenge, 2),
        (Reject::WrongProvider, 3),
    ];
    for (expected_rejection, changed_field) in cases {
        let mut p = claim(&c, 10);
        match changed_field {
            0 => p.issuer = [40; 16],
            1 => p.scope = [40; 16],
            2 => p.challenge = [40; 16],
            _ => p.provider = [40; 16],
        }
        let mut view = View::default();
        assert_eq!(view.observe(&c, &p, &keys, 150), Err(expected_rejection));
        assert_eq!(view.consumption_count(), 0);
    }
}

#[test]
fn invalid_relation_and_unapproved_key_are_not_model_acceptances() {
    let c = fixture(3);
    let mut p = claim(&c, 10);
    p.relation_observation = RelationObservation::ObservedInvalid;
    let mut view = View::default();
    assert_eq!(view.observe(&c, &p, &authorized_fixture(), 150), Err(Reject::InvalidRelation));
    assert_eq!(view.observe(&c, &claim(&c, 40), &authorized_fixture(), 150), Err(Reject::UnapprovedKey));
    assert_eq!(view.consumption_count(), 0);
}

#[test]
fn logical_challenge_id_changes_do_not_establish_more_storage() {
    let keys = authorized_fixture();
    let mut view = View::default();
    for value in [3, 4] {
        let c = fixture(value);
        assert_eq!(view.observe(&c, &claim(&c, 10), &keys, 150), Ok(Decision::FirstModelConsumption));
    }
    assert_eq!(view.consumption_count(), 2);
    // Same external resource anchor, distinct challenge IDs, no physical evidence.
}

#[test]
fn fork_local_consumption_is_not_additive_canonical_weight() {
    let c = fixture(3);
    let keys = authorized_fixture();
    let parent = View::default();
    let mut left = parent.clone();
    let mut right = parent;
    assert_eq!(left.observe(&c, &claim(&c, 10), &keys, 150), Ok(Decision::FirstModelConsumption));
    assert_eq!(right.observe(&c, &claim(&c, 10), &keys, 150), Ok(Decision::FirstModelConsumption));
    assert_eq!((left.consumption_count(), right.consumption_count()), (1, 1));
    // Both views represent alternatives; their sum is NOT canonical consensus weight.
}

#[test]
fn consumption_capacity_rejects_without_mutation() {
    let keys = authorized_fixture();
    let mut view = View::default();
    for value in 0..64 {
        let c = fixture(value);
        assert_eq!(view.observe(&c, &claim(&c, 10), &keys, 150), Ok(Decision::FirstModelConsumption));
    }
    let c = fixture(65);
    assert_eq!(view.observe(&c, &claim(&c, 10), &keys, 150), Err(Reject::Capacity));
    assert_eq!(view.consumption_count(), 64);
}

#[test]
fn unknown_key_cannot_self_enroll_through_a_presentation() {
    let c = fixture(3);
    let keys = KeyFamilies::default();
    let mut view = View::default();
    assert_eq!(view.observe(&c, &claim(&c, 10), &keys, 150), Err(Reject::UnapprovedKey));
    assert_eq!(view.consumption_count(), 0);
}
