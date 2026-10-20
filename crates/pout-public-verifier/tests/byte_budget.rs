//! Concurrent accounting regression definitions; no physical-link inference.
#[allow(dead_code)]
#[path = "../examples/support/retrieval_observation.rs"]
mod retrieval;
#[allow(dead_code)]
#[path = "../examples/support/byte_budget.rs"]
mod budget;
use budget::{Meter, MeterError, FRAME_BYTES, WINDOW_NANOS};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Barrier,
    },
};
fn no() -> AtomicBool {
    AtomicBool::new(false)
}
#[test]
fn concurrent_full_frame_reservation_is_atomic() {
    let meter = Meter::new(FRAME_BYTES, 0, WINDOW_NANOS).unwrap();
    let gate = Arc::new(Barrier::new(3));
    let handles: Vec<_> = (0..2).map(|_| {
        let meter = Arc::clone(&meter);
        let gate = Arc::clone(&gate);
        std::thread::spawn(move || {
            gate.wait();
            meter.reserve(FRAME_BYTES, || 1, &no())
        })
    }).collect();
    gate.wait();
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|r| matches!(r, Err(MeterError::CapacityDenied))).count(), 1);
    assert_eq!(meter.snapshot().unwrap().reserved, FRAME_BYTES);
    drop(results);
    assert_eq!(meter.snapshot().unwrap().released, FRAME_BYTES);
}
#[test]
fn completed_writes_never_refill_same_meter() {
    let meter = Meter::new(8, 0, 20).unwrap();
    let mut lease = meter.reserve(8, || 1, &no()).unwrap();
    assert_eq!(lease.write_to(&mut Vec::new(), &[5; 8], || 2, &no()), Ok(8));
    lease.check_completion(|| 2, &no()).unwrap();
    drop(lease);
    assert!(matches!(meter.reserve(8, || 3, &no()), Err(MeterError::CapacityDenied)));
    assert_eq!((meter.snapshot().unwrap().spent, meter.snapshot().unwrap().reserved), (8, 0));
}
struct Prefix;
impl Write for Prefix {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> { Ok(bytes.len().min(3)) }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
#[test]
fn partial_write_releases_only_known_unwritten_remainder() {
    let meter = Meter::new(8, 0, 20).unwrap();
    let mut lease = meter.reserve(8, || 1, &no()).unwrap();
    assert_eq!(lease.write_to(&mut Prefix, &[1; 8], || 2, &no()), Ok(3));
    assert_eq!(lease.check_completion(|| 2, &no()), Err(MeterError::Incomplete));
    drop(lease);
    let state = meter.snapshot().unwrap();
    assert_eq!((state.spent, state.released, state.reserved), (3, 5, 0));
}
#[test]
fn expiration_is_rechecked_after_waiting_for_lock() {
    let meter = Meter::new(8, 10, 20).unwrap();
    let calls = AtomicU64::new(0);
    assert!(matches!(meter.reserve(8, || if calls.fetch_add(1, Ordering::SeqCst) == 0 { 19 } else { 20 }, &no()), Err(MeterError::Expired)));
    assert_eq!(meter.snapshot().unwrap().reserved, 0);
}
struct Late<'a>(&'a AtomicU64);
impl Write for Late<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> { self.0.store(20, Ordering::SeqCst); Ok(bytes.len()) }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
#[test]
fn successful_late_output_is_spent_but_not_timely() {
    let meter = Meter::new(8, 10, 20).unwrap();
    let clock = AtomicU64::new(12);
    let mut lease = meter.reserve(8, || 11, &no()).unwrap();
    assert_eq!(lease.write_to(&mut Late(&clock), &[1; 8], || clock.load(Ordering::SeqCst), &no()), Ok(8));
    assert_eq!(lease.check_completion(|| 20, &no()), Err(MeterError::Expired));
    assert_eq!(lease.written_bytes(), 8);
    drop(lease);
    assert_eq!(meter.snapshot().unwrap().spent, 8);
}
struct Failed;
impl Write for Failed {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> { Err(io::Error::other("synthetic write error")) }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
#[test]
fn ambiguous_error_is_charged_separately_from_verified_output() {
    let meter = Meter::new(8, 0, 20).unwrap();
    let mut lease = meter.reserve(8, || 1, &no()).unwrap();
    assert_eq!(lease.write_to(&mut Failed, &[1; 4], || 2, &no()), Err(MeterError::WriteFailure));
    assert_eq!((lease.written_bytes(), lease.uncertain_bytes()), (0, 4));
    drop(lease);
    let state = meter.snapshot().unwrap();
    assert_eq!((state.spent, state.uncertain_spent, state.released), (4, 4, 4));
}
struct Panics;
impl Write for Panics {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> { panic!("expected synthetic writer panic") }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
#[test]
fn unwinding_does_not_refund_uncertain_output() {
    let meter = Meter::new(8, 0, 20).unwrap();
    let owned = Arc::clone(&meter);
    let outcome = std::panic::catch_unwind(move || {
        let mut lease = owned.reserve(8, || 1, &no()).unwrap();
        let _ = lease.write_to(&mut Panics, &[1; 8], || 2, &no());
    });
    assert!(outcome.is_err());
    assert_eq!((meter.snapshot().unwrap().spent, meter.snapshot().unwrap().uncertain_spent), (8, 8));
}
struct Inspect(Arc<Meter>);
impl Write for Inspect {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        assert_eq!(self.0.snapshot().unwrap().reserved, 8);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
#[test]
fn io_does_not_hold_accounting_mutex() {
    let meter = Meter::new(8, 0, 20).unwrap();
    let mut lease = meter.reserve(8, || 1, &no()).unwrap();
    assert_eq!(lease.write_to(&mut Inspect(Arc::clone(&meter)), &[1; 8], || 2, &no()), Ok(8));
}
#[test]
fn cancelled_or_oversized_admission_grants_no_capacity() {
    assert!(Meter::new(0, 0, 1).is_err());
    assert!(Meter::new(2 * FRAME_BYTES + 1, 0, 1).is_err());
    assert!(Meter::new(8, 0, WINDOW_NANOS + 1).is_err());
    let meter = Meter::new(8, 0, 20).unwrap();
    assert!(matches!(meter.reserve(8, || 1, &AtomicBool::new(true)), Err(MeterError::Cancelled)));
    assert!(matches!(meter.reserve(FRAME_BYTES + 1, || 1, &no()), Err(MeterError::Bounds)));
    assert_eq!(meter.snapshot().unwrap().reserved, 0);
}
#[test]
fn assigned_second_meter_is_extra_allowance_not_independent_storage() {
    let first = Meter::new(8, 0, 20).unwrap();
    let second = Meter::new(8, 0, 20).unwrap();
    let same_data = [7; 8];
    for meter in [first, second] {
        let mut lease = meter.reserve(8, || 1, &no()).unwrap();
        let mut output = Vec::new();
        assert_eq!(lease.write_to(&mut output, &same_data, || 2, &no()), Ok(8));
        assert_eq!(output, same_data);
    }
}
