//! Explicit canonical research encoding, version 1. All integer fields are LE.
//!
//! This is not an operational network protocol. Input lengths/counts are bounded
//! before allocation. Result decoding checks syntax, ordering, membership and
//! state-root consistency, not execution against an absent original block.

use crate::model::{
    Block, Execution, InvalidInput, Key, Operation, OperationResult, State, MAX_BLOCK_BYTES,
    MAX_EXECUTION_BYTES, MAX_OPERATIONS, MAX_RESULTS_BYTES, MAX_STATE_BYTES, MAX_STATE_KEYS,
    VERSION,
};

pub const STATE_DOMAIN: &[u8] = b"POUT/CWC/STATE\0";
pub const BLOCK_DOMAIN: &[u8] = b"POUT/CWC/BLOCK\0";
pub const RESULTS_DOMAIN: &[u8] = b"POUT/CWC/RESULTS\0";
pub const EXECUTION_DOMAIN: &[u8] = b"POUT/CWC/EXECUTION\0";
pub const ROOT_DOMAIN: &[u8] = b"POUT/CWC/STATE_ROOT\0";

fn begin(domain: &[u8], count: usize) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(domain);
    output.extend_from_slice(&VERSION.to_le_bytes());
    // Every caller has checked the appropriate maximum <= 1,024.
    output.extend_from_slice(&(count as u16).to_le_bytes());
    output
}

fn put_value(output: &mut Vec<u8>, key: &Key, value: u64) {
    output.extend_from_slice(key);
    output.extend_from_slice(&value.to_le_bytes());
}

pub fn encode_state(state: &State) -> Vec<u8> {
    let mut output = begin(STATE_DOMAIN, state.entries().len());
    for (key, value) in state.entries() {
        put_value(&mut output, key, *value);
    }
    output
}

pub fn encode_block(block: &Block) -> Vec<u8> {
    let mut output = begin(BLOCK_DOMAIN, block.operations().len());
    for operation in block.operations() {
        match operation {
            Operation::AssertEquals { key, value } => {
                output.push(0);
                put_value(&mut output, key, *value);
            }
            Operation::Set { key, value } => {
                output.push(1);
                put_value(&mut output, key, *value);
            }
            Operation::Add { key, amount } => {
                output.push(2);
                put_value(&mut output, key, *amount);
            }
            Operation::Move {
                source,
                destination,
                amount,
            } => {
                output.push(3);
                output.extend_from_slice(source);
                put_value(&mut output, destination, *amount);
            }
        }
    }
    output
}

fn validate_results(results: &[OperationResult]) -> Result<(), InvalidInput> {
    if results.len() > MAX_OPERATIONS {
        return Err(InvalidInput::LimitExceeded);
    }
    for (index, result) in results.iter().enumerate() {
        if usize::from(result.index) != index || !(1..=2).contains(&result.values.len()) {
            return Err(InvalidInput::InvalidResult);
        }
        if result.values.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
            return Err(InvalidInput::NonCanonical);
        }
    }
    Ok(())
}

pub fn encode_results(results: &[OperationResult]) -> Result<Vec<u8>, InvalidInput> {
    validate_results(results)?;
    let mut output = begin(RESULTS_DOMAIN, results.len());
    for result in results {
        output.extend_from_slice(&result.index.to_le_bytes());
        output.push(result.values.len() as u8);
        for (key, value) in &result.values {
            put_value(&mut output, key, *value);
        }
    }
    Ok(output)
}

/// Domain binds the research schema/version, canonical length and state bytes.
pub fn state_root(state: &State) -> [u8; 32] {
    let encoded = encode_state(state);
    let mut hasher = blake3::Hasher::new();
    hasher.update(ROOT_DOMAIN);
    hasher.update(&VERSION.to_le_bytes());
    hasher.update(&(encoded.len() as u32).to_le_bytes());
    hasher.update(&encoded);
    *hasher.finalize().as_bytes()
}

pub fn encode_execution(execution: &Execution) -> Result<Vec<u8>, InvalidInput> {
    if execution.state_root != state_root(&execution.final_state) {
        return Err(InvalidInput::RootMismatch);
    }
    let state = encode_state(&execution.final_state);
    let results = encode_results(&execution.results)?;
    if execution.results.iter().any(|result| {
        result
            .values
            .iter()
            .any(|(key, _)| execution.final_state.get(key).is_none())
    }) {
        return Err(InvalidInput::InvalidResult);
    }
    let mut output = Vec::new();
    output.extend_from_slice(EXECUTION_DOMAIN);
    output.extend_from_slice(&VERSION.to_le_bytes());
    output.extend_from_slice(&(state.len() as u32).to_le_bytes());
    output.extend_from_slice(&state);
    output.extend_from_slice(&(results.len() as u32).to_le_bytes());
    output.extend_from_slice(&results);
    output.extend_from_slice(&execution.state_root);
    if output.len() > MAX_EXECUTION_BYTES {
        return Err(InvalidInput::LimitExceeded);
    }
    Ok(output)
}

struct Reader<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(input: &'a [u8], maximum: usize) -> Result<Self, InvalidInput> {
        if input.len() > maximum {
            return Err(InvalidInput::LimitExceeded);
        }
        Ok(Self { input, offset: 0 })
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], InvalidInput> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(InvalidInput::Malformed)?;
        let value = self
            .input
            .get(self.offset..end)
            .ok_or(InvalidInput::Malformed)?;
        self.offset = end;
        Ok(value)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], InvalidInput> {
        self.take(N)?
            .try_into()
            .map_err(|_| InvalidInput::Malformed)
    }

    fn u8(&mut self) -> Result<u8, InvalidInput> {
        Ok(self.array::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, InvalidInput> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, InvalidInput> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, InvalidInput> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn header(&mut self, domain: &[u8]) -> Result<(), InvalidInput> {
        if self.take(domain.len())? != domain {
            return Err(InvalidInput::Malformed);
        }
        if self.u16()? != VERSION {
            return Err(InvalidInput::UnsupportedVersion);
        }
        Ok(())
    }

    fn count(&mut self, maximum: usize) -> Result<usize, InvalidInput> {
        let count = usize::from(self.u16()?);
        if count > maximum {
            return Err(InvalidInput::LimitExceeded);
        }
        Ok(count)
    }

    fn finish(self) -> Result<(), InvalidInput> {
        if self.offset != self.input.len() {
            return Err(InvalidInput::NonCanonical);
        }
        Ok(())
    }
}

fn reserve<T>(count: usize) -> Result<Vec<T>, InvalidInput> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| InvalidInput::LocalResourceFailure)?;
    Ok(values)
}

pub fn decode_state(input: &[u8]) -> Result<State, InvalidInput> {
    let mut reader = Reader::new(input, MAX_STATE_BYTES)?;
    reader.header(STATE_DOMAIN)?;
    let count = reader.count(MAX_STATE_KEYS)?;
    // Require all fixed-size entries to exist before input-driven allocation.
    if reader.input.len() - reader.offset < count * 40 {
        return Err(InvalidInput::Malformed);
    }
    let mut entries = reserve(count)?;
    for _ in 0..count {
        let key: Key = reader.array()?;
        let value = reader.u64()?;
        if let Some((previous, _)) = entries.last() {
            if previous >= &key {
                return Err(InvalidInput::NonCanonical);
            }
        }
        entries.push((key, value));
    }
    reader.finish()?;
    State::new(entries)
}

pub fn decode_block(input: &[u8]) -> Result<Block, InvalidInput> {
    let mut reader = Reader::new(input, MAX_BLOCK_BYTES)?;
    reader.header(BLOCK_DOMAIN)?;
    let count = reader.count(MAX_OPERATIONS)?;
    // Every opcode needs at least 41 bytes; max count and byte bound are checked.
    if reader.input.len() - reader.offset < count * 41 {
        return Err(InvalidInput::Malformed);
    }
    let mut operations = reserve(count)?;
    for _ in 0..count {
        let operation = match reader.u8()? {
            0 => Operation::AssertEquals {
                key: reader.array()?,
                value: reader.u64()?,
            },
            1 => Operation::Set {
                key: reader.array()?,
                value: reader.u64()?,
            },
            2 => Operation::Add {
                key: reader.array()?,
                amount: reader.u64()?,
            },
            3 => Operation::Move {
                source: reader.array()?,
                destination: reader.array()?,
                amount: reader.u64()?,
            },
            _ => return Err(InvalidInput::Malformed),
        };
        operations.push(operation);
    }
    reader.finish()?;
    Block::new(operations)
}

pub fn decode_results(input: &[u8]) -> Result<Vec<OperationResult>, InvalidInput> {
    let mut reader = Reader::new(input, MAX_RESULTS_BYTES)?;
    reader.header(RESULTS_DOMAIN)?;
    let count = reader.count(MAX_OPERATIONS)?;
    if reader.input.len() - reader.offset < count * 43 {
        return Err(InvalidInput::Malformed);
    }
    let mut results = reserve(count)?;
    for expected in 0..count {
        let index = reader.u16()?;
        let values_count = usize::from(reader.u8()?);
        if usize::from(index) != expected || !(1..=2).contains(&values_count) {
            return Err(InvalidInput::InvalidResult);
        }
        if reader.input.len() - reader.offset < values_count * 40 {
            return Err(InvalidInput::Malformed);
        }
        let mut values = reserve(values_count)?;
        for _ in 0..values_count {
            let key: Key = reader.array()?;
            let value = reader.u64()?;
            if values.last().is_some_and(|(previous, _)| previous >= &key) {
                return Err(InvalidInput::NonCanonical);
            }
            values.push((key, value));
        }
        results.push(OperationResult { index, values });
    }
    reader.finish()?;
    Ok(results)
}

/// Decode a canonical result record and check its state-root consistency.
///
/// This does **not** validate a state transition or attest that the operations of
/// any candidate block were executed: neither the initial state nor that block
/// is present. Only executing the serial/parallel validator produces that result.
pub fn decode_execution(input: &[u8]) -> Result<Execution, InvalidInput> {
    let mut reader = Reader::new(input, MAX_EXECUTION_BYTES)?;
    reader.header(EXECUTION_DOMAIN)?;
    let state_length = usize::try_from(reader.u32()?).map_err(|_| InvalidInput::LimitExceeded)?;
    if state_length > MAX_STATE_BYTES {
        return Err(InvalidInput::LimitExceeded);
    }
    let final_state = decode_state(reader.take(state_length)?)?;
    let results_length = usize::try_from(reader.u32()?).map_err(|_| InvalidInput::LimitExceeded)?;
    if results_length > MAX_RESULTS_BYTES {
        return Err(InvalidInput::LimitExceeded);
    }
    let results = decode_results(reader.take(results_length)?)?;
    let state_root = reader.array()?;
    reader.finish()?;
    let execution = Execution {
        final_state,
        results,
        state_root,
    };
    // Validate supplied root and all result references. This is not re-execution.
    encode_execution(&execution)?;
    Ok(execution)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_encodings_have_literal_bytes_and_round_trip() {
        let state = State::new(Vec::new()).unwrap();
        let block = Block::new(Vec::new()).unwrap();
        assert_eq!(encode_state(&state), b"POUT/CWC/STATE\0\x01\x00\x00\x00");
        assert_eq!(encode_block(&block), b"POUT/CWC/BLOCK\0\x01\x00\x00\x00");
        assert_eq!(
            encode_results(&[]).unwrap(),
            b"POUT/CWC/RESULTS\0\x01\x00\x00\x00"
        );
        assert_eq!(decode_state(&encode_state(&state)), Ok(state));
        assert_eq!(decode_block(&encode_block(&block)), Ok(block));
    }

    #[test]
    fn huge_counts_are_rejected_before_payload_allocation() {
        let mut state = STATE_DOMAIN.to_vec();
        state.extend_from_slice(&VERSION.to_le_bytes());
        state.extend_from_slice(&u16::MAX.to_le_bytes());
        assert_eq!(decode_state(&state), Err(InvalidInput::LimitExceeded));
        let mut block = BLOCK_DOMAIN.to_vec();
        block.extend_from_slice(&VERSION.to_le_bytes());
        block.extend_from_slice(&u16::MAX.to_le_bytes());
        assert_eq!(decode_block(&block), Err(InvalidInput::LimitExceeded));
        let mut results = RESULTS_DOMAIN.to_vec();
        results.extend_from_slice(&VERSION.to_le_bytes());
        results.extend_from_slice(&u16::MAX.to_le_bytes());
        assert_eq!(decode_results(&results), Err(InvalidInput::LimitExceeded));
    }

    #[test]
    fn maximum_typed_sizes_fit_all_encoded_bounds() {
        let entries: Vec<_> = (0..MAX_STATE_KEYS)
            .map(|index| {
                let mut key = [0u8; 32];
                key[..2].copy_from_slice(&(index as u16).to_be_bytes());
                (key, u64::MAX)
            })
            .collect();
        let state = State::new(entries).unwrap();
        let operations = vec![
            Operation::Move {
                source: state.entries()[0].0,
                destination: state.entries()[1].0,
                amount: 0,
            };
            MAX_OPERATIONS
        ];
        let block = Block::new(operations).unwrap();
        let results: Vec<_> = (0..MAX_OPERATIONS)
            .map(|index| OperationResult {
                index: index as u16,
                values: state.entries()[..2].to_vec(),
            })
            .collect();
        let execution = Execution {
            state_root: state_root(&state),
            final_state: state.clone(),
            results,
        };
        assert_eq!(encode_state(&state).len(), 40_979);
        assert_eq!(encode_block(&block).len(), 18_707);
        assert_eq!(encode_results(&execution.results).unwrap().len(), 21_269);
        let encoded = encode_execution(&execution).unwrap();
        assert_eq!(encoded.len(), 62_309);
        assert_eq!(decode_execution(&encoded), Ok(execution));
    }
}
