// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The proof checker: it derives, node by node, the sequent a proof term
//! proves under the rules of the dyadic sequent calculus and accepts the
//! term only if the root derives the proof's sequent. It is the reference
//! for what a proof is, so it shares no code with any search engine and must
//! never: an engine's proofs are validated by something that cannot repeat
//! the engine's mistakes.
//!
//! The pass is bottom-up, one node at a time in arena order, so it costs one
//! multiset operation per node and never recurses. What a subproof proves is
//! a `Derived` sequent; the one rule whose conclusion the premises do not
//! determine, `⊤`, is handled by a flag that says the subproof proves its
//! sequent under any further context.

use super::multiset::Multiset;
use super::{Node, NodeId, Proof, Side};
use crate::fragment::Mode;
use crate::occurrences::{Forest, OccId, OccSet};
use crate::sequents::Kind;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// What a subproof proves, as the checker derives it from the node's
/// premises: the dyadic sequent `⊢ Θ ; Γ`, or with `any` set `⊢ Θ ; Γ, Δ` for
/// every `Δ`, because a `⊤` leaf above absorbs whatever context it is given.
///
/// `Θ` is the least unrestricted zone the subproof needs, the occurrences it
/// copies without moving them into `Θ` itself; since weakening and
/// contraction on `Θ` are implicit, any larger zone does as well. So a proof
/// is correct when its root needs an empty `Θ` and derives the sequent's
/// formulas, and the checker never has to know the actual zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Derived {
    /// The unrestricted zone the subproof needs.
    pub(crate) theta: OccSet,
    /// The linear zone.
    pub(crate) gamma: Multiset,
    /// Whether a `⊤` above absorbs any further linear context.
    pub(crate) any: bool,
}

impl Derived {
    /// Returns the sequent as ids for an error report.
    fn to_dyadic(&self) -> Dyadic {
        Dyadic {
            theta: self.theta.iter().collect(),
            gamma: self.gamma.as_slice().to_vec(),
            any: self.any,
        }
    }
}

/// A dyadic sequent as the checker derived it, by occurrence ids: the
/// unrestricted zone `Θ`, the linear zone `Γ` with repeats, and whether a
/// `⊤` above absorbs any further linear context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dyadic {
    /// The unrestricted zone, ascending.
    pub theta: Vec<OccId>,
    /// The linear zone, ascending, with repeats.
    pub gamma: Vec<OccId>,
    /// Whether the linear zone may hold anything more.
    pub any: bool,
}

impl Display for Dyadic {
    /// Writes `⊢ Θ ; Γ` with the zones as ids, omitting `Θ ;` when it is
    /// empty, and `…` after `Γ` when anything more is allowed.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let list = |f: &mut Formatter<'_>, ids: &[OccId]| {
            for (i, o) in ids.iter().enumerate() {
                write!(f, "{}{}", if i == 0 { " " } else { ", " }, o.get())?;
            }
            Ok(())
        };
        f.write_str("⊢")?;
        if !self.theta.is_empty() {
            list(f, &self.theta)?;
            f.write_str(" ;")?;
        }
        list(f, &self.gamma)?;
        if self.any {
            f.write_str(if self.gamma.is_empty() {
                " …"
            } else {
                ", …"
            })?;
        }
        Ok(())
    }
}

/// Why a proof term is not a proof of its sequent: the node at fault, what
/// the checker had derived for its premises, and what the rule required.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckError {
    /// The node at fault; the root when the proof concludes the wrong sequent
    /// or the mode cannot be checked.
    pub node: NodeId,
    /// The node's rule instance.
    pub rule: Node,
    /// The sequents derived for the node's premises, in the node's order.
    pub premises: Vec<Dyadic>,
    /// What the rule required and did not get.
    pub problem: Problem,
}

/// What a rule required and did not get.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// The mode forbids the rule: weakening of a formula that is not a `?`
    /// outside affine mode, or Mix without Mix.
    Forbidden,
    /// The mode is intuitionistic, whose one-succedent condition the checker
    /// does not test yet; only classical proofs are checked.
    Intuitionistic,
    /// The occurrence is not of the kind the rule acts on.
    Kind(OccId),
    /// The two literals of an axiom are not an atom and its negation.
    NotDual,
    /// A premise, by its index in the node, lacks the occurrence the rule
    /// consumes from it.
    Missing {
        /// Which premise.
        premise: usize,
        /// What it lacks.
        occurrence: OccId,
    },
    /// The premise of a promotion has a linear zone besides the promoted
    /// subformula.
    NotEmpty,
    /// The premises of `&` do not share their linear zone.
    Differ,
    /// A copy of an occurrence that is not the subformula of a `?`.
    NotUnderQuest(OccId),
    /// The root derives this sequent, which is not the proof's: its linear
    /// zone differs from the sequent's formulas, or its unrestricted zone
    /// holds a copied occurrence that no `?` rule below moved there.
    Conclusion(Dyadic),
}

impl Display for CheckError {
    /// Writes the node, its premises and the problem, as in `node 2 (⊗ on 1
    /// from 0, 1) with premises ⊢ 0, 3 and ⊢ 3, 4: premise 0 lacks
    /// occurrence 2`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "node {} ({})", self.node.get(), self.rule)?;
        for (i, p) in self.premises.iter().enumerate() {
            write!(
                f,
                "{}{p}",
                match i {
                    0 => " with premises ",
                    _ => " and ",
                }
            )?;
        }
        f.write_str(": ")?;
        use Problem::*;
        match &self.problem {
            Forbidden => write!(f, "the mode forbids the rule"),
            Intuitionistic => write!(f, "intuitionistic proofs cannot be checked yet"),
            Kind(o) => write!(f, "occurrence {} is not what the rule acts on", o.get()),
            NotDual => write!(f, "the literals are not an atom and its negation"),
            Missing {
                premise,
                occurrence,
            } => write!(f, "premise {premise} lacks occurrence {}", occurrence.get()),
            NotEmpty => write!(f, "the linear zone is not empty"),
            Differ => write!(f, "the premises differ"),
            NotUnderQuest(o) => write!(f, "occurrence {} is not under a ?", o.get()),
            Conclusion(d) => write!(f, "the proof concludes {d}, not the sequent"),
        }
    }
}

impl std::error::Error for CheckError {}

/// Checks that a proof proves its sequent: every node applies its rule to
/// what its premises derive, the mode allows the rule, and the root derives
/// exactly the sequent's formulas with nothing left in the unrestricted
/// zone. Returns the first node that fails, in arena order, with what it
/// needed. Intuitionistic mode is refused: its one-succedent condition is
/// not tested yet.
pub fn check(proof: &Proof, mode: Mode) -> Result<(), CheckError> {
    if mode.intuitionistic {
        return Err(CheckError {
            node: proof.root(),
            rule: proof.node(proof.root()),
            premises: vec![],
            problem: Problem::Intuitionistic,
        });
    }
    let derived = derive(proof, mode)?;
    conclude(proof, mode, &derived)
}

/// Derives what every node proves, in arena order, or reports the first
/// node that misapplies its rule or uses one the mode forbids.
pub(crate) fn derive(proof: &Proof, mode: Mode) -> Result<Vec<Derived>, CheckError> {
    let mut derived = Vec::with_capacity(proof.nodes().len());
    for id in proof.ids() {
        let d = Step::new(proof, mode, id, &derived).derive()?;
        derived.push(d);
    }
    Ok(derived)
}

/// Checks that the root derives the proof's sequent.
pub(crate) fn conclude(proof: &Proof, mode: Mode, derived: &[Derived]) -> Result<(), CheckError> {
    let root = proof.root();
    let d = &derived[root.index()];
    let roots = Multiset::of(proof.forest().roots().iter().copied());
    let concludes = if d.any {
        d.gamma.is_subset(&roots)
    } else {
        d.gamma == roots
    };
    if d.theta.is_empty() && concludes {
        Ok(())
    } else {
        Err(Step::new(proof, mode, root, derived).fail(Problem::Conclusion(d.to_dyadic())))
    }
}

/// One node being checked, with what its premises derived.
struct Step<'a> {
    /// The forest the occurrences index.
    forest: &'a Forest,
    /// The rules in force.
    mode: Mode,
    /// The node.
    id: NodeId,
    /// Its rule instance.
    node: Node,
    /// What the nodes before it derived.
    derived: &'a [Derived],
}

impl<'a> Step<'a> {
    /// The step for node `id`, whose premises have their entries in
    /// `derived`.
    fn new(proof: &'a Proof, mode: Mode, id: NodeId, derived: &'a [Derived]) -> Self {
        Self {
            forest: proof.forest(),
            mode,
            id,
            node: proof.node(id),
            derived,
        }
    }

    /// The error for this node.
    fn fail(&self, problem: Problem) -> CheckError {
        CheckError {
            node: self.id,
            rule: self.node,
            premises: self
                .node
                .premises()
                .map(|p| self.derived[p.index()].to_dyadic())
                .collect(),
            problem,
        }
    }

    /// What premise `p` derived.
    fn premise(&self, p: NodeId) -> Derived {
        self.derived[p.index()].clone()
    }

    /// Fails unless `o` has the kind the rule acts on.
    fn expect(&self, o: OccId, kind: Kind) -> Result<(), CheckError> {
        if self.forest.kind(o) == kind {
            Ok(())
        } else {
            Err(self.fail(Problem::Kind(o)))
        }
    }

    /// Consumes one copy of `o` from the linear zone of `d`, the derived
    /// sequent of premise `premise`; a `⊤` above stands in for a missing
    /// one.
    fn take(&self, d: &mut Derived, o: OccId, premise: usize) -> Result<(), CheckError> {
        if d.gamma.remove(o) || d.any {
            Ok(())
        } else {
            Err(self.fail(Problem::Missing {
                premise,
                occurrence: o,
            }))
        }
    }

    /// The left subformula of `o`, which must have one.
    fn left(&self, o: OccId) -> OccId {
        self.forest.left(o).unwrap()
    }

    /// The right subformula of `o`, which must have one.
    fn right(&self, o: OccId) -> OccId {
        self.forest.right(o).unwrap()
    }

    /// The sequent with only `o` in its linear zone.
    fn just(&self, o: OccId, any: bool) -> Derived {
        Derived {
            theta: self.forest.empty_set(),
            gamma: Multiset::of([o]),
            any,
        }
    }

    /// The sequent both premises of a two-premise rule give together: the
    /// unrestricted zones united, the linear zones summed, absorbing if
    /// either is.
    fn join(&self, l: Derived, r: Derived) -> Derived {
        Derived {
            theta: &l.theta | &r.theta,
            gamma: l.gamma.sum(&r.gamma),
            any: l.any || r.any,
        }
    }

    /// Derives what the node proves from what its premises derived.
    fn derive(self) -> Result<Derived, CheckError> {
        use Node::*;
        let f = self.forest;
        match self.node {
            Ax(a, b) => {
                for o in [a, b] {
                    if !f.is_literal(o) {
                        return Err(self.fail(Problem::Kind(o)));
                    }
                }
                if f.atom(a) != f.atom(b) || f.sign(a) == f.sign(b) {
                    return Err(self.fail(Problem::NotDual));
                }
                Ok(Derived {
                    theta: f.empty_set(),
                    gamma: Multiset::of([a, b]),
                    any: false,
                })
            }
            One(o) => {
                self.expect(o, Kind::One)?;
                Ok(self.just(o, false))
            }
            Top(o) => {
                self.expect(o, Kind::Top)?;
                Ok(self.just(o, true))
            }
            Bot(o, p) => {
                self.expect(o, Kind::Bot)?;
                let mut d = self.premise(p);
                d.gamma.insert(o);
                Ok(d)
            }
            Par(o, p) => {
                self.expect(o, Kind::Par)?;
                let mut d = self.premise(p);
                self.take(&mut d, self.left(o), 0)?;
                self.take(&mut d, self.right(o), 0)?;
                d.gamma.insert(o);
                Ok(d)
            }
            Tensor(o, l, r) => {
                self.expect(o, Kind::Tensor)?;
                let (mut dl, mut dr) = (self.premise(l), self.premise(r));
                self.take(&mut dl, self.left(o), 0)?;
                self.take(&mut dr, self.right(o), 1)?;
                let mut d = self.join(dl, dr);
                d.gamma.insert(o);
                Ok(d)
            }
            With(o, l, r) => {
                self.expect(o, Kind::With)?;
                let (mut dl, mut dr) = (self.premise(l), self.premise(r));
                self.take(&mut dl, self.left(o), 0)?;
                self.take(&mut dr, self.right(o), 1)?;
                // The conclusion's context is what both premises can prove
                // it under: an absorbing premise adapts to the other one.
                let (gamma, any) = match (dl.any, dr.any) {
                    (false, false) if dl.gamma == dr.gamma => (dl.gamma, false),
                    (true, false) if dl.gamma.is_subset(&dr.gamma) => (dr.gamma, false),
                    (false, true) if dr.gamma.is_subset(&dl.gamma) => (dl.gamma, false),
                    (true, true) => (dl.gamma.union(&dr.gamma), true),
                    _ => return Err(self.fail(Problem::Differ)),
                };
                let mut gamma = gamma;
                gamma.insert(o);
                Ok(Derived {
                    theta: &dl.theta | &dr.theta,
                    gamma,
                    any,
                })
            }
            Plus(o, side, p) => {
                self.expect(o, Kind::Plus)?;
                let mut d = self.premise(p);
                let chosen = match side {
                    Side::Left => self.left(o),
                    Side::Right => self.right(o),
                };
                self.take(&mut d, chosen, 0)?;
                d.gamma.insert(o);
                Ok(d)
            }
            Bang(o, p) => {
                self.expect(o, Kind::Bang)?;
                let mut d = self.premise(p);
                self.take(&mut d, self.left(o), 0)?;
                if !d.gamma.is_empty() {
                    return Err(self.fail(Problem::NotEmpty));
                }
                // Promotion fixes the linear zone: a ⊤ above cannot absorb
                // past it.
                Ok(Derived {
                    theta: d.theta,
                    gamma: Multiset::of([o]),
                    any: false,
                })
            }
            Quest(o, p) => {
                self.expect(o, Kind::Quest)?;
                let mut d = self.premise(p);
                d.theta.remove(self.left(o));
                d.gamma.insert(o);
                Ok(d)
            }
            Copy(a, p) => {
                if f.parent(a).map(|q| f.kind(q)) != Some(Kind::Quest) {
                    return Err(self.fail(Problem::NotUnderQuest(a)));
                }
                let mut d = self.premise(p);
                self.take(&mut d, a, 0)?;
                d.theta.insert(a);
                Ok(d)
            }
            Weaken(o, p) => {
                // Weakening a `?` formula is a rule of every mode.
                if !self.mode.affine && f.kind(o) != Kind::Quest {
                    return Err(self.fail(Problem::Forbidden));
                }
                let mut d = self.premise(p);
                d.gamma.insert(o);
                Ok(d)
            }
            Mix(l, r) => {
                if !self.mode.mix {
                    return Err(self.fail(Problem::Forbidden));
                }
                Ok(self.join(self.premise(l), self.premise(r)))
            }
        }
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::{Error, Sequent};

    /// Wraps a raw occurrence id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Wraps a raw node id.
    const fn n(id: u32) -> NodeId {
        NodeId::new(id)
    }

    /// Builds a proof of the sequent `input` parses to, with the last node
    /// as the root. Occurrence ids are the preorder numbering, which each
    /// test lists in a comment.
    fn proof(input: &str, nodes: Vec<Node>) -> Proof {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let root = n(nodes.len() as u32 - 1);
        Proof::new(Forest::new(&s).unwrap(), nodes, root).unwrap()
    }

    /// Every rule applied correctly is accepted: the multiplicatives and
    /// their units, the additives with `⊤` absorbing its context, the
    /// exponentials in dyadic form, weakening in affine mode and Mix.
    #[test]
    fn accepts_every_rule() {
        use Node::*;
        let classical = Mode::CLASSICAL;
        for (input, nodes, mode) in [
            // ⊢ ~A, A ⊗ ~B, B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))],
                classical,
            ),
            // ⊢ A ⅋ ~A: 0 ⅋, 1 A, 2 ~A
            (
                "|- A par ~A",
                vec![Ax(o(2), o(1)), Par(o(0), n(0))],
                classical,
            ),
            // ⊢ 1, ⊥
            ("|- 1, bot", vec![One(o(0)), Bot(o(1), n(0))], classical),
            // ⊢ A & B, ~A ⊕ ~B: 0 &, 1 A, 2 B, 3 ⊕, 4 ~A, 5 ~B
            (
                "|- A & B, ~A + ~B",
                vec![
                    Ax(o(1), o(4)),
                    Plus(o(3), Side::Left, n(0)),
                    Ax(o(2), o(5)),
                    Plus(o(3), Side::Right, n(2)),
                    With(o(0), n(1), n(3)),
                ],
                classical,
            ),
            // ⊢ ⊤, A: ⊤ absorbs A.
            ("|- top, A", vec![Top(o(0))], classical),
            // ⊢ A & ⊤, ~A: 0 &, 1 A, 2 ⊤, 3 ~A: the ⊤ side adapts to the
            // other.
            (
                "|- A & top, ~A",
                vec![Ax(o(1), o(3)), Top(o(2)), With(o(0), n(0), n(1))],
                classical,
            ),
            // ⊢ ⊤ ⊗ A, ~A, B: 0 ⊗, 1 ⊤, 2 A, 3 ~A, 4 B: the ⊤ side of the
            // split takes B.
            (
                "|- top * A, ~A, B",
                vec![Top(o(1)), Ax(o(2), o(3)), Tensor(o(0), n(0), n(1))],
                classical,
            ),
            // ⊢ ?~A, A: 0 ?, 1 ~A, 2 A: a copy, then the ? step.
            (
                "!A |- A",
                vec![Ax(o(1), o(2)), Copy(o(1), n(0)), Quest(o(0), n(1))],
                classical,
            ),
            // ⊢ ?~A, A ⊗ A: 0 ?, 1 ~A, 2 ⊗, 3 A, 4 A: two copies of one
            // occurrence.
            (
                "!A |- A * A",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    Ax(o(1), o(4)),
                    Copy(o(1), n(2)),
                    Tensor(o(2), n(1), n(3)),
                    Quest(o(0), n(4)),
                ],
                classical,
            ),
            // ⊢ ?~A, 1: 0 ?, 1 ~A, 2 1: an unused ? formula.
            ("!A |- 1", vec![One(o(2)), Quest(o(0), n(0))], classical),
            // ⊢ ?~A, ?(A ⊗ ~B), !B: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 !, 7 B:
            // promotion under two ? formulas.
            (
                "!A, !(A -o B) |- !B",
                vec![
                    Ax(o(1), o(4)),
                    Copy(o(1), n(0)),
                    Ax(o(5), o(7)),
                    Tensor(o(3), n(1), n(2)),
                    Copy(o(3), n(3)),
                    Bang(o(6), n(4)),
                    Quest(o(2), n(5)),
                    Quest(o(0), n(6)),
                ],
                classical,
            ),
            // ⊢ !(A ⅋ ~A): 0 !, 1 ⅋, 2 A, 3 ~A: promotion of a closed
            // formula.
            (
                "|- !(A par ~A)",
                vec![Ax(o(2), o(3)), Par(o(1), n(0)), Bang(o(0), n(1))],
                classical,
            ),
            // ⊢ ~A, ~B, A: weakening of ~B.
            (
                "A, B |- A",
                vec![Ax(o(0), o(2)), Weaken(o(1), n(0))],
                classical.affine(),
            ),
            // ⊢ 1, ?A: weakening of a ? formula needs no affine mode.
            ("|- 1, ?A", vec![One(o(0)), Weaken(o(1), n(0))], classical),
            // ⊢ ?⊤, !0: 0 ?, 1 ⊤, 2 !, 3 0: the ⊤ stands in for the 0 the
            // promotion consumes, and the zone is empty after it.
            (
                "|- ?top, !0",
                vec![
                    Top(o(1)),
                    Copy(o(1), n(0)),
                    Bang(o(2), n(1)),
                    Quest(o(0), n(2)),
                ],
                classical,
            ),
            // ⊢ ⊤ & ⊤, A: both sides absorb.
            (
                "|- top & top, A",
                vec![Top(o(1)), Top(o(2)), With(o(0), n(0), n(1))],
                classical,
            ),
            // ⊢ ⊤, ?A: 0 ⊤, 1 ?, 2 A: a copy the ⊤ stands in for.
            (
                "|- top, ?A",
                vec![Top(o(0)), Copy(o(2), n(0)), Quest(o(1), n(1))],
                classical,
            ),
            // ⊢ ?(?~A), A ⊗ A: 0 ?, 1 ?, 2 ~A, 3 ⊗, 4 A, 5 A: the inner ?
            // is copied twice and its ? step taken twice on one path.
            (
                "|- ?(?~A), A * A",
                vec![
                    Ax(o(2), o(4)),
                    Copy(o(2), n(0)),
                    Ax(o(2), o(5)),
                    Copy(o(2), n(2)),
                    Tensor(o(3), n(1), n(3)),
                    Quest(o(1), n(4)),
                    Copy(o(1), n(5)),
                    Quest(o(1), n(6)),
                    Copy(o(1), n(7)),
                    Quest(o(0), n(8)),
                ],
                classical,
            ),
            // ⊢ ?A, ?~A: 0 ?, 1 A, 2 ?, 3 ~A: one subproof shared by both
            // sides of a Mix.
            (
                "|- ?A, ?~A",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    Copy(o(3), n(1)),
                    Mix(n(2), n(2)),
                    Quest(o(2), n(3)),
                    Quest(o(0), n(4)),
                ],
                classical.with_mix(),
            ),
            // ⊢ ~A ⅋ ~B, A ⅋ B: 0 ⅋, 1 ~A, 2 ~B, 3 ⅋, 4 A, 5 B: needs Mix.
            (
                "A * B |- A par B",
                vec![
                    Ax(o(1), o(4)),
                    Ax(o(2), o(5)),
                    Mix(n(0), n(1)),
                    Par(o(0), n(2)),
                    Par(o(3), n(3)),
                ],
                classical.with_mix(),
            ),
        ] {
            let p = proof(input, nodes);
            assert_eq!(p.check(mode), Ok(()), "{input:?}");
        }
    }

    /// Every rule misapplied is rejected at the node that misapplies it,
    /// with the problem named; a rule the mode forbids is rejected too, and
    /// so is a root that concludes something else.
    #[test]
    fn rejects_every_misuse() {
        use Node::*;
        use Problem::*;
        let classical = Mode::CLASSICAL;
        for (input, nodes, mode, node, problem) in [
            // Axiom on two copies of one literal.
            ("|- A, A", vec![Ax(o(0), o(1))], classical, 0, NotDual),
            // Axiom on a connective: 0 ⊗, 1 A, 2 B, 3 ~A.
            (
                "|- A * B, ~A",
                vec![Ax(o(0), o(3))],
                classical,
                0,
                Kind(o(0)),
            ),
            // ⊗ with its premises swapped: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B.
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(1), n(0))],
                classical,
                2,
                Missing {
                    premise: 0,
                    occurrence: o(2),
                },
            ),
            // ⅋ on a ⊗ occurrence.
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Par(o(1), n(0))],
                classical,
                1,
                Kind(o(1)),
            ),
            // ⅋ whose premise lacks a subformula: 0 A, 1 ⅋, 2 ~A, 3 B.
            (
                "|- A, ~A par B",
                vec![Ax(o(0), o(2)), Par(o(1), n(0))],
                classical,
                1,
                Missing {
                    premise: 0,
                    occurrence: o(3),
                },
            ),
            // 1 on ⊥.
            ("|- bot", vec![One(o(0))], classical, 0, Kind(o(0))),
            // ⊥ on 1.
            (
                "|- 1, bot",
                vec![One(o(0)), Bot(o(0), n(0))],
                classical,
                1,
                Kind(o(0)),
            ),
            // & whose premises consume different contexts: 0 &, 1 A, 2 B,
            // 3 ~A, 4 ~B.
            (
                "|- A & B, ~A, ~B",
                vec![Ax(o(1), o(3)), Ax(o(2), o(4)), With(o(0), n(0), n(1))],
                classical,
                2,
                Differ,
            ),
            // & whose ⊤ side needs more than the other side has: the root
            // then concludes ⊢ A & ⊤, ~A without ~B.
            (
                "|- A & top, ~A, ~B",
                vec![Ax(o(1), o(3)), Top(o(2)), With(o(0), n(0), n(1))],
                classical,
                2,
                Conclusion(Dyadic {
                    theta: vec![],
                    gamma: vec![o(0), o(3)],
                    any: false,
                }),
            ),
            // ⊕ on the side the premise does not prove: 0 ⊕, 1 A, 2 B, 3 ~A.
            (
                "|- A + B, ~A",
                vec![Ax(o(1), o(3)), Plus(o(0), Side::Right, n(0))],
                classical,
                1,
                Missing {
                    premise: 0,
                    occurrence: o(2),
                },
            ),
            // ⊤ on 0.
            ("|- 0", vec![Top(o(0))], classical, 0, Kind(o(0))),
            // Promotion with a linear formula in the context: 0 ~A, 1 !, 2 A.
            (
                "A |- !A",
                vec![Ax(o(0), o(2)), Bang(o(1), n(0))],
                classical,
                1,
                NotEmpty,
            ),
            // A copy of a formula under !: 0 !, 1 ~A, 2 A.
            (
                "?A |- A",
                vec![Ax(o(1), o(2)), Copy(o(1), n(0))],
                classical,
                1,
                NotUnderQuest(o(1)),
            ),
            // A copy of a formula the premise does not hold: 0 ?, 1 ~A, 2 1.
            (
                "!A |- 1",
                vec![One(o(2)), Copy(o(1), n(0))],
                classical,
                1,
                Missing {
                    premise: 0,
                    occurrence: o(1),
                },
            ),
            // A copy without the ? step below it: the root still needs ~A
            // in the unrestricted zone.
            (
                "!A |- A",
                vec![Ax(o(1), o(2)), Copy(o(1), n(0))],
                classical,
                1,
                Conclusion(Dyadic {
                    theta: vec![o(1)],
                    gamma: vec![o(2)],
                    any: false,
                }),
            ),
            // ? on a ! occurrence.
            (
                "|- !(A par ~A)",
                vec![Ax(o(2), o(3)), Par(o(1), n(0)), Quest(o(0), n(1))],
                classical,
                2,
                Kind(o(0)),
            ),
            // Weakening outside affine mode.
            (
                "A, B |- A",
                vec![Ax(o(0), o(2)), Weaken(o(1), n(0))],
                classical,
                1,
                Forbidden,
            ),
            // Mix without Mix.
            (
                "|- A, ~A, B, ~B",
                vec![Ax(o(0), o(1)), Ax(o(2), o(3)), Mix(n(0), n(1))],
                classical,
                2,
                Forbidden,
            ),
            // A proof of a smaller sequent.
            (
                "|- A, ~A, 1",
                vec![Ax(o(0), o(1))],
                classical,
                0,
                Conclusion(Dyadic {
                    theta: vec![],
                    gamma: vec![o(0), o(1)],
                    any: false,
                }),
            ),
            // Intuitionistic mode is not checked yet, whatever the root.
            (
                "|- A par ~A",
                vec![Ax(o(2), o(1)), Par(o(0), n(0))],
                Mode::INTUITIONISTIC,
                1,
                Intuitionistic,
            ),
        ] {
            let p = proof(input, nodes);
            let e = p.check(mode).unwrap_err();
            let message = e.to_string();
            assert_eq!(
                (e.node, e.problem),
                (n(node), problem),
                "{input:?}: {message}"
            );
        }
    }

    /// The error names the node, its premises and the problem.
    #[test]
    fn error_message() {
        use Node::*;
        // ⊢ ~A, A ⊗ ~B, B with the ⊗ premises swapped.
        let p = proof(
            "A, A -o B |- B",
            vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(1), n(0))],
        );
        assert_eq!(
            p.check(Mode::CLASSICAL).unwrap_err().to_string(),
            "node 2 (⊗ on 1 from 1, 0) with premises ⊢ 3, 4 and ⊢ 0, 2: \
             premise 0 lacks occurrence 2"
        );
        // ⊢ ?~A, A with the ? step missing: a dyadic sequent with Θ.
        let p = proof("!A |- A", vec![Ax(o(1), o(2)), Copy(o(1), n(0))]);
        assert_eq!(
            p.check(Mode::CLASSICAL).unwrap_err().to_string(),
            "node 1 (copy on 1 from 0) with premises ⊢ 1, 2: \
             the proof concludes ⊢ 1 ; 2, not the sequent"
        );
        assert_eq!(
            Dyadic {
                theta: vec![],
                gamma: vec![o(0)],
                any: true
            }
            .to_string(),
            "⊢ 0, …"
        );
    }

    /// A proof keeps only the nodes its root reaches, renumbered in order,
    /// and refuses a node that points outside the arena or the forest, or
    /// at a later node.
    #[test]
    fn construction() {
        use Node::*;
        let s: Sequent = "A, A -o B |- B".parse().unwrap();
        let f = Forest::new(&s).unwrap();
        // Node 1 is unused.
        let p = Proof::new(
            f.clone(),
            vec![
                Ax(o(0), o(2)),
                One(o(4)),
                Ax(o(3), o(4)),
                Tensor(o(1), n(0), n(2)),
            ],
            n(3),
        )
        .unwrap();
        assert_eq!(
            p.nodes(),
            [Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))]
        );
        assert_eq!(p.root(), n(2));
        assert_eq!(p.check(Mode::CLASSICAL), Ok(()));
        // The root is beyond the arena.
        assert!(matches!(
            Proof::new(f.clone(), vec![Ax(o(0), o(2))], n(1)),
            Err(Error::NodeIndexOutOfBounds(1, 1))
        ));
        // A premise does not precede its conclusion.
        assert!(matches!(
            Proof::new(f.clone(), vec![Par(o(1), n(0))], n(0)),
            Err(Error::PremiseIndexNotDecreasing(0, 0))
        ));
        // An occurrence beyond the forest.
        assert!(matches!(
            Proof::new(f, vec![Ax(o(0), o(5))], n(0)),
            Err(Error::OccurrenceIndexOutOfBounds(5, 5))
        ));
    }
}
