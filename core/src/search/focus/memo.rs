// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The memo of stable sequents. A proved sequent is a fact about its two
//! zones that holds regardless of how the search reached it; a failed one
//! is a fact only relative to the copy budget that was left when it failed,
//! unless the search space below it was explored without ever hitting the
//! budget. A complete failure holds as well for every sequent that differs
//! in interchangeable members of the linear zone only, so it is kept under
//! a key with every such member replaced by the first of its class (the
//! *canonical* key). A proof names its occurrences and stays under the
//! sequent's own key, and so does a failure cut by the budget: shared, it
//! would answer for a relative that the search of the sequent itself
//! leads to, level after level, and keep both from ever being complete.
//! One table holds all three: a key that is not canonical holds a proof
//! or a cut failure, a canonical one may also hold a complete failure,
//! which is all a relative reads there. The table has a cap in entries
//! and takes its memory from the search's account; when the search runs on
//! several threads it is one shard of a sharded map, [`Shared`], whose
//! merge under the shard's lock keeps the same invariant: an entry's
//! validity only ever grows.

use super::context::Context;
use crate::occurrences::{OccId, OccSet};
use crate::proofs::NodeId;
use crate::search::memory::{Account, bytes_of};
use std::hash::BuildHasher as _;
use std::sync::Mutex;

/// A stable sequent as the memo keys it: the unrestricted zone and the
/// linear zone.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Key {
    /// The unrestricted zone.
    pub(crate) theta: OccSet,
    /// The linear zone.
    pub(crate) gamma: Context,
}

impl Key {
    /// Makes this key a copy of the zones, reusing the buffers.
    pub(crate) fn assign(&mut self, theta: &OccSet, gamma: &Context) {
        self.theta.clone_from(theta);
        self.gamma.clone_from(gamma);
    }
}

/// What the search found out about a stable sequent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    /// Provable, by the subproof rooted at this node of the engine's arena.
    Proved(NodeId),
    /// Unprovable, as far as the copy budget allowed.
    Failed(Failure),
}

/// How a stable sequent failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Failure {
    /// Every branch below it was explored to its end without hitting the
    /// copy budget: unprovable at any budget.
    Complete,
    /// Some branch below it was cut by the copy budget when this many
    /// copies were left: unprovable with at most that many copies left.
    Exhausted(u32),
}

impl Entry {
    /// The entry as one word of a record: the node of a proof, or a tag
    /// above the low half with the budget of a cut failure in it.
    fn code(self) -> u64 {
        match self {
            Self::Proved(node) => u64::from(node.get()),
            Self::Failed(Failure::Complete) => 1 << 32,
            Self::Failed(Failure::Exhausted(left)) => (2 << 32) | u64::from(left),
        }
    }

    /// The entry a record's word stands for.
    fn of(code: u64) -> Self {
        match code >> 32 {
            0 => Self::Proved(NodeId::new(code as u32)),
            1 => Self::Failed(Failure::Complete),
            _ => Self::Failed(Failure::Exhausted(code as u32)),
        }
    }
}

/// What became of an insertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Inserted {
    /// The entry is recorded, or the memo is switched off.
    Done,
    /// The table is full, by its entries or by the memory left: emptied,
    /// it takes the entry.
    Full,
    /// The table is empty and the memory left has no room for its first
    /// entry.
    NoRoom,
}

/// The words of a record before its zones: the key's hash, the entry, and
/// where the linear zone's extra copies are.
const HEADER: usize = 3;

/// The words of a chunk of an engine's own memo: a mebibyte.
const CHUNK: usize = 1 << 17;

/// The words of a chunk of one shard of a shared memo.
const SHARD_CHUNK: usize = 1 << 13;

/// The memo: stable sequents mapped to what the search found out about
/// them, with at most `limit` entries, within the memory the search's
/// account has left. When the table is full it is emptied, which only
/// costs time: every entry is a fact the search can find again.
///
/// An entry is a record of words: its key's hash, the entry, the place of
/// the linear zone's extra copies, and the words of both zones. Records
/// lie in chunks of a fixed size, so the memo's memory is the chunks it
/// has asked for, an entry costs no allocation of its own, emptying the
/// table is resetting a count and zeroing its index, and dropping it frees
/// a few hundred blocks: a full memo of a gibibyte went in half a second
/// when every key was two allocations. The index is open addressing over
/// record numbers, which a search only ever looks up or adds to.
#[derive(Debug)]
pub(crate) struct Memo {
    /// Per slot, the number of a record plus one, or zero: linear probing
    /// from the hash's low bits. A power of two in length, at least twice
    /// the records, or empty.
    slots: Vec<u32>,
    /// The records, `1 << shift` to a chunk.
    chunks: Vec<Box<[u64]>>,
    /// The words of one record, known from the first key: zero until
    /// then.
    stride: usize,
    /// The binary logarithm of the records in a chunk.
    shift: u32,
    /// The words a chunk is to have at most, as far as a whole number of
    /// records allows.
    chunk: usize,
    /// The number of records. Below `u32::MAX`, which the slots can name:
    /// an insertion beyond is answered [`Inserted::Full`].
    len: usize,
    /// The extra copies of every record's linear zone, one after another.
    /// A record names its own by a 32-bit offset and a 32-bit length, so
    /// the list holds at most `u32::MAX` of them: a key that would take it
    /// beyond is answered [`Inserted::Full`], never recorded under an
    /// offset that wrapped, which would compare it with another record's
    /// copies.
    extras: Vec<(OccId, u32)>,
    /// The number of entries the table holds at most; zero switches the memo
    /// off.
    limit: usize,
    /// The most entries the table held at once.
    peak: usize,
    /// How many lookups found an entry that applied.
    hits: u64,
}

impl Memo {
    /// Returns an empty memo holding at most `limit` entries.
    pub(crate) fn new(limit: usize) -> Self {
        Self::with_chunk(limit, CHUNK)
    }

    /// Returns an empty memo holding at most `limit` entries in chunks of
    /// about `chunk` words.
    fn with_chunk(limit: usize, chunk: usize) -> Self {
        Self {
            slots: Vec::new(),
            chunks: Vec::new(),
            stride: 0,
            shift: 0,
            chunk,
            len: 0,
            extras: Vec::new(),
            limit: limit.min(u32::MAX as usize - 1),
            peak: 0,
            hits: 0,
        }
    }

    /// The hash of a key.
    fn hash(key: &Key) -> u64 {
        crate::hash::BuildHasher::default().hash_one(key)
    }

    /// The words of record `e`.
    fn record(&self, e: usize) -> &[u64] {
        let at = (e & ((1 << self.shift) - 1)) * self.stride;
        &self.chunks[e >> self.shift][at..at + self.stride]
    }

    /// The words of record `e`, to write.
    fn record_mut(&mut self, e: usize) -> &mut [u64] {
        let at = (e & ((1 << self.shift) - 1)) * self.stride;
        &mut self.chunks[e >> self.shift][at..at + self.stride]
    }

    /// Whether a record is the key's.
    fn matches(&self, record: &[u64], key: &Key) -> bool {
        let width = (self.stride - HEADER) / 2;
        let (start, len) = ((record[2] >> 32) as usize, record[2] as u32 as usize);
        record[HEADER..HEADER + width] == *key.theta.words()
            && record[HEADER + width..] == *key.gamma.set().words()
            && self.extras[start..start + len] == *key.gamma.extra()
    }

    /// The record of a key with the hash given, if there is one.
    fn find(&self, key: &Key, hash: u64) -> Option<usize> {
        if self.slots.is_empty() {
            return None;
        }
        let mask = self.slots.len() - 1;
        let mut slot = hash as usize & mask;
        loop {
            let e = (self.slots[slot] as usize).checked_sub(1)?;
            let record = self.record(e);
            if record[0] == hash && self.matches(record, key) {
                return Some(e);
            }
            slot = (slot + 1) & mask;
        }
    }

    /// Returns what is known about a stable sequent that applies with
    /// `remaining` copies left: a proof or a complete failure always, a
    /// failure cut by the budget only when at most as many copies are left
    /// now as were then, since a larger budget could prove more.
    pub(crate) fn get(&mut self, key: &Key, remaining: u32) -> Option<Entry> {
        self.get_hashed(key, Self::hash(key), remaining)
    }

    /// [`get`](Self::get) for a key whose hash is known.
    fn get_hashed(&mut self, key: &Key, hash: u64, remaining: u32) -> Option<Entry> {
        let entry = Entry::of(self.record(self.find(key, hash)?)[1]);
        if let Entry::Failed(Failure::Exhausted(then)) = entry
            && remaining > then
        {
            return None;
        }
        self.hits += 1;
        Some(entry)
    }

    /// Returns whether a complete failure is recorded under a canonical
    /// key: the answer for every sequent the key stands for. Whatever else
    /// the key holds is about the canonical sequent alone.
    pub(crate) fn refuted(&mut self, key: &Key) -> bool {
        self.refuted_hashed(key, Self::hash(key))
    }

    /// [`refuted`](Self::refuted) for a key whose hash is known.
    fn refuted_hashed(&mut self, key: &Key, hash: u64) -> bool {
        let refuted = self
            .find(key, hash)
            .is_some_and(|e| self.record(e)[1] == Entry::Failed(Failure::Complete).code());
        self.hits += u64::from(refuted);
        refuted
    }

    /// Records what the search found out about a stable sequent, unless
    /// the table has no room for a new key: then it says so and the caller
    /// empties it ([`clear`](Self::clear)) and inserts again. A proof or a
    /// complete failure replaces anything; a failure cut by the budget
    /// only raises the budget an earlier such failure recorded. The memory
    /// a new chunk or a larger index takes is charged to `account`, and
    /// only taken while it leaves an eighth of the bound to what the
    /// search cannot empty ([`Account::spares`]).
    pub(crate) fn insert(&mut self, key: &Key, entry: Entry, account: &Account) -> Inserted {
        self.insert_hashed(key, Self::hash(key), entry, account)
    }

    /// [`insert`](Self::insert) for a key whose hash is known.
    fn insert_hashed(&mut self, key: &Key, hash: u64, entry: Entry, account: &Account) -> Inserted {
        if self.limit == 0 {
            return Inserted::Done;
        }
        if let Some(e) = self.find(key, hash) {
            let old = Entry::of(self.record(e)[1]);
            debug_assert!(
                !matches!(
                    (old, entry),
                    (Entry::Proved(_), Entry::Failed(Failure::Complete))
                ),
                "a complete failure of a proved sequent"
            );
            match (old, entry) {
                (
                    Entry::Failed(Failure::Exhausted(then)),
                    Entry::Failed(Failure::Exhausted(now)),
                ) if now <= then => {}
                (Entry::Proved(_) | Entry::Failed(Failure::Complete), Entry::Failed(_)) => {}
                _ => self.record_mut(e)[1] = entry.code(),
            }
            return Inserted::Done;
        }
        let extras_fit = self.extras.len() + key.gamma.extra().len() <= u32::MAX as usize;
        if self.len >= self.limit || !extras_fit || !self.reserve(key, account) {
            return if self.len == 0 {
                Inserted::NoRoom
            } else {
                Inserted::Full
            };
        }
        let extras = bytes_of(&self.extras);
        let span = ((self.extras.len() as u64) << 32) | key.gamma.extra().len() as u64;
        self.extras.extend_from_slice(key.gamma.extra());
        account.resize(extras, bytes_of(&self.extras));
        let e = self.len;
        let width = (self.stride - HEADER) / 2;
        let record = self.record_mut(e);
        record[0] = hash;
        record[1] = entry.code();
        record[2] = span;
        record[HEADER..HEADER + width].copy_from_slice(key.theta.words());
        record[HEADER + width..].copy_from_slice(key.gamma.set().words());
        self.len += 1;
        self.index(e, hash);
        self.peak = self.peak.max(self.len);
        Inserted::Done
    }

    /// Puts record `e` into the first free slot from its hash on.
    fn index(&mut self, e: usize, hash: u64) {
        let mask = self.slots.len() - 1;
        let mut slot = hash as usize & mask;
        while self.slots[slot] != 0 {
            slot = (slot + 1) & mask;
        }
        self.slots[slot] = e as u32 + 1;
    }

    /// Makes room for one more record, a key like `key`: a chunk when the
    /// last is full and an index of twice the size when half its slots are
    /// taken, each only if the account has the memory. Returns whether
    /// there is room.
    fn reserve(&mut self, key: &Key, account: &Account) -> bool {
        if self.stride == 0 {
            self.stride = HEADER + 2 * key.theta.words().len();
            // Under a small bound the chunks are small: a sixteenth of
            // the room there is, so that the table can grow in steps.
            let room = account.limit().saturating_sub(account.used()) / 16;
            let words = (room / size_of::<u64>() as u64).min(self.chunk as u64) as usize;
            self.shift = (words / self.stride).max(1).ilog2();
        }
        if (self.len + 1) * 2 > self.slots.len() {
            let wider = (self.slots.len() * 2).max(16);
            if !account.spares(wider * size_of::<u32>()) {
                return false;
            }
            let narrower = bytes_of(&self.slots);
            self.slots = vec![0; wider];
            account.resize(narrower, bytes_of(&self.slots));
            for e in 0..self.len {
                let hash = self.record(e)[0];
                self.index(e, hash);
            }
        }
        if self.len == self.chunks.len() << self.shift {
            let words = self.stride << self.shift;
            if !account.spares(words * size_of::<u64>()) {
                return false;
            }
            account.charge(words * size_of::<u64>());
            self.chunks.push(vec![0; words].into_boxed_slice());
        }
        true
    }

    /// Empties the table and keeps its memory for the entries to come: a
    /// count is reset and the index zeroed.
    pub(crate) fn clear(&mut self) {
        self.len = 0;
        self.slots.fill(0);
        self.extras.clear();
    }

    /// Empties the table and gives its memory back.
    pub(crate) fn release(&mut self, account: &Account) {
        let held = bytes_of(&self.slots)
            + bytes_of(&self.extras)
            + self.chunks.len() * (self.stride << self.shift) * size_of::<u64>();
        account.release(held);
        self.len = 0;
        self.slots = Vec::new();
        self.extras = Vec::new();
        self.chunks = Vec::new();
    }

    /// Returns the most entries the table held at once.
    pub(crate) fn peak(&self) -> usize {
        self.peak
    }

    /// Returns how many lookups found an entry that applied.
    pub(crate) fn hits(&self) -> u64 {
        self.hits
    }
}

/// How many shards a shared memo has: a key's top hash bits pick one.
const SHARDS: usize = 64;

/// The memo shared by the workers of a parallel search: [`SHARDS`] memos
/// behind one lock each, a key's top hash bits choosing the shard. A
/// lookup or an insertion holds one lock for the time of one map
/// operation, so the merge of `insert` is atomic per key and two workers
/// that decide the same sequent at once cost duplicated work, never a
/// weaker entry. The cap is per shard, and a shard that is full, by its
/// entries or by the memory left, is emptied by itself.
#[derive(Debug)]
pub(crate) struct Shared {
    /// The shards.
    shards: Box<[Mutex<Memo>]>,
}

impl Shared {
    /// Returns an empty shared memo holding at most `limit` entries in all.
    pub(crate) fn new(limit: usize) -> Self {
        let per_shard = limit.div_ceil(SHARDS);
        Self {
            shards: (0..SHARDS)
                .map(|_| {
                    Mutex::new(Memo::with_chunk(
                        if limit == 0 { 0 } else { per_shard },
                        SHARD_CHUNK,
                    ))
                })
                .collect(),
        }
    }

    /// The shard of a key with the hash given.
    fn shard(&self, hash: u64) -> std::sync::MutexGuard<'_, Memo> {
        Self::lock(&self.shards[(hash >> (64 - SHARDS.trailing_zeros())) as usize])
    }

    /// Locks a shard, recovering the memo from a worker that panicked
    /// while holding the lock: every entry is a fact the worker had
    /// finished writing before the panic could interrupt it.
    fn lock(shard: &Mutex<Memo>) -> std::sync::MutexGuard<'_, Memo> {
        shard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// [`Memo::get`] on the key's shard.
    pub(crate) fn get(&self, key: &Key, remaining: u32) -> Option<Entry> {
        let hash = Memo::hash(key);
        self.shard(hash).get_hashed(key, hash, remaining)
    }

    /// [`Memo::refuted`] on the key's shard.
    pub(crate) fn refuted(&self, key: &Key) -> bool {
        let hash = Memo::hash(key);
        self.shard(hash).refuted_hashed(key, hash)
    }

    /// [`Memo::insert`] on the key's shard, which is emptied first when it
    /// is full. An entry that even the empty shard has no memory for is
    /// dropped: the other shards hold it, and the search says so when it
    /// finds itself over its bound.
    pub(crate) fn insert(&self, key: &Key, entry: Entry, account: &Account) {
        let hash = Memo::hash(key);
        let mut shard = self.shard(hash);
        if shard.insert_hashed(key, hash, entry, account) == Inserted::Full {
            shard.clear();
            shard.insert_hashed(key, hash, entry, account);
        }
    }

    /// Empties every shard and gives its memory back.
    pub(crate) fn release(&self, account: &Account) {
        for shard in &self.shards {
            Self::lock(shard).release(account);
        }
    }

    /// Returns the sum over the shards of the most entries each held at
    /// once: an upper bound on the most entries the memo held at once.
    pub(crate) fn peak(&self) -> usize {
        self.shards.iter().map(|s| Self::lock(s).peak()).sum()
    }

    /// Returns how many lookups found an entry that applied.
    pub(crate) fn hits(&self) -> u64 {
        self.shards.iter().map(|s| Self::lock(s).hits()).sum()
    }
}

/// The memo an engine consults: its own, or the one a parallel search
/// shares among its workers.
#[derive(Debug)]
pub(crate) enum Table<'a> {
    /// The engine's own memo.
    Own(Memo),
    /// The memo of a parallel search.
    Shared(&'a Shared),
}

impl Table<'_> {
    /// [`Memo::get`].
    pub(crate) fn get(&mut self, key: &Key, remaining: u32) -> Option<Entry> {
        match self {
            Self::Own(memo) => memo.get(key, remaining),
            Self::Shared(shared) => shared.get(key, remaining),
        }
    }

    /// [`Memo::refuted`].
    pub(crate) fn refuted(&mut self, key: &Key) -> bool {
        match self {
            Self::Own(memo) => memo.refuted(key),
            Self::Shared(shared) => shared.refuted(key),
        }
    }

    /// [`Memo::insert`]. A shared memo makes its own room and always
    /// answers [`Inserted::Done`].
    pub(crate) fn insert(&mut self, key: &Key, entry: Entry, account: &Account) -> Inserted {
        match self {
            Self::Own(memo) => memo.insert(key, entry, account),
            Self::Shared(shared) => {
                shared.insert(key, entry, account);
                Inserted::Done
            }
        }
    }

    /// [`Memo::clear`] on an engine's own memo; a shared memo empties its
    /// shards itself.
    pub(crate) fn clear(&mut self) {
        if let Self::Own(memo) = self {
            memo.clear();
        }
    }

    /// Empties the memo and gives its memory back.
    pub(crate) fn release(&mut self, account: &Account) {
        match self {
            Self::Own(memo) => memo.release(account),
            Self::Shared(shared) => shared.release(account),
        }
    }

    /// [`Memo::peak`].
    pub(crate) fn peak(&self) -> usize {
        match self {
            Self::Own(memo) => memo.peak(),
            Self::Shared(shared) => shared.peak(),
        }
    }

    /// [`Memo::hits`].
    pub(crate) fn hits(&self) -> u64 {
        match self {
            Self::Own(memo) => memo.hits(),
            Self::Shared(shared) => shared.hits(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Under a canonical key only a complete failure answers for the
    /// sequents the key stands for: a cut failure and a proof there are
    /// the canonical sequent's own.
    #[test]
    fn complete_failures_are_shared() {
        let mut key = Key {
            theta: OccSet::empty(8),
            gamma: Context::empty(8),
        };
        key.gamma.insert(OccId::new(1));
        let account = Account::new(None);
        let mut memo = Memo::new(10);
        assert!(!memo.refuted(&key));
        memo.insert(&key, Entry::Failed(Failure::Exhausted(1)), &account);
        assert!(!memo.refuted(&key));
        memo.insert(&key, Entry::Failed(Failure::Complete), &account);
        assert!(memo.refuted(&key));
        let mut proved = Memo::new(10);
        proved.insert(&key, Entry::Proved(NodeId::new(4)), &account);
        assert!(!proved.refuted(&key));
        assert_eq!((memo.hits(), proved.hits()), (1, 0));
    }

    /// A failure cut by the budget is a hit only with at most as many
    /// copies left, a complete failure and a proof always; a later entry
    /// only strengthens what is known.
    #[test]
    fn bounded_failures() {
        let account = Account::new(None);
        let mut memo = Memo::new(10);
        let mut key = Key {
            theta: OccSet::empty(8),
            gamma: Context::empty(8),
        };
        key.gamma.insert(OccId::new(1));
        assert_eq!(memo.get(&key, 0), None);
        memo.insert(&key, Entry::Failed(Failure::Exhausted(1)), &account);
        assert_eq!(memo.get(&key, 2), None, "more copies left now");
        assert_eq!(
            memo.get(&key, 1),
            Some(Entry::Failed(Failure::Exhausted(1)))
        );
        assert_eq!(
            memo.get(&key, 0),
            Some(Entry::Failed(Failure::Exhausted(1)))
        );
        memo.insert(&key, Entry::Failed(Failure::Exhausted(0)), &account);
        assert_eq!(
            memo.get(&key, 1),
            Some(Entry::Failed(Failure::Exhausted(1))),
            "a smaller budget does not weaken the entry"
        );
        memo.insert(&key, Entry::Failed(Failure::Exhausted(3)), &account);
        assert_eq!(
            memo.get(&key, 3),
            Some(Entry::Failed(Failure::Exhausted(3)))
        );
        memo.insert(&key, Entry::Failed(Failure::Complete), &account);
        assert_eq!(memo.get(&key, 100), Some(Entry::Failed(Failure::Complete)));
        memo.insert(&key, Entry::Failed(Failure::Exhausted(5)), &account);
        assert_eq!(
            memo.get(&key, 100),
            Some(Entry::Failed(Failure::Complete)),
            "a complete failure stays"
        );
        let mut other = Key {
            theta: OccSet::empty(8),
            gamma: Context::empty(8),
        };
        other.gamma.insert(OccId::new(2));
        let proved = Entry::Proved(NodeId::new(7));
        memo.insert(&other, proved, &account);
        assert_eq!(memo.get(&other, 0), Some(proved));
        memo.insert(&other, Entry::Failed(Failure::Exhausted(9)), &account);
        assert_eq!(memo.get(&other, 0), Some(proved), "a proof stays");
        assert_eq!(memo.hits(), 8);
        assert_eq!(memo.peak(), 2);
    }

    /// A key with the one member given in its linear zone.
    fn key(member: u32) -> Key {
        let mut key = Key {
            theta: OccSet::empty(1024),
            gamma: Context::empty(1024),
        };
        key.gamma.insert(OccId::new(member));
        key
    }

    /// A table full by its entries says so and takes the entry once it is
    /// emptied, keeping its memory; one whose next chunk the account has
    /// no room for is full in the same way, and an empty one that cannot
    /// have its first chunk has no room; a release gives everything back.
    #[test]
    fn full_and_emptied() {
        let entry = Entry::Failed(Failure::Complete);
        let account = Account::new(None);
        let mut memo = Memo::new(2);
        assert_eq!(memo.insert(&key(1), entry, &account), Inserted::Done);
        assert_eq!(memo.insert(&key(2), entry, &account), Inserted::Done);
        assert_eq!(memo.insert(&key(1), entry, &account), Inserted::Done);
        assert_eq!(memo.insert(&key(3), entry, &account), Inserted::Full);
        let held = account.used();
        memo.clear();
        assert_eq!(memo.insert(&key(3), entry, &account), Inserted::Done);
        assert!(memo.refuted(&key(3)) && !memo.refuted(&key(1)));
        assert_eq!(account.used(), held, "the emptied table keeps its memory");
        memo.release(&account);
        assert_eq!(account.used(), 0);
        assert!(!memo.refuted(&key(3)));

        // Under a bound the table grows in chunks of a sixteenth of the
        // room, here one record of 35 words, until one no longer fits.
        let account = Account::new(Some(4096));
        let mut memo = Memo::new(1000);
        let fitted = (0..1000)
            .take_while(|&member| memo.insert(&key(member), entry, &account) == Inserted::Done)
            .count();
        assert!((6..16).contains(&fitted), "{fitted} entries in 4 KiB");
        assert!(!account.over() && !account.spares(35 * 8));
        assert_eq!(memo.insert(&key(1000), entry, &account), Inserted::Full);
        memo.clear();
        assert_eq!(memo.insert(&key(1000), entry, &account), Inserted::Done);
        assert_eq!(memo.peak(), fitted);
        let mut starved = Memo::new(100);
        assert_eq!(
            starved.insert(&key(0), entry, &Account::new(Some(100))),
            Inserted::NoRoom
        );
    }

    /// A key with extra copies in its linear zone is another key than the
    /// one without, and each is found again among many.
    #[test]
    fn records() {
        let account = Account::new(None);
        let mut memo = Memo::with_chunk(1000, 64);
        let mut twice = key(7);
        twice.gamma.insert(OccId::new(7));
        memo.insert(&twice, Entry::Proved(NodeId::new(9)), &account);
        for member in 0..100 {
            memo.insert(
                &key(member),
                Entry::Failed(Failure::Exhausted(member)),
                &account,
            );
        }
        assert_eq!(memo.get(&twice, 0), Some(Entry::Proved(NodeId::new(9))));
        for member in 0..100 {
            assert_eq!(
                memo.get(&key(member), 0),
                Some(Entry::Failed(Failure::Exhausted(member)))
            );
        }
        assert_eq!(memo.get(&key(100), 0), None);
    }
}
