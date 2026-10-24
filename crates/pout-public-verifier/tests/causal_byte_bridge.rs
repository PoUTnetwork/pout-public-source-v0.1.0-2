//! Eight inspections/guards; one failure-sticky M19 causal corpus initialization.
#[path = "../examples/support/causal_byte_bridge.rs"]
mod experiment;
#[path = "../examples/support/fixture_once.rs"]
mod fixture_once;

use experiment::{Actor, Category, Class, Error, Limits, Reject, Report, Route};
use std::sync::{atomic::AtomicBool, Once, OnceLock};

// Independent Python struct.pack('<I', len(field)) framing, not Rust encode().
// Public golden generation and hashes are in research/milestones/reproduce.py.
const GOLDEN_NODE_INDEX2: &str = "26000000506f55542d72657365617263682f4444412d45506f522d65786163742d696e74656e742f76320900000065706f722d6e6f64651600000073796e7468657469632d6d31392d696e74656e742d3020000000313131313131313131313131313131313131313131313131313131313131313120000000424242424242424242424242424242424242424242424242424242424242424204000000020000004c00000002000000000000008000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000";
const GOLDEN_RANK_FULL_VECTOR: &str = "22000000506f55542d72657365617263682f65786163742d696e74656e742d72616e6b2f76318c01000027000000506f55542d72657365617263682f76657269666965642d636f6d6d6f6e2d7665727465782f76329900000026000000506f55542d72657365617263682f4444412d45506f522d65786163742d696e74656e742f76320900000065706f722d6e6f64651600000073796e7468657469632d6d31392d696e74656e742d30200000003131313131313131313131313131313131313131313131313131313131313131200000004242424242424242424242424242424242424242424242424242424242424242c0000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000";

fn golden_bytes<const N: usize>(hex: &str) -> [u8; N] {
    assert_eq!(hex.len(), N * 2);
    let mut output = [0; N];
    for (slot, pair) in output.iter_mut().zip(hex.as_bytes().as_chunks::<2>().0) {
        *slot = u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap();
    }
    output
}

fn report() -> &'static Report {
    static REPORT: OnceLock<Result<Report, fixture_once::InitFailure<Error>>> = OnceLock::new();
    static OUTPUT: Once = Once::new();
    let report = fixture_once::get_or_init(&REPORT, || {
        experiment::run(Limits::default(), &AtomicBool::new(false))
    })
    .expect("one retained bounded causal result or sanitized initialization failure");
    OUTPUT.call_once(|| {
        let encoded = experiment::format_report(report).unwrap();
        assert!(encoded.len() <= experiment::MAX_REPORT_BYTES);
        println!("POUT_CAUSAL_BYTE_BRIDGE_REPORT {encoded}");
    });
    report
}

#[test]
fn complete_byte_and_typed_traces_match_with_full_global_accounting() {
    let node = golden_bytes::<241>(GOLDEN_NODE_INDEX2);
    let rank = golden_bytes::<438>(GOLDEN_RANK_FULL_VECTOR);
    assert_eq!(experiment::qualify_independent_golden(&node, &rank), Ok(()));
    let report = report();
    assert!(report.complete && report.every_ordered_event_compared);
    assert_eq!((report.n, report.targets, report.traces.len()), (6, 2, 8));
    assert!(report.total_payload_bytes <= experiment::MAX_PAYLOAD_BYTES);
    assert_eq!(report.native_hash_curve_snark_dag_cwc_calls, 0);
    assert_eq!(report.consensus_eligibility, "NotEstablished");
    assert_eq!(report.consensus_weight, None);
    assert!(report.scripted_replies_are_not_an_independent_uniform_law);
    assert!(report.parser_is_not_requested_target_authorization);
    assert!(report.global_charging_is_not_private_expenditure);
    for trace in &report.traces {
        assert!(trace.full_ordered_trace_equal && trace.expected_counters_equal);
        assert!(trace.steps.len() <= experiment::MAX_EVENTS);
        assert!(trace
            .steps
            .iter()
            .all(|row| row.full_key_reply_and_classification_equal));
        assert_eq!(trace.steps.len(), trace.work.requests);
        assert_eq!(
            trace.work.node_calls,
            trace.work.fresh_nodes + trace.work.cached_nodes
        );
        assert_eq!(
            trace.work.rank_calls,
            trace.work.fresh_ranks + trace.work.cached_ranks
        );
    }
    let first = &report.traces[0];
    assert_eq!((first.work.node_calls, first.work.rank_calls), (6, 1));
    assert_eq!(first.steps[6].before, Class::Complete);
}

#[test]
fn helper_and_generic_labels_cannot_hide_work_or_target_selection() {
    let report = report();
    let helper = &report.traces[1];
    assert_eq!(
        (
            helper.per_actor[0].node_calls,
            helper.per_actor[1].node_calls
        ),
        (6, 0)
    );
    assert_eq!(helper.work.node_calls, 6);
    assert!(helper
        .steps
        .iter()
        .all(|row| row.category == Category::GenericHash));
    assert!(helper.steps[..6]
        .iter()
        .all(|row| row.actual_route == Route::Node && row.actor == Actor::Helper));
    let selected = &report.traces[2];
    assert_eq!(selected.selected_target, Some(1));
    assert_eq!(
        (selected.work.node_calls, selected.per_actor[1].node_calls),
        (7, 5)
    );
    assert_eq!(selected.per_actor[0].node_calls, 2);
    assert_eq!(selected.steps.last().unwrap().before, Class::Complete);
}

#[test]
fn cached_rank_completion_keeps_first_missing_classification() {
    let trace = &report().traces[3];
    let first = &trace.steps[0];
    let repeated = &trace.steps[7];
    assert!(first.fresh);
    assert_eq!(first.before, Class::Missing { index: 0 });
    assert_eq!(repeated.before, Class::Complete);
    assert_eq!(repeated.first_before, Class::Missing { index: 0 });
    assert!(!repeated.fresh);
    assert_eq!((trace.work.fresh_ranks, trace.work.cached_ranks), (1, 1));
    assert_eq!(
        trace
            .steps
            .iter()
            .filter(|row| row.actual_route == Route::Rank
                && row.fresh
                && row.first_before == Class::Complete)
            .count(),
        0
    );
    assert_eq!(trace.current_complete_rank_occurrences, 1);
    assert_eq!(trace.fresh_rank_first_complete, 0);
}

#[test]
fn off_path_queries_and_full_vector_keys_remain_distinct() {
    let report = report();
    let off_path = &report.traces[4];
    assert_eq!(off_path.work.node_calls, 6);
    assert_eq!(off_path.steps[6].before, Class::Missing { index: 1 });
    let vectors = &report.traces[5];
    assert_eq!(
        (
            vectors.work.rank_calls,
            vectors.work.fresh_ranks,
            vectors.work.cached_ranks
        ),
        (2, 2, 0)
    );
    assert!(vectors
        .steps
        .iter()
        .all(|row| row.target == Some(1) && row.fresh));
    assert_eq!(vectors.steps[1].before, Class::Missing { index: 1 });
    assert_eq!(vectors.steps[2].before, Class::Mismatch { index: 0 });
}

#[test]
fn permitted_categories_share_one_global_cache_without_service_credit() {
    let trace = &report().traces[6];
    assert_eq!(
        (
            trace.work.node_calls,
            trace.work.fresh_nodes,
            trace.work.cached_nodes
        ),
        (12, 6, 6)
    );
    assert_eq!(
        (
            trace.work.rank_calls,
            trace.work.fresh_ranks,
            trace.work.cached_ranks
        ),
        (2, 1, 1)
    );
    assert!(trace.steps[..7]
        .iter()
        .all(|row| row.category == Category::Storage));
    assert!(trace.steps[7..]
        .iter()
        .all(|row| row.category == Category::Delivery && !row.fresh));
    assert_eq!(
        (trace.per_actor[0].requests, trace.per_actor[2].requests),
        (7, 7)
    );
    assert_eq!(trace.naive_category_distinct_keys, 14);
    assert_eq!(trace.actual_global_distinct_keys, 7);
}

#[test]
fn malformed_reserved_bytes_and_catalog_rejections_do_not_sample_oracles() {
    let trace = &report().traces[7];
    assert_eq!(
        (trace.work.requests, trace.work.rejected_before_sample),
        (9, 9)
    );
    assert_eq!((trace.work.node_calls, trace.work.rank_calls), (0, 0));
    let expected = [
        Reject::MalformedReservedNode,
        Reject::MalformedReservedNode,
        Reject::MalformedReservedNode,
        Reject::MalformedReservedNode,
        Reject::MalformedReservedRank,
        Reject::MalformedReservedRank,
        Reject::MalformedReservedRank,
        Reject::OutsideCatalog,
        Reject::UnsupportedOutsideDomain,
    ];
    for (step, expected) in trace.steps.iter().zip(expected) {
        assert_eq!(step.rejection, Some(expected));
        assert!(!step.fresh);
    }
    assert!(trace.steps[..7]
        .iter()
        .all(|row| row.category == Category::GenericHash && row.actual_route != Route::Other));
}

#[test]
fn finite_input_bounds_are_admitted_before_execution() {
    let limits = Limits::default();
    for field in 0..4 {
        for larger in [false, true] {
            let mut changed = limits;
            let slot = match field {
                0 => &mut changed.traces,
                1 => &mut changed.events_per_trace,
                2 => &mut changed.payload_bytes,
                _ => &mut changed.key_bytes,
            };
            if larger {
                *slot += 1;
            } else {
                *slot -= 1;
            }
            assert_eq!(
                experiment::admission(changed, &AtomicBool::new(false)),
                Err(Error::Bounds)
            );
        }
    }
}

#[test]
fn cancellation_is_local_before_corpus_initialization() {
    assert_eq!(
        experiment::admission(Limits::default(), &AtomicBool::new(true)),
        Err(Error::Cancelled)
    );
    assert!(matches!(
        experiment::run(Limits::default(), &AtomicBool::new(true)),
        Err(Error::Cancelled)
    ));
}
