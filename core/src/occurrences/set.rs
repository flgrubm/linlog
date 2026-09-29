// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::{Forest, OccId};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign};

/// A set of occurrence ids of one forest: a bitset whose width is fixed when
/// it is made, one bit per occurrence. Sets of the same forest have the same
/// width, so they compare, hash and combine word by word.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OccSet {
    /// Bit `i` of word `i / 64` is set when occurrence `i` is a member.
    words: Box<[u64]>,
}

impl OccSet {
    /// Returns the empty set over `len` occurrence ids.
    pub fn empty(len: usize) -> Self {
        Self {
            words: vec![0; len.div_ceil(64)].into_boxed_slice(),
        }
    }

    /// Returns the set over `len` occurrence ids that holds the given ones.
    pub fn of(len: usize, ids: impl IntoIterator<Item = OccId>) -> Self {
        let mut set = Self::empty(len);
        set.extend(ids);
        set
    }

    /// Returns the number of ids the set can hold, which is its width rounded
    /// up to a multiple of 64.
    pub fn capacity(&self) -> usize {
        self.words.len() * 64
    }

    /// Returns the words of the bitset, lowest ids first.
    pub fn words(&self) -> &[u64] {
        &self.words
    }

    /// Returns the word and the bit mask that hold `o`.
    fn locate(o: OccId) -> (usize, u64) {
        (o.index() / 64, 1 << (o.get() % 64))
    }

    /// Adds `o` and returns whether it was absent.
    pub fn insert(&mut self, o: OccId) -> bool {
        let (w, bit) = Self::locate(o);
        let absent = self.words[w] & bit == 0;
        self.words[w] |= bit;
        absent
    }

    /// Removes `o` and returns whether it was present.
    pub fn remove(&mut self, o: OccId) -> bool {
        let (w, bit) = Self::locate(o);
        let present = self.words[w] & bit != 0;
        self.words[w] &= !bit;
        present
    }

    /// Adds `o` if it is absent and removes it otherwise.
    pub fn toggle(&mut self, o: OccId) {
        let (w, bit) = Self::locate(o);
        self.words[w] ^= bit;
    }

    /// Returns whether `o` is a member.
    pub fn contains(&self, o: OccId) -> bool {
        let (w, bit) = Self::locate(o);
        self.words[w] & bit != 0
    }

    /// Removes every member.
    pub fn clear(&mut self) {
        self.words.fill(0);
    }

    /// Returns whether the set has no member.
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    /// Returns the number of members.
    pub fn len(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// Returns the smallest member, if any.
    pub fn first(&self) -> Option<OccId> {
        self.iter().next()
    }

    /// Returns the members in ascending order.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            words: &self.words,
            index: 0,
            current: self.words.first().copied().unwrap_or(0),
        }
    }

    /// Adds every member of `other`.
    pub fn union_with(&mut self, other: &Self) {
        self.zip_with(other, |a, b| a | b);
    }

    /// Keeps only the members `other` has too.
    pub fn intersect_with(&mut self, other: &Self) {
        self.zip_with(other, |a, b| a & b);
    }

    /// Removes every member of `other`.
    pub fn difference_with(&mut self, other: &Self) {
        self.zip_with(other, |a, b| a & !b);
    }

    /// Combines the sets word by word; both must have the same width.
    fn zip_with(&mut self, other: &Self, f: impl Fn(u64, u64) -> u64) {
        debug_assert_eq!(self.words.len(), other.words.len(), "sets of one forest");
        for (a, b) in self.words.iter_mut().zip(&other.words) {
            *a = f(*a, *b);
        }
    }

    /// Returns whether every member is a member of `other`.
    pub fn is_subset(&self, other: &Self) -> bool {
        self.words
            .iter()
            .zip(&other.words)
            .all(|(a, b)| a & !b == 0)
    }

    /// Returns whether no member is a member of `other`.
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.words.iter().zip(&other.words).all(|(a, b)| a & b == 0)
    }
}

impl Extend<OccId> for OccSet {
    /// Adds every id.
    fn extend<I: IntoIterator<Item = OccId>>(&mut self, ids: I) {
        for o in ids {
            self.insert(o);
        }
    }
}

impl<'a> IntoIterator for &'a OccSet {
    type Item = OccId;
    type IntoIter = Iter<'a>;

    /// Returns the members in ascending order.
    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl Debug for OccSet {
    /// Writes the members in ascending order, as `{0, 3, 7}`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_set().entries(self.iter().map(|o| o.get())).finish()
    }
}

impl BitOrAssign<&OccSet> for OccSet {
    /// Adds every member of `other`.
    fn bitor_assign(&mut self, other: &OccSet) {
        self.union_with(other);
    }
}

impl BitAndAssign<&OccSet> for OccSet {
    /// Keeps only the members `other` has too.
    fn bitand_assign(&mut self, other: &OccSet) {
        self.intersect_with(other);
    }
}

impl SubAssign<&OccSet> for OccSet {
    /// Removes every member of `other`.
    fn sub_assign(&mut self, other: &OccSet) {
        self.difference_with(other);
    }
}

impl BitOr for &OccSet {
    type Output = OccSet;

    /// The union.
    fn bitor(self, other: Self) -> OccSet {
        let mut set = self.clone();
        set |= other;
        set
    }
}

impl BitAnd for &OccSet {
    type Output = OccSet;

    /// The intersection.
    fn bitand(self, other: Self) -> OccSet {
        let mut set = self.clone();
        set &= other;
        set
    }
}

impl Sub for &OccSet {
    type Output = OccSet;

    /// The difference: the members of the left set that the right one lacks.
    fn sub(self, other: Self) -> OccSet {
        let mut set = self.clone();
        set -= other;
        set
    }
}

/// The members of an [`OccSet`] in ascending order.
#[derive(Clone, Debug)]
pub struct Iter<'a> {
    /// The words still to visit, from `index` on.
    words: &'a [u64],
    /// The word `current` was taken from.
    index: usize,
    /// The bits of word `index` not yet yielded.
    current: u64,
}

impl Iterator for Iter<'_> {
    type Item = OccId;

    /// Returns the next member.
    fn next(&mut self) -> Option<OccId> {
        while self.current == 0 {
            self.index += 1;
            self.current = *self.words.get(self.index)?;
        }
        let bit = self.current.trailing_zeros();
        self.current &= self.current - 1;
        Some(OccId::new((self.index * 64) as u32 + bit))
    }

    /// The exact number of members left.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rest = self.words.get(self.index + 1..).unwrap_or_default();
        let n = self.current.count_ones() as usize
            + rest.iter().map(|w| w.count_ones() as usize).sum::<usize>();
        (n, Some(n))
    }
}

impl ExactSizeIterator for Iter<'_> {}

impl Forest {
    /// Returns the empty set over this forest's occurrences.
    pub fn empty_set(&self) -> OccSet {
        OccSet::empty(self.len())
    }

    /// Returns the set of the root formulas: the sequent itself, as proof
    /// search starts on it.
    pub fn root_set(&self) -> OccSet {
        OccSet::of(self.len(), self.roots().iter().copied())
    }
}

/// One step of a submask enumeration: the submask reached and the position
/// that changed to reach it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flip {
    /// The submask, as a bit per position of the member list.
    pub mask: u64,
    /// The position whose membership changed since the previous submask.
    pub position: u32,
}

/// Returns every non-empty submask of a list of `len` members, `len < 64`,
/// in Gray-code order: each differs from the one before in exactly one
/// position, which the [`Flip`] names, so a set and its complement in the
/// list are kept up to date by toggling one occurrence per step. The
/// enumeration starts from the empty submask, which is not returned.
pub fn submasks(len: usize) -> Submasks {
    assert!(
        len < 64,
        "a member list to enumerate has at most 63 members"
    );
    Submasks {
        next: 1,
        end: 1 << len,
    }
}

/// The submasks of a member list in Gray-code order, see [`submasks`].
#[derive(Clone, Debug)]
pub struct Submasks {
    /// The index of the next submask in the Gray sequence.
    next: u64,
    /// One past the index of the last submask.
    end: u64,
}

impl Iterator for Submasks {
    type Item = Flip;

    /// Returns the next submask.
    fn next(&mut self) -> Option<Flip> {
        if self.next == self.end {
            return None;
        }
        let i = self.next;
        self.next += 1;
        Some(Flip {
            mask: i ^ (i >> 1),
            position: i.trailing_zeros(),
        })
    }

    /// The exact number of submasks left.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = (self.end - self.next) as usize;
        (n, Some(n))
    }
}

impl ExactSizeIterator for Submasks {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::HashSet;

    /// Wraps a raw id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Members go in and out, across word boundaries, and come back in
    /// order.
    #[test]
    fn membership() {
        let mut s = OccSet::empty(130);
        assert_eq!(s.capacity(), 192);
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
        assert_eq!(s.first(), None);
        assert!(s.insert(o(129)));
        assert!(s.insert(o(64)));
        assert!(s.insert(o(3)));
        assert!(!s.insert(o(3)));
        assert!(s.contains(o(3)) && s.contains(o(64)) && s.contains(o(129)));
        assert!(!s.contains(o(63)) && !s.contains(o(65)));
        assert_eq!(s.len(), 3);
        assert_eq!(s.first(), Some(o(3)));
        assert_eq!(s.iter().collect::<Vec<_>>(), [o(3), o(64), o(129)]);
        assert_eq!(s.iter().len(), 3);
        assert_eq!(format!("{s:?}"), "{3, 64, 129}");
        assert!(s.remove(o(64)));
        assert!(!s.remove(o(64)));
        s.toggle(o(0));
        s.toggle(o(3));
        assert_eq!(s.iter().collect::<Vec<_>>(), [o(0), o(129)]);
        s.clear();
        assert!(s.is_empty());
        assert_eq!(OccSet::empty(0).iter().count(), 0);
    }

    /// Union, intersection, difference and the inclusion tests agree with
    /// their definitions on members.
    #[test]
    fn algebra() {
        let a = OccSet::of(70, [o(1), o(2), o(65)]);
        let b = OccSet::of(70, [o(2), o(3), o(69)]);
        assert_eq!(
            (&a | &b).iter().collect::<Vec<_>>(),
            [o(1), o(2), o(3), o(65), o(69)]
        );
        assert_eq!((&a & &b).iter().collect::<Vec<_>>(), [o(2)]);
        assert_eq!((&a - &b).iter().collect::<Vec<_>>(), [o(1), o(65)]);
        assert!(!a.is_subset(&b));
        assert!((&a & &b).is_subset(&a));
        assert!(!a.is_disjoint(&b));
        assert!((&a - &b).is_disjoint(&b));
        let mut c = a.clone();
        c |= &b;
        assert_eq!(c, &a | &b);
        c &= &b;
        assert_eq!(c, b);
        c -= &a;
        assert_eq!(c, &b - &a);
        let mut d = OccSet::empty(70);
        d.extend([o(65), o(1), o(2)]);
        assert_eq!(d, a);
    }

    /// Equal sets are equal and hash alike; the words are what gets hashed.
    #[test]
    fn hashing() {
        let mut seen = HashSet::default();
        assert!(seen.insert(OccSet::of(100, [o(5), o(80)])));
        assert!(!seen.insert(OccSet::of(100, [o(80), o(5)])));
        assert!(seen.insert(OccSet::of(100, [o(5)])));
        assert_eq!(OccSet::of(100, [o(5), o(80)]).words(), [1 << 5, 1 << 16]);
    }

    /// The forest hands out sets of its width, and its roots as a set.
    #[cfg(feature = "parse")]
    #[test]
    fn forest_sets() {
        let s: crate::Sequent = "A, A -o B |- B".parse().unwrap();
        let f = Forest::new(&s).unwrap();
        assert_eq!(f.empty_set().capacity(), 64);
        assert!(f.empty_set().is_empty());
        assert_eq!(f.root_set().iter().collect::<Vec<_>>(), f.roots());
    }

    /// Every submask of a list shows up exactly once, and each step flips
    /// exactly the position it names.
    #[test]
    fn gray_submasks() {
        for len in 0..6 {
            let mut seen = vec![false; 1 << len];
            seen[0] = true;
            let mut previous = 0u64;
            let steps = submasks(len);
            assert_eq!(steps.len(), (1 << len) - 1);
            for flip in steps {
                assert_eq!(
                    previous ^ flip.mask,
                    1 << flip.position,
                    "one flip per step"
                );
                assert!(!seen[flip.mask as usize], "each submask once");
                seen[flip.mask as usize] = true;
                previous = flip.mask;
            }
            assert!(seen.iter().all(|&s| s), "every submask");
        }
    }

    /// Toggling the named member keeps a set and its complement in step
    /// with the mask.
    #[test]
    fn gray_split() {
        let members = [o(2), o(7), o(9), o(70)];
        let all = OccSet::of(80, members);
        let mut left = OccSet::empty(80);
        let mut right = all.clone();
        for flip in submasks(members.len()) {
            let m = members[flip.position as usize];
            left.toggle(m);
            right.toggle(m);
            let expected: Vec<OccId> = (0..members.len())
                .filter(|&i| flip.mask >> i & 1 == 1)
                .map(|i| members[i])
                .collect();
            assert_eq!(left.iter().collect::<Vec<_>>(), expected);
            assert_eq!(&left | &right, all);
            assert!(left.is_disjoint(&right));
        }
    }
}
