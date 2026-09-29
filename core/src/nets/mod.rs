// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof nets for unit-free MLL, with or without Mix: proof structures over
//! the occurrence forest, the correctness criterion, and the conversions
//! to and from proof terms.
//!
//! A [`ProofStructure`] is the sequent's formula trees, which the forest
//! already holds, plus axiom links between dual literal occurrences. It is
//! complete when every literal is linked. A complete structure is a proof
//! net when it passes the Danos–Regnier criterion, which
//! [`is_correct`](ProofStructure::is_correct) decides independently of any
//! search and of the proof checker: no switching cycle, and without Mix
//! exactly one connected component under every switching. A proof net
//! sequentializes into a proof term that the checker accepts, and a proof
//! term of MLL desequentializes into the net of its axiom links; two
//! derivations that differ only by the order of their rules give the same
//! net.

/// The coloured structure graph and the criterion's tests on it.
mod graph;
/// From a proof net to a proof term.
mod sequentialize;
/// The `⅋`-free skeleton as a union-find with undo.
mod skeleton;

pub use graph::Scratch;

use crate::Error;
use crate::fragment::Fragment;
use crate::occurrences::{Forest, OccId};
use crate::proofs::{Node, Proof};
use crate::sequents::{Kind, Sequent};
use graph::Graph;
use skeleton::Skeleton;
use std::fmt::{Display, Formatter, Result as FmtResult};
use thiserror::Error as ThisError;

/// The raw index that stands for "no occurrence".
const NONE: u32 = u32::MAX;

/// Why a proof structure is not a proof net, or why a list of links is not
/// a proof structure. Occurrences print as ids; [`describe`](Self::describe)
/// prints them as formulas.
#[derive(ThisError, Clone, Debug, PartialEq, Eq)]
pub enum NetError {
    /// A link names an occurrence that is not a literal.
    #[error("occurrence {} is not a literal", .0.get())]
    NotLiteral(OccId),
    /// A link joins two literals that are not `a` and `~a` for one atom.
    #[error("the literals {} and {} are not dual", .0.get(), .1.get())]
    NotDual(OccId, OccId),
    /// A literal appears in two links.
    #[error("literal {} is linked twice", .0.get())]
    LinkedTwice(OccId),
    /// A literal has no link, so the structure is incomplete.
    #[error("literal {} has no axiom link", .0.get())]
    Unlinked(OccId),
    /// The structure has no occurrence at all: no rule concludes the empty
    /// sequent.
    #[error("the structure is empty, and no rule concludes the empty sequent")]
    Empty,
    /// A cycle survives some switching: the occurrences it runs through,
    /// in order along the cycle.
    #[error("a switching cycle runs through {}", ids(.0))]
    SwitchingCycle(Vec<OccId>),
    /// Every switching falls into several parts. The parts are those of the
    /// switching that keeps the left premise of every `⅋`, each given by
    /// the occurrences in it that have no parent edge there: the roots and
    /// the right premises of `⅋` nodes.
    #[error("every switching falls into {} parts; keeping every left premise, they are {}", .0.len(), parts(.0))]
    Disconnected(Vec<Vec<OccId>>),
}

/// Writes occurrence ids separated by commas.
fn ids(ids: &[OccId]) -> String {
    ids.iter()
        .map(|o| o.get().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Writes lists of occurrence ids as `{0, 1} and {2}`.
fn parts(parts: &[Vec<OccId>]) -> String {
    parts
        .iter()
        .map(|p| format!("{{{}}}", ids(p)))
        .collect::<Vec<_>>()
        .join(" and ")
}

impl NetError {
    /// Returns the error as a value that prints occurrences as formulas
    /// followed by their id in brackets, as in `~A[0]`.
    pub fn describe<'a>(&'a self, forest: &'a Forest) -> Described<'a> {
        Described {
            error: self,
            forest,
        }
    }
}

/// A [`NetError`] printed with the formulas of its occurrences.
#[derive(Clone, Copy, Debug)]
pub struct Described<'a> {
    /// The error.
    error: &'a NetError,
    /// The forest of the structure that failed.
    forest: &'a Forest,
}

impl Display for Described<'_> {
    /// Writes the error with `formula[id]` in place of every occurrence id.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let occ = |o: &OccId| format!("{}[{}]", self.forest.formula(*o), o.get());
        let list = |os: &[OccId]| os.iter().map(occ).collect::<Vec<_>>().join(", ");
        use NetError::*;
        match self.error {
            NotLiteral(o) => write!(f, "occurrence {} is not a literal", occ(o)),
            NotDual(x, y) => write!(f, "the literals {} and {} are not dual", occ(x), occ(y)),
            LinkedTwice(o) => write!(f, "literal {} is linked twice", occ(o)),
            Unlinked(o) => write!(f, "literal {} has no axiom link", occ(o)),
            Empty => write!(f, "{}", self.error),
            SwitchingCycle(cycle) => write!(f, "a switching cycle runs through {}", list(cycle)),
            Disconnected(parts) => {
                let parts: Vec<String> = parts.iter().map(|p| format!("{{{}}}", list(p))).collect();
                write!(
                    f,
                    "every switching falls into {} parts; keeping every left premise, they are {}",
                    parts.len(),
                    parts.join(" and ")
                )
            }
        }
    }
}

/// A proof structure of unit-free MLL over the occurrence forest of its
/// sequent: the formula trees, which the forest holds, plus axiom links
/// between dual literal occurrences, and whether the Mix rule is allowed,
/// which decides what [`is_correct`](Self::is_correct) requires.
///
/// Links are added with [`link`](Self::link) and taken back in the reverse
/// order with [`unlink`](Self::unlink), both in constant time, so that a
/// search can try linkings by backtracking. The structure keeps the
/// `⅋`-free skeleton (the premise edges of `⊗` nodes and the links) as a
/// union-find, so that [`same_component`](Self::same_component) answers in
/// logarithmic time whether a candidate link would close a cycle that no
/// switching breaks.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Forest, OccId, ProofStructure, Sequent};
///
/// // ⊢ ~A, A ⊗ ~B, B, with the occurrences 0: ~A, 1: A ⊗ ~B, 2: A,
/// // 3: ~B, 4: B.
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let o = OccId::new;
/// let net = ProofStructure::from_links(Forest::new(&sequent)?, false, &[(o(0), o(2)), (o(3), o(4))])?;
/// assert!(net.is_complete());
/// assert_eq!(net.partner(o(2)), Some(o(0)));
/// # Ok::<(), linlog::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct ProofStructure {
    /// The forest of the sequent.
    forest: Forest,
    /// Whether the Mix rule is allowed, in which case a proof net need not
    /// be connected.
    mix: bool,
    /// Per occurrence, the literal it is linked to, or `NONE`.
    partner: Box<[u32]>,
    /// The links in the order they were made, each as the pair given to
    /// [`link`](Self::link).
    links: Vec<(OccId, OccId)>,
    /// The coloured structure graph.
    graph: Graph,
    /// The `⅋`-free skeleton.
    skeleton: Skeleton,
}

impl ProofStructure {
    /// Returns the structure over the forest with no link yet. Fails if the
    /// sequent lies outside unit-free MLL, where there are no proof nets.
    pub fn new(forest: Forest, mix: bool) -> Result<Self, Error> {
        let fragment = forest.sequent().fragment();
        if !Fragment::MLL.contains(fragment) {
            return Err(Error::NetFragment(fragment));
        }
        let n = forest.len();
        let mut skeleton = Skeleton::new(n);
        for o in forest.ids() {
            if let Some(p) = forest.parent(o)
                && forest.kind(p) == Kind::Tensor
            {
                skeleton.union(p.get(), o.get());
            }
        }
        let literals = forest.all_literals().len();
        skeleton.commit(literals / 2);
        Ok(Self {
            graph: Graph::new(&forest),
            forest,
            mix,
            partner: vec![NONE; n].into_boxed_slice(),
            links: Vec::with_capacity(literals / 2),
            skeleton,
        })
    }

    /// Returns the structure with the given links. Fails as
    /// [`new`](Self::new) does, if a link names an occurrence outside the
    /// forest, and with a [`NetError`] if a link does not join two dual
    /// literals or a literal is linked twice.
    pub fn from_links(forest: Forest, mix: bool, links: &[(OccId, OccId)]) -> Result<Self, Error> {
        let mut net = Self::new(forest, mix)?;
        for &(x, y) in links {
            net.check_link(x, y)?;
            net.link(x, y);
        }
        Ok(net)
    }

    /// Desequentializes a proof of MLL: reads the axiom links off its `Ax`
    /// nodes and builds their structure over a copy of the proof's forest.
    /// Every proof the checker accepts gives a proof net; two proofs that
    /// differ only in the order of their rules give the same one. Fails as
    /// [`from_links`](Self::from_links) does, and with the criterion's
    /// error if the links are not a proof net, which happens only for a
    /// proof the checker rejects or one that uses Mix when `mix` is off.
    /// The checker is not run: the net of a term the checker would reject
    /// can still be a proof net, and is returned as one.
    pub fn from_proof(proof: &Proof, mix: bool) -> Result<Self, Error> {
        let links: Vec<(OccId, OccId)> = proof
            .nodes()
            .iter()
            .filter_map(|n| match *n {
                Node::Ax(x, y) => Some((x, y)),
                _ => None,
            })
            .collect();
        let net = Self::from_links(proof.forest().clone(), mix, &links)?;
        net.is_correct()?;
        Ok(net)
    }

    /// Fails if `x` and `y` are not two unlinked dual literals of the
    /// forest.
    fn check_link(&self, x: OccId, y: OccId) -> Result<(), Error> {
        let f = &self.forest;
        for o in [x, y] {
            if o.index() >= f.len() {
                return Err(Error::OccurrenceIndexOutOfBounds(o.index(), f.len()));
            }
            if !f.is_literal(o) {
                return Err(NetError::NotLiteral(o).into());
            }
        }
        if !self.dual(x, y) {
            return Err(NetError::NotDual(x, y).into());
        }
        for o in [x, y] {
            if self.partner(o).is_some() {
                return Err(NetError::LinkedTwice(o).into());
            }
        }
        Ok(())
    }

    /// Returns whether two literal occurrences are `a` and `~a` for one atom.
    fn dual(&self, x: OccId, y: OccId) -> bool {
        let f = &self.forest;
        f.atom(x) == f.atom(y) && f.sign(x) != f.sign(y)
    }

    /// Returns the forest of the sequent.
    pub fn forest(&self) -> &Forest {
        &self.forest
    }

    /// Returns the sequent.
    pub fn sequent(&self) -> &Sequent {
        self.forest.sequent()
    }

    /// Returns whether the Mix rule is allowed.
    pub fn mix(&self) -> bool {
        self.mix
    }

    /// Returns the literal an occurrence is linked to, or `None` for an
    /// unlinked literal or a connective.
    pub fn partner(&self, o: OccId) -> Option<OccId> {
        match self.partner[o.index()] {
            NONE => None,
            p => Some(OccId::new(p)),
        }
    }

    /// Returns the links in the order they were made.
    pub fn links(&self) -> &[(OccId, OccId)] {
        &self.links
    }

    /// Returns whether every literal is linked.
    pub fn is_complete(&self) -> bool {
        2 * self.links.len() == self.forest.all_literals().len()
    }

    /// Returns the literals without a link, grouped by atom and sign as
    /// [`Forest::all_literals`] lists them.
    pub fn unlinked(&self) -> impl Iterator<Item = OccId> {
        self.forest
            .all_literals()
            .iter()
            .copied()
            .filter(|&l| self.partner[l.index()] == NONE)
    }

    /// Links two unlinked dual literals, in constant time. The caller
    /// guarantees the literals are dual and unlinked; a debug build checks.
    pub fn link(&mut self, x: OccId, y: OccId) {
        debug_assert!(
            self.check_link(x, y).is_ok(),
            "{x:?} and {y:?} cannot be linked"
        );
        self.partner[x.index()] = y.get();
        self.partner[y.index()] = x.get();
        self.links.push((x, y));
        self.skeleton.union(x.get(), y.get());
        self.graph.link(x.get(), y.get());
    }

    /// Takes back the last link made and returns it, or `None` if there is
    /// no link. Links are undone in the reverse order of their making, as a
    /// backtracking search does.
    pub fn unlink(&mut self) -> Option<(OccId, OccId)> {
        let (x, y) = self.links.pop()?;
        self.partner[x.index()] = NONE;
        self.partner[y.index()] = NONE;
        self.skeleton.undo();
        self.graph.unlink(x.get(), y.get());
        Some((x, y))
    }

    /// Returns whether two occurrences are joined by a path that uses no
    /// premise edge of a `⅋`: `⊗` premise edges and links only. Linking two
    /// such literals closes a cycle that every switching keeps.
    pub fn same_component(&self, x: OccId, y: OccId) -> bool {
        self.skeleton.same(x.get(), y.get())
    }

    /// Returns working memory for the tests of the criterion, sized for
    /// this structure; one serves any number of calls.
    pub fn scratch(&self) -> Scratch {
        self.graph.scratch()
    }

    /// Returns whether no cycle survives any switching, by the deletion
    /// procedure of Yeo's theorem: a vertex that every part of the graph
    /// without it meets in edges of one colour only lies on no switching
    /// cycle and goes; when none is left to delete, what remains carries a
    /// switching cycle, or nothing remains. Meaningful on a partial
    /// structure too: an unlinked literal lies on no cycle. Allocates
    /// nothing; the scratch comes from [`scratch`](Self::scratch).
    pub fn is_acyclic(&self, scratch: &mut Scratch) -> bool {
        scratch.restore();
        self.graph.acyclic(&self.forest, scratch)
    }

    /// Decides whether the structure is a proof net, independently of any
    /// search and of the proof checker: every literal is linked, no cycle
    /// survives any switching, and, unless Mix is allowed, every switching
    /// is connected, which given acyclicity is the count of edges a
    /// switching keeps being one less than the number of occurrences. The
    /// error names an unlinked literal, a switching cycle or the parts the
    /// structure falls into.
    pub fn is_correct(&self) -> Result<(), NetError> {
        if self.forest.is_empty() {
            return Err(NetError::Empty);
        }
        if let Some(l) = self.unlinked().next() {
            return Err(NetError::Unlinked(l));
        }
        let mut scratch = self.scratch();
        if !self.is_acyclic(&mut scratch) {
            return Err(NetError::SwitchingCycle(
                self.graph.cycle(&self.forest, &mut scratch),
            ));
        }
        if !self.mix && self.graph.switched_edges(self.links.len()) + 1 != self.forest.len() {
            return Err(NetError::Disconnected(
                self.graph.parts(&self.forest, &mut scratch),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fragment::Mode;
    use crate::proofs::NodeId;

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

    /// Links are made and taken back in stack order, the partners and the
    /// skeleton follow, and completeness is every literal linked.
    #[cfg(feature = "parse")]
    #[test]
    fn link_and_unlink() {
        // ⊢ ~A, A ⊗ ~B, B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B.
        let mut net = ProofStructure::new(forest("A, A -o B |- B"), false).unwrap();
        assert!(!net.is_complete());
        assert_eq!(net.unlinked().collect::<Vec<_>>(), [o(2), o(0), o(4), o(3)]);
        assert!(net.same_component(o(2), o(3)), "the ⊗ joins its premises");
        assert!(!net.same_component(o(0), o(2)));

        net.link(o(0), o(2));
        assert_eq!(net.partner(o(2)), Some(o(0)));
        assert_eq!(net.partner(o(1)), None);
        assert!(net.same_component(o(0), o(3)), "through the link and the ⊗");
        net.link(o(4), o(3));
        assert!(net.is_complete());
        assert_eq!(net.links(), [(o(0), o(2)), (o(4), o(3))]);

        assert_eq!(net.unlink(), Some((o(4), o(3))));
        assert_eq!(net.partner(o(4)), None);
        assert!(!net.same_component(o(4), o(0)));
        assert_eq!(net.unlink(), Some((o(0), o(2))));
        assert!(!net.same_component(o(0), o(3)));
        assert_eq!(net.unlink(), None);
        assert_eq!(net.unlinked().count(), 4);
    }

    /// Two derivations that differ only in the order of their rules give
    /// the same net, and a proof that is not one of MLL is refused.
    #[cfg(feature = "parse")]
    #[test]
    fn desequentialize() {
        use Node::*;
        let (n, mode) = (NodeId::new, Mode::CLASSICAL);
        // ⊢ A ⊗ B, ~A, ~B ⊗ C, ~C: 0 ⊗, 1 A, 2 B, 3 ~A, 4 ⊗, 5 ~B, 6 C, 7 ~C.
        let f = forest("|- A * B, ~A, ~B * C, ~C");
        let first_tensor_first = Proof::new(
            f.clone(),
            vec![
                Ax(o(1), o(3)),
                Ax(o(2), o(5)),
                Ax(o(6), o(7)),
                Tensor(o(4), n(1), n(2)),
                Tensor(o(0), n(0), n(3)),
            ],
            n(4),
        )
        .unwrap();
        let second_tensor_first = Proof::new(
            f.clone(),
            vec![
                Ax(o(1), o(3)),
                Ax(o(2), o(5)),
                Tensor(o(0), n(0), n(1)),
                Ax(o(6), o(7)),
                Tensor(o(4), n(2), n(3)),
            ],
            n(4),
        )
        .unwrap();
        assert_eq!(first_tensor_first.check(mode), Ok(()));
        assert_eq!(second_tensor_first.check(mode), Ok(()));
        let a = ProofStructure::from_proof(&first_tensor_first, false).unwrap();
        let b = ProofStructure::from_proof(&second_tensor_first, false).unwrap();
        assert_eq!(links(&a), links(&b));
        assert_eq!(links(&a), [(o(1), o(3)), (o(2), o(5)), (o(6), o(7))]);

        // A proof with Mix desequentializes only when Mix is allowed.
        let f = forest("|- A par B, ~A, ~B");
        let with_mix = Proof::new(
            f,
            vec![
                Ax(o(1), o(3)),
                Ax(o(2), o(4)),
                Mix(n(0), n(1)),
                Par(o(0), n(2)),
            ],
            n(3),
        )
        .unwrap();
        assert!(ProofStructure::from_proof(&with_mix, true).is_ok());
        assert!(matches!(
            ProofStructure::from_proof(&with_mix, false),
            Err(Error::InvalidNet(NetError::Disconnected(_)))
        ));
        // Outside MLL there is no net.
        let f = forest("|- A & A, ~A");
        let additive = Proof::new(
            f,
            vec![Ax(o(1), o(3)), Ax(o(2), o(3)), With(o(0), n(0), n(1))],
            n(2),
        )
        .unwrap();
        assert!(matches!(
            ProofStructure::from_proof(&additive, false),
            Err(Error::NetFragment(_))
        ));
    }

    /// Random derivations of MLL, with and without Mix, round-trip through
    /// their net: the net is correct, the sequentialized proof passes the
    /// checker, and it has the same net as the derivation it came from.
    #[cfg(feature = "parse")]
    #[test]
    fn round_trip_random_derivations() {
        use crate::search::generate::{self, Rng, Rules};
        use crate::search::{Options, prove};
        for (seed, mix) in [(3, false), (4, true)] {
            let rules = Rules {
                units: false,
                additives: false,
                mix,
            };
            let mode = if mix {
                Mode::CLASSICAL.with_mix()
            } else {
                Mode::CLASSICAL
            };
            let mut rng = Rng::new(seed);
            for _ in 0..60 {
                let budget = 2 + rng.below(14);
                let formulas = generate::provable(&mut rng, rules, 3, budget);
                let text = generate::sequent(&formulas);
                let s: Sequent = text.parse().unwrap();
                let outcome = prove(&s, mode, &Options::default()).unwrap();
                let proof = outcome.verdict.proof().expect("the sequent is provable");
                assert_eq!(proof.check(mode), Ok(()), "{text:?}");
                let net = ProofStructure::from_proof(proof, mix).unwrap_or_else(|e| panic!("{text:?}: {e}"));
                let back = net.sequentialize().unwrap_or_else(|e| panic!("{text:?}: {e}"));
                assert_eq!(back.check(mode), Ok(()), "{text:?}");
                let again = ProofStructure::from_proof(&back, mix).unwrap();
                assert_eq!(links(&again), links(&net), "{text:?}");
            }
        }
    }

    /// The links of a structure as sorted pairs, for comparing nets.
    fn links(net: &ProofStructure) -> Vec<(OccId, OccId)> {
        let mut links: Vec<(OccId, OccId)> = net
            .links()
            .iter()
            .map(|&(x, y)| (x.min(y), x.max(y)))
            .collect();
        links.sort();
        links
    }

    /// A list of links is validated: literals only, dual, each at most
    /// once, inside the forest; and only unit-free MLL has structures.
    #[cfg(feature = "parse")]
    #[test]
    fn validation() {
        // ⊢ ~A, A, A ⊗ ~B, B: 0 ~A, 1 A, 2 ⊗, 3 A, 4 ~B, 5 B.
        let f = || forest("A, A -o B |- B, A");
        let net = ProofStructure::from_links(f(), false, &[(o(0), o(1))]).unwrap();
        assert!(!net.is_complete());
        for (links, error) in [
            (vec![(o(2), o(0))], NetError::NotLiteral(o(2))),
            (vec![(o(0), o(4))], NetError::NotDual(o(0), o(4))),
            (vec![(o(1), o(3))], NetError::NotDual(o(1), o(3))),
            (vec![(o(0), o(1)), (o(0), o(3))], NetError::LinkedTwice(o(0))),
        ] {
            match ProofStructure::from_links(f(), false, &links) {
                Err(Error::InvalidNet(e)) => assert_eq!(e, error, "{links:?}"),
                other => panic!("{links:?}: {other:?}"),
            }
        }
        assert!(matches!(
            ProofStructure::from_links(f(), false, &[(o(0), o(9))]),
            Err(Error::OccurrenceIndexOutOfBounds(9, 6))
        ));
        for input in ["|- 1", "|- A & B", "|- !A"] {
            assert!(
                matches!(
                    ProofStructure::new(forest(input), true),
                    Err(Error::NetFragment(_))
                ),
                "{input}"
            );
        }
        assert!(ProofStructure::new(forest("|-"), true).is_ok());
    }
}
