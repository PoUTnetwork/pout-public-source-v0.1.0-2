//! Finite accounting-controls experiment over synthetic, externally asserted facts.
//!
//! This module performs NO Groth16 verification, resource measurement or consensus
//! admission. The caller's `relation_observation` is an explicit model assumption.
//! Time, issuer, key/family authorization and resource binding are externally
//! trusted inputs; this code cannot authenticate those external authorities.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

type Id16 = [u8; 16];
type Id32 = [u8; 32];
const MAX_CONSUMPTIONS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationObservation {
    AssumedValidForModel,
    ObservedInvalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Decision {
    FirstModelConsumption,
    AlreadyConsumed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reject {
    InvalidWindow,
    NotYetIssued,
    Expired,
    WrongIssuer,
    WrongScope,
    WrongChallenge,
    WrongProvider,
    WrongResource,
    UnapprovedKey,
    InvalidRelation,
    Capacity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedContext {
    pub issuer: Id16,
    pub scope: Id16,
    pub challenge: Id16,
    pub provider: Id16,
    pub resource_anchor: Id32,
    pub issued_at: u64,
    pub expires_at: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Presentation {
    pub issuer: Id16,
    pub scope: Id16,
    pub challenge: Id16,
    pub provider: Id16,
    pub claimed_resource: Id32,
    pub verifier_key: Id32,
    pub relation_observation: RelationObservation,
}

#[derive(Clone, Debug, Default)]
pub struct KeyFamilies {
    // Populated by a synthetic externally authorized operator. Not self-enrollment.
    enrolled: BTreeMap<Id32, Id32>,
}

impl KeyFamilies {
    pub fn enroll_for_research(&mut self, key: Id32, family: Id32) {
        self.enrolled.insert(key, family);
    }

    pub fn family_for(&self, key: &Id32) -> Option<Id32> {
        self.enrolled.get(key).copied()
    }
}

#[derive(Clone, Debug, Default)]
pub struct View {
    consumed: BTreeSet<Id32>,
}

impl View {
    pub fn consumption_count(&self) -> usize {
        self.consumed.len()
    }

    pub fn observe(
        &mut self,
        expected: &ExpectedContext,
        claim: &Presentation,
        authorized: &KeyFamilies,
        assumed_trusted_now: u64,
    ) -> Result<Decision, Reject> {
        // Deterministic fail-closed validation before any state change.
        if expected.issued_at >= expected.expires_at {
            return Err(Reject::InvalidWindow);
        }
        if assumed_trusted_now < expected.issued_at {
            return Err(Reject::NotYetIssued);
        }
        if assumed_trusted_now >= expected.expires_at {
            return Err(Reject::Expired);
        }
        if claim.issuer != expected.issuer {
            return Err(Reject::WrongIssuer);
        }
        if claim.scope != expected.scope {
            return Err(Reject::WrongScope);
        }
        if claim.challenge != expected.challenge {
            return Err(Reject::WrongChallenge);
        }
        if claim.provider != expected.provider {
            return Err(Reject::WrongProvider);
        }
        if claim.claimed_resource != expected.resource_anchor {
            return Err(Reject::WrongResource);
        }
        let family = authorized
            .family_for(&claim.verifier_key)
            .ok_or(Reject::UnapprovedKey)?;
        if claim.relation_observation != RelationObservation::AssumedValidForModel {
            return Err(Reject::InvalidRelation);
        }

        let id = logical_consumption_id(expected, &family);
        if self.consumed.contains(&id) {
            return Ok(Decision::AlreadyConsumed);
        }
        if self.consumed.len() >= MAX_CONSUMPTIONS {
            return Err(Reject::Capacity);
        }
        self.consumed.insert(id);
        Ok(Decision::FirstModelConsumption)
    }
}

pub fn logical_consumption_id(expected: &ExpectedContext, family: &Id32) -> Id32 {
    let mut h = Sha256::new();
    h.update(b"PoUT/public-research/accounting-family/v1");
    h.update(expected.issuer);
    h.update(expected.scope);
    h.update(expected.challenge);
    h.update(expected.resource_anchor);
    h.update(family);
    h.finalize().into()
}

pub fn fixture(value: u8) -> ExpectedContext {
    ExpectedContext {
        issuer: [1; 16],
        scope: [2; 16],
        challenge: [value; 16],
        provider: [4; 16],
        resource_anchor: [5; 32],
        issued_at: 100,
        expires_at: 200,
    }
}

pub fn claim(context: &ExpectedContext, key: u8) -> Presentation {
    Presentation {
        issuer: context.issuer,
        scope: context.scope,
        challenge: context.challenge,
        provider: context.provider,
        claimed_resource: context.resource_anchor,
        verifier_key: [key; 32],
        relation_observation: RelationObservation::AssumedValidForModel,
    }
}

pub fn authorized_fixture() -> KeyFamilies {
    let mut keys = KeyFamilies::default();
    keys.enroll_for_research([10; 32], [90; 32]);
    keys.enroll_for_research([11; 32], [90; 32]);
    keys.enroll_for_research([12; 32], [91; 32]);
    keys
}
