//! Synthetic, in-memory demonstration. No chain, assets or network calls.
use pout_cwc_research::{
    dependency_plan, execute_parallel, execute_serial, Block, Operation, Outcome, State,
};
use std::sync::atomic::AtomicBool;

fn key(label: u8) -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[1] = label;
    bytes
}

fn accepted(outcome: Outcome) -> pout_cwc_research::Execution {
    match outcome {
        Outcome::Accepted(result) => result,
        other => panic!("demo expected acceptance, got {other:?}"),
    }
}

fn main() {
    let (a, b) = (key(1), key(2));
    let initial = State::new(vec![(a, 10), (b, 20)]).expect("bounded synthetic state");
    let block = Block::new(vec![
        Operation::Add { key: a, amount: 5 },
        Operation::Add { key: b, amount: 7 },
        Operation::Move {
            source: a,
            destination: b,
            amount: 3,
        },
    ])
    .expect("bounded synthetic block");
    let cancel = AtomicBool::new(false);
    let plan = dependency_plan(&block);
    println!("Isolated CWC research demo: artificial counters, in-memory only");
    println!("Initial: A=10 B=20");
    println!("Operations: 0 Add(A,5); 1 Add(B,7); 2 Move(A,B,3)");
    println!(
        "Derived dependencies: {:?}; waves: {:?}",
        plan.edges, plan.waves
    );
    let serial = accepted(execute_serial(&initial, &block));
    for workers in [1, 2, 4] {
        let parallel = accepted(execute_parallel(&initial, &block, workers, &cancel));
        assert_eq!(serial, parallel);
        assert_eq!(parallel.final_state.get(&a), Some(12));
        assert_eq!(parallel.final_state.get(&b), Some(30));
        println!("Workers={workers}: serial A=12 B=30; parallel A=12 B=30; results/state/root equal=true");
    }
    let root: String = serial
        .state_root
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("Final state root: {root}");
    let c = key(6);
    let complex_initial = State::new(vec![(a, 10), (b, 20), (c, 30)]).unwrap();
    let complex_before = complex_initial.clone();
    let complex = Block::new(vec![
        Operation::Add { key: a, amount: 5 },
        Operation::Add { key: c, amount: 2 },
        Operation::Move {
            source: a,
            destination: b,
            amount: 3,
        },
        Operation::AssertEquals { key: c, value: 32 },
        Operation::Move {
            source: b,
            destination: c,
            amount: 4,
        },
    ])
    .unwrap();
    let complex_plan = dependency_plan(&complex);
    println!("Extended initial: A=10 B=20 C=30");
    println!("Extended operations: 0 Add(A,5); 1 Add(C,2); 2 Move(A,B,3); 3 AssertEquals(C,32); 4 Move(B,C,4)");
    println!(
        "Extended dependencies: {:?}; waves: {:?}",
        complex_plan.edges, complex_plan.waves
    );
    assert_eq!(complex_plan.waves, vec![vec![0, 1], vec![2, 3], vec![4]]);
    let complex_serial = accepted(execute_serial(&complex_initial, &complex));
    for workers in [1, 2, 4] {
        let parallel = accepted(execute_parallel(
            &complex_initial,
            &complex,
            workers,
            &cancel,
        ));
        assert_eq!(complex_serial, parallel);
        let values = [a, b, c].map(|key| parallel.final_state.get(&key).unwrap());
        assert_eq!(values, [12, 19, 36]);
        println!("Extended workers={workers}: serial={:?}; parallel={values:?}; results/state/root equal={}", [a,b,c].map(|key| complex_serial.final_state.get(&key).unwrap()), complex_serial == parallel);
    }
    assert_eq!(complex_initial, complex_before);
    let (reserve, recipient_a, recipient_b) = (key(3), key(4), key(5));
    let reserved = State::new(vec![(reserve, 100), (recipient_a, 0), (recipient_b, 0)]).unwrap();
    let before = reserved.clone();
    let invalid = Block::new(vec![
        Operation::Move {
            source: reserve,
            destination: recipient_a,
            amount: 80,
        },
        Operation::Move {
            source: reserve,
            destination: recipient_b,
            amount: 80,
        },
    ])
    .unwrap();
    assert_eq!(execute_serial(&reserved, &invalid), Outcome::Rejected);
    assert_eq!(
        execute_parallel(&reserved, &invalid, 2, &cancel),
        Outcome::Rejected
    );
    assert_eq!(reserved, before);
    println!("Shared reserve: 100; moves 80+80: both Rejected; original reserve=100, recipients=0,0 unchanged");
    let cancelled = AtomicBool::new(true);
    assert!(matches!(
        execute_parallel(&initial, &block, 2, &cancelled),
        Outcome::Aborted(_)
    ));
    assert_eq!(initial.get(&a), Some(10));
    assert_eq!(initial.get(&b), Some(20));
    println!("Pre-cancelled attempt: Aborted; original A=10 B=20 unchanged; retry accepted");
    assert_eq!(
        accepted(execute_parallel(&initial, &block, 2, &cancel)),
        serial
    );
    println!("This tested executor does not establish consensus security or a speedup.");
}
