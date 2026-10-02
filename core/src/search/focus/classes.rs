// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Interchangeable occurrences. Two occurrences of the same formula are
//! distinct members of a sequent, but nothing a proof does tells them
//! apart: every rule looks at an occurrence's connective, its subformulas
//! and, two-sided, its position, and all of these are the same for both.
//! So a sequent stays as provable, by a proof of the same shape, when a
//! member is replaced by another occurrence of the same formula in the
//! same position, and the search need only try one of each kind.

use crate::occurrences::{Forest, OccId, Position, Reading};

/// The class of every occurrence of a forest: the occurrences of the same
/// term, and under an intuitionistic reading in the same position, are
/// *interchangeable* and share a class, which is named by its first
/// occurrence. A position below an occurrence is determined by the
/// occurrence's term and position, so the subformulas of two
/// interchangeable occurrences are interchangeable pairwise.
#[derive(Clone, Debug)]
pub(crate) struct Classes {
    /// Per occurrence, the first occurrence of its class.
    class: Box<[OccId]>,
    /// Whether no two occurrences are interchangeable.
    distinct: bool,
}

impl Classes {
    /// Computes the classes of a forest, by term alone or, given the
    /// reading, by term and position.
    pub(crate) fn new(forest: &Forest, reading: Option<&Reading>) -> Self {
        /// No occurrence yet.
        const NONE: u32 = u32::MAX;
        // Per term, the first occurrence in input and in output position.
        let mut first = vec![[NONE; 2]; forest.sequent().terms().len()];
        let class: Box<[OccId]> = forest
            .ids()
            .map(|o| {
                let output = reading.is_some_and(|r| r.position(o) == Position::Output);
                let slot = &mut first[forest.term(o).index()][usize::from(output)];
                if *slot == NONE {
                    *slot = o.get();
                }
                OccId::new(*slot)
            })
            .collect();
        let distinct = forest.ids().all(|o| class[o.index()] == o);
        Self { class, distinct }
    }

    /// Returns whether no two occurrences of the forest are
    /// interchangeable, so that every occurrence is its own class.
    pub(crate) fn distinct(&self) -> bool {
        self.distinct
    }

    /// Returns the class of an occurrence: its first interchangeable
    /// occurrence.
    pub(crate) fn of(&self, o: OccId) -> OccId {
        self.class[o.index()]
    }

    /// Returns whether two occurrences are interchangeable.
    pub(crate) fn same(&self, x: OccId, y: OccId) -> bool {
        self.of(x) == self.of(y)
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;

    /// Equal formulas share a class wherever they occur, and a reading
    /// separates the positions: here the `⊤` that is the hypothesis `0`
    /// from the `⊤` that is the goal.
    #[test]
    fn classes() {
        use crate::sequents::Kind;
        let s: Sequent = "0, a, a |- top".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let roots = |kind| -> Vec<OccId> {
            let roots = forest.roots().iter().copied();
            roots.filter(|&r| forest.kind(r) == kind).collect()
        };
        let (tops, literals) = (roots(Kind::Top), roots(Kind::DualVar));
        assert_eq!((tops.len(), literals.len()), (2, 2));
        let classes = Classes::new(&forest, None);
        assert!(classes.same(tops[0], tops[1]));
        assert!(classes.same(literals[0], literals[1]));
        assert!(!classes.same(tops[0], literals[0]));
        assert_eq!(classes.of(literals[1]), literals[0]);
        assert!(!classes.distinct());
        let reading = Reading::new(&forest).unwrap();
        let classes = Classes::new(&forest, Some(&reading));
        assert!(!classes.same(tops[0], tops[1]));
        assert!(classes.same(literals[0], literals[1]));
    }
}
