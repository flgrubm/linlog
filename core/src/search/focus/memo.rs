// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The memo of stable sequents. An entry is a fact about a set of
//! occurrences that holds regardless of how the search reached it, so the
//! table is a plain map with a cap; a sharded map can replace it when the
//! search runs on several threads.

use crate::hash::HashMap;
use crate::occurrences::OccSet;
use crate::proofs::NodeId;

/// What the search found out about a stable sequent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    /// Provable, by the subproof rooted at this node of the engine's arena.
    Proved(NodeId),
    /// Unprovable.
    Failed,
}

/// The memo: stable sequents mapped to what the search found out about
/// them, with at most `limit` entries. When the table is full it is
/// emptied, which only costs time: every entry is a fact the search can
/// find again.
#[derive(Debug)]
pub(crate) struct Memo {
    /// The entries.
    map: HashMap<OccSet, Entry>,
    /// The number of entries the table holds at most; zero switches the memo
    /// off.
    limit: usize,
    /// The most entries the table held at once.
    peak: usize,
    /// How many lookups found an entry.
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

    /// Returns what is known about a stable sequent.
    pub(crate) fn get(&mut self, key: &OccSet) -> Option<Entry> {
        let entry = self.map.get(key).copied();
        if entry.is_some() {
            self.hits += 1;
        }
        entry
    }

    /// Records what the search found out about a stable sequent, making
    /// room by emptying the table if it is full.
    pub(crate) fn insert(&mut self, key: &OccSet, entry: Entry) {
        if self.limit == 0 {
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

    /// Returns how many lookups found an entry.
    pub(crate) fn hits(&self) -> u64 {
        self.hits
    }
}
