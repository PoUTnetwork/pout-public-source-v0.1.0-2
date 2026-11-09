fn audited_stock(ledger: &PowerLedger, origins: &BTreeSet<Hash32>) -> BTreeMap<Owner, u64> {
    let mut stock = BTreeMap::new();
    for output in ledger
        .outputs()
        .filter(|output| output.sector.is_some() && origins.contains(&output.origin))
    {
        *stock.entry(output.owner).or_insert(0) += output.capacity;
    }
    stock
}

pub(crate) fn evaluate(
    ledger: &PowerLedger,
    history: &AuditHistory,
    profile: &AcceptanceProfile,
    origins: &BTreeSet<Hash32>,
) -> ResourceEvaluation {
    let by_owner = audited_stock(ledger, origins);
    let audited_units: u64 = by_owner.values().sum();
    let mut unmet = Vec::new();
    if history.audited_epochs < profile.min_audited_epochs {
        unmet.push(Criterion::AuditedEpochs);
    }
    if history.distinct_beacons < profile.min_distinct_beacons {
        unmet.push(Criterion::DistinctBeacons);
    }
    if history.failures > profile.max_failures {
        unmet.push(Criterion::Failures);
    }
    if audited_units < profile.min_audited_units.max(1) {
        unmet.push(Criterion::AuditedUnits);
    }
    if ledger.check_invariants().is_err() {
        unmet.push(Criterion::LedgerInvariants);
    }
    let established = unmet.is_empty();
    ResourceEvaluation {
        mode: MODE,
        scope: SCOPE,
        consensus_eligibility: if established {
            Eligibility::Established
        } else {
            Eligibility::NotEstablished
        },
        consensus_weight: established.then_some(audited_units),
        weight_by_owner: if established {
            by_owner
        } else {
            BTreeMap::new()
        },
        live_units_informational: ledger.live_total(),
        audited_units,
        history: *history,
        unmet,
        open_production_gates: OPEN_PRODUCTION_GATES.to_vec(),
    }
}

pub const OPEN_PRODUCTION_GATES: [ProductionGate; 7] = [
    ProductionGate::ExternalSecurityReview,
    ProductionGate::MultiDeviceTest,
    ProductionGate::BeaconUnpredictability,
    ProductionGate::DelayPrimitiveGate,
    ProductionGate::DepthRobustnessGate,
    ProductionGate::ResourceTheorem,
    ProductionGate::HostedCi,
];
