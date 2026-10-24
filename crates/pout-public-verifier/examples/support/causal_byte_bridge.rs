//! M19: a finite causal byte/typed-key comparison, not a random-oracle proof.
//! Replies are a declared deterministic toy table; there is no native hashing,
//! proof generation, physical-resource measurement, or economic admission.

use std::sync::atomic::{AtomicBool, Ordering};

use thiserror::Error;

pub const MAX_REPORT_BYTES: usize = 65_536;
pub const MAX_TRACES: usize = 8;
pub const MAX_EVENTS: usize = 64;
pub const MAX_PAYLOAD_BYTES: usize = 131_072;
pub const MAX_KEY_BYTES: usize = 4_096;
const N: usize = 6;
const NODE: &[u8] = b"PoUT-research/DDA-EPoR-exact-intent/v2";
const RANK: &[u8] = b"PoUT-research/exact-intent-rank/v1";
const VERTEX: &[u8] = b"PoUT-research/verified-common-vertex/v2";
type Label = [u8; 32];
const ZERO: Label = [0; 32];
const ONE: Label = {
    let mut value = [0; 32];
    value[0] = 0x80;
    value
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum Error {
    #[error("finite causal experiment bound")]
    Bounds,
    #[error("bounded local allocation failed")]
    LocalResource,
    #[error("local cancellation, not a mathematical rejection")]
    Cancelled,
    #[error("independent trace or frozen expectation disagreed")]
    Invariant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub traces: usize,
    pub events_per_trace: usize,
    pub payload_bytes: usize,
    pub key_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            traces: MAX_TRACES,
            events_per_trace: MAX_EVENTS,
            payload_bytes: MAX_PAYLOAD_BYTES,
            key_bytes: MAX_KEY_BYTES,
        }
    }
}

pub fn admission(limits: Limits, cancel: &AtomicBool) -> Result<(), Error> {
    if cancel.load(Ordering::Acquire) {
        return Err(Error::Cancelled);
    }
    if limits != Limits::default() {
        return Err(Error::Bounds);
    }
    Ok(())
}

fn need(value: bool) -> Result<(), Error> {
    if value {
        Ok(())
    } else {
        Err(Error::Invariant)
    }
}

fn reserve<T>(capacity: usize) -> Result<Vec<T>, Error> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| Error::LocalResource)?;
    Ok(values)
}

fn bytes_copy(value: &[u8]) -> Result<Vec<u8>, Error> {
    if value.len() > MAX_KEY_BYTES {
        return Err(Error::Bounds);
    }
    let mut out = reserve(value.len())?;
    out.extend_from_slice(value);
    Ok(out)
}

fn frame(fields: &[&[u8]]) -> Result<Vec<u8>, Error> {
    if fields.len() > 7 || fields.iter().any(|field| field.len() > 2_048) {
        return Err(Error::Bounds);
    }
    let size = fields.iter().try_fold(0_usize, |sum, field| {
        sum.checked_add(4)
            .and_then(|n| n.checked_add(field.len()))
            .ok_or(Error::Bounds)
    })?;
    if size > MAX_KEY_BYTES {
        return Err(Error::Bounds);
    }
    let mut out = reserve(size)?;
    for field in fields {
        let len = u32::try_from(field.len()).map_err(|_| Error::Bounds)?;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(field);
    }
    Ok(out)
}

fn target(index: u8) -> Result<Vec<u8>, Error> {
    let mut intent = *b"synthetic-m19-intent-0";
    *intent.last_mut().ok_or(Error::Invariant)? = b'0'.checked_add(index).ok_or(Error::Bounds)?;
    frame(&[NODE, b"epor-node", &intent, &[0x31; 32], &[0x42; 32]])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Key {
    Node {
        target: u8,
        index: u8,
        prefix: [Label; N],
    },
    Rank {
        target: u8,
        values: [Label; N],
    },
}

fn node_key(target: u8, index: usize, values: [Label; N]) -> Result<Key, Error> {
    if index >= N {
        return Err(Error::Bounds);
    }
    let mut prefix = [ZERO; N];
    prefix[..index].copy_from_slice(&values[..index]);
    Ok(Key::Node {
        target,
        index: u8::try_from(index).map_err(|_| Error::Bounds)?,
        prefix,
    })
}

fn encode(key: Key) -> Result<Vec<u8>, Error> {
    match key {
        Key::Node {
            target: id,
            index,
            prefix,
        } => {
            let index = usize::from(index);
            if index >= N {
                return Err(Error::Bounds);
            }
            let mut parents = reserve(4 + 36 * index)?;
            parents.extend_from_slice(&(index as u32).to_le_bytes());
            for (j, label) in prefix[..index].iter().enumerate() {
                parents.extend_from_slice(&(j as u32).to_le_bytes());
                parents.extend_from_slice(label);
            }
            let suffix = frame(&[&(index as u32).to_le_bytes(), &parents])?;
            let mut out = target(id)?;
            if out.len().checked_add(suffix.len()).ok_or(Error::Bounds)? > MAX_KEY_BYTES {
                return Err(Error::Bounds);
            }
            out.try_reserve_exact(suffix.len())
                .map_err(|_| Error::LocalResource)?;
            out.extend_from_slice(&suffix);
            Ok(out)
        }
        Key::Rank { target: id, values } => {
            let mut vector = [0; 32 * N];
            for (slot, value) in vector.as_chunks_mut::<32>().0.iter_mut().zip(values) {
                slot.copy_from_slice(&value);
            }
            frame(&[RANK, &frame(&[VERTEX, &target(id)?, &vector])?])
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Route {
    Node,
    Rank,
    Other,
}

fn route(bytes: &[u8]) -> Route {
    let prefix = |domain: &[u8]| {
        bytes.starts_with(&(domain.len() as u32).to_le_bytes())
            && bytes.get(4..4 + domain.len()) == Some(domain)
    };
    if prefix(NODE) {
        Route::Node
    } else if prefix(RANK) {
        Route::Rank
    } else {
        Route::Other
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reject {
    MalformedReservedNode,
    MalformedReservedRank,
    OutsideCatalog,
    UnsupportedOutsideDomain,
}

struct Fields<'a> {
    parts: [&'a [u8]; 7],
    ends: [usize; 7],
    len: usize,
}

fn read_le(bytes: &[u8]) -> Option<usize> {
    let value: [u8; 4] = bytes.try_into().ok()?;
    usize::try_from(u32::from_le_bytes(value)).ok()
}

fn fields(bytes: &[u8]) -> Option<Fields<'_>> {
    if bytes.len() > MAX_KEY_BYTES {
        return None;
    }
    let mut result = Fields {
        parts: [&[]; 7],
        ends: [0; 7],
        len: 0,
    };
    let mut offset = 0_usize;
    while offset < bytes.len() {
        if result.len >= 7 {
            return None;
        }
        let header_end = offset.checked_add(4)?;
        let size = read_le(bytes.get(offset..header_end)?)?;
        if size > 2_048 {
            return None;
        }
        let end = header_end.checked_add(size)?;
        result.parts[result.len] = bytes.get(header_end..end)?;
        result.ends[result.len] = end;
        result.len += 1;
        offset = end;
    }
    Some(result)
}

fn catalog_target(bytes: &[u8], catalog: &[Vec<u8>; 2], malformed: Reject) -> Result<u8, Reject> {
    let f = fields(bytes).ok_or(malformed)?;
    if f.len != 5
        || f.parts[0] != NODE
        || f.parts[1] != b"epor-node"
        || f.parts[2].is_empty()
        || f.parts[2].len() > 1_024
        || f.parts[3].len() != 32
        || f.parts[4].len() != 32
    {
        return Err(malformed);
    }
    catalog
        .iter()
        .position(|value| value.as_slice() == bytes)
        .and_then(|id| u8::try_from(id).ok())
        .ok_or(Reject::OutsideCatalog)
}

fn parse(bytes: &[u8], catalog: &[Vec<u8>; 2]) -> Result<Key, Reject> {
    match route(bytes) {
        Route::Other => Err(Reject::UnsupportedOutsideDomain),
        Route::Node => {
            let malformed = Reject::MalformedReservedNode;
            let f = fields(bytes).ok_or(malformed)?;
            if f.len != 7 || f.parts[5].len() != 4 {
                return Err(malformed);
            }
            let index = read_le(f.parts[5]).ok_or(malformed)?;
            if index >= N || f.parts[6].len() != 4 + 36 * index {
                return Err(malformed);
            }
            if read_le(&f.parts[6][..4]) != Some(index) {
                return Err(malformed);
            }
            let mut prefix = [ZERO; N];
            for (j, label) in prefix[..index].iter_mut().enumerate() {
                let start = 4 + 36 * j;
                if read_le(&f.parts[6][start..start + 4]) != Some(j) {
                    return Err(malformed);
                }
                label.copy_from_slice(&f.parts[6][start + 4..start + 36]);
            }
            let id = catalog_target(&bytes[..f.ends[4]], catalog, malformed)?;
            Ok(Key::Node {
                target: id,
                index: index as u8,
                prefix,
            })
        }
        Route::Rank => {
            let malformed = Reject::MalformedReservedRank;
            let f = fields(bytes).ok_or(malformed)?;
            if f.len != 2 {
                return Err(malformed);
            }
            let inner = fields(f.parts[1]).ok_or(malformed)?;
            if inner.len != 3 || inner.parts[0] != VERTEX || inner.parts[2].len() != 32 * N {
                return Err(malformed);
            }
            let id = catalog_target(inner.parts[1], catalog, malformed)?;
            let mut values = [ZERO; N];
            for (value, bytes) in values
                .iter_mut()
                .zip(inner.parts[2].as_chunks::<32>().0.iter())
            {
                value.copy_from_slice(bytes);
            }
            Ok(Key::Rank { target: id, values })
        }
    }
}

/// Two externally supplied raw-byte goldens qualify framing independently of
/// the shared Rust target encoder. This performs no oracle/hash operation and
/// does not initialize the causal corpus. ONE occurs at vector positions0,2,5;
/// Node index2 tests a nonzero LE32 boundary and two ordered parent entries.
pub fn qualify_independent_golden(node: &[u8], rank: &[u8]) -> Result<(), Error> {
    if node.len() != 241 || rank.len() != 438 {
        return Err(Error::Bounds);
    }
    let values = [ONE, ZERO, ONE, ZERO, ZERO, ONE];
    let node_key = node_key(0, 2, values)?;
    let rank_key = Key::Rank { target: 0, values };
    need(encode(node_key)?.as_slice() == node)?;
    need(encode(rank_key)?.as_slice() == rank)?;
    let catalog = [target(0)?, target(1)?];
    need(parse(node, &catalog) == Ok(node_key))?;
    need(parse(rank, &catalog) == Ok(rank_key))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Class {
    NotRank,
    Complete,
    Missing { index: u8 },
    Mismatch { index: u8 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Actor {
    Helper,
    Recipient,
    Verifier,
}

impl Actor {
    fn slot(self) -> usize {
        match self {
            Self::Helper => 0,
            Self::Recipient => 1,
            Self::Verifier => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Category {
    Storage,
    Delivery,
    GenericHash,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Work {
    pub requests: usize,
    pub node_calls: usize,
    pub fresh_nodes: usize,
    pub cached_nodes: usize,
    pub rank_calls: usize,
    pub fresh_ranks: usize,
    pub cached_ranks: usize,
    pub rejected_before_sample: usize,
}

impl Work {
    fn add_event(&mut self, event: Event) -> Result<(), Error> {
        self.requests = self.requests.checked_add(1).ok_or(Error::Bounds)?;
        if event.reject.is_some() {
            self.rejected_before_sample += 1;
        } else if event.route == Route::Node {
            self.node_calls += 1;
            self.fresh_nodes += usize::from(event.fresh);
            self.cached_nodes += usize::from(!event.fresh);
        } else {
            need(event.route == Route::Rank)?;
            self.rank_calls += 1;
            self.fresh_ranks += usize::from(event.fresh);
            self.cached_ranks += usize::from(!event.fresh);
        }
        need(self.requests <= MAX_EVENTS)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Event {
    actor: Actor,
    category: Category,
    route: Route,
    key: Option<Key>,
    reply: Option<Label>,
    fresh: bool,
    before: Class,
    first_before: Class,
    reject: Option<Reject>,
}

fn event_base(actor: Actor, category: Category, route: Route) -> Event {
    Event {
        actor,
        category,
        route,
        key: None,
        reply: None,
        fresh: false,
        before: Class::NotRank,
        first_before: Class::NotRank,
        reject: None,
    }
}

struct ByteEntry {
    bytes: Vec<u8>,
    reply: Label,
    first: Class,
}

/// Only this module receives the scripted table rule. Drivers get replies to
/// charged requests, never an unqueried table or hidden truth inspection API.
mod byte_oracle {
    use super::*;

    pub(super) struct Machine {
        catalog: [Vec<u8>; 2],
        cache: Vec<ByteEntry>,
    }

    impl Machine {
        pub(super) fn new() -> Result<Self, Error> {
            Ok(Self {
                catalog: [target(0)?, target(1)?],
                cache: reserve(MAX_EVENTS)?,
            })
        }

        fn classify(&self, id: u8, values: [Label; N]) -> Result<Class, Error> {
            for (j, expected_value) in values.iter().enumerate() {
                let key = encode(node_key(id, j, values)?)?;
                match self.cache.iter().find(|entry| entry.bytes == key) {
                    None => return Ok(Class::Missing { index: j as u8 }),
                    Some(entry) if entry.reply != *expected_value => {
                        return Ok(Class::Mismatch { index: j as u8 });
                    }
                    Some(_) => {}
                }
            }
            Ok(Class::Complete)
        }

        pub(super) fn query(
            &mut self,
            bytes: &[u8],
            actor: Actor,
            category: Category,
        ) -> Result<Event, Error> {
            let mut event = event_base(actor, category, route(bytes));
            let key = match parse(bytes, &self.catalog) {
                Ok(key) => key,
                Err(reject) => {
                    event.reject = Some(reject);
                    return Ok(event);
                }
            };
            event.key = Some(key);
            event.before = match key {
                Key::Rank { target, values } => self.classify(target, values)?,
                Key::Node { .. } => Class::NotRank,
            };
            if let Some(entry) = self.cache.iter().find(|entry| entry.bytes == bytes) {
                event.reply = Some(entry.reply);
                event.first_before = entry.first;
            } else {
                if self.cache.len() >= MAX_EVENTS {
                    return Err(Error::Bounds);
                }
                // Fixed nonrandom toy table: target0's first Node is ONE;
                // every other Node and every Rank reply is ZERO.
                let reply = if matches!(
                    key,
                    Key::Node {
                        target: 0,
                        index: 0,
                        ..
                    }
                ) {
                    ONE
                } else {
                    ZERO
                };
                event.fresh = true;
                event.reply = Some(reply);
                event.first_before = event.before;
                self.cache.push(ByteEntry {
                    bytes: bytes_copy(bytes)?,
                    reply,
                    first: event.before,
                });
            }
            Ok(event)
        }
    }
}

/// Independent typed addressing: no byte encoder/parser or byte-cache lookup.
/// It deliberately spells out proposed prefixes and scripted replies separately.
struct Reference {
    entries: Vec<(Key, Label, Class)>,
}

impl Reference {
    fn new() -> Result<Self, Error> {
        Ok(Self {
            entries: reserve(MAX_EVENTS)?,
        })
    }

    fn query(
        &mut self,
        key: Option<Key>,
        rejected: Option<(Route, Reject)>,
        actor: Actor,
        category: Category,
    ) -> Result<Event, Error> {
        if let Some((route, reject)) = rejected {
            let mut event = event_base(actor, category, route);
            event.reject = Some(reject);
            return Ok(event);
        }
        let key = key.ok_or(Error::Invariant)?;
        let mut event = event_base(
            actor,
            category,
            if matches!(key, Key::Node { .. }) {
                Route::Node
            } else {
                Route::Rank
            },
        );
        event.key = Some(key);
        if let Key::Rank { target, values } = key {
            event.before = Class::Complete;
            for (j, expected_value) in values.iter().enumerate() {
                let mut p = [ZERO; 6];
                p[..j].copy_from_slice(&values[..j]);
                let required = Key::Node {
                    target,
                    index: j as u8,
                    prefix: p,
                };
                let known = self.entries.iter().find(|row| row.0 == required);
                if known.is_none() {
                    event.before = Class::Missing { index: j as u8 };
                    break;
                }
                if known.ok_or(Error::Invariant)?.1 != *expected_value {
                    event.before = Class::Mismatch { index: j as u8 };
                    break;
                }
            }
        }
        match self.entries.iter().find(|row| row.0 == key) {
            Some((_, reply, first)) => {
                event.reply = Some(*reply);
                event.first_before = *first;
            }
            None => {
                if self.entries.len() >= MAX_EVENTS {
                    return Err(Error::Bounds);
                }
                let mut reply = [0; 32];
                let coordinate = match key {
                    Key::Node { target, index, .. } => u16::from(target) + u16::from(index),
                    Key::Rank { .. } => u16::MAX,
                };
                if coordinate == 0 {
                    reply[0] = 128;
                }
                event.fresh = true;
                event.reply = Some(reply);
                event.first_before = event.before;
                self.entries.push((key, reply, event.before));
            }
        }
        Ok(event)
    }
}

#[derive(Debug)]
pub struct Step {
    pub actor: Actor,
    pub category: Category,
    pub actual_route: Route,
    pub target: Option<u8>,
    pub index: Option<u8>,
    pub fresh: bool,
    pub before: Class,
    pub first_before: Class,
    pub rejection: Option<Reject>,
    pub full_key_reply_and_classification_equal: bool,
}

#[derive(Debug)]
pub struct TraceReport {
    pub name: &'static str,
    pub work: Work,
    /// Helper, Recipient, Verifier, in that order.
    pub per_actor: [Work; 3],
    pub steps: Vec<Step>,
    pub full_ordered_trace_equal: bool,
    pub expected_counters_equal: bool,
    pub selected_target: Option<u8>,
    /// Deliberately naive design reference: namespaces identical bytes by a
    /// caller category. This is not a protocol credit or consensus weight.
    pub naive_category_distinct_keys: usize,
    pub actual_global_distinct_keys: usize,
    /// A retrospective proxy can count a cached reply after later preparation.
    pub current_complete_rank_occurrences: usize,
    pub fresh_rank_first_complete: usize,
}

struct Driver<'a> {
    bytes: byte_oracle::Machine,
    reference: Reference,
    report: TraceReport,
    reference_work: Work,
    reference_actors: [Work; 3],
    naive_categories: Vec<(Category, Vec<u8>)>,
    payload: &'a mut usize,
    cancel: &'a AtomicBool,
}

impl<'a> Driver<'a> {
    fn new(
        name: &'static str,
        payload: &'a mut usize,
        cancel: &'a AtomicBool,
    ) -> Result<Self, Error> {
        Ok(Self {
            bytes: byte_oracle::Machine::new()?,
            reference: Reference::new()?,
            report: TraceReport {
                name,
                work: Work::default(),
                per_actor: [Work::default(); 3],
                steps: reserve(MAX_EVENTS)?,
                full_ordered_trace_equal: true,
                expected_counters_equal: false,
                selected_target: None,
                naive_category_distinct_keys: 0,
                actual_global_distinct_keys: 0,
                current_complete_rank_occurrences: 0,
                fresh_rank_first_complete: 0,
            },
            reference_work: Work::default(),
            reference_actors: [Work::default(); 3],
            naive_categories: reserve(MAX_EVENTS)?,
            payload,
            cancel,
        })
    }

    fn step(
        &mut self,
        raw: &[u8],
        key: Option<Key>,
        rejected: Option<(Route, Reject)>,
        actor: Actor,
        category: Category,
    ) -> Result<Event, Error> {
        if self.cancel.load(Ordering::Acquire) {
            return Err(Error::Cancelled);
        }
        if self.report.steps.len() >= MAX_EVENTS || raw.len() > MAX_KEY_BYTES {
            return Err(Error::Bounds);
        }
        let payload = self.payload.checked_add(raw.len()).ok_or(Error::Bounds)?;
        if payload > MAX_PAYLOAD_BYTES {
            return Err(Error::Bounds);
        }
        *self.payload = payload;
        let actual = self.bytes.query(raw, actor, category)?;
        let expected = self.reference.query(key, rejected, actor, category)?;
        need(actual == expected)?;
        if actual.reject.is_none()
            && !self
                .naive_categories
                .iter()
                .any(|(label, bytes)| *label == category && bytes == raw)
        {
            if self.naive_categories.len() >= MAX_EVENTS {
                return Err(Error::Bounds);
            }
            self.naive_categories.push((category, bytes_copy(raw)?));
        }
        self.report.work.add_event(actual)?;
        self.report.per_actor[actor.slot()].add_event(actual)?;
        self.reference_work.add_event(expected)?;
        self.reference_actors[actor.slot()].add_event(expected)?;
        let (target, index) = match actual.key {
            Some(Key::Node { target, index, .. }) => (Some(target), Some(index)),
            Some(Key::Rank { target, .. }) => (Some(target), None),
            None => (None, None),
        };
        self.report.steps.push(Step {
            actor,
            category,
            actual_route: actual.route,
            target,
            index,
            fresh: actual.fresh,
            before: actual.before,
            first_before: actual.first_before,
            rejection: actual.reject,
            full_key_reply_and_classification_equal: true,
        });
        Ok(actual)
    }

    fn query(&mut self, key: Key, actor: Actor, category: Category) -> Result<Label, Error> {
        self.step(&encode(key)?, Some(key), None, actor, category)?
            .reply
            .ok_or(Error::Invariant)
    }

    fn prepare(&mut self, id: u8, actor: Actor, category: Category) -> Result<[Label; N], Error> {
        let mut values = [ZERO; N];
        let mut j = 0;
        while j < N {
            values[j] = self.query(node_key(id, j, values)?, actor, category)?;
            j += 1;
        }
        Ok(values)
    }

    fn finish(
        mut self,
        nodes: usize,
        fresh_nodes: usize,
        ranks: usize,
        fresh_ranks: usize,
        rejects: usize,
    ) -> Result<TraceReport, Error> {
        let expected = Work {
            requests: nodes + ranks + rejects,
            node_calls: nodes,
            fresh_nodes,
            cached_nodes: nodes.checked_sub(fresh_nodes).ok_or(Error::Bounds)?,
            rank_calls: ranks,
            fresh_ranks,
            cached_ranks: ranks.checked_sub(fresh_ranks).ok_or(Error::Bounds)?,
            rejected_before_sample: rejects,
        };
        need(self.report.work == expected)?;
        need(self.report.work == self.reference_work)?;
        need(self.report.per_actor == self.reference_actors)?;
        need(self.report.steps.len() == expected.requests)?;
        self.report.naive_category_distinct_keys = self.naive_categories.len();
        self.report.actual_global_distinct_keys = expected
            .fresh_nodes
            .checked_add(expected.fresh_ranks)
            .ok_or(Error::Bounds)?;
        self.report.current_complete_rank_occurrences = self
            .report
            .steps
            .iter()
            .filter(|step| step.actual_route == Route::Rank && step.before == Class::Complete)
            .count();
        self.report.fresh_rank_first_complete = self
            .report
            .steps
            .iter()
            .filter(|step| {
                step.actual_route == Route::Rank
                    && step.fresh
                    && step.first_before == Class::Complete
            })
            .count();
        self.report.expected_counters_equal = true;
        Ok(self.report)
    }
}

#[derive(Debug)]
pub struct Report {
    pub complete: bool,
    pub n: usize,
    pub targets: usize,
    pub traces: Vec<TraceReport>,
    pub total_payload_bytes: usize,
    pub every_ordered_event_compared: bool,
    pub scripted_replies_are_not_an_independent_uniform_law: bool,
    pub parser_is_not_requested_target_authorization: bool,
    pub global_charging_is_not_private_expenditure: bool,
    pub native_hash_curve_snark_dag_cwc_calls: usize,
    pub consensus_eligibility: &'static str,
    pub consensus_weight: Option<u64>,
}

pub fn run(limits: Limits, cancel: &AtomicBool) -> Result<Report, Error> {
    admission(limits, cancel)?;
    let mut traces = reserve(MAX_TRACES)?;
    let mut payload = 0;
    {
        let mut d = Driver::new("six_sequential_nodes_then_rank", &mut payload, cancel)?;
        let values = d.prepare(0, Actor::Recipient, Category::Storage)?;
        d.query(
            Key::Rank { target: 0, values },
            Actor::Recipient,
            Category::Storage,
        )?;
        need(d.report.steps[6].before == Class::Complete)?;
        traces.push(d.finish(6, 6, 1, 1, 0)?);
    }
    {
        let mut d = Driver::new("generic_helper_work_is_global_work", &mut payload, cancel)?;
        let values = d.prepare(0, Actor::Helper, Category::GenericHash)?;
        d.query(
            Key::Rank { target: 0, values },
            Actor::Recipient,
            Category::GenericHash,
        )?;
        need(d.report.per_actor[0].node_calls == 6 && d.report.per_actor[1].node_calls == 0)?;
        traces.push(d.finish(6, 6, 1, 1, 0)?);
    }
    {
        let mut d = Driver::new(
            "helper_selected_target_keeps_charged_history",
            &mut payload,
            cancel,
        )?;
        let first = d.query(
            node_key(0, 0, [ZERO; N])?,
            Actor::Helper,
            Category::GenericHash,
        )?;
        let second = d.query(
            node_key(1, 0, [ZERO; N])?,
            Actor::Helper,
            Category::GenericHash,
        )?;
        let id = if first == ZERO {
            0
        } else if second == ZERO {
            1
        } else {
            return Err(Error::Invariant);
        };
        d.report.selected_target = Some(id);
        let mut values = [ZERO; N];
        values[0] = if id == 0 { first } else { second };
        let mut j = 1;
        while j < N {
            values[j] = d.query(
                node_key(id, j, values)?,
                Actor::Recipient,
                Category::Delivery,
            )?;
            j += 1;
        }
        d.query(
            Key::Rank { target: id, values },
            Actor::Recipient,
            Category::Delivery,
        )?;
        need(
            id == 1
                && d.report.per_actor[0].node_calls == 2
                && d.report.per_actor[1].node_calls == 5,
        )?;
        traces.push(d.finish(7, 7, 1, 1, 0)?);
    }
    {
        let mut d = Driver::new(
            "guessed_rank_then_completion_keeps_first_class",
            &mut payload,
            cancel,
        )?;
        let guessed = Key::Rank {
            target: 1,
            values: [ZERO; N],
        };
        d.query(guessed, Actor::Recipient, Category::GenericHash)?;
        let values = d.prepare(1, Actor::Helper, Category::Storage)?;
        need(values == [ZERO; N])?;
        d.query(guessed, Actor::Recipient, Category::Delivery)?;
        need(d.report.steps[0].first_before == Class::Missing { index: 0 })?;
        need(
            d.report.steps[7].before == Class::Complete
                && d.report.steps[7].first_before == Class::Missing { index: 0 }
                && !d.report.steps[7].fresh,
        )?;
        traces.push(d.finish(6, 6, 2, 1, 0)?);
    }
    {
        let mut d = Driver::new("off_proposed_path_is_not_known_path", &mut payload, cancel)?;
        // The root answer is ONE, but all subsequent queries explicitly use
        // ZERO as the first parent; cached values at the same indices do not
        // substitute for the different proposed prefix keys.
        for j in 0..N {
            d.query(
                node_key(0, j, [ZERO; N])?,
                Actor::Recipient,
                Category::GenericHash,
            )?;
        }
        let mut proposed = [ZERO; N];
        proposed[0] = ONE;
        d.query(
            Key::Rank {
                target: 0,
                values: proposed,
            },
            Actor::Recipient,
            Category::Storage,
        )?;
        need(d.report.steps[6].before == Class::Missing { index: 1 })?;
        traces.push(d.finish(6, 6, 1, 1, 0)?);
    }
    {
        let mut d = Driver::new(
            "different_full_vectors_are_different_rank_keys",
            &mut payload,
            cancel,
        )?;
        d.query(
            node_key(1, 0, [ZERO; N])?,
            Actor::Recipient,
            Category::Storage,
        )?;
        d.query(
            Key::Rank {
                target: 1,
                values: [ZERO; N],
            },
            Actor::Recipient,
            Category::Storage,
        )?;
        let mut alternate = [ZERO; N];
        alternate[0] = ONE;
        d.query(
            Key::Rank {
                target: 1,
                values: alternate,
            },
            Actor::Recipient,
            Category::Storage,
        )?;
        need(d.report.steps.iter().all(|step| step.fresh))?;
        need(
            d.report.steps[1].before == Class::Missing { index: 1 }
                && d.report.steps[2].before == Class::Mismatch { index: 0 },
        )?;
        traces.push(d.finish(1, 1, 2, 2, 0)?);
    }
    {
        let mut d = Driver::new(
            "permitted_categories_share_exact_key_cache",
            &mut payload,
            cancel,
        )?;
        let values = d.prepare(0, Actor::Helper, Category::Storage)?;
        d.query(
            Key::Rank { target: 0, values },
            Actor::Helper,
            Category::Storage,
        )?;
        let copied = d.prepare(0, Actor::Verifier, Category::Delivery)?;
        need(copied == values)?;
        d.query(
            Key::Rank {
                target: 0,
                values: copied,
            },
            Actor::Verifier,
            Category::Delivery,
        )?;
        traces.push(d.finish(12, 6, 2, 1, 0)?);
    }
    {
        let mut d = Driver::new(
            "reserved_malformed_and_catalog_rejections",
            &mut payload,
            cancel,
        )?;
        let base = encode(node_key(0, 1, [ZERO; N])?)?;
        let f = fields(&base).ok_or(Error::Invariant)?;
        let mut duplicate = bytes_copy(f.parts[6])?;
        duplicate[..4].copy_from_slice(&2_u32.to_le_bytes());
        let bad_duplicate = frame(&[
            f.parts[0], f.parts[1], f.parts[2], f.parts[3], f.parts[4], f.parts[5], &duplicate,
        ])?;
        d.step(
            &bad_duplicate,
            None,
            Some((Route::Node, Reject::MalformedReservedNode)),
            Actor::Helper,
            Category::GenericHash,
        )?;
        let mut parent = bytes_copy(f.parts[6])?;
        parent[4..8].copy_from_slice(&1_u32.to_le_bytes());
        let bad_parent = frame(&[
            f.parts[0], f.parts[1], f.parts[2], f.parts[3], f.parts[4], f.parts[5], &parent,
        ])?;
        d.step(
            &bad_parent,
            None,
            Some((Route::Node, Reject::MalformedReservedNode)),
            Actor::Helper,
            Category::GenericHash,
        )?;
        let endian = frame(&[
            f.parts[0],
            f.parts[1],
            f.parts[2],
            f.parts[3],
            f.parts[4],
            &1_u32.to_be_bytes(),
            f.parts[6],
        ])?;
        d.step(
            &endian,
            None,
            Some((Route::Node, Reject::MalformedReservedNode)),
            Actor::Helper,
            Category::GenericHash,
        )?;
        let mut trailing = bytes_copy(&base)?;
        trailing
            .try_reserve_exact(1)
            .map_err(|_| Error::LocalResource)?;
        trailing.push(0x58);
        d.step(
            &trailing,
            None,
            Some((Route::Node, Reject::MalformedReservedNode)),
            Actor::Helper,
            Category::GenericHash,
        )?;
        let short = frame(&[RANK, &frame(&[VERTEX, &target(0)?, &[0; 191]])?])?;
        d.step(
            &short,
            None,
            Some((Route::Rank, Reject::MalformedReservedRank)),
            Actor::Helper,
            Category::GenericHash,
        )?;
        let wrong_vertex = frame(&[RANK, &frame(&[b"wrong-vertex", &target(0)?, &[0; 192]])?])?;
        d.step(
            &wrong_vertex,
            None,
            Some((Route::Rank, Reject::MalformedReservedRank)),
            Actor::Helper,
            Category::GenericHash,
        )?;
        let mut rank_trailing = encode(Key::Rank {
            target: 0,
            values: [ZERO; N],
        })?;
        rank_trailing
            .try_reserve_exact(1)
            .map_err(|_| Error::LocalResource)?;
        rank_trailing.push(0x58);
        d.step(
            &rank_trailing,
            None,
            Some((Route::Rank, Reject::MalformedReservedRank)),
            Actor::Helper,
            Category::GenericHash,
        )?;
        let outside = encode(node_key(2, 0, [ZERO; N])?)?;
        d.step(
            &outside,
            None,
            Some((Route::Node, Reject::OutsideCatalog)),
            Actor::Recipient,
            Category::Storage,
        )?;
        let unrelated = frame(&[b"unrelated-domain", b"public"])?;
        d.step(
            &unrelated,
            None,
            Some((Route::Other, Reject::UnsupportedOutsideDomain)),
            Actor::Recipient,
            Category::GenericHash,
        )?;
        traces.push(d.finish(0, 0, 0, 0, 9)?);
    }
    need(traces.len() == MAX_TRACES && payload <= MAX_PAYLOAD_BYTES)?;
    Ok(Report {
        complete: true,
        n: N,
        targets: 2,
        traces,
        total_payload_bytes: payload,
        every_ordered_event_compared: true,
        scripted_replies_are_not_an_independent_uniform_law: true,
        parser_is_not_requested_target_authorization: true,
        global_charging_is_not_private_expenditure: true,
        native_hash_curve_snark_dag_cwc_calls: 0,
        consensus_eligibility: "NotEstablished",
        consensus_weight: None,
    })
}

/// Public text renderer. The finite experiment itself is unchanged.
/// Explicit field rendering keeps this export on the existing dependency set.
pub fn format_report(report: &Report) -> Result<String, Error> {
    use std::fmt::Write;
    let mut out = String::new();
    out.try_reserve_exact(MAX_REPORT_BYTES)
        .map_err(|_| Error::LocalResource)?;
    writeln!(out, "complete={} n={} targets={} payload_bytes={}",
        report.complete, report.n, report.targets, report.total_payload_bytes)
        .map_err(|_| Error::LocalResource)?;
    writeln!(out, "ordered={} scripted_not_uniform={} parser_not_authority={} global_not_private_cost={} native_calls={} eligibility={} weight={:?}",
        report.every_ordered_event_compared,
        report.scripted_replies_are_not_an_independent_uniform_law,
        report.parser_is_not_requested_target_authorization,
        report.global_charging_is_not_private_expenditure,
        report.native_hash_curve_snark_dag_cwc_calls,
        report.consensus_eligibility, report.consensus_weight)
        .map_err(|_| Error::LocalResource)?;
    for trace in &report.traces {
        writeln!(out, "trace={} work={:?} actors={:?} ordered={} counters={} target={:?} category_keys={} global_keys={} currently_complete={} fresh_first_complete={}",
            trace.name, trace.work, trace.per_actor, trace.full_ordered_trace_equal,
            trace.expected_counters_equal, trace.selected_target,
            trace.naive_category_distinct_keys, trace.actual_global_distinct_keys,
            trace.current_complete_rank_occurrences, trace.fresh_rank_first_complete)
            .map_err(|_| Error::LocalResource)?;
        for step in &trace.steps {
            writeln!(out, "  actor={:?} category={:?} route={:?} target={:?} index={:?} fresh={} before={:?} first={:?} rejection={:?} equal={}",
                step.actor, step.category, step.actual_route, step.target, step.index,
                step.fresh, step.before, step.first_before, step.rejection,
                step.full_key_reply_and_classification_equal)
                .map_err(|_| Error::LocalResource)?;
        }
    }
    if out.len() > MAX_REPORT_BYTES {
        return Err(Error::Bounds);
    }
    Ok(out)
}
