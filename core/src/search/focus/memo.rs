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
//! which is all a relative reads there. The table is a plain map with a
//! cap; when the search runs on
//! several threads it is one shard of a sharded map, [`Shared`], whose
//! merge under the shard's lock keeps the same invariant: an entry's
//! validity only ever grows.

use super::context::Context;
use crate::hash::HashMap;
use crate::occurrences::OccSet;
use crate::proofs::NodeId;
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

/// The memo: stable sequents mapped to what the search found out about
/// them, with at most `limit` entries. When the table is full it is
/// emptied, which only costs time: every entry is a fact the search can
/// find again.
#[derive(Debug)]
pub(crate) struct Memo {
    /// The entries.
    map: HashMap<Key, Entry>,
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
        Self {
            map: HashMap::default(),
            limit,
            peak: 0,
            hits: 0,
        }
    }

    /// Returns what is known about a stable sequent that applies with
    /// `remaining` copies left: a proof or a complete failure always, a
    /// failure cut by the budget only when at most as many copies are left
    /// now as were then, since a larger budget could prove more.
    pub(crate) fn get(&mut self, key: &Key, remaining: u32) -> Option<Entry> {
        let entry = self.map.get(key).copied()?;
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
        let refuted = self.map.get(key) == Some(&Entry::Failed(Failure::Complete));
        self.hits += u64::from(refuted);
        refuted
    }

    /// Records what the search found out about a stable sequent, making
    /// room by emptying the table if it is full. A proof or a complete
    /// failure replaces anything; a failure cut by the budget only raises
    /// the budget an earlier such failure recorded.
    pub(crate) fn insert(&mut self, key: &Key, entry: Entry) {
        if self.limit == 0 {
            return;
        }
        if let Some(old) = self.map.get_mut(key) {
            match (*old, entry) {
                (
                    Entry::Failed(Failure::Exhausted(then)),
                    Entry::Failed(Failure::Exhausted(now)),
                ) if now <= then => {}
                (Entry::Proved(_) | Entry::Failed(Failure::Complete), Entry::Failed(_)) => {}
                _ => *old = entry,
            }
            return;
        }
        if self.map.len() >= self.limit {
            self.map.clear();
        }
        self.map.insert(key.clone(), entry);
        self.peak = self.peak.max(self.map.len());
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
/// weaker entry. The cap is per shard.
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
                .map(|_| Mutex::new(Memo::new(if limit == 0 { 0 } else { per_shard })))
                .collect(),
        }
    }

    /// The shard of a key.
    fn shard(&self, key: &Key) -> &Mutex<Memo> {
        let hash = crate::hash::BuildHasher::default().hash_one(key);
        &self.shards[(hash >> (64 - SHARDS.trailing_zeros())) as usize]
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
        Self::lock(self.shard(key)).get(key, remaining)
    }

    /// [`Memo::refuted`] on the key's shard.
    pub(crate) fn refuted(&self, key: &Key) -> bool {
        Self::lock(self.shard(key)).refuted(key)
    }

    /// [`Memo::insert`] on the key's shard.
    pub(crate) fn insert(&self, key: &Key, entry: Entry) {
        Self::lock(self.shard(key)).insert(key, entry);
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

    /// [`Memo::insert`].
    pub(crate) fn insert(&mut self, key: &Key, entry: Entry) {
        match self {
            Self::Own(memo) => memo.insert(key, entry),
            Self::Shared(shared) => shared.insert(key, entry),
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
    use crate::occurrences::OccId;

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
        let mut memo = Memo::new(10);
        assert!(!memo.refuted(&key));
        memo.insert(&key, Entry::Failed(Failure::Exhausted(1)));
        assert!(!memo.refuted(&key));
        memo.insert(&key, Entry::Failed(Failure::Complete));
        assert!(memo.refuted(&key));
        let mut proved = Memo::new(10);
        proved.insert(&key, Entry::Proved(NodeId::new(4)));
        assert!(!proved.refuted(&key));
        assert_eq!((memo.hits(), proved.hits()), (1, 0));
    }

    /// A failure cut by the budget is a hit only with at most as many
    /// copies left, a complete failure and a proof always; a later entry
    /// only strengthens what is known.
    #[test]
    fn bounded_failures() {
        let mut memo = Memo::new(10);
        let mut key = Key {
            theta: OccSet::empty(8),
            gamma: Context::empty(8),
        };
        key.gamma.insert(OccId::new(1));
        assert_eq!(memo.get(&key, 0), None);
        memo.insert(&key, Entry::Failed(Failure::Exhausted(1)));
        assert_eq!(memo.get(&key, 2), None, "more copies left now");
        assert_eq!(
            memo.get(&key, 1),
            Some(Entry::Failed(Failure::Exhausted(1)))
        );
        assert_eq!(
            memo.get(&key, 0),
            Some(Entry::Failed(Failure::Exhausted(1)))
        );
        memo.insert(&key, Entry::Failed(Failure::Exhausted(0)));
        assert_eq!(
            memo.get(&key, 1),
            Some(Entry::Failed(Failure::Exhausted(1))),
            "a smaller budget does not weaken the entry"
        );
        memo.insert(&key, Entry::Failed(Failure::Exhausted(3)));
        assert_eq!(
            memo.get(&key, 3),
            Some(Entry::Failed(Failure::Exhausted(3)))
        );
        memo.insert(&key, Entry::Failed(Failure::Complete));
        assert_eq!(memo.get(&key, 100), Some(Entry::Failed(Failure::Complete)));
        memo.insert(&key, Entry::Failed(Failure::Exhausted(5)));
        assert_eq!(
            memo.get(&key, 100),
            Some(Entry::Failed(Failure::Complete)),
            "a complete failure stays"
        );
        let proved = Entry::Proved(NodeId::new(7));
        memo.insert(&key, proved);
        assert_eq!(memo.get(&key, 0), Some(proved));
        memo.insert(&key, Entry::Failed(Failure::Complete));
        assert_eq!(memo.get(&key, 0), Some(proved), "a proof stays");
        assert_eq!(memo.hits(), 8);
        assert_eq!(memo.peak(), 1);
    }
}
