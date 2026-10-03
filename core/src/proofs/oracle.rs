// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The checker as it was first written, kept for the tests: it derives a
//! sequent for every node and keeps them all, each with a set as wide as
//! the forest, so it is quadratic in memory and easy to read. The checker
//! proper must accept and reject exactly what this one does, with the same
//! error.

use super::check::{CheckError, Dyadic, Problem};
use super::multiset::Multiset;
use super::{Node, NodeId, Proof, Side};
#[cfg(feature = "parse")]
use crate::Sequent;
use crate::fragment::Mode;
use crate::occurrences::{Forest, OccId, OccSet, Position, Reading};
use crate::sequents::Kind;

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

/// Checks that a proof proves its sequent: every node applies its rule to
/// what its premises derive, the mode allows the rule, and the root derives
/// exactly the sequent's formulas with nothing left in the unrestricted
/// zone; in intuitionistic mode also that the sequent has an intuitionistic
/// reading and every sequent of the proof one formula on the right of `⊢`.
/// Returns the first node that fails, in arena order, with what it needed.
pub(crate) fn check(proof: &Proof, mode: Mode) -> Result<(), CheckError> {
    let reading = if mode.intuitionistic {
        match Reading::new(proof.forest()) {
            Ok(reading) => Some(reading),
            Err(e) => {
                return Err(CheckError {
                    node: proof.root(),
                    rule: proof.node(proof.root()),
                    premises: vec![],
                    problem: Problem::Shape(e),
                });
            }
        }
    } else {
        None
    };
    let derived = derive(proof, mode, reading.as_ref())?;
    conclude(proof, mode, &derived)
}

/// Derives what every node proves, in arena order, or reports the first
/// node that misapplies its rule or uses one the mode forbids; with a
/// reading, also the first sequent that breaks the one-succedent condition.
pub(crate) fn derive(
    proof: &Proof,
    mode: Mode,
    reading: Option<&Reading>,
) -> Result<Vec<Derived>, CheckError> {
    let mut derived = Vec::with_capacity(proof.nodes().len());
    for id in proof.ids() {
        let d = Step::new(proof, mode, id, &derived, reading).derive()?;
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
        Err(Step::new(proof, mode, root, derived, None).fail(Problem::Conclusion(d.to_dyadic())))
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
    /// The intuitionistic reading, in intuitionistic mode.
    reading: Option<&'a Reading<'a>>,
}

impl<'a> Step<'a> {
    /// The step for node `id`, whose premises have their entries in
    /// `derived`.
    fn new(
        proof: &'a Proof,
        mode: Mode,
        id: NodeId,
        derived: &'a [Derived],
        reading: Option<&'a Reading<'a>>,
    ) -> Self {
        Self {
            forest: proof.forest(),
            mode,
            id,
            node: proof.node(id),
            derived,
            reading,
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
    /// one, but never for a second goal.
    fn take(&self, d: &mut Derived, o: OccId, premise: usize) -> Result<(), CheckError> {
        if d.gamma.remove(o) {
            return Ok(());
        }
        if !d.any {
            return Err(self.fail(Problem::Missing {
                premise,
                occurrence: o,
            }));
        }
        // The premise's sequent holds `o` besides its zone: one goal at
        // most.
        if let Some(reading) = self.reading
            && reading.position(o) == Position::Output
            && self.outputs(&d.gamma) > 0
        {
            return Err(self.fail(Problem::Succedents(2)));
        }
        Ok(())
    }

    /// Counts the formulas of a linear zone in output position, in
    /// intuitionistic mode.
    fn outputs(&self, gamma: &Multiset) -> usize {
        self.reading
            .map_or(0, |r| r.outputs(gamma.as_slice().iter().copied()))
    }

    /// Checks the one-succedent condition on what the node derived: one
    /// output at most, and exactly one unless a `⊤` above supplies it.
    fn one_succedent(&self, d: Derived) -> Result<Derived, CheckError> {
        if self.reading.is_none() {
            return Ok(d);
        }
        let outputs = self.outputs(&d.gamma);
        if outputs > 1 || (outputs == 0 && !d.any) {
            return Err(self.fail(Problem::Succedents(outputs)));
        }
        Ok(d)
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
        let d = self.rule()?;
        self.one_succedent(d)
    }

    /// Applies the node's rule to what its premises derived.
    fn rule(&self) -> Result<Derived, CheckError> {
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
                // Weakening a `?` formula is a rule of every mode; the goal
                // is never weakened.
                if !self.mode.affine && f.kind(o) != Kind::Quest {
                    return Err(self.fail(Problem::Forbidden));
                }
                if self
                    .reading
                    .is_some_and(|r| r.position(o) == Position::Output)
                {
                    return Err(self.fail(Problem::Succedents(0)));
                }
                let mut d = self.premise(p);
                d.gamma.insert(o);
                Ok(d)
            }
            Mix(l, r) => {
                // Mix has no intuitionistic form: a premise would lack the
                // goal.
                if !self.mode.mix || self.mode.intuitionistic {
                    return Err(self.fail(Problem::Forbidden));
                }
                Ok(self.join(self.premise(l), self.premise(r)))
            }
        }
    }
}

/// Returns proofs to test a pass over proofs on, each with the mode it was
/// found in: those the engines find of generated sequents, classical and
/// intuitionistic, linear and affine, and of the smallest instance of
/// every family.
#[cfg(feature = "parse")]
pub(crate) fn proofs() -> Vec<(Proof, Mode)> {
    use crate::search::generate::{self, IllRules, Rng, Rules};
    use crate::search::{Options, Verdict, prove_until};
    let classical = Mode::CLASSICAL;
    let mut cases: Vec<(String, Mode, Options)> = vec![];
    for (k, rules) in Rules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(900 + k as u64);
        for _ in 0..25 {
            let budget = 2 + rng.below(9);
            let provable = generate::provable(&mut rng, rules, 3, budget);
            let mode = if rules.mix {
                classical.with_mix()
            } else {
                classical
            };
            let options = Options::default().copies(provable.copies);
            let text = generate::sequent(&provable.formulas);
            cases.push((text.clone(), mode, options.clone()));
            cases.push((text, mode.affine(), options));
        }
    }
    for (k, rules) in IllRules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(950 + k as u64);
        for _ in 0..25 {
            let budget = 2 + rng.below(9);
            let ill = generate::ill(&mut rng, rules, 3, budget);
            let options = Options::default().copies(ill.copies);
            let text = generate::two_sided(&ill.hypotheses, &ill.goal);
            cases.push((text.clone(), Mode::INTUITIONISTIC, options.clone()));
            cases.push((text, Mode::INTUITIONISTIC.affine(), options));
        }
    }
    let mut proofs: Vec<(Proof, Mode)> = vec![];
    for (text, mode, options) in cases {
        let sequent: Sequent = text.parse().unwrap();
        let mut polls = 0;
        let outcome = prove_until(&sequent, mode, &options, || {
            polls += 1;
            polls > 20_000
        });
        if let Ok(Verdict::Proved(proof)) = outcome.map(|o| o.verdict) {
            proofs.push((*proof, mode));
        }
    }
    for family in crate::families::FAMILIES {
        let instance = family.instance(family.sizes[0], 0);
        let options = match instance.copies {
            Some(copies) => Options::default().copies(copies),
            None => Options::default(),
        };
        let mut polls = 0;
        let outcome = prove_until(&instance.sequent, instance.mode, &options, || {
            polls += 1;
            polls > 20_000
        });
        if let Ok(Verdict::Proved(proof)) = outcome.map(|o| o.verdict) {
            proofs.push((*proof, instance.mode));
        }
    }
    proofs
}
