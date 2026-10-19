//! Intentionally simple block-index serial reference; no dependency scheduler.

use crate::model::{
    evaluate_operation, finish_execution, validate_candidate, AbortReason, Block, Outcome, State,
};

pub fn execute_serial(initial: &State, block: &Block) -> Outcome {
    if validate_candidate(initial, block).is_err() {
        return Outcome::Rejected;
    }
    let mut candidate = match initial.try_clone() {
        Ok(candidate) => candidate,
        Err(reason) => return Outcome::Aborted(reason),
    };
    let mut results = Vec::new();
    if results.try_reserve_exact(block.operations().len()).is_err() {
        return Outcome::Aborted(AbortReason::LocalResourceFailure);
    }
    for (index, operation) in block.operations().iter().enumerate() {
        // The constructor enforces at most 256 operations.
        let evaluation = match evaluate_operation(&candidate, operation, index as u16) {
            Ok(evaluation) => evaluation,
            Err(_) => return Outcome::Rejected,
        };
        for (key, value) in &evaluation.writes {
            if candidate.set_existing(key, *value).is_err() {
                return Outcome::Rejected;
            }
        }
        results.push(evaluation.result);
    }
    Outcome::Accepted(finish_execution(candidate, results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Operation;

    #[test]
    fn result_values_include_assert_reads_and_canonical_move_writes() {
        let low = [1u8; 32];
        let high = [2u8; 32];
        let original = State::new(vec![(high, 10), (low, 20)]).unwrap();
        let block = Block::new(vec![
            Operation::AssertEquals {
                key: high,
                value: 10,
            },
            Operation::Move {
                source: high,
                destination: low,
                amount: 3,
            },
        ])
        .unwrap();
        let Outcome::Accepted(execution) = execute_serial(&original, &block) else {
            panic!("hand-calculated candidate must be accepted");
        };
        assert_eq!(execution.results[0].values, vec![(high, 10)]);
        assert_eq!(execution.results[1].values, vec![(low, 23), (high, 7)]);
        assert_eq!(execution.final_state.entries(), &[(low, 23), (high, 7)]);
        assert_eq!(original.entries(), &[(low, 20), (high, 10)]);
    }

    #[test]
    fn shared_reserve_overdraw_rejects_without_exposing_a_prefix() {
        let reserve = [0u8; 32];
        let a = [1u8; 32];
        let b = [2u8; 32];
        let original = State::new(vec![(reserve, 100), (a, 0), (b, 0)]).unwrap();
        let before = crate::encode_state(&original);
        let block = Block::new(vec![
            Operation::Move {
                source: reserve,
                destination: a,
                amount: 80,
            },
            Operation::Move {
                source: reserve,
                destination: b,
                amount: 80,
            },
        ])
        .unwrap();
        assert_eq!(execute_serial(&original, &block), Outcome::Rejected);
        assert_eq!(crate::encode_state(&original), before);
    }
}
