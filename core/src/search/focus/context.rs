// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The linear zone of a sequent inside the focused engine: a multiset of
//! occurrence ids, kept as a bitset of the occurrences present plus a list
//! of the extra copies, which is empty until a copy from the unrestricted
//! zone repeats an occurrence, so that the common case costs a bitset and
//! only a repeat allocates.

use crate::occurrences::{OccId, OccSet};

/// A multiset of occurrence ids over one forest.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Context {
    /// The occurrences present at least once.
    set: OccSet,
    /// Per occurrence present more than once, in ascending id order, how
    /// many copies beyond the first it has.
    extra: Vec<(OccId, u32)>,
}

impl Context {
    /// Returns the empty zone over `len` occurrence ids.
    pub(crate) fn empty(len: usize) -> Self {
        Self {
            set: OccSet::empty(len),
            extra: Vec::new(),
        }
    }

    /// Returns the occurrences present, each once.
    pub(crate) fn set(&self) -> &OccSet {
        &self.set
    }

    /// Returns the extra copies: per occurrence present more than once, in
    /// ascending id order, how many copies beyond the first it has.
    pub(crate) fn extra(&self) -> &[(OccId, u32)] {
        &self.extra
    }

    /// Where `o` is or would be in the extra list.
    fn slot(&self, o: OccId) -> Result<usize, usize> {
        self.extra.binary_search_by_key(&o, |&(x, _)| x)
    }

    /// Adds one copy of `o`.
    pub(crate) fn insert(&mut self, o: OccId) {
        if self.set.insert(o) {
            return;
        }
        match self.slot(o) {
            Ok(i) => self.extra[i].1 += 1,
            Err(i) => self.extra.insert(i, (o, 1)),
        }
    }

    /// Removes one copy of `o`, which must be a member.
    pub(crate) fn remove(&mut self, o: OccId) {
        debug_assert!(self.set.contains(o), "removing a member");
        if let Ok(i) = self.slot(o) {
            if self.extra[i].1 == 1 {
                self.extra.remove(i);
            } else {
                self.extra[i].1 -= 1;
            }
        } else {
            self.set.remove(o);
        }
    }

    /// Returns whether `o` is a member.
    pub(crate) fn contains(&self, o: OccId) -> bool {
        self.set.contains(o)
    }

    /// Returns how many copies of `o` there are.
    pub(crate) fn count(&self, o: OccId) -> u32 {
        if !self.set.contains(o) {
            return 0;
        }
        1 + self.slot(o).map_or(0, |i| self.extra[i].1)
    }

    /// Removes every member.
    pub(crate) fn clear(&mut self) {
        self.set.clear();
        self.extra.clear();
    }

    /// Makes this zone a copy of `other`, reusing the buffers.
    pub(crate) fn clone_from(&mut self, other: &Self) {
        self.set.clone_from(&other.set);
        self.extra.clone_from(&other.extra);
    }

    /// Returns whether the zone has no member.
    pub(crate) fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Returns the number of members, copies counted.
    pub(crate) fn len(&self) -> usize {
        self.set.len() + self.extra.iter().map(|&(_, n)| n as usize).sum::<usize>()
    }

    /// Returns the members in ascending order, an occurrence repeated as
    /// often as it is present.
    pub(crate) fn iter(&self) -> impl Iterator<Item = OccId> + '_ {
        self.set.iter().flat_map(move |o| {
            let copies = 1 + self.slot(o).map_or(0, |i| self.extra[i].1);
            std::iter::repeat_n(o, copies as usize)
        })
    }

    /// Returns whether every member of `other` is a member here at least as
    /// often: the multiset inclusion `other ⊆ self`.
    pub(crate) fn includes(&self, other: &Self) -> bool {
        other.set.is_subset(&self.set)
            && other
                .extra
                .iter()
                .all(|&(o, n)| self.slot(o).is_ok_and(|i| self.extra[i].1 >= n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wraps a raw id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Copies go in and out one at a time, the count and the length follow,
    /// and the extra list appears only for repeats.
    #[test]
    fn copies() {
        let mut c = Context::empty(70);
        assert!(c.is_empty());
        c.insert(o(65));
        c.insert(o(3));
        assert!(c.extra().is_empty());
        c.insert(o(3));
        c.insert(o(3));
        assert_eq!(c.extra(), [(o(3), 2)]);
        assert_eq!(c.count(o(3)), 3);
        assert_eq!(c.count(o(65)), 1);
        assert_eq!(c.count(o(4)), 0);
        assert_eq!(c.len(), 4);
        assert_eq!(c.iter().collect::<Vec<_>>(), [o(3), o(3), o(3), o(65)]);
        c.remove(o(3));
        assert_eq!(c.extra(), [(o(3), 1)]);
        c.remove(o(3));
        assert!(c.extra().is_empty());
        assert!(c.contains(o(3)));
        c.remove(o(3));
        assert!(!c.contains(o(3)));
        assert_eq!(c.iter().collect::<Vec<_>>(), [o(65)]);
        let mut d = Context::empty(70);
        d.clone_from(&c);
        assert_eq!(c, d);
        c.clear();
        assert!(c.is_empty());
    }

    /// Multiset inclusion counts copies.
    #[test]
    fn inclusion() {
        let mut a = Context::empty(10);
        let mut b = Context::empty(10);
        a.insert(o(1));
        a.insert(o(1));
        a.insert(o(2));
        b.insert(o(1));
        assert!(a.includes(&b));
        assert!(!b.includes(&a));
        b.insert(o(1));
        assert!(a.includes(&b));
        b.insert(o(1));
        assert!(!a.includes(&b));
        assert!(a.includes(&a));
    }
}
