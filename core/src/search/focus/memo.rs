// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The memo of stable sequents. A proved sequent is a fact about its two
//! zones that holds regardless of how the search reached it; a failed one
//! is a fact only relative to the copy budget that was left when it failed,
//! unless the search space below it was explored without ever hitting the
//! budget. The table is a plain map with a cap; a sharded map can replace
//! it when the search runs on several threads.

use super::context::Context;
use crate::hash::HashMap;
use crate::occurrences::OccSet;
use crate::proofs::NodeId;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::occurrences::OccId;

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
