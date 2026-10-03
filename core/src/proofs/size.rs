// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The size of a derivation, computed from the proof term without building
//! the derivation: one pass of the checker, with a handful of numbers kept
//! per node. A term stores a subproof that several nodes share once and
//! the derivation unfolds it at every use, so a derivation can be
//! exponentially larger than its term; the numbers here are what says so
//! before anything is built. All of them saturate.

use super::check::{self, CheckError, Facts, Observer, State};
use super::{Derivation, Node, NodeId, Proof, Side};
use crate::fragment::Mode;
use crate::occurrences::{Forest, OccId, Position, Reading};
use crate::sequents::Term;

/// How large the derivation of a proof is, as
/// [`Proof::derivation_size`] computes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Size {
    /// The inferences of the derivation. Exact.
    pub inferences: u64,
    /// The characters of every inference's sequent added up, each sequent
    /// written one-sided with its formulas separated by a comma and a
    /// space: what a renderer writes, up to its rule names, its layout and
    /// its own spelling of the connectives. Exact when `exact` is set, and
    /// never less than the sum otherwise.
    pub characters: u64,
    /// The inferences on the longest branch. Exact. The text tree has two
    /// lines for each.
    pub height: u64,
    /// The characters of the widest sequent the pass saw, which the text
    /// tree is at least as wide as: a lower bound of its width.
    pub width: u64,
    /// Whether `characters` is the exact sum. It is an upper bound when a
    /// `&` has several `?` formulas that only one of its premises uses:
    /// the weakenings above the other premise are then each counted with
    /// the sequent of the last.
    pub exact: bool,
}

impl Size {
    /// What a derivation takes in memory and in most of its written forms
    /// for every character of its sequents: the occurrence ids of the
    /// sequents, the rule lines of the text tree, the markup of the
    /// exports.
    pub const BYTES_PER_CHARACTER: u64 = 8;

    /// What a derivation takes for every inference beside its sequent.
    pub const BYTES_PER_INFERENCE: u64 = 128;

    /// Returns an estimate, good to the order of magnitude, of the bytes
    /// the derivation takes to build and to write in any format:
    /// [`BYTES_PER_CHARACTER`](Self::BYTES_PER_CHARACTER) for each
    /// character of a sequent and
    /// [`BYTES_PER_INFERENCE`](Self::BYTES_PER_INFERENCE) for each
    /// inference. It is what [`ViewOptions::limit`](super::ViewOptions)
    /// bounds.
    pub fn bytes(&self) -> u64 {
        self.characters
            .saturating_mul(Self::BYTES_PER_CHARACTER)
            .saturating_add(self.inferences.saturating_mul(Self::BYTES_PER_INFERENCE))
    }

    /// Returns the lines of the text tree: two for every inference of the
    /// longest branch.
    pub fn lines(&self) -> u64 {
        self.height.saturating_mul(2)
    }
}

impl Proof {
    /// Returns how large the derivation of the proof is, the two-sided one
    /// with `two_sided` set, without building it, or the checker's
    /// complaint about the proof: the proof passes the checker once, and
    /// a subproof that several nodes share is counted at every use, as the
    /// derivation repeats it.
    pub fn derivation_size(&self, two_sided: bool) -> Result<Size, CheckError> {
        let mode = if two_sided {
            Derivation::TWO_SIDED
        } else {
            Derivation::ONE_SIDED
        };
        let reading = check::reading(self, mode)?;
        measure(self, self.forest().roots(), mode, reading.as_ref())
    }
}

/// What the pass keeps for a node: the size of the subtree it unfolds
/// into, under the sequent it derives itself, and how that grows with
/// what a `⊤` in it absorbs.
#[derive(Clone, Copy, Debug, Default)]
struct Sub {
    /// The inferences.
    inferences: u64,
    /// The characters of its sequents, with nothing absorbed.
    characters: u64,
    /// The inferences whose sequent holds a hypothesis the subtree absorbs
    /// (any formula, one-sided).
    reached: u64,
    /// The inferences whose sequent holds the goal the subtree absorbs.
    reached_goal: u64,
    /// The inferences on its longest branch.
    height: u64,
    /// The characters of its widest sequent, with nothing absorbed.
    width: u64,
    /// The characters of the sequent it derives.
    weight: u64,
    /// Those of the formula in output position among them.
    goal: u64,
    /// Whether a `⊤` in it absorbs any context.
    absorbs: bool,
}

/// The observer that adds up the sizes.
struct Measure<'a> {
    /// The proof.
    proof: &'a Proof,
    /// The reading, for a two-sided derivation.
    reading: Option<&'a Reading<'a>>,
    /// The characters of every occurrence's formula, and two for its
    /// separator.
    weights: Vec<u32>,
    /// What the pass found for each node so far.
    subs: Vec<Sub>,
    /// Whether every sum is exact so far.
    exact: bool,
}

/// Returns, for every occurrence of a forest, the characters of its
/// formula in one-sided notation plus two for the separator after it.
fn weights(forest: &Forest) -> Vec<u32> {
    let sequent = forest.sequent();
    let names: Vec<u32> = sequent
        .atom_names()
        .iter()
        .map(|name| name.chars().count() as u32)
        .collect();
    // Per term, the characters at the top of a formula and inside one,
    // where a binary formula is bracketed; a subterm precedes its parents.
    let mut top = Vec::<u32>::with_capacity(sequent.terms().len());
    let mut inner = Vec::<u32>::with_capacity(sequent.terms().len());
    for term in sequent.terms() {
        use Term::*;
        let (width, binary) = match *term {
            Var(a) => (names[a.index()], false),
            DualVar(a) => (names[a.index()].saturating_add(1), false),
            One | Bot | Top | Zero => (1, false),
            Tensor(k, l) | Par(k, l) | With(k, l) | Plus(k, l) => (
                inner[k.index()]
                    .saturating_add(3)
                    .saturating_add(inner[l.index()]),
                true,
            ),
            Bang(k) | Quest(k) => (inner[k.index()].saturating_add(1), false),
        };
        top.push(width);
        inner.push(if binary {
            width.saturating_add(2)
        } else {
            width
        });
    }
    forest
        .ids()
        .map(|o| top[forest.term(o).index()].saturating_add(2))
        .collect()
}

impl Measure<'_> {
    /// The characters of an occurrence's formula and its separator.
    fn characters(&self, o: OccId) -> u64 {
        u64::from(self.weights[o.index()])
    }

    /// How many inferences of `sub` hold an absorbed copy of `o`: those
    /// its goal reaches when `o` is in output position, those a hypothesis
    /// reaches otherwise.
    fn reach(&self, sub: &Sub, o: OccId) -> u64 {
        match self.reading {
            Some(reading) if reading.position(o) == Position::Output => sub.reached_goal,
            _ => sub.reached,
        }
    }

    /// The characters a premise's subtree gains from a subformula `o` its
    /// sequent lacked, which a `⊤` in it absorbs.
    fn absorbed(&self, sub: &Sub, o: OccId, absent: bool) -> u64 {
        if absent {
            self.characters(o).saturating_mul(self.reach(sub, o))
        } else {
            0
        }
    }

    /// The size of a node with one premise that unfolds into `own`
    /// inferences below the premise's subtree, their sequents `extra`
    /// characters over `own` times the node's own, and whose premise
    /// absorbs `absorbed` characters more.
    fn unary(sub: &Sub, premise: &Sub, own: u64, extra: u64, absorbed: u64) -> Sub {
        Sub {
            inferences: premise.inferences.saturating_add(own),
            characters: premise
                .characters
                .saturating_add(own.saturating_mul(sub.weight))
                .saturating_add(extra)
                .saturating_add(absorbed),
            reached: premise.reached.saturating_add(own),
            reached_goal: premise.reached_goal.saturating_add(own),
            height: premise.height.saturating_add(own),
            width: premise.width.max(sub.weight.saturating_add(extra)),
            ..*sub
        }
    }

    /// What a premise of a `&` adds to it: its own subtree under the
    /// conclusion's context, which holds `beyond` characters more than the
    /// premise derives, `beyond_goal` of them the goal's, and `unused`
    /// formulas `?A` that only the other premise uses, weakened above this
    /// one unless a `⊤` absorbs them. Returns the inferences, the
    /// characters and the height.
    fn padded(
        &mut self,
        premise: &Sub,
        beyond: u64,
        beyond_goal: u64,
        unused: u64,
    ) -> (u64, u64, u64) {
        if premise.absorbs {
            let hypotheses = beyond.saturating_sub(beyond_goal);
            let characters = premise
                .characters
                .saturating_add(hypotheses.saturating_mul(premise.reached))
                .saturating_add(beyond_goal.saturating_mul(premise.reached_goal));
            return (premise.inferences, characters, premise.height);
        }
        // Every weakening is counted with the sequent of the last, which
        // is the sum itself when there is one at most.
        self.exact &= unused <= 1;
        let weakened = premise.weight.saturating_add(beyond);
        (
            premise.inferences.saturating_add(unused),
            premise
                .characters
                .saturating_add(unused.saturating_mul(weakened)),
            premise.height.saturating_add(unused),
        )
    }
}

impl Observer for Measure<'_> {
    fn weight(&self, o: OccId) -> u64 {
        self.characters(o)
    }

    fn derived(&mut self, id: NodeId, state: &State, facts: &Facts<'_>) {
        use Node::*;
        let forest = self.proof.forest();
        let (left, right) = (
            |o: OccId| forest.left(o).unwrap(),
            |o: OccId| forest.right(o).unwrap(),
        );
        let at = |p: NodeId| self.subs[p.index()];
        let sub = Sub {
            weight: state.weight(),
            goal: state.goal_weight(),
            absorbs: state.absorbs(),
            ..Sub::default()
        };
        let sub = match self.proof.node(id) {
            Ax(..) | One(_) | Top(_) => Sub {
                inferences: 1,
                characters: sub.weight,
                reached: 1,
                reached_goal: 1,
                height: 1,
                width: sub.weight,
                ..sub
            },
            Bot(_, p) | Weaken(_, p) => Self::unary(&sub, &at(p), 1, 0, 0),
            Par(o, p) => {
                let premise = at(p);
                let absorbed = self
                    .absorbed(&premise, left(o), facts.absent[0])
                    .saturating_add(self.absorbed(&premise, right(o), facts.absent[1]));
                Self::unary(&sub, &premise, 1, 0, absorbed)
            }
            Plus(o, side, p) => {
                let chosen = match side {
                    Side::Left => left(o),
                    Side::Right => right(o),
                };
                let premise = at(p);
                let absorbed = self.absorbed(&premise, chosen, facts.absent[0]);
                Self::unary(&sub, &premise, 1, 0, absorbed)
            }
            Bang(o, p) => {
                let premise = at(p);
                let absorbed = self.absorbed(&premise, left(o), facts.absent[0]);
                Self::unary(&sub, &premise, 1, 0, absorbed)
            }
            // A `?` step whose formula is used above is no inference: the
            // sequent is the premise's.
            Quest(_, p) if facts.used => Sub { ..at(p) },
            Quest(_, p) => Self::unary(&sub, &at(p), 1, 0, 0),
            Copy(a, p) => {
                let premise = at(p);
                let absorbed = self.absorbed(&premise, a, facts.absent[0]);
                if facts.used {
                    // A dereliction, whose sequent holds the `?` formula
                    // twice, and the contraction below it.
                    let q = self.characters(forest.parent(a).unwrap());
                    Self::unary(&sub, &premise, 2, q, absorbed)
                } else {
                    Self::unary(&sub, &premise, 1, 0, absorbed)
                }
            }
            node @ (Tensor(_, l, r) | Mix(l, r)) => {
                let (pl, pr) = (at(l), at(r));
                // The rule's sequent holds every shared `?` formula twice,
                // and one contraction below it per formula takes the
                // second away, in ascending order.
                let mut shared: Vec<OccId> = facts.shared.to_vec();
                shared.sort_unstable();
                let own = shared.len() as u64 + 1;
                let mut doubled: u64 = 0;
                for (j, &a) in shared.iter().enumerate() {
                    let q = self.characters(forest.parent(a).unwrap());
                    doubled = doubled.saturating_add(q.saturating_mul(j as u64 + 1));
                }
                let widest: u64 = shared
                    .iter()
                    .map(|&a| self.characters(forest.parent(a).unwrap()))
                    .fold(sub.weight, u64::saturating_add);
                let absorbed = match node {
                    Tensor(o, ..) => self
                        .absorbed(&pl, left(o), facts.absent[0])
                        .saturating_add(self.absorbed(&pr, right(o), facts.absent[1])),
                    _ => 0,
                };
                // What the rule's conclusion absorbs goes to the left
                // premise if it absorbs, else to the right one; two-sided,
                // with both absorbing, the goal goes to the premise that
                // has none.
                let (reached, reached_goal) = if self.reading.is_some() && pl.absorbs && pr.absorbs
                {
                    let goal = if facts.left_goal { &pr } else { &pl };
                    (pl.reached, goal.reached_goal)
                } else if pl.absorbs {
                    (pl.reached, pl.reached_goal)
                } else {
                    (pr.reached, pr.reached_goal)
                };
                Sub {
                    inferences: pl
                        .inferences
                        .saturating_add(pr.inferences)
                        .saturating_add(own),
                    characters: pl
                        .characters
                        .saturating_add(pr.characters)
                        .saturating_add(own.saturating_mul(sub.weight))
                        .saturating_add(doubled)
                        .saturating_add(absorbed),
                    reached: reached.saturating_add(own),
                    reached_goal: reached_goal.saturating_add(own),
                    height: pl.height.max(pr.height).saturating_add(own),
                    width: pl.width.max(pr.width).max(widest),
                    ..sub
                }
            }
            With(o, l, r) => {
                let (pl, pr) = (at(l), at(r));
                let needs = state.needs() as u64;
                // What a premise's conclusion holds beyond what the
                // premise derives: the conclusion's context with the
                // premise's subformula in place of the `&`.
                let output = self
                    .reading
                    .is_some_and(|reading| reading.position(o) == Position::Output);
                let beyond = |premise: &Sub, child: OccId| {
                    let (whole, part) = (self.characters(o), self.characters(child));
                    let all = (sub.weight.saturating_add(part))
                        .saturating_sub(whole.saturating_add(premise.weight));
                    let goal = if output {
                        sub.goal.saturating_add(part).saturating_sub(whole)
                    } else {
                        sub.goal
                    };
                    (all, goal.saturating_sub(premise.goal).min(all))
                };
                let ((all_l, goal_l), (all_r, goal_r)) =
                    (beyond(&pl, left(o)), beyond(&pr, right(o)));
                let (il, cl, hl) = self.padded(
                    &pl,
                    all_l,
                    goal_l,
                    needs.saturating_sub(facts.needs[0] as u64),
                );
                let (ir, cr, hr) = self.padded(
                    &pr,
                    all_r,
                    goal_r,
                    needs.saturating_sub(facts.needs[1] as u64),
                );
                Sub {
                    inferences: il.saturating_add(ir).saturating_add(1),
                    characters: cl.saturating_add(cr).saturating_add(sub.weight),
                    reached: pl.reached.saturating_add(pr.reached).saturating_add(1),
                    reached_goal: pl
                        .reached_goal
                        .saturating_add(pr.reached_goal)
                        .saturating_add(1),
                    height: hl.max(hr).saturating_add(1),
                    width: pl.width.max(pr.width).max(sub.weight),
                    ..sub
                }
            }
        };
        self.subs[id.index()] = sub;
    }
}

/// Returns the size of the derivation that a proof concluding `goal`
/// unfolds into, two-sided under a reading, or the checker's complaint
/// about the proof in `mode`.
pub(crate) fn measure(
    proof: &Proof,
    goal: &[OccId],
    mode: Mode,
    reading: Option<&Reading>,
) -> Result<Size, CheckError> {
    let mut measure = Measure {
        proof,
        reading,
        weights: weights(proof.forest()),
        subs: vec![Sub::default(); proof.nodes().len()],
        exact: true,
    };
    check::examine(proof, goal, mode, reading, &mut measure)?;
    let root = measure.subs[proof.root().index()];
    // What the conclusion holds beyond what the root derives, a `⊤`
    // absorbs.
    let all: u64 = goal.iter().map(|&o| measure.characters(o)).sum();
    let goal: u64 = goal
        .iter()
        .filter(|&&o| reading.is_some_and(|r| r.position(o) == Position::Output))
        .map(|&o| measure.characters(o))
        .sum();
    let beyond = all.saturating_sub(root.weight);
    let beyond_goal = goal.saturating_sub(root.goal).min(beyond);
    Ok(Size {
        inferences: root.inferences,
        characters: root
            .characters
            .saturating_add((beyond - beyond_goal).saturating_mul(root.reached))
            .saturating_add(beyond_goal.saturating_mul(root.reached_goal)),
        height: root.height,
        width: root.width.max(all),
        exact: measure.exact,
    })
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use crate::proofs::{Derivation, ViewOptions, oracle};
    use crate::search::{Options, Verdict, prove};
    use crate::{Mode, Sequent};

    /// The inferences, the characters of the sequents, the height and the
    /// widest sequent of a derivation, counted on the derivation itself.
    fn counted(derivation: &Derivation) -> (u64, u64, u64, u64) {
        let forest = derivation.forest();
        let mut heights = vec![0u64; derivation.inferences().len()];
        let (mut characters, mut widest) = (0, 0);
        for (i, inference) in derivation.inferences().iter().enumerate() {
            let above = inference.premises.iter().map(|p| heights[p.index()]).max();
            heights[i] = above.unwrap_or(0) + 1;
            let width: u64 = inference
                .sequent
                .iter()
                .map(|&o| forest.formula(o).to_string().chars().count() as u64 + 2)
                .sum();
            characters += width;
            widest = widest.max(width);
        }
        (
            heights.len() as u64,
            characters,
            heights[derivation.root().index()],
            widest,
        )
    }

    /// The size computed from a proof is the size of the derivation built
    /// from it, one-sided and two-sided: the inferences and the height
    /// exactly, the characters exactly unless it says otherwise and never
    /// fewer, and a width between the conclusion's and the widest
    /// sequent's.
    #[test]
    fn is_the_size_of_the_derivation_built() {
        let never = || false;
        let mut bounds = 0;
        let mut proofs = oracle::proofs();
        // A `&` whose right premise alone uses two `?` formulas, weakened
        // above the left one: the case the sum bounds.
        let sequent: Sequent = "!A, !B |- (C -o C) & (A * B)".parse().unwrap();
        let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default()).unwrap();
        let Verdict::Proved(proof) = outcome.verdict else {
            panic!("provable");
        };
        proofs.push((*proof, Mode::CLASSICAL));
        for (proof, mode) in proofs {
            let mut views = vec![(
                false,
                Derivation::new(&proof, &ViewOptions::UNBOUNDED, never).unwrap(),
            )];
            if mode.intuitionistic {
                views.push((
                    true,
                    Derivation::two_sided(&proof, &ViewOptions::UNBOUNDED, never).unwrap(),
                ));
            }
            for (two_sided, derivation) in views {
                let size = proof.derivation_size(two_sided).unwrap();
                let (inferences, characters, height, widest) = counted(&derivation);
                let conclusion = counted_root(&derivation);
                let context = format!("{} ({mode}, two-sided: {two_sided})", proof.sequent());
                assert_eq!(size.inferences, inferences, "{context}");
                assert_eq!(size.height, height, "{context}");
                if size.exact {
                    assert_eq!(size.characters, characters, "{context}");
                } else {
                    bounds += 1;
                    assert!(size.characters >= characters, "{context}");
                }
                assert!(
                    conclusion <= size.width && size.width <= widest,
                    "{context}: {conclusion} <= {} <= {widest}",
                    size.width
                );
            }
        }
        assert!(bounds > 0, "no sample whose sum is a bound");
    }

    /// The characters of a derivation's conclusion.
    fn counted_root(derivation: &Derivation) -> u64 {
        let forest = derivation.forest();
        derivation
            .inference(derivation.root())
            .sequent
            .iter()
            .map(|&o| forest.formula(o).to_string().chars().count() as u64 + 2)
            .sum()
    }
}
