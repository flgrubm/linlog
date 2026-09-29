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
}

impl Sequentialization<'_> {
    /// Adds a node and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        NodeId::new(self.nodes.len() as u32 - 1)
    }

    /// Proves the sub-net with the given conclusions and returns the node
    /// concluding it.
    fn sequentialize(&mut self, mut gamma: Vec<OccId>) -> NodeId {
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

        let parts = graph.search(&mut self.scratch, gamma.iter().map(|o| o.get()));
        let mut node = if parts > 1 {
            debug_assert!(self.net.mix, "a proof net without Mix is connected");
            let mut labels: Vec<OccId> = Vec::new();
            for &c in &gamma {
                let label = self.scratch.component(c);
                if !labels.contains(&label) {
                    labels.push(label);
                }
            }
            let parts: Vec<Vec<OccId>> = labels
                .iter()
                .map(|&l| {
                    gamma
                        .iter()
                        .copied()
                        .filter(|&c| self.scratch.component(c) == l)
                        .collect()
                })
                .collect();
            let mut parts = parts.into_iter();
            let mut node = self.sequentialize(parts.next().unwrap());
            for part in parts {
                let other = self.sequentialize(part);
                node = self.push(Node::Mix(node, other));
            }
            node
        } else if gamma.iter().all(|&c| f.is_literal(c)) {
            debug_assert!(gamma.len() == 2 && self.net.partner(gamma[0]) == Some(gamma[1]));
            let (x, y) = (gamma[0].min(gamma[1]), gamma[0].max(gamma[1]));
            self.push(Node::Ax(x, y))
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
            let (left, right) = (self.sequentialize(left), self.sequentialize(right));
            self.push(Node::Tensor(t, left, right))
        };
        for &p in pars.iter().rev() {
            node = self.push(Node::Par(p, node));
        }
        node
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::super::ProofStructure;
    use crate::fragment::Mode;
    use crate::occurrences::{Forest, OccId};
    use crate::proofs::{Node, NodeId};
    use crate::sequents::Sequent;

    /// Wraps a raw id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Builds the structure of `input` with the links, or panics.
    fn net(input: &str, mix: bool, links: &[(u32, u32)]) -> ProofStructure {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let links: Vec<(OccId, OccId)> = links.iter().map(|&(x, y)| (o(x), o(y))).collect();
        ProofStructure::from_links(Forest::new(&s).unwrap(), mix, &links).unwrap()
    }

    /// The classic nets sequentialize into the expected terms: the ⅋ below
    /// the ⊗ that is not splitting until it is opened, a Mix for a
    /// disconnected net, and an error for a structure that is no net.
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
