//! Finite correctness evidence, not a proof over all candidate blocks.
//!
//! The oracle below uses independent wide-integer arithmetic and a deliberately
//! small linear map; it does not call either executor's operation evaluator.

use std::sync::atomic::AtomicBool;

use pout_cwc_research::{
    decode_block, decode_execution, decode_results, decode_state, dependency_plan, encode_block,
    encode_execution, encode_results, encode_state, execute_parallel, execute_serial, Block,
    Execution, Key, Operation, OperationResult, Outcome, State,
};

const WORKERS: [usize; 3] = [1, 2, 4];
const SEEDED_CASES: usize = 128;
const CORPUS_SEED: u64 = 0x4357_435f_3230_3236;

fn key(label: u16) -> Key {
    let mut bytes = [0_u8; 32];
    bytes[..2].copy_from_slice(&label.to_be_bytes());
    bytes
}

#[derive(Clone, Debug)]
enum Toy {
    Assert(u16, u64),
    Set(u16, u64),
    Add(u16, u64),
    Move(u16, u16, u64),
}

impl Toy {
    fn typed(&self) -> Operation {
        match *self {
            Self::Assert(k, value) => Operation::AssertEquals { key: key(k), value },
            Self::Set(k, value) => Operation::Set { key: key(k), value },
            Self::Add(k, amount) => Operation::Add {
                key: key(k),
                amount,
            },
            Self::Move(a, b, amount) => Operation::Move {
                source: key(a),
                destination: key(b),
                amount,
            },
        }
    }
}

#[derive(Debug)]
struct Expected {
    state: Vec<(Key, u64)>,
    results: Vec<OperationResult>,
}

fn oracle_state_bytes(expected: &Expected) -> Vec<u8> {
    // Written directly from the version-1 specification, without using the
    // production state constructor, encoder or root helper.
    let mut encoded = b"POUT/CWC/STATE\0\x01\x00".to_vec();
    encoded.extend_from_slice(&u16::try_from(expected.state.len()).unwrap().to_le_bytes());
    for (key, value) in &expected.state {
        encoded.extend_from_slice(key);
        encoded.extend_from_slice(&value.to_le_bytes());
    }
    encoded
}

fn oracle_result_bytes(expected: &Expected) -> Vec<u8> {
    let mut encoded = b"POUT/CWC/RESULTS\0\x01\x00".to_vec();
    encoded.extend_from_slice(&u16::try_from(expected.results.len()).unwrap().to_le_bytes());
    for result in &expected.results {
        encoded.extend_from_slice(&result.index.to_le_bytes());
        encoded.push(u8::try_from(result.values.len()).unwrap());
        for (key, value) in &result.values {
            encoded.extend_from_slice(key);
            encoded.extend_from_slice(&value.to_le_bytes());
        }
    }
    encoded
}

fn oracle_root(state_bytes: &[u8]) -> [u8; 32] {
    let mut preimage = b"POUT/CWC/STATE_ROOT\0\x01\x00".to_vec();
    preimage.extend_from_slice(&u32::try_from(state_bytes.len()).unwrap().to_le_bytes());
    preimage.extend_from_slice(state_bytes);
    *blake3::hash(&preimage).as_bytes()
}

fn reference(initial: &[(u16, u64)], operations: &[Toy]) -> Option<Expected> {
    let mut values = initial.to_vec();
    let mut results = Vec::new();
    for (index, op) in operations.iter().enumerate() {
        let lookup = |label| values.iter().position(|(k, _)| *k == label);
        let mut observed = match *op {
            Toy::Assert(k, expected) => {
                let position = lookup(k)?;
                if values[position].1 != expected {
                    return None;
                }
                vec![(key(k), expected)]
            }
            Toy::Set(k, value) => {
                let position = lookup(k)?;
                values[position].1 = value;
                vec![(key(k), value)]
            }
            Toy::Add(k, amount) => {
                let position = lookup(k)?;
                let sum = u128::from(values[position].1) + u128::from(amount);
                if sum > u128::from(u64::MAX) {
                    return None;
                }
                let sum = u64::try_from(sum).expect("oracle checked the upper bound");
                values[position].1 = sum;
                vec![(key(k), sum)]
            }
            Toy::Move(a, b, amount) => {
                if a == b {
                    return None;
                }
                let source = lookup(a)?;
                let destination = lookup(b)?;
                let debit = u128::from(amount);
                let old_source = u128::from(values[source].1);
                let credit = u128::from(values[destination].1) + debit;
                if debit > old_source || credit > u128::from(u64::MAX) {
                    return None;
                }
                let new_source = u64::try_from(old_source - debit).expect("bounded difference");
                let new_destination = u64::try_from(credit).expect("oracle checked upper bound");
                values[source].1 = new_source;
                values[destination].1 = new_destination;
                vec![(key(a), new_source), (key(b), new_destination)]
            }
        };
        observed.sort_unstable_by_key(|(k, _)| *k);
        results.push(OperationResult {
            index: u16::try_from(index).expect("bounded test case"),
            values: observed,
        });
    }
    let mut state: Vec<_> = values.into_iter().map(|(k, v)| (key(k), v)).collect();
    state.sort_unstable_by_key(|(k, _)| *k);
    Some(Expected { state, results })
}

fn check_candidate(initial: &[(u16, u64)], operations: &[Toy], label: &str) -> bool {
    let input = State::new(initial.iter().map(|(k, v)| (key(*k), *v)).collect())
        .expect("unique bounded initial state");
    let unchanged = input.clone();
    let block = Block::new(operations.iter().map(Toy::typed).collect()).expect("bounded corpus");
    let expected = reference(initial, operations);
    let serial = execute_serial(&input, &block);
    assert_eq!(input, unchanged, "serial mutated committed input: {label}");
    match (&serial, &expected) {
        (Outcome::Accepted(execution), Some(oracle)) => {
            assert_eq!(
                execution.final_state.entries(),
                oracle.state,
                "state {label}"
            );
            assert_eq!(
                execution.results, oracle.results,
                "operation results {label}"
            );
            let state_bytes = oracle_state_bytes(oracle);
            assert_eq!(
                encode_state(&execution.final_state),
                state_bytes,
                "canonical state bytes {label}"
            );
            assert_eq!(
                encode_results(&execution.results).unwrap(),
                oracle_result_bytes(oracle),
                "canonical result bytes {label}"
            );
            assert_eq!(
                execution.state_root,
                oracle_root(&state_bytes),
                "root {label}"
            );
        }
        (Outcome::Rejected, None) => {}
        _ => panic!("hand oracle disagreement {label}: {serial:?}; expected {expected:?}"),
    }
    for workers in WORKERS {
        let parallel = execute_parallel(&input, &block, workers, &AtomicBool::new(false));
        assert_eq!(parallel, serial, "workers={workers}, candidate={label}");
        assert_eq!(
            input, unchanged,
            "parallel mutated committed input: {label}"
        );
    }
    expected.is_some()
}

#[test]
fn hand_calculated_demo_and_all_operation_results_agree() {
    let initial = [(1, 10), (2, 20)];
    let operations = [Toy::Add(1, 5), Toy::Add(2, 7), Toy::Move(1, 2, 3)];
    assert!(check_candidate(&initial, &operations, "documented demo"));
    let oracle = reference(&initial, &operations).expect("hand-valid example");
    assert_eq!(oracle.state, vec![(key(1), 12), (key(2), 30)]);
    assert_eq!(
        oracle.results,
        vec![
            OperationResult {
                index: 0,
                values: vec![(key(1), 15)]
            },
            OperationResult {
                index: 1,
                values: vec![(key(2), 27)]
            },
            OperationResult {
                index: 2,
                values: vec![(key(1), 12), (key(2), 30)]
            },
        ]
    );
    let plan = dependency_plan(&Block::new(operations.iter().map(Toy::typed).collect()).unwrap());
    assert_eq!(plan.edges, vec![(0, 2), (1, 2)]);
    assert_eq!(plan.waves, vec![vec![0, 1], vec![2]]);
    // Wave completion order differs from canonical result order: op2 can run
    // beside op0 while op1 must wait for op0's write.
    let noncontiguous = [Toy::Set(1, 11), Toy::Assert(1, 11), Toy::Add(2, 1)];
    assert!(check_candidate(
        &initial,
        &noncontiguous,
        "noncontiguous waves"
    ));
    let plan =
        dependency_plan(&Block::new(noncontiguous.iter().map(Toy::typed).collect()).unwrap());
    assert_eq!(plan.waves, vec![vec![0, 2], vec![1]]);
}

#[test]
fn shared_reserve_conflicts_reject_without_publishing_either_credit() {
    let initial = [(0, 100), (1, 0), (2, 0)];
    let operations = [Toy::Move(0, 1, 80), Toy::Move(0, 2, 80)];
    assert!(!check_candidate(
        &initial,
        &operations,
        "reserve 100, two debits of 80"
    ));
    let plan = dependency_plan(&Block::new(operations.iter().map(Toy::typed).collect()).unwrap());
    assert_eq!(plan.edges, vec![(0, 1)]);
    assert_eq!(plan.waves, vec![vec![0], vec![1]]);
}

#[test]
fn deterministic_edge_and_arithmetic_cases_match_the_independent_oracle() {
    type HandCase = (&'static str, Vec<(u16, u64)>, Vec<Toy>);
    let cases: Vec<HandCase> = vec![
        ("empty", vec![], vec![]),
        (
            "independent",
            vec![(0, 1), (1, 2), (2, 3)],
            vec![Toy::Add(0, 4), Toy::Set(1, 8), Toy::Assert(2, 3)],
        ),
        (
            "hot key",
            vec![(0, 0)],
            (0..32).map(|_| Toy::Add(0, 1)).collect(),
        ),
        (
            "read after write",
            vec![(0, 1)],
            vec![Toy::Set(0, 2), Toy::Assert(0, 2)],
        ),
        (
            "write after read",
            vec![(0, 1)],
            vec![Toy::Assert(0, 1), Toy::Set(0, 2)],
        ),
        (
            "write after write",
            vec![(0, 1)],
            vec![Toy::Set(0, 2), Toy::Set(0, 3)],
        ),
        (
            "no-op repeated",
            vec![(0, 5)],
            vec![Toy::Set(0, 5), Toy::Add(0, 0), Toy::Assert(0, 5)],
        ),
        (
            "overlapping moves",
            vec![(0, 5), (1, 6), (2, 7)],
            vec![Toy::Move(0, 1, 3), Toy::Move(1, 2, 9), Toy::Move(2, 0, 4)],
        ),
        ("missing read", vec![(0, 5)], vec![Toy::Assert(9, 0)]),
        ("missing set", vec![(0, 5)], vec![Toy::Set(9, 0)]),
        ("missing add", vec![(0, 5)], vec![Toy::Add(9, 0)]),
        ("missing source", vec![(0, 5)], vec![Toy::Move(9, 0, 0)]),
        (
            "missing destination",
            vec![(0, 5)],
            vec![Toy::Move(0, 9, 0)],
        ),
        ("aliased move", vec![(0, 5)], vec![Toy::Move(0, 0, 0)]),
        ("underflow", vec![(0, 5), (1, 0)], vec![Toy::Move(0, 1, 6)]),
        ("add overflow", vec![(0, u64::MAX)], vec![Toy::Add(0, 1)]),
        (
            "destination overflow",
            vec![(0, 1), (1, u64::MAX)],
            vec![Toy::Move(0, 1, 1)],
        ),
        (
            "maximum no-op",
            vec![(0, u64::MAX)],
            vec![Toy::Add(0, 0), Toy::Assert(0, u64::MAX)],
        ),
        ("assert rejection", vec![(0, 1)], vec![Toy::Assert(0, 2)]),
        (
            "early failure later independent success",
            vec![(0, 1), (1, 0)],
            vec![Toy::Assert(0, 2), Toy::Set(1, 99)],
        ),
        (
            "later failure discards earlier writes",
            vec![(0, 1), (1, 0)],
            vec![Toy::Set(1, 99), Toy::Assert(0, 2)],
        ),
        (
            "reverse lexical move",
            vec![(2, 5), (0, 1)],
            vec![Toy::Move(2, 0, 3)],
        ),
    ];
    for (label, initial, operations) in &cases {
        check_candidate(initial, operations, label);
    }
    println!(
        "hand cases={}, serial paths={}, parallel worker paths={}",
        cases.len(),
        cases.len(),
        cases.len() * WORKERS.len()
    );
}

#[test]
fn derived_edges_include_read_write_write_write_and_write_after_read() {
    let cases = [
        (Toy::Set(0, 1), Toy::Assert(0, 1)),
        (Toy::Set(0, 1), Toy::Set(0, 2)),
        (Toy::Assert(0, 1), Toy::Set(0, 2)),
        (Toy::Move(0, 1, 0), Toy::Move(2, 0, 0)),
    ];
    for (first, second) in cases {
        let block = Block::new(vec![first.typed(), second.typed()]).unwrap();
        assert_eq!(dependency_plan(&block).edges, vec![(0, 1)]);
    }
    let block = Block::new(vec![Toy::Assert(0, 1).typed(), Toy::Assert(0, 1).typed()]).unwrap();
    assert!(dependency_plan(&block).edges.is_empty());
    assert_eq!(dependency_plan(&block).waves, vec![vec![0, 1]]);
}

#[test]
fn exhaustive_two_key_length_three_corpus_matches_hand_oracle() {
    // Nine states: A and B independently take values 0, 1, 2. Eight operations
    // from this fixed alphabet are composed at lengths 0, 1, 2, 3. This is a
    // finite enumeration, not a proof for arbitrary state or candidate input.
    let alphabet = [
        Toy::Assert(0, 0),
        Toy::Set(0, 1),
        Toy::Add(0, 1),
        Toy::Add(0, u64::MAX),
        Toy::Move(0, 1, 1),
        Toy::Move(1, 0, 1),
        Toy::Set(1, 0),
        Toy::Assert(1, 2),
    ];
    let mut candidates = 0;
    let mut accepted = 0;
    let mut historical_candidates = 0;
    let mut historical_accepted = 0;
    for a in 0..=2 {
        for b in 0..=2 {
            let initial = [(0, a), (1, b)];
            let mut exercise = |ops: &[Toy]| {
                let label = format!("finite corpus initial={initial:?}, ops={ops:?}");
                let valid = usize::from(check_candidate(&initial, ops, &label));
                accepted += valid;
                candidates += 1;
                if ops.len() <= 2 {
                    historical_candidates += 1;
                    historical_accepted += valid;
                }
            };
            exercise(&[]);
            for first in &alphabet {
                exercise(std::slice::from_ref(first));
                for second in &alphabet {
                    exercise(&[first.clone(), second.clone()]);
                    for third in &alphabet {
                        exercise(&[first.clone(), second.clone(), third.clone()]);
                    }
                }
            }
        }
    }
    // The first prototype's exact short corpus is still included unchanged.
    assert_eq!(historical_candidates, 657);
    assert_eq!(historical_accepted, 295);
    assert_eq!(candidates, 5265);
    assert_eq!(
        accepted, 1458,
        "independent Python integer enumeration crosscheck"
    );
    println!(
        "exhaustive candidates={candidates}, accepted={accepted}, rejected={}, triples=4608, historical length0..2 candidates={historical_candidates}, historical accepted={historical_accepted}, serial paths={candidates}, parallel paths={} (workers 1,2,4)",
        candidates - accepted,
        candidates * WORKERS.len()
    );
}

#[test]
fn three_operation_three_key_dependencies_and_failures_match_independent_oracle() {
    // Labels 0, 1, 2 are A, B, C. These fixed hand cases complement the
    // two-key enumeration: three disjoint accesses require three actual keys.
    // Edges are hand-written, not obtained from either executor or a planner.
    type TripleCase = (
        &'static str,
        [u64; 3],
        [Toy; 3],
        bool,
        &'static [(u16, u16)],
    );
    let cases: Vec<TripleCase> = vec![
        (
            "three independent operations",
            [5, 6, 7],
            [Toy::Add(0, 1), Toy::Set(1, 8), Toy::Assert(2, 7)],
            true,
            &[],
        ),
        (
            "three operations on one hot key",
            [5, 6, 7],
            [Toy::Add(0, 1), Toy::Set(0, 8), Toy::Assert(0, 8)],
            true,
            &[(0, 1), (0, 2), (1, 2)],
        ),
        (
            "join: independent writes followed by overlapping Move",
            [5, 6, 7],
            [Toy::Add(0, 1), Toy::Add(1, 1), Toy::Move(0, 1, 2)],
            true,
            &[(0, 2), (1, 2)],
        ),
        (
            "fork: Move followed by independent reads of its outputs",
            [5, 6, 7],
            [Toy::Move(0, 1, 3), Toy::Assert(0, 2), Toy::Assert(1, 9)],
            true,
            &[(0, 1), (0, 2)],
        ),
        (
            "read after write with independent third operation",
            [5, 6, 7],
            [Toy::Set(0, 9), Toy::Assert(0, 9), Toy::Add(2, 1)],
            true,
            &[(0, 1)],
        ),
        (
            "write after read with independent third operation",
            [5, 6, 7],
            [Toy::Assert(0, 5), Toy::Set(0, 9), Toy::Add(2, 1)],
            true,
            &[(0, 1)],
        ),
        (
            "write after write with independent third operation",
            [5, 6, 7],
            [Toy::Set(0, 9), Toy::Add(0, 1), Toy::Add(2, 1)],
            true,
            &[(0, 1)],
        ),
        (
            "shared reserve cannot fund two credits of 80",
            [100, 0, 0],
            [Toy::Move(0, 1, 80), Toy::Move(0, 2, 80), Toy::Assert(1, 80)],
            false,
            &[(0, 1), (0, 2)],
        ),
        (
            "shared reserve with sufficient funds",
            [100, 0, 0],
            [Toy::Move(0, 1, 40), Toy::Move(0, 2, 50), Toy::Assert(0, 10)],
            true,
            &[(0, 1), (0, 2), (1, 2)],
        ),
        (
            "three overlapping Moves",
            [5, 6, 7],
            [Toy::Move(0, 1, 3), Toy::Move(1, 2, 9), Toy::Move(2, 0, 4)],
            true,
            &[(0, 1), (0, 2), (1, 2)],
        ),
        (
            "failed assertion before two otherwise valid writes",
            [5, 6, 7],
            [Toy::Assert(0, 9), Toy::Add(1, 1), Toy::Set(2, 8)],
            false,
            &[],
        ),
        (
            "failed assertion between otherwise valid writes",
            [5, 6, 7],
            [Toy::Set(1, 8), Toy::Assert(0, 9), Toy::Add(2, 1)],
            false,
            &[],
        ),
        (
            "failed final assertion discards two otherwise valid writes",
            [5, 6, 7],
            [Toy::Set(1, 8), Toy::Add(2, 1), Toy::Assert(0, 9)],
            false,
            &[],
        ),
        (
            "add overflow with independent otherwise valid operations",
            [u64::MAX, 6, 7],
            [Toy::Add(0, 1), Toy::Set(1, 8), Toy::Assert(2, 7)],
            false,
            &[],
        ),
        (
            "Move destination overflow with later valid dependency",
            [5, u64::MAX, 7],
            [Toy::Move(0, 1, 1), Toy::Set(2, 8), Toy::Assert(2, 8)],
            false,
            &[(1, 2)],
        ),
        (
            "Move underflow with later valid dependency",
            [1, 6, 7],
            [Toy::Move(0, 1, 2), Toy::Set(2, 8), Toy::Assert(2, 8)],
            false,
            &[(1, 2)],
        ),
        (
            "missing-key assertion with later otherwise valid writes",
            [5, 6, 7],
            [Toy::Assert(9, 0), Toy::Add(0, 1), Toy::Set(1, 8)],
            false,
            &[],
        ),
    ];
    let mut accepted = 0;
    for (name, values, operations, valid, edges) in &cases {
        let initial = [(0, values[0]), (1, values[1]), (2, values[2])];
        let label = format!("{name}: initial={initial:?}, ops={operations:?}");
        let actual = check_candidate(&initial, operations, &label);
        assert_eq!(actual, *valid, "hand-classified validity: {label}");
        accepted += usize::from(actual);
        let block = Block::new(operations.iter().map(Toy::typed).collect()).unwrap();
        assert_eq!(dependency_plan(&block).edges, *edges, "edges: {label}");
    }
    assert_eq!(cases.len(), 17);
    assert_eq!(accepted, 9);
    println!(
        "three-key hand triples={}, accepted={accepted}, rejected={}, serial paths={}, parallel paths={} (workers 1,2,4)",
        cases.len(),
        cases.len() - accepted,
        cases.len(),
        cases.len() * WORKERS.len()
    );
}

fn next(seed: &mut u64) -> u64 {
    // Recorded deterministic generator; no execution-semantic randomness.
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

#[test]
fn fixed_seed_mixed_corpus_matches_hand_oracle() {
    let mut seed = CORPUS_SEED;
    let mut accepted = 0;
    let mut operation_count = 0;
    for case in 0..SEEDED_CASES {
        let mut initial = Vec::new();
        for label in 0..4 {
            let raw = next(&mut seed);
            initial.push((
                label,
                if raw.is_multiple_of(19) {
                    u64::MAX
                } else {
                    raw % 9
                },
            ));
        }
        let count = usize::try_from(next(&mut seed) % 17).unwrap();
        let mut operations = Vec::new();
        for _ in 0..count {
            let opcode = next(&mut seed) % 4;
            let label = u16::try_from(next(&mut seed) % 5).unwrap();
            let destination = u16::try_from(next(&mut seed) % 5).unwrap();
            let raw = next(&mut seed);
            let value = if raw.is_multiple_of(17) {
                u64::MAX
            } else {
                raw % 12
            };
            operations.push(match opcode {
                0 => Toy::Assert(label, value),
                1 => Toy::Set(label, value),
                2 => Toy::Add(label, value),
                _ => Toy::Move(label, destination, value),
            });
        }
        operation_count += operations.len();
        accepted += usize::from(check_candidate(
            &initial,
            &operations,
            &format!("seeded case {case}"),
        ));
    }
    assert_eq!(operation_count, 1066);
    assert_eq!(accepted, 8, "independent Python integer oracle crosscheck");
    println!(
        "seed={CORPUS_SEED:#018x}, candidates={SEEDED_CASES}, operations={operation_count}, accepted={accepted}, rejected={}, serial paths={SEEDED_CASES}, parallel paths={} (workers 1,2,4)",
        SEEDED_CASES - accepted,
        SEEDED_CASES * WORKERS.len()
    );
}

#[test]
fn negative_control_omitting_shared_reserve_conflict_would_false_accept() {
    let initial = [(0, 100), (1, 0), (2, 0)];
    let operations = [Toy::Move(0, 1, 80), Toy::Move(0, 2, 80)];
    // Deliberately incorrect scheduler: both debits see the same snapshot.
    // Combining their apparently successful outputs would mint a toy credit.
    let first = reference(&initial, &operations[..1]).unwrap();
    let second = reference(&initial, &operations[1..]).unwrap();
    assert_eq!(first.state, vec![(key(0), 20), (key(1), 80), (key(2), 0)]);
    assert_eq!(second.state, vec![(key(0), 20), (key(1), 0), (key(2), 80)]);
    let incorrectly_merged = [(key(0), 20), (key(1), 80), (key(2), 80)];
    assert_eq!(
        incorrectly_merged
            .iter()
            .map(|(_, value)| value)
            .sum::<u64>(),
        180
    );
    assert!(reference(&initial, &operations).is_none());
    assert!(!check_candidate(
        &initial,
        &operations,
        "negative control double-consumed reserve"
    ));
}

fn accepted(outcome: Outcome) -> Execution {
    match outcome {
        Outcome::Accepted(execution) => execution,
        other => panic!("expected accepted candidate, got {other:?}"),
    }
}

#[test]
fn canonical_results_are_in_block_order_and_move_values_in_key_order() {
    let initial = State::new(vec![(key(9), 10), (key(2), 4)]).unwrap();
    let block = Block::new(vec![Toy::Move(9, 2, 3).typed(), Toy::Assert(9, 7).typed()]).unwrap();
    let serial = accepted(execute_serial(&initial, &block));
    assert_eq!(
        serial.results,
        vec![
            OperationResult {
                index: 0,
                values: vec![(key(2), 7), (key(9), 7)]
            },
            OperationResult {
                index: 1,
                values: vec![(key(9), 7)]
            },
        ]
    );
    for workers in WORKERS {
        assert_eq!(
            accepted(execute_parallel(
                &initial,
                &block,
                workers,
                &AtomicBool::new(false)
            )),
            serial
        );
    }
}

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).expect("literal hexadecimal");
            let low = (pair[1] as char).to_digit(16).expect("literal hexadecimal");
            u8::try_from((high << 4) | low).unwrap()
        })
        .collect()
}

#[test]
fn independent_known_bytes_and_state_root_for_hand_calculated_demo() {
    // Literal vectors assembled independently with Python struct '<HH', '<Q'
    // and the written domains, not by serializing the Rust model under test.
    let state_bytes = unhex(concat!(
        "504f55542f4357432f53544154450001000200",
        "0001000000000000000000000000000000000000000000000000000000000000",
        "0c00000000000000",
        "0002000000000000000000000000000000000000000000000000000000000000",
        "1e00000000000000",
    ));
    let block_bytes = unhex(concat!(
        "504f55542f4357432f424c4f434b0001000300",
        "02",
        "0001000000000000000000000000000000000000000000000000000000000000",
        "0500000000000000",
        "02",
        "0002000000000000000000000000000000000000000000000000000000000000",
        "0700000000000000",
        "03",
        "0001000000000000000000000000000000000000000000000000000000000000",
        "0002000000000000000000000000000000000000000000000000000000000000",
        "0300000000000000",
    ));
    let result_bytes = unhex(concat!(
        "504f55542f4357432f524553554c54530001000300",
        "000001",
        "0001000000000000000000000000000000000000000000000000000000000000",
        "0f00000000000000",
        "010001",
        "0002000000000000000000000000000000000000000000000000000000000000",
        "1b00000000000000",
        "020002",
        "0001000000000000000000000000000000000000000000000000000000000000",
        "0c00000000000000",
        "0002000000000000000000000000000000000000000000000000000000000000",
        "1e00000000000000",
    ));
    assert_eq!(state_bytes.len(), 99);
    assert_eq!(block_bytes.len(), 174);
    assert_eq!(result_bytes.len(), 190);
    let initial = State::new(vec![(key(2), 20), (key(1), 10)]).unwrap();
    let block = Block::new(vec![
        Toy::Add(1, 5).typed(),
        Toy::Add(2, 7).typed(),
        Toy::Move(1, 2, 3).typed(),
    ])
    .unwrap();
    assert_eq!(encode_block(&block), block_bytes);
    assert_eq!(decode_block(&block_bytes).unwrap(), block);
    let serial = accepted(execute_serial(&initial, &block));
    assert_eq!(encode_state(&serial.final_state), state_bytes);
    assert_eq!(decode_state(&state_bytes).unwrap(), serial.final_state);
    assert_eq!(encode_results(&serial.results).unwrap(), result_bytes);
    assert_eq!(decode_results(&result_bytes).unwrap(), serial.results);
    // Fixed root checked by a standalone direct-BLAKE3 program over the literal
    // preimage (no CWC encoder/evaluator linked); see cwc-prototype-vectors.json.
    let mut preimage = b"POUT/CWC/STATE_ROOT\0".to_vec();
    preimage.extend_from_slice(&1_u16.to_le_bytes());
    preimage.extend_from_slice(&99_u32.to_le_bytes());
    preimage.extend_from_slice(&state_bytes);
    assert_eq!(preimage.len(), 125);
    assert_eq!(serial.state_root, *blake3::hash(&preimage).as_bytes());
    assert_eq!(
        serial.state_root.as_slice(),
        unhex("8a08205899bde1c3126dbabf3d60153b216ecc844c070ede9f44f305719adb67")
    );
    let encoded_execution = encode_execution(&serial).unwrap();
    assert_eq!(decode_execution(&encoded_execution).unwrap(), serial);
    for workers in WORKERS {
        let parallel = accepted(execute_parallel(
            &initial,
            &block,
            workers,
            &AtomicBool::new(false),
        ));
        assert_eq!(encode_execution(&parallel).unwrap(), encoded_execution);
    }
}

#[test]
fn noncanonical_malformed_truncated_and_trailing_encodings_are_rejected() {
    let initial = State::new(vec![(key(0), 10), (key(1), 20)]).unwrap();
    let block = Block::new(vec![Toy::Add(0, 5).typed(), Toy::Move(0, 1, 3).typed()]).unwrap();
    let execution = accepted(execute_serial(&initial, &block));
    let state_bytes = encode_state(&initial);
    let block_bytes = encode_block(&block);
    let results_bytes = encode_results(&execution.results).unwrap();
    let execution_bytes = encode_execution(&execution).unwrap();
    for end in 0..state_bytes.len() {
        assert!(
            decode_state(&state_bytes[..end]).is_err(),
            "state truncation {end}"
        );
    }
    for end in 0..block_bytes.len() {
        assert!(
            decode_block(&block_bytes[..end]).is_err(),
            "block truncation {end}"
        );
    }
    for end in 0..results_bytes.len() {
        assert!(
            decode_results(&results_bytes[..end]).is_err(),
            "results truncation {end}"
        );
    }
    for end in 0..execution_bytes.len() {
        assert!(
            decode_execution(&execution_bytes[..end]).is_err(),
            "execution truncation {end}"
        );
    }
    let mut bad = state_bytes.clone();
    bad.push(0);
    assert!(decode_state(&bad).is_err());
    let mut bad = block_bytes.clone();
    bad.push(0);
    assert!(decode_block(&bad).is_err());
    let mut bad = results_bytes.clone();
    bad.push(0);
    assert!(decode_results(&bad).is_err());
    let mut bad = execution_bytes.clone();
    bad.push(0);
    assert!(decode_execution(&bad).is_err());
    let mut bad = state_bytes.clone();
    let domain_size = b"POUT/CWC/STATE\0".len();
    bad[domain_size..domain_size + 2].copy_from_slice(&2_u16.to_le_bytes());
    assert!(decode_state(&bad).is_err());
    let mut bad = block_bytes.clone();
    let header = b"POUT/CWC/BLOCK\0".len();
    bad[header..header + 2].copy_from_slice(&2_u16.to_le_bytes());
    assert!(decode_block(&bad).is_err());
    let mut bad = results_bytes;
    let header = b"POUT/CWC/RESULTS\0".len();
    bad[header..header + 2].copy_from_slice(&2_u16.to_le_bytes());
    assert!(decode_results(&bad).is_err());
    let mut bad = execution_bytes.clone();
    let header = b"POUT/CWC/EXECUTION\0".len();
    bad[header..header + 2].copy_from_slice(&2_u16.to_le_bytes());
    assert!(decode_execution(&bad).is_err());
    let mut bad = execution_bytes.clone();
    bad[header + 2..header + 6].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(
        decode_execution(&bad).is_err(),
        "untrusted inner-state length"
    );
    let mut bad = block_bytes;
    bad[b"POUT/CWC/BLOCK\0".len() + 4] = 255;
    assert!(decode_block(&bad).is_err());
    let header = b"POUT/CWC/STATE\0".len() + 4;
    let mut bad = state_bytes.clone();
    let first = bad[header..header + 40].to_vec();
    bad[header + 40..header + 80].copy_from_slice(&first);
    assert!(
        decode_state(&bad).is_err(),
        "duplicate serialized initial keys"
    );
    let mut bad = state_bytes;
    bad[header..header + 80].rotate_left(40);
    assert!(
        decode_state(&bad).is_err(),
        "unsorted serialized initial keys"
    );
    let mut bad = execution_bytes;
    *bad.last_mut().unwrap() ^= 1;
    assert!(
        decode_execution(&bad).is_err(),
        "root does not bind embedded state"
    );
    let mut invalid_execution = execution;
    invalid_execution.results[1].index = 0;
    assert!(
        encode_execution(&invalid_execution).is_err(),
        "result indices must be canonical"
    );
}

#[test]
fn finite_model_limits_accept_boundary_and_reject_excess_before_execution() {
    let entries: Vec<_> = (0_u16..1024)
        .map(|index| (key(index), u64::from(index)))
        .collect();
    let state = State::new(entries.clone()).unwrap();
    assert_eq!(decode_state(&encode_state(&state)).unwrap(), state);
    let mut excessive = entries;
    excessive.push((key(1024), 0));
    assert!(State::new(excessive).is_err());
    assert!(State::new(vec![(key(0), 1), (key(0), 2)]).is_err());
    // Two-key operations attain this model's largest typed block and result,
    // while the full initial map attains its largest state encoding.
    let operations = vec![Toy::Move(0, 1, 0).typed(); 256];
    let block = Block::new(operations.clone()).unwrap();
    let serial = execute_serial(&state, &block);
    assert!(matches!(serial, Outcome::Accepted(_)));
    let execution = match &serial {
        Outcome::Accepted(execution) => execution,
        _ => unreachable!("acceptance asserted above"),
    };
    assert_eq!(encode_state(&execution.final_state).len(), 40_979);
    assert_eq!(encode_block(&block).len(), 18_707);
    assert_eq!(encode_results(&execution.results).unwrap().len(), 21_269);
    let encoded = encode_execution(execution).unwrap();
    assert_eq!(encoded.len(), 62_309);
    assert_eq!(decode_execution(&encoded).unwrap(), *execution);
    for workers in WORKERS {
        assert_eq!(
            execute_parallel(&state, &block, workers, &AtomicBool::new(false)),
            serial
        );
    }
    let mut excessive = operations;
    excessive.push(Toy::Add(0, 0).typed());
    assert!(Block::new(excessive).is_err());
    assert!(decode_block(&vec![0; 65537]).is_err());
    assert!(decode_execution(&vec![0; 65537]).is_err());
    let mut claimed = b"POUT/CWC/BLOCK\0".to_vec();
    claimed.extend_from_slice(&1_u16.to_le_bytes());
    claimed.extend_from_slice(&257_u16.to_le_bytes());
    assert!(decode_block(&claimed).is_err());
    let mut claimed = b"POUT/CWC/STATE\0".to_vec();
    claimed.extend_from_slice(&1_u16.to_le_bytes());
    claimed.extend_from_slice(&1025_u16.to_le_bytes());
    assert!(decode_state(&claimed).is_err());
    let mut claimed = b"POUT/CWC/RESULTS\0".to_vec();
    claimed.extend_from_slice(&1_u16.to_le_bytes());
    claimed.extend_from_slice(&257_u16.to_le_bytes());
    assert!(decode_results(&claimed).is_err());
}
