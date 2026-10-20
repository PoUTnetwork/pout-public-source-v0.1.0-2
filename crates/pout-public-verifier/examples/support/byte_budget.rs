//! Nonrefilling, caller-assigned byte windows, not measured physical bandwidth.
//!
//! Meter assignment and monotonic time are trusted inputs. All accepted output
//! must pass through the meter; arbitrary out-of-band writers are not contained.
use super::retrieval;
use std::{
    fmt,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

pub const FRAME_BYTES: u64 = (retrieval::HEADER_BYTES + retrieval::CHUNK_BYTES) as u64;
pub const WINDOW_NANOS: u64 = 60_000_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterError {
    Bounds,
    CapacityDenied,
    Expired,
    Cancelled,
    Accounting,
    Backend,
    WriteFailure,
    Incomplete,
}
impl fmt::Display for MeterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for MeterError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Snapshot {
    pub capacity: u64,
    pub reserved: u64,
    pub spent: u64,
    pub released: u64,
    pub uncertain_spent: u64,
    pub admitted: u64,
    pub capacity_denials: u64,
}
pub struct Meter {
    state: Mutex<Snapshot>,
    start: u64,
    expires: u64,
}
impl Meter {
    pub fn new(capacity: u64, start: u64, expires: u64) -> Result<Arc<Self>, MeterError> {
        if capacity == 0
            || capacity > 2 * FRAME_BYTES
            || expires <= start
            || expires.checked_sub(start).ok_or(MeterError::Bounds)? > WINDOW_NANOS
        {
            return Err(MeterError::Bounds);
        }
        Ok(Arc::new(Self {
            state: Mutex::new(Snapshot {
                capacity,
                reserved: 0,
                spent: 0,
                released: 0,
                uncertain_spent: 0,
                admitted: 0,
                capacity_denials: 0,
            }),
            start,
            expires,
        }))
    }
    fn check(&self, now: u64, cancel: &AtomicBool) -> Result<(), MeterError> {
        if cancel.load(Ordering::Acquire) {
            return Err(MeterError::Cancelled);
        }
        if now < self.start || now >= self.expires {
            return Err(MeterError::Expired);
        }
        Ok(())
    }
    pub fn snapshot(&self) -> Result<Snapshot, MeterError> {
        self.state
            .lock()
            .map(|state| *state)
            .map_err(|_| MeterError::Backend)
    }
    pub fn reserve(
        self: &Arc<Self>,
        amount: u64,
        clock: impl Fn() -> u64,
        cancel: &AtomicBool,
    ) -> Result<Lease, MeterError> {
        self.check(clock(), cancel)?;
        if amount == 0 || amount > FRAME_BYTES {
            return Err(MeterError::Bounds);
        }
        let mut state = self.state.lock().map_err(|_| MeterError::Backend)?;
        self.check(clock(), cancel)?;
        let used = state
            .spent
            .checked_add(state.reserved)
            .ok_or(MeterError::Accounting)?;
        if used.checked_add(amount).ok_or(MeterError::Accounting)? > state.capacity {
            state.capacity_denials = state
                .capacity_denials
                .checked_add(1)
                .ok_or(MeterError::Accounting)?;
            return Err(MeterError::CapacityDenied);
        }
        let reserved = state
            .reserved
            .checked_add(amount)
            .ok_or(MeterError::Accounting)?;
        let admitted = state
            .admitted
            .checked_add(1)
            .ok_or(MeterError::Accounting)?;
        state.reserved = reserved;
        state.admitted = admitted;
        Ok(Lease {
            meter: Arc::clone(self),
            remaining: amount,
            written: 0,
            uncertain: 0,
            in_flight: 0,
            failed: false,
            completion_error: None,
        })
    }
}

/// Non-Clone lease: known successful writes cannot be refunded. Ambiguous or
/// panicking writes conservatively consume their offered bound. Uncertain
/// spending is not a measurement of actual received bytes.
pub struct Lease {
    meter: Arc<Meter>,
    remaining: u64,
    written: u64,
    uncertain: u64,
    in_flight: u64,
    failed: bool,
    completion_error: Option<MeterError>,
}
impl Lease {
    fn charge(&mut self, amount: u64, uncertain: bool) -> Result<(), MeterError> {
        if amount > self.remaining {
            return Err(MeterError::Accounting);
        }
        let mut state = self.meter.state.lock().map_err(|_| MeterError::Backend)?;
        let reserved = state
            .reserved
            .checked_sub(amount)
            .ok_or(MeterError::Accounting)?;
        let spent = state
            .spent
            .checked_add(amount)
            .ok_or(MeterError::Accounting)?;
        let unknown = state
            .uncertain_spent
            .checked_add(if uncertain { amount } else { 0 })
            .ok_or(MeterError::Accounting)?;
        if spent.checked_add(reserved).ok_or(MeterError::Accounting)? > state.capacity {
            return Err(MeterError::Accounting);
        }
        state.reserved = reserved;
        state.spent = spent;
        state.uncertain_spent = unknown;
        self.remaining -= amount;
        if uncertain {
            self.uncertain += amount;
        } else {
            self.written += amount;
        }
        Ok(())
    }
    fn failed_in_flight(&mut self) -> Result<(), MeterError> {
        self.failed = true;
        let amount = self.in_flight;
        self.charge(amount, true)?;
        self.in_flight = 0;
        Ok(())
    }
    pub fn write_to(
        &mut self,
        writer: &mut impl Write,
        bytes: &[u8],
        clock: impl Fn() -> u64,
        cancel: &AtomicBool,
    ) -> Result<usize, MeterError> {
        if let Some(error) = self.completion_error {
            return Err(error);
        }
        self.meter.check(clock(), cancel)?;
        if self.failed || self.in_flight != 0 {
            return Err(MeterError::Accounting);
        }
        let offered = u64::try_from(bytes.len()).map_err(|_| MeterError::Bounds)?;
        if offered > self.remaining {
            return Err(MeterError::Bounds);
        }
        // No meter mutex is held across the supplied Write implementation.
        self.in_flight = offered;
        match writer.write(bytes) {
            Ok(n) if n <= bytes.len() => {
                if let Err(error) = self.charge(n as u64, false) {
                    self.failed = true;
                    self.written += n as u64;
                    self.completion_error = Some(error);
                    return Ok(n);
                }
                self.in_flight = 0;
                if let Err(error) = self.meter.check(clock(), cancel) {
                    self.completion_error = Some(error);
                }
                Ok(n)
            }
            Ok(_) => {
                self.failed_in_flight()?;
                Err(MeterError::Accounting)
            }
            Err(_) => {
                self.failed_in_flight()?;
                Err(MeterError::WriteFailure)
            }
        }
    }
    pub fn written_bytes(&self) -> u64 {
        self.written
    }
    pub fn uncertain_bytes(&self) -> u64 {
        self.uncertain
    }
    pub fn check_completion(
        &self,
        clock: impl Fn() -> u64,
        cancel: &AtomicBool,
    ) -> Result<(), MeterError> {
        if let Some(error) = self.completion_error {
            return Err(error);
        }
        if self.failed || self.in_flight != 0 {
            return Err(MeterError::Accounting);
        }
        if self.remaining != 0 {
            return Err(MeterError::Incomplete);
        }
        self.meter.check(clock(), cancel)
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        if self.in_flight != 0 && self.failed_in_flight().is_err() {
            return;
        }
        if let Ok(mut state) = self.meter.state.lock() {
            if let (Some(reserved), Some(released)) = (
                state.reserved.checked_sub(self.remaining),
                state.released.checked_add(self.remaining),
            ) {
                state.reserved = reserved;
                state.released = released;
                self.remaining = 0;
            }
        }
        // Poisoned or uncertain state does not create new available credit.
    }
}
