// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The count invariants proof search prunes with: per occurrence, an
//! interval of the balance every atom can reach below it, and the weight
//! the `MLL` count equation sums. Both are necessary conditions on a
//! provable sequent, computed once per forest and summed per sequent.

use crate::occurrences::{Forest, OccId};
use crate::sequents::{Atom, Kind};

/// What a formula occurrence can contribute to the per-atom balance of a
/// sequent, and to the `MLL` count equation, precomputed for every
/// occurrence of a forest.
///
/// **Intervals.** For an atom `a`, the balance of a sequent is the number of
/// `a` literals minus the number of `~a` literals it decomposes into; every
/// axiom consumes one of each, so a provable sequent has balance zero for
/// every atom. Under `&` and `⊕` only one side of the connective is
/// decomposed, so a formula's contribution is an interval `lo ..= hi`:
/// a literal is `±1`, `⊗` and `⅋` sum their sides, `&` and `⊕` take the
/// hull of theirs, and units contribute nothing. A sequent whose summed
/// interval excludes zero for some atom is unprovable. `⊤` proves any
/// sequent it appears in, so an occurrence with a `⊤` below it *absorbs*:
/// its row is meaningless and any sequent containing it passes. Rows are
/// sparse, over the atoms that occur below the occurrence.
///
/// **Weight.** In the multiplicative fragment with units, every cut-free
/// proof of a sequent of `c` formulas with `t` tensors, `p` pars, `u` ones
/// and `b` bottoms below them satisfies `c = t − p − u + b + 2`, or `≥` with
/// Mix, because every leaf is an axiom or a `1` and every `⊗` or Mix joins
/// two subproofs. The weight of an occurrence is `t − p − u + b` over its
/// subtree, so the equation is a sum over the members of a sequent. It is
/// unsound as soon as additives or exponentials appear: `⊢ a ⊕ b, ~a` is
/// provable with `b` unbalanced, and a `⊤` closes any sequent.
#[derive(Clone, Debug)]
pub(crate) struct Counts {
    /// Where each occurrence's row starts in the entry arrays; the last
    /// entry is the total.
    row_start: Box<[u32]>,
    /// The atom of each row entry.
    atom: Box<[Atom]>,
    /// The least balance of each row entry's atom.
    lo: Box<[i32]>,
    /// The greatest balance of each row entry's atom.
    hi: Box<[i32]>,
    /// Per occurrence, whether a `⊤` lies at or below it.
    absorbs: Box<[bool]>,
    /// Per occurrence, `t − p − u + b` over its subtree.
    weight: Box<[i32]>,
    /// The number of atoms of the sequent, the width of a [`Tally`].
    num_atoms: usize,
}

/// One entry of a row: the atom and the interval of its balance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The atom.
    pub(crate) atom: Atom,
    /// The least balance.
    pub(crate) lo: i32,
    /// The greatest balance.
    pub(crate) hi: i32,
}

impl Counts {
    /// Computes the rows and weights of every occurrence of a forest.
    pub(crate) fn new(forest: &Forest) -> Self {
        let n = forest.len();
        // Every descendant has a larger id than its ancestor, so a pass from
        // the last id down sees the children before the parent.
        let mut rows: Vec<Vec<Entry>> = vec![Vec::new(); n];
        let mut absorbs = vec![false; n];
        let mut weight = vec![0i32; n];
        for o in forest.ids().rev() {
            use Kind::*;
            let kind = forest.kind(o);
            let (row, absorb, w) = match kind {
                Var | DualVar => {
                    let sign = if kind == Var { 1 } else { -1 };
                    let atom = forest.atom(o).unwrap();
                    (
                        vec![Entry {
                            atom,
                            lo: sign,
                            hi: sign,
                        }],
                        false,
                        0,
                    )
                }
                One => (Vec::new(), false, -1),
                Bot => (Vec::new(), false, 1),
                Top => (Vec::new(), true, 0),
                Zero => (Vec::new(), false, 0),
                Bang | Quest => {
                    // A copy below `?` breaks the balance; the engine that
                    // handles exponentials skips such atoms. Here the row is
                    // the subformula's.
                    let c = forest.left(o).unwrap();
                    (
                        rows[c.index()].clone(),
                        absorbs[c.index()],
                        weight[c.index()],
                    )
                }
                Tensor | Par | With | Plus => {
                    let (l, r) = (forest.left(o).unwrap(), forest.right(o).unwrap());
                    let (li, ri) = (l.index(), r.index());
                    let sum = matches!(kind, Tensor | Par);
                    let absorb = absorbs[li] || absorbs[ri];
                    let row = if absorb {
                        Vec::new()
                    } else {
                        merge(&rows[li], &rows[ri], sum)
                    };
                    let w = weight[li]
                        + weight[ri]
                        + if kind == Tensor {
                            1
                        } else if kind == Par {
                            -1
                        } else {
                            0
                        };
                    (row, absorb, w)
                }
            };
            rows[o.index()] = row;
            absorbs[o.index()] = absorb;
            weight[o.index()] = w;
        }

        let mut row_start = Vec::with_capacity(n + 1);
        let (mut atom, mut lo, mut hi) = (Vec::new(), Vec::new(), Vec::new());
        for row in &rows {
            row_start.push(atom.len() as u32);
            for e in row {
                atom.push(e.atom);
                lo.push(e.lo);
                hi.push(e.hi);
            }
        }
        row_start.push(atom.len() as u32);
        Self {
            row_start: row_start.into_boxed_slice(),
            atom: atom.into_boxed_slice(),
            lo: lo.into_boxed_slice(),
            hi: hi.into_boxed_slice(),
            absorbs: absorbs.into_boxed_slice(),
            weight: weight.into_boxed_slice(),
            num_atoms: forest.sequent().atom_names().len(),
        }
    }

    /// Returns the row of an occurrence: its interval per atom occurring
    /// below it, in ascending atom order. Empty when the occurrence absorbs.
    pub(crate) fn row(&self, o: OccId) -> impl Iterator<Item = Entry> + '_ {
        let (start, end) = (
            self.row_start[o.index()] as usize,
            self.row_start[o.index() + 1] as usize,
        );
        (start..end).map(move |i| Entry {
            atom: self.atom[i],
            lo: self.lo[i],
            hi: self.hi[i],
        })
    }

    /// Returns whether a `⊤` lies at or below the occurrence, so that any
    /// sequent containing it passes the interval check.
    pub(crate) fn absorbs(&self, o: OccId) -> bool {
        self.absorbs[o.index()]
    }

    /// Returns `t − p − u + b` over the occurrence's subtree.
    pub(crate) fn weight(&self, o: OccId) -> i32 {
        self.weight[o.index()]
    }

    /// Returns an empty tally of this forest's width.
    pub(crate) fn tally(&self) -> Tally {
        Tally {
            lo: vec![0; self.num_atoms],
            hi: vec![0; self.num_atoms],
            bad: 0,
            absorbers: 0,
            len: 0,
            weight: 0,
        }
    }
}

/// Merges two rows sorted by atom: the sum of the intervals for `⊗` and
/// `⅋`, their hull for `&` and `⊕`, with an atom missing from one side
/// contributing `0 ..= 0` there.
fn merge(a: &[Entry], b: &[Entry], sum: bool) -> Vec<Entry> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    let zero = |atom| Entry { atom, lo: 0, hi: 0 };
    while i < a.len() || j < b.len() {
        let (x, y) = match (a.get(i), b.get(j)) {
            (Some(&x), Some(&y)) if x.atom == y.atom => {
                i += 1;
                j += 1;
                (x, y)
            }
            (Some(&x), y) if y.is_none_or(|y| x.atom < y.atom) => {
                i += 1;
                (x, zero(x.atom))
            }
            (_, Some(&y)) => {
                j += 1;
                (zero(y.atom), y)
            }
            (_, None) => unreachable!("a side still has entries"),
        };
        out.push(Entry {
            atom: x.atom,
            lo: if sum { x.lo + y.lo } else { x.lo.min(y.lo) },
            hi: if sum { x.hi + y.hi } else { x.hi.max(y.hi) },
        });
    }
    out
}

/// The running sums of a set of occurrences: the summed interval per atom,
/// how many atoms' intervals exclude zero, how many members absorb, and the
/// member count and summed weight the `MLL` equation compares. Members are
/// added and removed one at a time, at the cost of their row, so a split
/// enumeration keeps two tallies in step with its two sides.
#[derive(Clone, Debug)]
pub(crate) struct Tally {
    /// Per atom, the summed least balance.
    lo: Vec<i32>,
    /// Per atom, the summed greatest balance.
    hi: Vec<i32>,
    /// The number of atoms whose summed interval excludes zero.
    bad: u32,
    /// The number of members that absorb.
    absorbers: u32,
    /// The number of members.
    len: u32,
    /// The summed weight of the members.
    weight: i32,
}

impl Tally {
    /// Removes every member.
    pub(crate) fn clear(&mut self) {
        self.lo.fill(0);
        self.hi.fill(0);
        self.bad = 0;
        self.absorbers = 0;
        self.len = 0;
        self.weight = 0;
    }

    /// Adds a member.
    pub(crate) fn add(&mut self, counts: &Counts, o: OccId) {
        self.apply(counts, o, 1);
    }

    /// Removes a member.
    pub(crate) fn remove(&mut self, counts: &Counts, o: OccId) {
        self.apply(counts, o, -1);
    }

    /// Adds (`sign` 1) or removes (`sign` −1) a member's row, weight and
    /// absorption.
    fn apply(&mut self, counts: &Counts, o: OccId, sign: i32) {
        for e in counts.row(o) {
            let a = e.atom.index();
            let was_bad = self.lo[a] > 0 || self.hi[a] < 0;
            self.lo[a] += sign * e.lo;
            self.hi[a] += sign * e.hi;
            let is_bad = self.lo[a] > 0 || self.hi[a] < 0;
            if is_bad && !was_bad {
                self.bad += 1;
            } else if was_bad && !is_bad {
                self.bad -= 1;
            }
        }
        if counts.absorbs(o) {
            self.absorbers = self.absorbers.wrapping_add_signed(sign);
        }
        self.len = self.len.wrapping_add_signed(sign);
        self.weight += sign * counts.weight(o);
    }

    /// Returns whether a member absorbs, that is, has a `⊤` below it.
    pub(crate) fn absorbs(&self) -> bool {
        self.absorbers > 0
    }

    /// Returns whether the members pass the interval check: one of them
    /// absorbs, or zero lies in the summed interval of every atom.
    pub(crate) fn balanced(&self) -> bool {
        self.absorbs() || self.bad == 0
    }

    /// Returns whether the members pass the `MLL` count equation
    /// `c = t − p − u + b + 2`, as an equality or, with Mix, as `≥`. Only
    /// meaningful for members without additives or exponentials below them.
    pub(crate) fn equation(&self, mix: bool) -> bool {
        let c = i64::from(self.len);
        let rhs = i64::from(self.weight) + 2;
        if mix { c >= rhs } else { c == rhs }
    }

    /// Returns whether the `MLL` count equation leaves room for a Mix among
    /// the members: `c > t − p − u + b + 2`, since every Mix adds two to the
    /// left side.
    pub(crate) fn admits_mix(&self) -> bool {
        i64::from(self.len) > i64::from(self.weight) + 2
    }

    /// Returns the number of members.
    pub(crate) fn len(&self) -> u32 {
        self.len
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;

    /// Builds the forest and counts of `input`.
    fn counts(input: &str) -> (Forest, Counts) {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let f = Forest::new(&s).unwrap();
        let c = Counts::new(&f);
        (f, c)
    }

    /// Tallies every root of the sequent.
    fn roots(input: &str) -> (Forest, Counts, Tally) {
        let (f, c) = counts(input);
        let mut t = c.tally();
        for &r in f.roots() {
            t.add(&c, r);
        }
        (f, c, t)
    }

    /// Rows sum under the multiplicatives, take the hull under the
    /// additives, and are sparse over the atoms below.
    #[test]
    fn rows() {
        // ⊢ (a ⊗ ~a) ⅋ b, (a & ~b) ⊕ ~a, 1
        let (f, c) = counts("|- (a * ~a) par b, (a & ~b) + ~a, 1");
        let (a, b) = (
            f.sequent().atom("a").unwrap(),
            f.sequent().atom("b").unwrap(),
        );
        let row = |o: u32| c.row(OccId::new(o)).collect::<Vec<_>>();
        let e = |atom, lo, hi| Entry { atom, lo, hi };
        assert_eq!(row(0), [e(a, 0, 0), e(b, 1, 1)], "the ⅋");
        assert_eq!(row(1), [e(a, 0, 0)], "the ⊗");
        assert_eq!(row(2), [e(a, 1, 1)]);
        assert_eq!(row(3), [e(a, -1, -1)]);
        assert_eq!(row(5), [e(a, -1, 1), e(b, -1, 0)], "the ⊕");
        assert_eq!(row(6), [e(a, 0, 1), e(b, -1, 0)], "the &");
        assert_eq!(row(10), [], "the 1");
        assert!(!c.absorbs(OccId::new(0)));
        assert_eq!(c.weight(OccId::new(0)), 0, "one ⊗, one ⅋");
        assert_eq!(c.weight(OccId::new(10)), -1);
    }

    /// The interval check accepts the balanced, rejects the unbalanced, and
    /// lets `⊢ a ⊕ b, ~a` through where the plain balance would not.
    #[test]
    fn intervals() {
        for (input, balanced) in [
            ("|- a, ~a", true),
            ("|- a, a", false),
            ("|- a, ~a, b", false),
            ("|- a + b, ~a", true),
            ("|- a & b, ~a", true),
            ("|- (a * b) + (a * c), ~a, ~b", true),
            ("|- (a * b) + (a * c), ~a, ~c, ~c", false),
            ("|- 1, bot, 0", true),
            ("|- a", false),
            ("|-", true),
        ] {
            let (_, _, t) = roots(input);
            assert_eq!(t.balanced(), balanced, "{input:?}");
        }
    }

    /// A `⊤` anywhere below a member absorbs any imbalance.
    #[test]
    fn top_absorbs() {
        for input in [
            "|- top, a",
            "|- top * a, b",
            "|- (a & top) + c, ~b",
            "|- top par a, b",
            "|- a & top, ~b",
            "|- top & a, ~b",
        ] {
            let (f, c, t) = roots(input);
            assert!(c.absorbs(f.roots()[0]), "{input:?}");
            assert!(t.balanced(), "{input:?}");
        }
    }

    /// The `MLL` equation accepts exactly the spec's unit checks, as an
    /// equality without Mix and as an inequality with it.
    #[test]
    fn equation() {
        for (input, holds, holds_with_mix) in [
            ("|- 1", true, true),
            ("|- bot, 1", true, true),
            ("|- 1 * 1", true, true),
            ("|- bot par 1", true, true),
            ("|- bot par bot", false, false),
            ("|- a, ~a", true, true),
            ("|- a * b, ~a, ~b", true, true),
            ("|- a par b, ~a, ~b", false, true),
            ("|- a, ~a, b, ~b", false, true),
            ("|- 1, a, ~a", false, true),
            ("|-", false, false),
        ] {
            let (_, _, t) = roots(input);
            assert_eq!(t.equation(false), holds, "{input:?}");
            assert_eq!(t.equation(true), holds_with_mix, "{input:?}");
        }
    }

    /// Members go in and out of a tally, and the counts follow.
    #[test]
    fn tally_moves() {
        let (f, c, mut t) = roots("|- a * b, ~a, ~b");
        assert!(t.balanced());
        assert_eq!(t.len(), 3);
        t.remove(&c, f.roots()[1]);
        assert!(!t.balanced(), "~a alone is unbalanced");
        assert_eq!(t.len(), 2);
        t.remove(&c, f.roots()[0]);
        t.remove(&c, f.roots()[2]);
        assert!(t.balanced(), "the empty set is balanced");
        assert!(!t.equation(false));
        t.add(&c, f.roots()[1]);
        t.add(&c, f.roots()[2]);
        assert!(!t.balanced());
        t.clear();
        assert!(t.balanced());
        assert_eq!(t.len(), 0);
    }
}
