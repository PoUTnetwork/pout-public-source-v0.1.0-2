//! Public M19 finite-model demonstration; no network or assets.
#[path = "support/causal_byte_bridge.rs"]
mod experiment;

use std::sync::atomic::AtomicBool;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node = include_bytes!("../../../tests/fixtures/milestones/m19-node-index2.frame");
    let rank = include_bytes!("../../../tests/fixtures/milestones/m19-rank-vector.frame");
    experiment::qualify_independent_golden(node, rank)?;
    let report = experiment::run(experiment::Limits::default(), &AtomicBool::new(false))?;
    print!("{}", experiment::format_report(&report)?);
    Ok(())
}
