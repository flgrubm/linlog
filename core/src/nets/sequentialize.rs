// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Sequentialization: from a proof net to a proof term, by the splitting
//! tensor lemma. A sub-net is given by its conclusions. While one is a `⅋`,
//! it is replaced by its premises and a `⅋` rule is emitted below whatever
//! proves the rest. With Mix allowed, a sub-net that falls into several
//! parts, as the plain graph of its formula trees and links, is proved
//! part by part and joined with Mix. Two literals alone are an axiom.
//! Otherwise some `⊗` conclusion is splitting: the net without it is two
//! nets, one holding each premise, which is the case exactly when its edge
//! to a premise is a bridge of the plain graph. A depth-first search per
//! stage finds the parts and the bridges at once, so the whole
//! sequentialization costs the square of the net's size at most.

use super::{NetError, ProofStructure, Scratch};
use crate::fragment::Mode;
use crate::occurrences::OccId;
use crate::proofs::{Node, NodeId, Proof};
use crate::sequents::Kind;

impl ProofStructure {
    /// Turns a proof net into a proof term of its sequent, which the proof
    /// checker accepts: `⅋`, `⊗`, axiom and, where the structure falls
    /// apart, Mix nodes. Fails as [`is_correct`](Self::is_correct) does if
    /// the structure is not a proof net.
    pub fn sequentialize(&self) -> Result<Proof, NetError> {
        self.is_correct()?;
        let mut run = Sequentialization {
            net: self,
            scratch: self.scratch(),
            nodes: Vec::with_capacity(self.forest.len()),
            steps: Vec::new(),
            proved: Vec::new(),
        };
        let root = run.sequentialize(self.forest.roots().to_vec());
        let proof = Proof::new(self.forest.clone(), run.nodes, root)
            .expect("premises precede conclusions and every occurrence is the forest's");
        debug_assert_eq!(proof.check(self.mode()), Ok(()));
        Ok(proof)
    }

    /// The mode a sequentialized proof holds in: classical, with Mix if
    /// the structure allows it.
    fn mode(&self) -> Mode {
        if self.mix {
            Mode::CLASSICAL.with_mix()
        } else {
            Mode::CLASSICAL
        }
    }
}

/// A sequentialization in progress.
struct Sequentialization<'a> {
    /// The proof net.
    net: &'a ProofStructure,
    /// Working memory for the searches; conclusions already handled are
    /// deleted in it, so a sub-net is exactly what its conclusions reach.
    scratch: Scratch,
    /// The proof's arena so far.
    nodes: Vec<Node>,
    /// What is yet to do, the next step last. A derivation is as high as
    /// its net is large, so the steps are kept here and not on the
    /// caller's stack.
    steps: Vec<Step>,
    /// The nodes concluding the sub-nets proved and not yet used as a
    /// premise, the last proved last.
    proved: Vec<NodeId>,
}

/// A step of a sequentialization.
enum Step {
    /// Prove the sub-net with these conclusions.
    Prove(Vec<OccId>),
    /// Join the last two sub-nets proved with Mix.
    Mix,
    /// Apply this `⊗` to the last two sub-nets proved, which hold its left
    /// and its right premise.
    Tensor(OccId),
    /// Apply these `⅋` to the last sub-net proved, the last of them first.
    Pars(Vec<OccId>),
}

impl Sequentialization<'_> {
    /// Adds a node and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        NodeId::new(self.nodes.len() as u32 - 1)
    }

    /// Takes the node concluding the last sub-net proved.
    fn premise(&mut self) -> NodeId {
        self.proved
            .pop()
            .expect("a sub-net proved for every premise")
    }

    /// Proves the sub-net with the given conclusions and returns the node
    /// concluding it.
    fn sequentialize(&mut self, gamma: Vec<OccId>) -> NodeId {
        self.steps.push(Step::Prove(gamma));
        while let Some(step) = self.steps.pop() {
            let node = match step {
                Step::Prove(gamma) => {
                    self.stage(gamma);
                    continue;
                }
                Step::Mix => {
                    let other = self.premise();
                    let node = self.premise();
                    self.push(Node::Mix(node, other))
                }
                Step::Tensor(t) => {
                    let right = self.premise();
                    let left = self.premise();
                    self.push(Node::Tensor(t, left, right))
                }
                Step::Pars(pars) => {
                    let mut node = self.premise();
                    for &p in pars.iter().rev() {
                        node = self.push(Node::Par(p, node));
                    }
                    node
                }
            };
            self.proved.push(node);
        }
        self.premise()
    }

    /// One stage, on the sub-net with the given conclusions: proves it by
    /// an axiom, or leaves the steps that prove it, which are its parts
    /// with a Mix after each but the first, or the two sides of its
    /// splitting `⊗` and the `⊗` after them; the `⅋` conclusions it opened
    /// come after either.
    fn stage(&mut self, mut gamma: Vec<OccId>) {
        let f = self.net.forest();
        let graph = &self.net.graph;
        let child = |o: OccId, left: bool| if left { f.left(o) } else { f.right(o) }.unwrap();

        // Every ⅋ conclusion is opened, outermost first; its rule is
        // applied last, below the rest.
        let mut pars = Vec::new();
        let mut i = 0;
        while i < gamma.len() {
            let c = gamma[i];
            if f.kind(c) == Kind::Par {
                gamma[i] = child(c, true);
                gamma.push(child(c, false));
                self.scratch.delete(c);
                pars.push(c);
            } else {
                i += 1;
            }
        }
        if !pars.is_empty() {
            self.steps.push(Step::Pars(pars));
        }

        let parts = graph.search(&mut self.scratch, gamma.iter().map(|o| o.get()));
        if parts > 1 {
            debug_assert!(self.net.mix, "a proof net without Mix is connected");
            let mut labels: Vec<OccId> = Vec::new();
            for &c in &gamma {
                let label = self.scratch.component(c);
                if !labels.contains(&label) {
                    labels.push(label);
                }
            }
            // The steps are taken from the last: the first part, then
            // every other one and the Mix that joins it to those before.
            for (i, &label) in labels.iter().enumerate().rev() {
                if i > 0 {
                    self.steps.push(Step::Mix);
                }
                let part = gamma.iter().copied();
                let part = part.filter(|&c| self.scratch.component(c) == label);
                self.steps.push(Step::Prove(part.collect()));
            }
        } else if gamma.iter().all(|&c| f.is_literal(c)) {
            debug_assert!(gamma.len() == 2 && self.net.partner(gamma[0]) == Some(gamma[1]));
            let (x, y) = (gamma[0].min(gamma[1]), gamma[0].max(gamma[1]));
            let axiom = self.push(Node::Ax(x, y));
            self.proved.push(axiom);
        } else {
            // The splitting ⊗ conclusion with the smallest id: one whose
            // premise edge is a bridge.
            let t = gamma
                .iter()
                .copied()
                .filter(|&c| f.kind(c) == Kind::Tensor)
                .filter(|&c| self.scratch.is_bridge(c.get(), child(c, true).get()))
                .min()
                .expect("a connected proof net whose conclusions are not all literals has a splitting ⊗");
            let (l, r) = (child(t, true), child(t, false));
            self.scratch.delete(t);
            graph.search(&mut self.scratch, [l.get()]);
            let (mut left, mut right) = (vec![l], vec![r]);
            for &c in &gamma {
                if c == t {
                    continue;
                }
                if self.scratch.reached(c) {
                    left.push(c);
                } else {
                    right.push(c);
                }
            }
            self.steps.push(Step::Tensor(t));
            self.steps.push(Step::Prove(right));
            self.steps.push(Step::Prove(left));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ProofStructure;
    use crate::fragment::Mode;
    use crate::occurrences::{Forest, OccId};
    #[cfg(feature = "parse")]
    use crate::proofs::{Node, NodeId};
    use crate::sequents::Sequent;

    /// Wraps a raw id.
    #[cfg(feature = "parse")]
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Builds the structure of `input` with the links, or panics.
    #[cfg(feature = "parse")]
    fn net(input: &str, mix: bool, links: &[(u32, u32)]) -> ProofStructure {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let links: Vec<(OccId, OccId)> = links.iter().map(|&(x, y)| (o(x), o(y))).collect();
        ProofStructure::from_links(Forest::new(&s).unwrap(), mix, &links).unwrap()
    }

    /// A net whose derivation is thousands of inferences high is
    /// sequentialized on a stack too small for a recursion to follow it,
    /// 64 bytes a level: the tensors of `⊢ a1 ⊗ (a2 ⊗ (… ⊗ an)), ~a1, …,
    /// ~an` split off one axiom at a time.
    #[test]
    fn a_high_derivation_needs_no_stack() {
        use crate::occurrences::Sign;
        use crate::sequents::{Atom, Term, TermId};
        const PAIRS: u32 = 2048;
        // Terms 2i and 2i + 1 are the two literals of atom i.
        let mut terms: Vec<Term> = (0..PAIRS)
            .map(Atom::new)
            .flat_map(|a| [Term::Var(a), Term::DualVar(a)])
            .collect();
        let mut chain = TermId::new(2 * (PAIRS - 1));
        for a in (0..PAIRS - 1).rev() {
            terms.push(Term::Tensor(TermId::new(2 * a), chain));
            chain = TermId::new(terms.len() as u32 - 1);
        }
        let mut roots = vec![chain];
        roots.extend((0..PAIRS).map(|a| TermId::new(2 * a + 1)));
        let atoms = (0..PAIRS).map(|a| format!("a{a}")).collect();
        let forest = Forest::new(&Sequent {
            terms,
            roots,
            atoms,
        })
        .unwrap();
        let literal = |a, sign| forest.literals(Atom::new(a), sign)[0];
        let links: Vec<(OccId, OccId)> = (0..PAIRS)
            .map(|a| (literal(a, Sign::Var), literal(a, Sign::DualVar)))
            .collect();
        let net = ProofStructure::from_links(forest, false, &links).unwrap();
        let thread = std::thread::Builder::new().stack_size(128 * 1024);
        let proof = thread.spawn(move || net.sequentialize()).unwrap();
        let proof = proof.join().unwrap().unwrap();
        assert_eq!(proof.nodes().len(), 2 * PAIRS as usize - 1);
        assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
    }

    /// The classic nets sequentialize into the expected terms: the ⅋ below
    /// the ⊗ that is not splitting until it is opened, a Mix for a
    /// disconnected net, and an error for a structure that is no net.
    #[cfg(feature = "parse")]
    #[test]
    fn classic_nets() {
        use Node::*;
        let n = NodeId::new;
        // ⊢ A ⊗ B, ~A ⅋ ~B: 0 ⊗, 1 A, 2 B, 3 ⅋, 4 ~A, 5 ~B. The ⊗ splits
        // only once the ⅋ is opened.
        let proof = net("|- A * B, ~A par ~B", false, &[(1, 4), (2, 5)])
            .sequentialize()
            .unwrap();
        assert_eq!(
            proof.nodes(),
            [
                Ax(o(1), o(4)),
                Ax(o(2), o(5)),
                Tensor(o(0), n(0), n(1)),
                Par(o(3), n(2)),
            ]
        );
        // ⊢ A ⅋ B, ~A, ~B with Mix: 0 ⅋, 1 A, 2 B, 3 ~A, 4 ~B.
        let proof = net("|- A par B, ~A, ~B", true, &[(1, 3), (2, 4)])
            .sequentialize()
            .unwrap();
        assert_eq!(
            proof.nodes(),
            [
                Ax(o(1), o(3)),
                Ax(o(2), o(4)),
                Mix(n(0), n(1)),
                Par(o(0), n(2))
            ]
        );
        assert_eq!(proof.check(Mode::CLASSICAL.with_mix()), Ok(()));
        // ⊢ A ⊗ B, C ⊗ (~A ⅋ ~B), ~C: 0 ⊗, 1 A, 2 B, 3 ⊗, 4 C, 5 ⅋, 6 ~A,
        // 7 ~B, 8 ~C: the first ⊗ is not splitting, the second is.
        let proof = net(
            "|- A * B, C * (~A par ~B), ~C",
            false,
            &[(1, 6), (2, 7), (4, 8)],
        )
        .sequentialize()
        .unwrap();
        assert_eq!(
            proof.nodes(),
            [
                Ax(o(4), o(8)),
                Ax(o(1), o(6)),
                Ax(o(2), o(7)),
                Tensor(o(0), n(1), n(2)),
                Par(o(5), n(3)),
                Tensor(o(3), n(0), n(4)),
            ]
        );
        assert!(
            net("|- A par B, ~A, ~B", false, &[(1, 3), (2, 4)])
                .sequentialize()
                .is_err()
        );
    }
}
