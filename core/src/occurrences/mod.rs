// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// The intuitionistic reading of a sequent.
pub mod reading;
/// Bitsets over occurrence ids.
pub mod set;

pub use reading::{DescribedShape, IllFormula, Position, Reading, ShapeError};
pub use set::OccSet;

use crate::Error;
use crate::sequents::{Atom, Formula, Kind, Sequent, TermId};
use std::cmp::Ordering;
use std::ops::Not;

/// The index of a subformula occurrence in a [`Forest`]: the position of the
/// occurrence in a depth-first preorder walk over the root formulas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OccId(u32);

impl OccId {
    /// Wraps a raw occurrence index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw occurrence index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing per-occurrence arrays.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The raw index that stands for "no occurrence" in the forest's arrays.
const NONE: u32 = u32::MAX;

/// Which literal of an atom `a` an occurrence is: `a` itself or its negation
/// `~a`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sign {
    /// The atom `a`, a `Var` term.
    Var,
    /// The negation `~a`, a `DualVar` term.
    DualVar,
}

impl Not for Sign {
    type Output = Self;

    /// The other literal of the same atom.
    fn not(self) -> Self {
        match self {
            Sign::Var => Sign::DualVar,
            Sign::DualVar => Sign::Var,
        }
    }
}

/// How the positive literal of every atom is chosen for focused proof
/// search. Focusing is complete for every choice, so the rules differ in
/// speed and, with exponentials, in the copies a branch of the proofs
/// they lead to needs: never in what is provable.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Bias {
    /// [`Factors`](Self::Factors) for a sequent without exponentials and
    /// [`Rarer`](Self::Rarer) for a search with weakening. For a sequent
    /// with a `!` or a `?` the forest's own bias is `Rarer`, and proof
    /// search runs a search under each rule and answers with the first
    /// that decides, so it decides whatever either does: the backward one
    /// within the copy bound, the forward one within a bound of its own
    /// where the formulas under `!` and `?` are Horn clauses.
    #[default]
    Auto,
    /// The literal with fewer occurrences in the sequent is positive, `Var`
    /// when both have the same number. With Horn-like hypotheses this
    /// mostly chains backward from the goal, which keeps the copies per
    /// branch low.
    Rarer,
    /// The literal that is more often a direct factor of a `⊗` is
    /// positive, an occurrence counting half for every `&` or `⊕` above
    /// it; the rarer literal on a tie. A `⊗` with a positive literal for a
    /// factor takes exactly the dual literal for it, so its split needs no
    /// search. With Horn-like hypotheses this chains forward from the
    /// facts, one copy per step on a single branch: fast, and in need of a
    /// copy bound as large as the number of steps.
    Factors,
}

/// The polarity of a formula in focused proof search: positive connectives
/// (`⊗ 1 ⊕ 0 !`) have non-invertible rules and are decomposed under focus,
/// negative ones (`⅋ ⊥ & ⊤ ?`) have invertible rules and are decomposed
/// eagerly. A literal's polarity is the bias its atom was given, see
/// [`Forest::bias`] and [`Bias`].
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Polarity {
    /// `⊗ 1 ⊕ 0 !`, and the literals the bias makes positive.
    Positive,
    /// `⅋ ⊥ & ⊤ ?`, and the literals the bias makes negative.
    Negative,
}

impl Not for Polarity {
    type Output = Self;

    /// The other polarity.
    fn not(self) -> Self {
        match self {
            Polarity::Positive => Polarity::Negative,
            Polarity::Negative => Polarity::Positive,
        }
    }
}

impl Kind {
    /// Returns the sign of a literal kind, or `None` for a connective.
    pub const fn sign(self) -> Option<Sign> {
        match self {
            Kind::Var => Some(Sign::Var),
            Kind::DualVar => Some(Sign::DualVar),
            _ => None,
        }
    }

    /// Returns the polarity of a connective, or `None` for a literal, whose
    /// polarity is a per-atom choice.
    pub const fn polarity(self) -> Option<Polarity> {
        use Kind::*;
        match self {
            Var | DualVar => None,
            Tensor | One | Plus | Zero | Bang => Some(Polarity::Positive),
            Par | Bot | With | Top | Quest => Some(Polarity::Negative),
        }
    }
}

/// The subformula occurrences of a sequent, numbered so that every subtree is
/// a contiguous range of ids, with what proof search asks of each occurrence
/// and of each atom.
///
/// The numbering is a depth-first preorder walk: the roots in the order
/// [`Sequent::roots`] lists them, and below a connective its left subterm
/// before its right one. So a root comes before everything below it, the
/// left child of `o` is `o + 1`, the right child follows the left child's
/// subtree, and the subtree of `o` is exactly the ids `o .. o + size(o)`.
/// Two occurrences of one arena term are distinct ids that share a
/// [`TermId`].
///
/// A forest owns a copy of its sequent, so that an occurrence can be printed
/// as a formula, and every other per-occurrence datum is an array of `u32`
/// or smaller.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Forest, Kind, Sequent, Sign};
///
/// // ⊢ ~A, A ⊗ ~B, B
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let forest = Forest::new(&sequent)?;
/// assert_eq!(forest.len(), 5);
///
/// let tensor = forest.roots()[1];
/// assert_eq!(forest.kind(tensor), Kind::Tensor);
/// assert_eq!(forest.size(tensor), 3);
/// let (left, right) = (forest.left(tensor).unwrap(), forest.right(tensor).unwrap());
/// assert_eq!(forest.formula(left).to_string(), "A");
/// assert_eq!(forest.formula(right).to_string(), "~B");
/// assert_eq!(forest.parent(right), Some(tensor));
/// assert_eq!(forest.lca(left, right), Some(tensor));
///
/// // Every literal of an atom, by sign, in id order.
/// let a = sequent.atom("A").unwrap();
/// assert_eq!(forest.literals(a, Sign::Var), [left]);
/// assert_eq!(forest.literals(a, Sign::DualVar), [forest.roots()[0]]);
/// # Ok::<(), linlog::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Forest {
    /// The sequent the occurrences are of.
    sequent: Sequent,
    /// The occurrence of each root formula, in the sequent's order.
    roots: Box<[OccId]>,
    /// Per occurrence, its arena term.
    term: Box<[TermId]>,
    /// Per occurrence, the kind of its term.
    kind: Box<[Kind]>,
    /// Per occurrence, its parent, or `NONE` for a root.
    parent: Box<[u32]>,
    /// Per occurrence, the root it lies below (itself for a root).
    root: Box<[OccId]>,
    /// Per occurrence, the number of occurrences in its subtree, itself
    /// included.
    size: Box<[u32]>,
    /// Per occurrence, the number of ancestors it has.
    depth: Box<[u32]>,
    /// Per occurrence, its polarity, with the bias applied to literals.
    polarity: Box<[Polarity]>,
    /// Per atom, the sign of its positive literal.
    bias: Box<[Sign]>,
    /// Every literal occurrence, grouped by atom, then by sign (`Var` first),
    /// in ascending id order within a group.
    literals: Box<[OccId]>,
    /// Where each group of `literals` starts: group `2a` holds the `Var`
    /// literals of atom `a`, group `2a + 1` its `DualVar` literals, and the
    /// last entry is the total.
    literal_start: Box<[u32]>,
}

impl Forest {
    /// Builds the forest of a sequent, keeping a copy of it. Fails only if
    /// the sequent has more subformula occurrences than a `u32` can index,
    /// which needs an arena that shares subterms deeply.
    pub fn new(sequent: &Sequent) -> Result<Self, Error> {
        Self::try_from(sequent.clone())
    }

    /// Returns the sequent the forest was built from.
    pub fn sequent(&self) -> &Sequent {
        &self.sequent
    }

    /// Returns the number of occurrences, which is the width of an
    /// occurrence set of this forest.
    pub fn len(&self) -> usize {
        self.term.len()
    }

    /// Returns whether the sequent has no formula.
    pub fn is_empty(&self) -> bool {
        self.term.is_empty()
    }

    /// Returns every occurrence id in ascending order.
    pub fn ids(&self) -> impl DoubleEndedIterator<Item = OccId> + ExactSizeIterator {
        (0..self.term.len() as u32).map(OccId::new)
    }

    /// Returns the occurrence of each root formula, in the order the sequent
    /// lists them.
    pub fn roots(&self) -> &[OccId] {
        &self.roots
    }

    /// Returns the arena term of an occurrence.
    pub fn term(&self, o: OccId) -> TermId {
        self.term[o.index()]
    }

    /// Returns the kind of an occurrence's term.
    pub fn kind(&self, o: OccId) -> Kind {
        self.kind[o.index()]
    }

    /// Returns the occurrence as a value that prints its formula.
    pub fn formula(&self, o: OccId) -> Formula<'_> {
        self.sequent.formula(self.term(o))
    }

    /// Returns the parent of an occurrence, or `None` for a root.
    pub fn parent(&self, o: OccId) -> Option<OccId> {
        match self.parent[o.index()] {
            NONE => None,
            p => Some(OccId::new(p)),
        }
    }

    /// Returns the root formula an occurrence lies below, or itself if it is
    /// a root.
    pub fn root(&self, o: OccId) -> OccId {
        self.root[o.index()]
    }

    /// Returns whether two occurrences lie below the same root formula.
    pub fn same_root(&self, x: OccId, y: OccId) -> bool {
        self.root(x) == self.root(y)
    }

    /// Returns the number of occurrences in the subtree of `o`, itself
    /// included.
    pub fn size(&self, o: OccId) -> u32 {
        self.size[o.index()]
    }

    /// Returns the number of ancestors of an occurrence: 0 for a root.
    pub fn depth(&self, o: OccId) -> u32 {
        self.depth[o.index()]
    }

    /// Returns the polarity of an occurrence; for a literal, that is the bias
    /// of its atom.
    pub fn polarity(&self, o: OccId) -> Polarity {
        self.polarity[o.index()]
    }

    /// Returns whether the occurrence is a literal, `a` or `~a`.
    pub fn is_literal(&self, o: OccId) -> bool {
        self.kind(o).is_literal()
    }

    /// Returns the atom of a literal occurrence, or `None` for a connective.
    pub fn atom(&self, o: OccId) -> Option<Atom> {
        self.sequent.term(self.term(o)).atom()
    }

    /// Returns the sign of a literal occurrence, or `None` for a connective.
    pub fn sign(&self, o: OccId) -> Option<Sign> {
        self.kind(o).sign()
    }

    /// Returns the first child: the only subformula of `!` and `?`, the left
    /// one of a binary connective, and `None` for a literal or a unit.
    pub fn left(&self, o: OccId) -> Option<OccId> {
        (self.kind(o).arity() >= 1).then(|| OccId::new(o.0 + 1))
    }

    /// Returns the right child of a binary connective, and `None` otherwise.
    pub fn right(&self, o: OccId) -> Option<OccId> {
        (self.kind(o).arity() == 2).then(|| {
            let left = o.0 + 1;
            OccId::new(left + self.size[left as usize])
        })
    }

    /// Returns the children of an occurrence in order, left before right.
    pub fn children(&self, o: OccId) -> impl Iterator<Item = OccId> {
        self.left(o).into_iter().chain(self.right(o))
    }

    /// Returns the subtree of `o` in preorder: `o` itself first, then every
    /// occurrence below it, which are the ids `o .. o + size(o)`.
    pub fn subtree(&self, o: OccId) -> impl DoubleEndedIterator<Item = OccId> + ExactSizeIterator {
        (o.0..o.0 + self.size(o)).map(OccId::new)
    }

    /// Returns whether `o` is `ancestor` itself or lies below it.
    pub fn is_below(&self, o: OccId, ancestor: OccId) -> bool {
        ancestor.0 <= o.0 && o.0 < ancestor.0 + self.size(ancestor)
    }

    /// Returns the lowest common ancestor of two occurrences, or `None` if
    /// they lie below different roots. Walks up from `x`, so it costs the
    /// depth of `x` at most.
    pub fn lca(&self, x: OccId, y: OccId) -> Option<OccId> {
        if !self.same_root(x, y) {
            return None;
        }
        let mut a = x;
        while !self.is_below(y, a) {
            a = self.parent(a)?;
        }
        Some(a)
    }

    /// Returns the sign of the positive literal of an atom under
    /// [`Bias::Auto`]; the literal of the other sign is negative. Focused
    /// search is complete whatever the choice, so it is made for speed,
    /// from the sequent alone.
    pub fn bias(&self, atom: Atom) -> Sign {
        self.bias[atom.index()]
    }

    /// Returns, per atom, the sign of its positive literal under the rule
    /// given.
    pub fn bias_under(&self, rule: Bias) -> Box<[Sign]> {
        if rule == Bias::Auto {
            return self.bias.clone();
        }
        let terms = self.sequent.terms();
        Self::signs(
            rule,
            &self.kind,
            &self.parent,
            |o| terms[self.term[o].index()].atom().unwrap(),
            |a| {
                let count = |sign| self.literals(Atom::new(a as u32), sign).len() as u32;
                (count(Sign::Var), count(Sign::DualVar))
            },
            self.bias.len(),
        )
    }

    /// The sign of the positive literal of every atom under a rule, from
    /// the kinds and parents of the occurrences, the atom of a literal
    /// occurrence, and the number of `Var` and of `DualVar` literals of an
    /// atom.
    fn signs(
        rule: Bias,
        kind: &[Kind],
        parent: &[u32],
        atom: impl Fn(usize) -> Atom,
        literals: impl Fn(usize) -> (u32, u32),
        num_atoms: usize,
    ) -> Box<[Sign]> {
        // A `⊗` with a positive literal for a factor has its split forced,
        // so the literal that is more often such a factor is the positive
        // one; an occurrence counts half for every additive choice above
        // it, of which a proof takes one side.
        let by_factors = match rule {
            Bias::Auto => !kind.iter().any(|k| matches!(k, Kind::Bang | Kind::Quest)),
            Bias::Rarer => false,
            Bias::Factors => true,
        };
        let mut factors = vec![0u64; 2 * num_atoms];
        if by_factors {
            /// The weight of an occurrence under no additive choice.
            const WHOLE: u64 = 1 << 31;
            let mut choices = vec![0u8; kind.len()];
            for (o, k) in kind.iter().enumerate() {
                if parent[o] == NONE {
                    continue;
                }
                let p = parent[o] as usize;
                let additive = matches!(kind[p], Kind::With | Kind::Plus);
                choices[o] = choices[p].saturating_add(u8::from(additive));
                if let Some(s) = k.sign()
                    && kind[p] == Kind::Tensor
                {
                    factors[2 * atom(o).index() + s as usize] +=
                        WHOLE.checked_shr(u32::from(choices[o])).unwrap_or(0);
                }
            }
        }
        (0..num_atoms)
            .map(|a| {
                let (vars, duals) = literals(a);
                match factors[2 * a].cmp(&factors[2 * a + 1]) {
                    Ordering::Greater => Sign::Var,
                    Ordering::Less => Sign::DualVar,
                    Ordering::Equal if duals < vars => Sign::DualVar,
                    Ordering::Equal => Sign::Var,
                }
            })
            .collect()
    }

    /// Returns the occurrences of one literal of an atom, `a` or `~a`, in
    /// ascending id order.
    pub fn literals(&self, atom: Atom, sign: Sign) -> &[OccId] {
        let group = 2 * atom.index() + sign as usize;
        let (start, end) = (self.literal_start[group], self.literal_start[group + 1]);
        &self.literals[start as usize..end as usize]
    }

    /// Returns the occurrences of the positive literal of an atom, in
    /// ascending id order.
    pub fn positive_literals(&self, atom: Atom) -> &[OccId] {
        self.literals(atom, self.bias(atom))
    }

    /// Returns the occurrences of the negative literal of an atom, in
    /// ascending id order.
    pub fn negative_literals(&self, atom: Atom) -> &[OccId] {
        self.literals(atom, !self.bias(atom))
    }

    /// Returns every literal occurrence, grouped by atom, `a` before `~a`
    /// within an atom, and in ascending id order within a group.
    pub fn all_literals(&self) -> &[OccId] {
        &self.literals
    }
}

impl TryFrom<Sequent> for Forest {
    type Error = Error;

    /// Builds the forest of a sequent, taking ownership of it. Fails only if
    /// the sequent has more subformula occurrences than a `u32` can index.
    fn try_from(sequent: Sequent) -> Result<Self, Error> {
        let terms = sequent.terms();

        // The subtree size of every arena term, in one pass over the arena:
        // a subterm precedes its parent. Saturation keeps a deep DAG from
        // overflowing; anything at or beyond NONE is too large either way.
        let mut term_size = vec![0u64; terms.len()];
        for (n, term) in terms.iter().enumerate() {
            term_size[n] = term
                .subterms()
                .fold(1u64, |sum, k| sum.saturating_add(term_size[k.index()]));
        }
        let total = sequent
            .roots()
            .iter()
            .fold(0u64, |sum, r| sum.saturating_add(term_size[r.index()]));
        if total >= u64::from(NONE) {
            return Err(Error::TooManyOccurrences(total));
        }
        let n = total as usize;

        let mut roots = Vec::with_capacity(sequent.roots().len());
        let mut term = Vec::with_capacity(n);
        let mut kind = Vec::with_capacity(n);
        let mut parent = Vec::with_capacity(n);
        let mut root = Vec::with_capacity(n);
        let mut size = Vec::with_capacity(n);
        let mut depth = Vec::with_capacity(n);

        // Preorder: the right child is pushed first so the left one is
        // visited, with its whole subtree, before it.
        let mut stack: Vec<(TermId, u32, u32)> = Vec::new();
        for &r in sequent.roots() {
            let root_id = OccId::new(term.len() as u32);
            roots.push(root_id);
            stack.push((r, NONE, 0));
            while let Some((t, p, d)) = stack.pop() {
                let o = term.len() as u32;
                let node = terms[t.index()];
                term.push(t);
                kind.push(node.kind());
                parent.push(p);
                root.push(root_id);
                size.push(term_size[t.index()] as u32);
                depth.push(d);
                let subterms: Vec<TermId> = node.subterms().collect();
                for &k in subterms.iter().rev() {
                    stack.push((k, o, d + 1));
                }
            }
        }
        debug_assert_eq!(term.len(), n);

        // Literals: count per atom and sign, bias each atom, then lay the
        // groups out by a counting sort.
        let num_atoms = sequent.atom_names().len();
        let mut literal_start = vec![0u32; 2 * num_atoms + 1];
        for (o, k) in kind.iter().enumerate() {
            if let Some(s) = k.sign() {
                let a = terms[term[o].index()].atom().unwrap();
                literal_start[2 * a.index() + s as usize + 1] += 1;
            }
        }
        let bias = Self::signs(
            Bias::Auto,
            &kind,
            &parent,
            |o| terms[term[o].index()].atom().unwrap(),
            |a| (literal_start[2 * a + 1], literal_start[2 * a + 2]),
            num_atoms,
        );
        for g in 1..literal_start.len() {
            literal_start[g] += literal_start[g - 1];
        }
        let mut next = literal_start.clone();
        let mut literals = vec![OccId::new(NONE); literal_start[2 * num_atoms] as usize];
        let mut polarity = Vec::with_capacity(n);
        for (o, k) in kind.iter().enumerate() {
            polarity.push(match k.polarity() {
                Some(p) => p,
                None => {
                    let a = terms[term[o].index()].atom().unwrap();
                    let s = k.sign().unwrap();
                    let group = 2 * a.index() + s as usize;
                    literals[next[group] as usize] = OccId::new(o as u32);
                    next[group] += 1;
                    if s == bias[a.index()] {
                        Polarity::Positive
                    } else {
                        Polarity::Negative
                    }
                }
            });
        }

        Ok(Self {
            sequent,
            roots: roots.into_boxed_slice(),
            term: term.into_boxed_slice(),
            kind: kind.into_boxed_slice(),
            parent: parent.into_boxed_slice(),
            root: root.into_boxed_slice(),
            size: size.into_boxed_slice(),
            depth: depth.into_boxed_slice(),
            polarity: polarity.into_boxed_slice(),
            bias,
            literals: literals.into_boxed_slice(),
            literal_start: literal_start.into_boxed_slice(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequents::Term;

    /// Wraps a raw id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Builds the forest of `input`, or panics with the parse error.
    #[cfg(feature = "parse")]
    fn forest(input: &str) -> Forest {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        Forest::new(&s).unwrap()
    }

    /// Checks the invariants that hold in every forest: preorder ranges,
    /// parents against children, depths, roots, sizes, and the literal
    /// tables against the occurrences.
    fn check_invariants(f: &Forest) {
        let n = f.len() as u32;
        assert_eq!(f.ids().count(), n as usize);
        assert_eq!(
            f.roots().iter().map(|&r| f.size(r)).sum::<u32>(),
            n,
            "the roots' subtrees partition the ids"
        );
        for (i, &r) in f.roots().iter().enumerate() {
            let expected = f.roots()[..i].iter().map(|&q| f.size(q)).sum::<u32>();
            assert_eq!(r, o(expected), "root {i} starts after the roots before it");
            assert_eq!(f.parent(r), None);
            assert_eq!(f.depth(r), 0);
            assert_eq!(f.root(r), r);
            assert_eq!(f.term(r), f.sequent().roots()[i]);
        }
        for x in f.ids() {
            let node = f.sequent().term(f.term(x));
            assert_eq!(f.kind(x), node.kind());
            let children: Vec<OccId> = f.children(x).collect();
            assert_eq!(children.len(), node.kind().arity() as usize);
            assert_eq!(children.first().copied(), f.left(x));
            assert_eq!(children.get(1).copied(), f.right(x));
            let subterms: Vec<TermId> = node.subterms().collect();
            for (&c, &t) in children.iter().zip(&subterms) {
                assert_eq!(f.parent(c), Some(x), "the child's parent is the node");
                assert_eq!(f.depth(c), f.depth(x) + 1);
                assert_eq!(f.root(c), f.root(x));
                assert_eq!(f.term(c), t, "children come in the term's order");
                assert!(f.is_below(c, x));
            }
            assert_eq!(
                f.size(x),
                1 + children.iter().map(|&c| f.size(c)).sum::<u32>()
            );
            if let Some(l) = f.left(x) {
                assert_eq!(l, o(x.get() + 1), "the left child follows its parent");
            }
            if let (Some(l), Some(r)) = (f.left(x), f.right(x)) {
                assert_eq!(
                    r,
                    o(l.get() + f.size(l)),
                    "the right child follows the left subtree"
                );
            }
            // Every id in the range descends from x through parents, and no
            // id outside does.
            for y in f.ids() {
                let mut a = Some(y);
                while let Some(b) = a
                    && b != x
                {
                    a = f.parent(b);
                }
                assert_eq!(a == Some(x), f.is_below(y, x), "{y:?} below {x:?}");
                assert_eq!(f.subtree(x).any(|z| z == y), f.is_below(y, x));
            }
            match f.parent(x) {
                Some(p) => assert_eq!(f.depth(x), f.depth(p) + 1),
                None => assert!(f.roots().contains(&x)),
            }
            // Polarity, atom and sign agree with the term.
            assert_eq!(f.atom(x), node.atom());
            assert_eq!(f.sign(x), node.kind().sign());
            assert_eq!(f.is_literal(x), node.kind().is_literal());
            match node.kind().polarity() {
                Some(p) => assert_eq!(f.polarity(x), p),
                None => {
                    let positive = f.sign(x) == Some(f.bias(f.atom(x).unwrap()));
                    assert_eq!(f.polarity(x) == Polarity::Positive, positive);
                }
            }
        }
        // The literal tables list exactly the literal occurrences, sorted.
        let mut all: Vec<OccId> = Vec::new();
        for a in 0..f.sequent().atom_names().len() {
            let atom = Atom::new(a as u32);
            for sign in [Sign::Var, Sign::DualVar] {
                let list = f.literals(atom, sign);
                assert!(list.windows(2).all(|w| w[0] < w[1]), "sorted");
                for &l in list {
                    assert_eq!(f.atom(l), Some(atom));
                    assert_eq!(f.sign(l), Some(sign));
                }
                all.extend_from_slice(list);
            }
            assert_eq!(f.positive_literals(atom), f.literals(atom, f.bias(atom)));
            assert_eq!(f.negative_literals(atom), f.literals(atom, !f.bias(atom)));
        }
        assert_eq!(all, f.all_literals());
        let mut literal_ids: Vec<OccId> = f.ids().filter(|&x| f.is_literal(x)).collect();
        all.sort();
        literal_ids.sort();
        assert_eq!(all, literal_ids);
    }

    /// The forest of the empty sequent has nothing in it.
    #[test]
    fn empty() {
        let f = Forest::new(&Sequent::new()).unwrap();
        assert!(f.is_empty());
        assert_eq!(f.len(), 0);
        assert!(f.roots().is_empty());
        assert!(f.all_literals().is_empty());
        check_invariants(&f);
    }

    /// The numbering of `⊢ ~A, A ⊗ ~B, B` is preorder with the left subterm
    /// first, and the queries return what the layout says.
    #[cfg(feature = "parse")]
    #[test]
    fn layout() {
        let f = forest("A, A -o B |- B");
        check_invariants(&f);
        assert_eq!(f.len(), 5);
        assert_eq!(f.roots(), [o(0), o(1), o(4)]);
        let kinds: Vec<Kind> = f.ids().map(|x| f.kind(x)).collect();
        use Kind::*;
        assert_eq!(kinds, [DualVar, Tensor, Var, DualVar, Var]);
        assert_eq!(f.formula(o(1)).to_string(), "A ⊗ ~B");
        assert_eq!(f.left(o(1)), Some(o(2)));
        assert_eq!(f.right(o(1)), Some(o(3)));
        assert_eq!(f.subtree(o(1)).collect::<Vec<_>>(), [o(1), o(2), o(3)]);
        assert_eq!(f.size(o(1)), 3);
        assert_eq!(f.depth(o(3)), 1);
        assert_eq!(f.root(o(3)), o(1));
        assert!(f.same_root(o(2), o(3)));
        assert!(!f.same_root(o(0), o(2)));
    }

    /// Nested and repeated formulas keep every invariant, including shared
    /// subterms and repeated roots.
    #[cfg(feature = "parse")]
    #[test]
    fn invariants_on_a_table() {
        for input in [
            "|- A",
            "|- A, A",
            "|- A * A",
            "|- (A * B), (A * B)",
            "|- !(A & B) + ?(A * B), 1, bot, top, 0",
            "A -o B -o C, (A -o B) -o C |- (A * B) + (0 & top)",
            "!A, ?B, A * B, C par D |- ~A, B^, A & B, C + D",
            "|- ((A * B) * (A * B)) * ((A * B) * (A * B))",
        ] {
            check_invariants(&forest(input));
        }
    }

    /// The lowest common ancestor is the deepest node above both, and
    /// nothing across roots.
    #[cfg(feature = "parse")]
    #[test]
    fn lowest_common_ancestor() {
        // ⊢ (A ⊗ B) ⅋ (C ⊗ D), E
        let f = forest("|- (A * B) par (C * D), E");
        check_invariants(&f);
        assert_eq!(f.lca(o(2), o(3)), Some(o(1)));
        assert_eq!(f.lca(o(2), o(5)), Some(o(0)));
        assert_eq!(f.lca(o(3), o(6)), Some(o(0)));
        assert_eq!(f.lca(o(1), o(4)), Some(o(0)));
        assert_eq!(f.lca(o(0), o(6)), Some(o(0)));
        assert_eq!(f.lca(o(6), o(0)), Some(o(0)));
        assert_eq!(f.lca(o(5), o(5)), Some(o(5)));
        assert_eq!(f.lca(o(2), o(7)), None);
        assert_eq!(f.lca(o(7), o(7)), Some(o(7)));
    }

    /// Where no literal is a factor of a `⊗`, the literal with fewer
    /// occurrences is positive and a tie makes the atom positive; every
    /// occurrence is listed once, by atom and sign.
    #[cfg(feature = "parse")]
    #[test]
    fn literals_and_bias() {
        // ⊢ ~A, ~A, A, B, ~C ⅋ C, ~C ⅋ C
        let f = forest("A, A |- A, B, C -o C, C -o C");
        check_invariants(&f);
        let s = f.sequent();
        let (a, b, c) = (
            s.atom("A").unwrap(),
            s.atom("B").unwrap(),
            s.atom("C").unwrap(),
        );
        assert_eq!(f.bias(a), Sign::Var, "one A against two ~A");
        assert_eq!(
            f.bias(b),
            Sign::DualVar,
            "no ~B at all, so ~B is the rarer one"
        );
        assert_eq!(f.bias(c), Sign::Var, "two of each");
        assert_eq!(f.literals(a, Sign::Var), [o(2)]);
        assert_eq!(f.literals(a, Sign::DualVar), [o(0), o(1)]);
        assert_eq!(f.positive_literals(a), [o(2)]);
        assert_eq!(f.negative_literals(a), [o(0), o(1)]);
        assert_eq!(f.literals(b, Sign::Var), [o(3)]);
        assert_eq!(f.literals(b, Sign::DualVar), []);
        assert_eq!(f.literals(c, Sign::Var), [o(6), o(9)]);
        assert_eq!(f.literals(c, Sign::DualVar), [o(5), o(8)]);
        assert_eq!(f.polarity(o(2)), Polarity::Positive);
        assert_eq!(f.polarity(o(0)), Polarity::Negative);
        assert_eq!(f.polarity(o(3)), Polarity::Negative, "B is never a focus");

        // ⊢ A, A, ~A: the negation is rarer, so it is positive.
        let f = forest("|- A, A, ~A");
        check_invariants(&f);
        let a = f.sequent().atom("A").unwrap();
        assert_eq!(f.bias(a), Sign::DualVar);
        assert_eq!(f.polarity(o(0)), Polarity::Negative);
        assert_eq!(f.polarity(o(2)), Polarity::Positive);
    }

    /// The literal that is more often a factor of a `⊗` is positive, an
    /// occurrence under an additive choice counting half; with an
    /// exponential in the sequent the rarer literal is.
    #[cfg(feature = "parse")]
    #[test]
    fn bias_by_tensor_factors() {
        let bias = |input: &str, atom: &str| {
            let f = forest(input);
            check_invariants(&f);
            f.bias(f.sequent().atom(atom).unwrap())
        };
        // `A` is a factor once and `~A` never, though `A` is not the rarer.
        let horn = "|- ~A, A * ~B, A * ~B, B";
        assert_eq!(bias(horn, "A"), Sign::Var);
        assert_eq!(bias(horn, "B"), Sign::DualVar);
        // `~D` is a factor under each side of a choice, `D` once outside
        // one: a tie, which the rarer literal wins.
        let choice = "|- (A * ~D) + (B * ~D), D * C, ~A, ~B, ~C";
        assert_eq!(bias(choice, "D"), Sign::Var);
        let exponential = "|- ?~A, A * ~B, A * ~B, B";
        assert_eq!(bias(exponential, "A"), Sign::DualVar);
        // The rules by name, whatever the sequent.
        let under = |input: &str, rule: Bias| {
            let f = forest(input);
            f.bias_under(rule)[f.sequent().atom("A").unwrap().index()]
        };
        assert_eq!(under(exponential, Bias::Factors), Sign::Var);
        assert_eq!(under(exponential, Bias::Auto), Sign::DualVar);
        assert_eq!(under(horn, Bias::Rarer), Sign::DualVar);
    }

    /// Connectives have their fixed polarities.
    #[cfg(feature = "parse")]
    #[test]
    fn connective_polarities() {
        let f = forest("|- (A * B) par (A & B) + (!A par ?B), 1, bot, top, 0");
        check_invariants(&f);
        let by_kind = |k: Kind| {
            f.ids()
                .filter(|&x| f.kind(x) == k)
                .map(|x| f.polarity(x))
                .collect::<Vec<_>>()
        };
        use Kind::*;
        use Polarity::*;
        for (k, p) in [
            (Tensor, Positive),
            (Plus, Positive),
            (One, Positive),
            (Zero, Positive),
            (Bang, Positive),
            (Par, Negative),
            (With, Negative),
            (Bot, Negative),
            (Top, Negative),
            (Quest, Negative),
        ] {
            assert!(!by_kind(k).is_empty(), "{k:?} occurs");
            assert!(by_kind(k).iter().all(|&q| q == p), "{k:?} is {p:?}");
        }
    }

    /// A shared subterm is walked once per occurrence, and a deeply shared
    /// arena is refused rather than mis-numbered.
    #[test]
    fn sharing() {
        // Var(0) shared by two roots and inside a tensor of itself.
        let s = Sequent {
            terms: vec![
                Term::Var(Atom::new(0)),
                Term::Tensor(TermId::new(0), TermId::new(0)),
            ],
            roots: vec![TermId::new(1), TermId::new(0), TermId::new(1)],
            atoms: vec!["A".into()],
        };
        let f = Forest::new(&s).unwrap();
        check_invariants(&f);
        assert_eq!(f.len(), 7);
        assert_eq!(f.roots(), [o(0), o(3), o(4)]);
        assert_eq!(
            f.literals(Atom::new(0), Sign::Var),
            [o(1), o(2), o(3), o(5), o(6)]
        );

        // A chain of 40 tensors of the term below doubles 40 times.
        let mut terms = vec![Term::One];
        for k in 0..40u32 {
            terms.push(Term::Tensor(TermId::new(k), TermId::new(k)));
        }
        let s = Sequent {
            terms,
            roots: vec![TermId::new(40)],
            atoms: vec![],
        };
        assert!(matches!(
            Forest::try_from(s),
            Err(Error::TooManyOccurrences(n)) if n == (1u64 << 41) - 1
        ));
    }
}
