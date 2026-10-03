// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The memory a search may hold: a count of the bytes its growing
//! structures have allocated, against the bound of the options. What
//! grows charges it where it allocates (the memo, the proof arena, the
//! counts, the buffers a level of recursion takes), by the capacity
//! allocated and not by what is in use, so the count follows the memory
//! the process holds; what was freed releases it. The count is atomic
//! because the workers of a parallel search share one account; one thread
//! pays an uncontended addition per allocation, which is rare by design.

use std::sync::atomic::{AtomicU64, Ordering};

/// The bytes a search holds, and the most it may.
#[derive(Debug)]
pub(crate) struct Account {
    /// The bound, or `u64::MAX` for none.
    limit: u64,
    /// The bytes charged and not released. A sum of sizes of live
    /// allocations, each below `isize::MAX`, of which an address space
    /// holds fewer than 2⁶⁴ bytes' worth: it cannot wrap.
    used: AtomicU64,
}

impl Account {
    /// An account with nothing charged and the bound given, or none.
    pub(crate) fn new(limit: Option<u64>) -> Self {
        Self {
            limit: limit.unwrap_or(u64::MAX),
            used: AtomicU64::new(0),
        }
    }

    /// An account for one of `parts` searches that run side by side: the
    /// same bound divided evenly, so that neither search's memory decides
    /// what the other may keep.
    pub(crate) fn share(&self, parts: u64) -> Self {
        Self {
            limit: if self.limit == u64::MAX {
                u64::MAX
            } else {
                self.limit / parts
            },
            used: AtomicU64::new(0),
        }
    }

    /// An account with the same bound that starts from what this one
    /// holds now: for a search that starts afresh beside what was set up
    /// for it, and whose own memory goes when it ends.
    pub(crate) fn fork(&self) -> Self {
        Self {
            limit: self.limit,
            used: AtomicU64::new(self.used()),
        }
    }

    /// Returns the bound, or `u64::MAX` for none.
    pub(crate) fn limit(&self) -> u64 {
        self.limit
    }

    /// Counts `bytes` more as held.
    pub(crate) fn charge(&self, bytes: usize) {
        self.used.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    /// Counts `bytes` as given back, which must have been charged.
    pub(crate) fn release(&self, bytes: usize) {
        self.used.fetch_sub(bytes as u64, Ordering::Relaxed);
    }

    /// Counts a buffer whose allocation went from `before` to `after`
    /// bytes.
    pub(crate) fn resize(&self, before: usize, after: usize) {
        if after > before {
            self.charge(after - before);
        } else if before > after {
            self.release(before - after);
        }
    }

    /// Returns the bytes held.
    pub(crate) fn used(&self) -> u64 {
        self.used.load(Ordering::Relaxed)
    }

    /// Returns whether `bytes` more would still be within the bound.
    pub(crate) fn fits(&self, bytes: usize) -> bool {
        self.used().saturating_add(bytes as u64) <= self.limit
    }

    /// Returns whether `bytes` more would leave an eighth of the bound
    /// free. The memo asks this before it grows: it would take whatever
    /// is left, and what cannot be emptied (the proofs kept, the buffers
    /// of the recursion) would then pass the bound with every node, each
    /// time at the price of the whole memo.
    pub(crate) fn spares(&self, bytes: usize) -> bool {
        self.used()
            .saturating_add(bytes as u64)
            .saturating_add(self.limit / 8)
            <= self.limit
    }

    /// Returns whether more is held than the bound allows.
    pub(crate) fn over(&self) -> bool {
        self.used() > self.limit
    }
}

/// What one engine charged an account for buffers of its own, which go
/// when the engine does: dropping this releases them. The workers of a
/// parallel search come and go by the thousand, each with its buffers.
#[derive(Debug)]
pub(crate) struct Charged<'a> {
    /// The account.
    account: &'a Account,
    /// The bytes charged through this value and not released.
    bytes: usize,
}

impl<'a> Charged<'a> {
    /// Nothing charged to `account` yet.
    pub(crate) fn new(account: &'a Account) -> Self {
        Self { account, bytes: 0 }
    }

    /// Returns the account.
    pub(crate) fn account(&self) -> &'a Account {
        self.account
    }

    /// Counts `bytes` more as held.
    pub(crate) fn charge(&mut self, bytes: usize) {
        self.bytes += bytes;
        self.account.charge(bytes);
    }

    /// Counts a buffer whose allocation went from `before` to `after`
    /// bytes.
    pub(crate) fn resize(&mut self, before: usize, after: usize) {
        self.bytes = self.bytes + after - before;
        self.account.resize(before, after);
    }
}

impl Drop for Charged<'_> {
    fn drop(&mut self) {
        self.account.release(self.bytes);
    }
}

/// Returns the bytes a vector's allocation takes.
pub(crate) fn bytes_of<T>(vector: &Vec<T>) -> usize {
    vector.capacity() * size_of::<T>()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A charge counts, a release takes it back, and the bound is
    /// inclusive; a share has its part of the bound and a count of its own.
    #[test]
    fn accounting() {
        let account = Account::new(Some(100));
        account.charge(60);
        assert!(account.fits(40) && !account.fits(41) && !account.over());
        assert!(account.spares(28) && !account.spares(29));
        account.resize(60, 110);
        assert!(account.over());
        account.release(20);
        assert_eq!(account.used(), 90);
        let half = account.share(2);
        assert!(half.fits(50) && !half.fits(51));
        assert!(Account::new(None).share(2).fits(usize::MAX));
        let before = account.used();
        let mut own = Charged::new(&account);
        own.charge(7);
        own.resize(7, 3);
        assert_eq!(account.used(), before + 3);
        drop(own);
        assert_eq!(account.used(), before);
    }
}
