//! Standalone model of externally pinned freshness, identifiers and consumption.
#[path = "support/resource_controls.rs"]
mod research;

use research::{authorized_fixture, claim, fixture, Decision, Reject, View};

fn run() -> Result<(), &'static str> {
    let expected = fixture(3);
    let keys = authorized_fixture();
    let mut view = View::default();
    if view.observe(&expected, &claim(&expected, 10), &keys, 100)
        != Ok(Decision::FirstModelConsumption)
    {
        return Err("fresh synthetic scope not recorded");
    }
    if view.observe(&expected, &claim(&expected, 11), &keys, 150)
        != Ok(Decision::AlreadyConsumed)
    {
        return Err("same family incorrectly counted twice");
    }
    if view.observe(&expected, &claim(&expected, 10), &keys, 200) != Err(Reject::Expired) {
        return Err("expiry boundary not enforced");
    }
    if view.consumption_count() != 1 {
        return Err("rejection mutated accounting");
    }
    println!("finite_accounting_model=complete");
    println!("fresh_scope=recorded");
    println!("same_family_rotation=deduplicated");
    println!("expiry_boundary=rejected");
    println!("cryptographic_verification=not_executed");
    println!("physical_resource_scarcity=not_established");
    println!("consensus_eligibility=NotEstablished");
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("model_error={error}");
            std::process::ExitCode::FAILURE
        }
    }
}
