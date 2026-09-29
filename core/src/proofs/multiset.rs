// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::occurrences::OccId;

/// A multiset of occurrence ids, kept as an ascending list with repeats: the
/// linear zone of a sequent, which holds an occurrence more than once after
/// copies from the unrestricted zone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Multiset(Vec<OccId>);

impl Multiset {
    /// Returns the empty multiset.
    pub(crate) const fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns the multiset of the given ids, in any order.
    pub(crate) fn of(ids: impl IntoIterator<Item = OccId>) -> Self {
        let mut ids: Vec<OccId> = ids.into_iter().collect();
        ids.sort_unstable();
        Self(ids)
    }

    /// Returns the members in ascending order, with repeats.
    pub(crate) fn as_slice(&self) -> &[OccId] {
        &self.0
    }

    /// Returns the members as a vector in ascending order, with repeats.
    pub(crate) fn into_vec(self) -> Vec<OccId> {
        self.0
    }

    /// Returns whether the multiset has no member.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the position of the first copy of `o`, if it is a member.
    pub(crate) fn position(&self, o: OccId) -> Option<usize> {
        let i = self.0.partition_point(|&x| x < o);
        (self.0.get(i) == Some(&o)).then_some(i)
    }

    /// Returns whether `o` is a member.
    pub(crate) fn contains(&self, o: OccId) -> bool {
        self.position(o).is_some()
    }

    /// Adds one copy of `o`.
    pub(crate) fn insert(&mut self, o: OccId) {
        let i = self.0.partition_point(|&x| x <= o);
        self.0.insert(i, o);
    }

    /// Adds `o` if it is not a member.
    pub(crate) fn ensure(&mut self, o: OccId) {
        if !self.contains(o) {
            self.insert(o);
        }
    }

    /// Removes one copy of `o` and returns whether there was one.
    pub(crate) fn remove(&mut self, o: OccId) -> bool {
        match self.position(o) {
            Some(i) => {
                self.0.remove(i);
                true
            }
            None => false,
        }
    }

    /// Returns the multiset with, per id, `f` of the two counts many copies.
    fn merge(&self, other: &Self, f: impl Fn(usize, usize) -> usize) -> Self {
        let (a, b) = (&self.0, &other.0);
        let mut out = Vec::with_capacity(a.len() + b.len());
        let (mut i, mut j) = (0, 0);
        while i < a.len() || j < b.len() {
            let o = match (a.get(i), b.get(j)) {
                (Some(&x), Some(&y)) => x.min(y),
                (Some(&x), None) => x,
                (None, Some(&y)) => y,
                (None, None) => unreachable!(),
            };
            let run = |v: &[OccId], k: usize| v[k..].iter().take_while(|&&x| x == o).count();
            let (ca, cb) = (run(a, i), run(b, j));
            out.extend(std::iter::repeat_n(o, f(ca, cb)));
            i += ca;
            j += cb;
        }
        Self(out)
    }

    /// Returns the multiset sum: the counts add up.
    pub(crate) fn sum(&self, other: &Self) -> Self {
        self.merge(other, |a, b| a + b)
    }

    /// Returns the multiset union: per id, the larger count.
    pub(crate) fn union(&self, other: &Self) -> Self {
        self.merge(other, usize::max)
    }

    /// Returns the multiset difference: per id, this count less the other's,
    /// if positive.
    pub(crate) fn difference(&self, other: &Self) -> Self {
        self.merge(other, usize::saturating_sub)
    }

    /// Returns whether no id has more copies here than in `other`.
    pub(crate) fn is_subset(&self, other: &Self) -> bool {
        self.difference(other).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wraps a raw id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Members are kept sorted with repeats, and the operations count.
    #[test]
    fn counts() {
        let mut m = Multiset::of([o(5), o(2), o(5)]);
        assert_eq!(m.as_slice(), [o(2), o(5), o(5)]);
        assert_eq!(m.position(o(5)), Some(1));
        assert_eq!(m.position(o(3)), None);
        m.insert(o(3));
        m.ensure(o(3));
        m.ensure(o(9));
        assert_eq!(m.as_slice(), [o(2), o(3), o(5), o(5), o(9)]);
        assert!(m.remove(o(5)));
        assert!(m.remove(o(5)));
        assert!(!m.remove(o(5)));
        assert_eq!(m.as_slice(), [o(2), o(3), o(9)]);

        let a = Multiset::of([o(1), o(1), o(2)]);
        let b = Multiset::of([o(1), o(2), o(2), o(4)]);
        assert_eq!(
            a.sum(&b).as_slice(),
            [o(1), o(1), o(1), o(2), o(2), o(2), o(4)]
        );
        assert_eq!(a.union(&b).as_slice(), [o(1), o(1), o(2), o(2), o(4)]);
        assert_eq!(a.difference(&b).as_slice(), [o(1)]);
        assert_eq!(b.difference(&a).as_slice(), [o(2), o(4)]);
        assert!(!a.is_subset(&b));
        assert!(a.difference(&b).is_subset(&a));
        assert!(Multiset::new().is_subset(&a));
        assert!(Multiset::new().is_empty());
    }
}
