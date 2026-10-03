// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The count invariants proof search prunes with: per occurrence, an
//! interval of the balance every atom can reach below it, and the weight
//! the `MLL` count equation sums. Both are necessary conditions on a
//! provable sequent, computed once per forest and summed per sequent.

use crate::occurrences::{Bias, Forest, OccId, Sign};
use crate::proofs::Side;
use crate::search::memory::{Account, bytes_of};
use crate::search::{Reason, set_up_stopped};
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
/// An atom with a literal below a `?` or `!` anywhere in the problem gets no
/// entry in any row: copies and discards break its balance. A `⊤` below a
/// `?` or `!` makes the check useless altogether, since a copy of it
/// absorbs any imbalance; [`absorbs_from_copies`](Self::absorbs_from_copies)
/// says so.
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
    /// Where the rows end in the entry arrays, in the order the rows were
    /// computed, the last occurrence's first: the row of occurrence `o` of
    /// `n` lies between the entries `n − 1 − o` and `n − o`. Every bound
    /// fits a `u32`: a set-up whose rows have more entries is refused.
    bound: Box<[u32]>,
    /// The atom of each row entry: its rank among the atoms that have rows
    /// at all, in the order of the sequent's atoms.
    atom: Box<[Atom]>,
    /// The least balance of each row entry's atom. A sum of ±1 over the
    /// literals below an occurrence, of which a forest that gets counts has
    /// fewer than 2³¹: within an `i32`.
    lo: Box<[i32]>,
    /// The greatest balance of each row entry's atom, bounded as the least.
    hi: Box<[i32]>,
    /// Per occurrence, whether a `⊤` lies at or below it.
    absorbs: Box<[bool]>,
    /// Per occurrence, `t − p − u + b` over its subtree: at most its
    /// connectives in absolute value, fewer than 2³¹.
    weight: Box<[i32]>,
    /// The number of atoms that have rows, the width of a [`Tally`]: those
    /// with no literal below a `!` or `?`.
    row_atoms: usize,
    /// Whether a `⊤` lies below a `?` or `!` somewhere in the problem, so
    /// that a copy can absorb any imbalance and the intervals prune
    /// nothing.
    absorbs_from_copies: bool,
    /// Per occurrence, whether it is a tensor of positive literals none of
    /// whose duals can lie in the unrestricted zone.
    literal_tensor: Box<[bool]>,
    /// Per atom, the sign of its positive literal in this search.
    positive: Box<[Sign]>,
}

/// One entry of a row: the atom and the interval of its balance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The atom, as its rank among the atoms that have rows.
    pub(crate) atom: Atom,
    /// The least balance.
    pub(crate) lo: i32,
    /// The greatest balance.
    pub(crate) hi: i32,
}

/// The rows of the occurrences computed so far, one after another.
#[derive(Default)]
struct Rows {
    /// The atom of each entry.
    atom: Vec<Atom>,
    /// The least balance of each entry.
    lo: Vec<i32>,
    /// The greatest balance of each entry.
    hi: Vec<i32>,
}

impl Rows {
    /// The entry at a place.
    fn at(&self, i: usize) -> Entry {
        Entry {
            atom: self.atom[i],
            lo: self.lo[i],
            hi: self.hi[i],
        }
    }

    /// Appends an entry.
    fn push(&mut self, e: Entry) {
        self.atom.push(e.atom);
        self.lo.push(e.lo);
        self.hi.push(e.hi);
    }

    /// The bytes the entries' allocations take.
    fn bytes(&self) -> usize {
        bytes_of(&self.atom) + bytes_of(&self.lo) + bytes_of(&self.hi)
    }

    /// Appends the merge of two rows that lie among the entries, each
    /// sorted by atom: the sum of the intervals for `⊗` and `⅋`, their
    /// hull for `&` and `⊕`, with an atom missing from one side
    /// contributing `0 ..= 0` there. Returns how many entries it read.
    fn merge(&mut self, a: std::ops::Range<usize>, b: std::ops::Range<usize>, sum: bool) -> usize {
        let (mut i, mut j) = (a.start, b.start);
        let zero = |atom| Entry { atom, lo: 0, hi: 0 };
        while i < a.end || j < b.end {
            let (x, y) = match (i < a.end, j < b.end) {
                (true, true) if self.atom[i] == self.atom[j] => {
                    i += 1;
                    j += 1;
                    (self.at(i - 1), self.at(j - 1))
                }
                (true, other) if !other || self.atom[i] < self.atom[j] => {
                    i += 1;
                    (self.at(i - 1), zero(self.atom[i - 1]))
                }
                _ => {
                    j += 1;
                    (zero(self.atom[j - 1]), self.at(j - 1))
                }
            };
            self.push(Entry {
                atom: x.atom,
                lo: if sum { x.lo + y.lo } else { x.lo.min(y.lo) },
                hi: if sum { x.hi + y.hi } else { x.hi.max(y.hi) },
            });
        }
        a.len() + b.len()
    }
}

impl Counts {
    /// Computes the rows and weights of every occurrence of a forest, and
    /// the literals that are positive under the bias given.
    pub(crate) fn new(forest: &Forest, bias: Bias) -> Self {
        Self::new_until(forest, bias, &Account::new(None), &mut || false).expect("nothing stops it")
    }

    /// Computes the rows and weights of every occurrence of a forest, and
    /// the literals that are positive under the bias given, polling `stop`
    /// on a large forest once every so many occurrences and row entries
    /// visited, as the passes around it are polled between them: this one
    /// is several, and the longest of a search's set-up. The rows together
    /// can be quadratic in the forest (a nest of `⊗` and `⅋` over distinct
    /// atoms has a row as long as its subtree at every level), so their
    /// memory is charged to `account` as it grows. Returns the reason the
    /// set-up gave up: the condition fired, the account is over its bound,
    /// or the forest or the rows are more than the counts can index.
    pub(crate) fn new_until(
        forest: &Forest,
        bias: Bias,
        account: &Account,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<Self, Reason> {
        /// How many occurrences are visited between two polls.
        const PERIOD: usize = 1 << 16;
        let mut visited = 0;
        let mut stopped = |steps: usize| {
            visited += steps;
            visited >= PERIOD && {
                visited = 0;
                set_up_stopped(forest, stop)
            }
        };
        let n = forest.len();
        // The balances and weights are sums over an occurrence's subtree.
        if n > i32::MAX as usize {
            return Err(Reason::IndexLimit);
        }
        let positive = forest.bias_under(bias);
        // An atom with a literal below a `?` or `!` anywhere in the problem
        // can be copied or discarded any number of times, so its balance
        // says nothing: such atoms get no row entries at all.
        let num_atoms = forest.sequent().atom_names().len();
        let fixed = n * (size_of::<u32>() + size_of::<i32>() + 2 * size_of::<bool>())
            + num_atoms * (2 * size_of::<bool>() + size_of::<u32>() + size_of::<Sign>());
        account.charge(fixed);
        if account.over() {
            return Err(Reason::MemoryLimit(account.limit()));
        }
        let mut exponential = vec![false; num_atoms];
        let mut absorbs_from_copies = false;
        // An occurrence's subtree is the ids that follow it, so one pass
        // that remembers where the outermost `!` or `?` it is inside ends
        // knows of every occurrence whether it lies below one: a walk of
        // each one's subtree was quadratic in a nest of them.
        let mut inside = 0;
        for o in forest.ids() {
            if o.index() < inside {
                if let Some(atom) = forest.atom(o) {
                    exponential[atom.index()] = true;
                }
                absorbs_from_copies |= forest.kind(o) == Kind::Top;
            }
            if matches!(forest.kind(o), Kind::Bang | Kind::Quest) {
                inside = inside.max(o.index() + forest.size(o) as usize);
            }
            if stopped(1) {
                return Err(Reason::Stopped);
            }
        }
        // The atoms that have rows, numbered in their own order: a tally
        // is as wide as they are many, and a Petri net has tens of
        // thousands of atoms and none with a row.
        let mut rank = vec![0u32; num_atoms];
        let mut row_atoms = 0;
        for (atom, rank) in rank.iter_mut().enumerate() {
            *rank = row_atoms as u32;
            row_atoms += usize::from(!exponential[atom]);
        }
        // The literals that a `?` can put into the unrestricted zone: those
        // directly under one.
        let mut unrestricted = vec![[false; 2]; num_atoms];
        for o in forest.ids() {
            if forest.kind(o) == Kind::Quest
                && let Some(below) = forest.left(o)
                && let (Some(atom), Some(sign)) = (forest.atom(below), forest.sign(below))
            {
                unrestricted[atom.index()][sign as usize] = true;
            }
        }
        // A positive literal whose dual only ever lies in the linear zone.
        let linear = |o: OccId| match (forest.atom(o), forest.sign(o)) {
            (Some(atom), Some(sign)) => {
                sign == positive[atom.index()] && !unrestricted[atom.index()][!sign as usize]
            }
            _ => false,
        };
        let mut literal_tensor = vec![false; n];
        // Every descendant has a larger id than its ancestor, so a pass from
        // the last id down sees the children before the parent, and a row
        // is merged from two that are written already.
        let mut rows = Rows::default();
        let mut bound = Vec::with_capacity(n + 1);
        bound.push(0u32);
        let row = |bound: &[u32], o: OccId| {
            let at = n - 1 - o.index();
            bound[at] as usize..bound[at + 1] as usize
        };
        let mut absorbs = vec![false; n];
        let mut weight = vec![0i32; n];
        let mut charged = 0;
        for o in forest.ids().rev() {
            use Kind::*;
            let kind = forest.kind(o);
            let mut steps = 1;
            let (absorb, w) = match kind {
                Var | DualVar => {
                    let sign = if kind == Var { 1 } else { -1 };
                    let atom = forest.atom(o).unwrap();
                    if !exponential[atom.index()] {
                        rows.push(Entry {
                            atom: Atom::new(rank[atom.index()]),
                            lo: sign,
                            hi: sign,
                        });
                    }
                    (false, 0)
                }
                One => (false, -1),
                Bot => (false, 1),
                Top => (true, 0),
                Zero => (false, 0),
                Bang | Quest => {
                    // The subformula's row is empty: its atoms are
                    // exponential.
                    let c = forest.left(o).unwrap();
                    (absorbs[c.index()], weight[c.index()])
                }
                Tensor | Par | With | Plus => {
                    let (l, r) = (forest.left(o).unwrap(), forest.right(o).unwrap());
                    let (li, ri) = (l.index(), r.index());
                    let absorb = absorbs[li] || absorbs[ri];
                    if !absorb {
                        let sum = matches!(kind, Tensor | Par);
                        steps += rows.merge(row(&bound, l), row(&bound, r), sum);
                    }
                    let w = weight[li]
                        + weight[ri]
                        + if kind == Tensor {
                            1
                        } else if kind == Par {
                            -1
                        } else {
                            0
                        };
                    (absorb, w)
                }
            };
            let Ok(end) = u32::try_from(rows.atom.len()) else {
                return Err(Reason::IndexLimit);
            };
            bound.push(end);
            absorbs[o.index()] = absorb;
            weight[o.index()] = w;
            literal_tensor[o.index()] = kind == Tensor
                && forest
                    .children(o)
                    .all(|c| linear(c) || literal_tensor[c.index()]);
            if rows.bytes() != charged {
                account.resize(charged, rows.bytes());
                charged = rows.bytes();
                if account.over() {
                    return Err(Reason::MemoryLimit(account.limit()));
                }
            }
            if stopped(steps) {
                return Err(Reason::Stopped);
            }
        }
        let counts = Self {
            bound: bound.into_boxed_slice(),
            atom: rows.atom.into_boxed_slice(),
            lo: rows.lo.into_boxed_slice(),
            hi: rows.hi.into_boxed_slice(),
            absorbs: absorbs.into_boxed_slice(),
            weight: weight.into_boxed_slice(),
            row_atoms,
            absorbs_from_copies,
            literal_tensor: literal_tensor.into_boxed_slice(),
            positive,
        };
        account.resize(
            charged,
            counts.atom.len() * (size_of::<Atom>() + 2 * size_of::<i32>()),
        );
        Ok(counts)
    }

    /// The places of an occurrence's row in the entry arrays.
    fn span(&self, o: OccId) -> (usize, usize) {
        let at = self.absorbs.len() - 1 - o.index();
        (self.bound[at] as usize, self.bound[at + 1] as usize)
    }

    /// Returns whether the occurrence is a positive literal: the literal
    /// of its atom that the bias of the search makes positive.
    pub(crate) fn positive(&self, forest: &Forest, o: OccId) -> bool {
        match (forest.atom(o), forest.sign(o)) {
            (Some(atom), Some(sign)) => sign == self.positive[atom.index()],
            _ => false,
        }
    }

    /// Returns whether the occurrence is a tensor of positive literals
    /// none of whose duals can lie in the unrestricted zone (none is
    /// directly under a `?`): in focus, such a tensor is proved by exactly
    /// one dual per literal, each from the linear zone.
    pub(crate) fn literal_tensor(&self, o: OccId) -> bool {
        self.literal_tensor[o.index()]
    }

    /// Returns the row of an occurrence: its interval per atom occurring
    /// below it, in ascending atom order. Empty when the occurrence absorbs.
    pub(crate) fn row(&self, o: OccId) -> impl Iterator<Item = Entry> + '_ {
        let (start, end) = self.span(o);
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

    /// Returns whether a `⊤` lies below a `?` or `!` somewhere in the
    /// problem: a copy from the unrestricted zone can then absorb any
    /// imbalance, so the interval check is unsound.
    pub(crate) fn absorbs_from_copies(&self) -> bool {
        self.absorbs_from_copies
    }

    /// Returns the first atom of the occurrence's row, as an index, or
    /// `u32::MAX` when the row is empty: members of a context sorted by it
    /// have every atom's members next to each other.
    pub(crate) fn first_atom(&self, o: OccId) -> u32 {
        let (start, end) = self.span(o);
        if start == end {
            u32::MAX
        } else {
            self.atom[start].index() as u32
        }
    }

    /// Returns the number of atoms in the occurrence's row.
    pub(crate) fn row_len(&self, o: OccId) -> u32 {
        let (start, end) = self.span(o);
        (end - start) as u32
    }

    /// Returns the bytes a [`Tally`] of this forest allocates.
    pub(crate) fn tally_bytes(&self) -> usize {
        2 * self.row_atoms * size_of::<i32>()
    }

    /// Returns the bytes the counts of a [`Split`] of this forest take.
    pub(crate) fn split_bytes(&self) -> usize {
        size_of::<Split>() + 6 * self.row_atoms * size_of::<i32>()
    }

    /// Returns the counts of a split with no member yet.
    pub(crate) fn split(&self) -> Split {
        Split {
            lo: [vec![0; self.row_atoms], vec![0; self.row_atoms]],
            hi: [vec![0; self.row_atoms], vec![0; self.row_atoms]],
            below: vec![0; self.row_atoms],
            above: vec![0; self.row_atoms],
            bad: [0; 2],
            absorbers: [0; 2],
            open_absorbers: 0,
            slack: [-2; 2],
            slack_below: 0,
            slack_above: 0,
            inert: false,
            touched: Vec::new(),
        }
    }

    /// Returns whether the occurrence's row has an interval that excludes
    /// zero by itself: only such a member can make a set fail the interval
    /// check.
    pub(crate) fn tight(&self, o: OccId) -> bool {
        self.row(o).any(|e| e.lo > 0 || e.hi < 0)
    }

    /// Returns an empty tally of this forest's width.
    pub(crate) fn tally(&self) -> Tally {
        Tally {
            lo: vec![0; self.row_atoms],
            hi: vec![0; self.row_atoms],
            touched: Vec::new(),
            bad: 0,
            absorbers: 0,
            len: 0,
            weight: 0,
        }
    }
}

/// The running sums of a set of occurrences: the summed interval per atom,
/// how many atoms' intervals exclude zero, how many members absorb, and the
/// member count and summed weight the `MLL` equation compares. Members are
/// added and removed one at a time, at the cost of their row.
#[derive(Clone, Debug)]
pub(crate) struct Tally {
    /// Per atom, the summed least balance.
    lo: Vec<i32>,
    /// Per atom, the summed greatest balance.
    hi: Vec<i32>,
    /// The atoms of the rows added or removed since the tally was last
    /// cleared, some more than once: the only atoms whose sums can be other
    /// than zero.
    touched: Vec<u32>,
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
    /// Removes every member, at the cost of the rows that came in since the
    /// last time and not of the sequent's atoms.
    pub(crate) fn clear(&mut self) {
        for a in self.touched.drain(..) {
            self.lo[a as usize] = 0;
            self.hi[a as usize] = 0;
        }
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
            self.touched.push(a as u32);
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

/// The counts of a split of a context in the making: the sums of the
/// members each side has so far, and what the members not yet assigned to
/// a side can still add to either. A side whose sums no assignment of the
/// open members can bring to pass the interval check or the count equation
/// makes the partial split infeasible, so a search over the members cuts
/// it there; with no member open, [`feasible`](Self::feasible) is the test
/// of a whole split: both sides pass the interval check and the equation.
///
/// The bounds are per atom and per side, each on its own: an open member
/// can lower a side's least balance of an atom by the negative part of its
/// own, and raise the greatest by the positive part of its own. They are
/// necessary conditions, never sufficient.
#[derive(Clone, Debug)]
pub(crate) struct Split {
    /// Per side and atom, the summed least balance of the side's members.
    lo: [Vec<i32>; 2],
    /// Per side and atom, the summed greatest balance of the side's members.
    hi: [Vec<i32>; 2],
    /// Per atom, the sum of the negative least balances of the open
    /// members: the most they can lower a side's least balance.
    below: Vec<i32>,
    /// Per atom, the sum of the positive greatest balances of the open
    /// members: the most they can raise a side's greatest balance.
    above: Vec<i32>,
    /// Per side, the number of atoms whose interval excludes zero whatever
    /// the open members do.
    bad: [u32; 2],
    /// Per side, the number of members that absorb.
    absorbers: [u32; 2],
    /// The number of open members that absorb.
    open_absorbers: u32,
    /// Per side, `c − (t − p − u + b) − 2` over its members, which the
    /// count equation wants zero, or at least zero with Mix.
    slack: [i64; 2],
    /// The sum of the negative contributions of the open members to a
    /// side's slack.
    slack_below: i64,
    /// The sum of their positive contributions.
    slack_above: i64,
    /// Whether no prune can cut any assignment of these members, so that
    /// the counts need not follow the search.
    inert: bool,
    /// The atoms of the rows of every member placed or opened since the
    /// counts were last cleared, some more than once: the only atoms whose
    /// sums can be other than zero.
    touched: Vec<u32>,
}

impl Split {
    /// Removes every member, at the cost of the rows that came in since the
    /// last time and not of the sequent's atoms: a Petri net has thousands
    /// of atoms and a split touches a few.
    pub(crate) fn clear(&mut self) {
        for a in self.touched.drain(..) {
            let a = a as usize;
            for side in 0..2 {
                self.lo[side][a] = 0;
                self.hi[side][a] = 0;
            }
            self.below[a] = 0;
            self.above[a] = 0;
        }
        self.bad = [0; 2];
        self.absorbers = [0; 2];
        self.open_absorbers = 0;
        self.slack = [-2; 2];
        self.slack_below = 0;
        self.slack_above = 0;
        self.inert = false;
    }

    /// Says, once every member is placed or open, whether no prune in
    /// force can cut any assignment of them: [`feasible`](Self::feasible)
    /// then answers yes without the counts, which stop following the
    /// members' moves.
    pub(crate) fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
    }

    /// Whether no assignment of the open members can bring the side's
    /// interval of the atom to contain zero.
    fn excludes(&self, side: usize, atom: usize) -> bool {
        self.lo[side][atom] + self.below[atom] > 0 || self.hi[side][atom] + self.above[atom] < 0
    }

    /// Moves a member between the open ones and a side, or adds it to
    /// either, in one pass over its row: `side` gains it `settled` times
    /// (1, 0 or −1) and the open members gain it `opened` times, and the
    /// sides' counts of excluded atoms are brought up to date.
    fn shift(&mut self, counts: &Counts, o: OccId, side: Side, settled: i32, opened: i32) {
        let side = side as usize;
        for e in counts.row(o) {
            let a = e.atom.index();
            let was = [self.excludes(0, a), self.excludes(1, a)];
            self.lo[side][a] += settled * e.lo;
            self.hi[side][a] += settled * e.hi;
            self.below[a] += opened * e.lo.min(0);
            self.above[a] += opened * e.hi.max(0);
            for (side, was) in was.into_iter().enumerate() {
                match (was, self.excludes(side, a)) {
                    (false, true) => self.bad[side] += 1,
                    (true, false) => self.bad[side] -= 1,
                    _ => {}
                }
            }
        }
        if counts.absorbs(o) {
            self.absorbers[side] = self.absorbers[side].wrapping_add_signed(settled);
            self.open_absorbers = self.open_absorbers.wrapping_add_signed(opened);
        }
        let slack = 1 - i64::from(counts.weight(o));
        self.slack[side] += i64::from(settled) * slack;
        self.slack_below += i64::from(opened) * slack.min(0);
        self.slack_above += i64::from(opened) * slack.max(0);
    }

    /// Adds a member to a side for good: a subformula of the `⊗`, or a
    /// member the search does not move.
    pub(crate) fn place(&mut self, counts: &Counts, o: OccId, side: Side) {
        self.touched
            .extend(counts.row(o).map(|e| e.atom.index() as u32));
        self.shift(counts, o, side, 1, 0);
    }

    /// Adds a member that a search will assign to a side.
    pub(crate) fn open(&mut self, counts: &Counts, o: OccId) {
        self.touched
            .extend(counts.row(o).map(|e| e.atom.index() as u32));
        self.shift(counts, o, Side::Left, 0, 1);
    }

    /// Assigns an open member to a side.
    pub(crate) fn assign(&mut self, counts: &Counts, o: OccId, side: Side) {
        if !self.inert {
            self.shift(counts, o, side, 1, -1);
        }
    }

    /// Moves an assigned member from the other side to this one.
    pub(crate) fn flip(&mut self, counts: &Counts, o: OccId, side: Side) {
        if self.inert {
            return;
        }
        let (to, from) = (side as usize, 1 - side as usize);
        for e in counts.row(o) {
            let a = e.atom.index();
            let was = [self.excludes(0, a), self.excludes(1, a)];
            self.lo[from][a] -= e.lo;
            self.hi[from][a] -= e.hi;
            self.lo[to][a] += e.lo;
            self.hi[to][a] += e.hi;
            for (side, was) in was.into_iter().enumerate() {
                match (was, self.excludes(side, a)) {
                    (false, true) => self.bad[side] += 1,
                    (true, false) => self.bad[side] -= 1,
                    _ => {}
                }
            }
        }
        if counts.absorbs(o) {
            self.absorbers[from] -= 1;
            self.absorbers[to] += 1;
        }
        let slack = 1 - i64::from(counts.weight(o));
        self.slack[from] -= slack;
        self.slack[to] += slack;
    }

    /// Takes an assigned member back from its side: it is open again.
    pub(crate) fn unassign(&mut self, counts: &Counts, o: OccId, side: Side) {
        if !self.inert {
            self.shift(counts, o, side, -1, 1);
        }
    }

    /// Returns whether some assignment of the open members may still pass
    /// the prunes in force on both sides: the interval check (a side
    /// passes with an absorbing member, which each open one can give to
    /// one side only) and the count equation, `≥` with Mix. With no open
    /// member, whether the split passes them.
    pub(crate) fn feasible(&self, intervals: bool, equation: bool, mix: bool) -> bool {
        if self.inert {
            return true;
        }
        if intervals {
            let needy = (0..2)
                .filter(|&side| self.absorbers[side] == 0 && self.bad[side] > 0)
                .count();
            if needy > self.open_absorbers as usize {
                return false;
            }
        }
        !equation
            || self.slack.iter().all(|slack| {
                slack + self.slack_above >= 0 && (mix || slack + self.slack_below <= 0)
            })
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
        let c = Counts::new(&f, Bias::Auto);
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
        assert!(c.tight(OccId::new(2)), "a literal's interval excludes zero");
        assert!(!c.tight(OccId::new(5)), "the ⊕ can be balanced by itself");
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
