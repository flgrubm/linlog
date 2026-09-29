// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The coloured structure graph of a proof structure and the tests the
//! correctness criterion runs on it.
//!
//! The vertices are the occurrences. The edges are the premise edges of
//! every `⊗` and `⅋` and the axiom links. The colouring: the two premise
//! edges of a `⅋` share a colour of their own, and every other edge has a
//! colour of its own, so a cycle is *properly coloured* (no two consecutive
//! edges alike) exactly when it enters and leaves no `⅋` through its two
//! premises, that is, when some switching keeps it. A switching cycle is a
//! properly coloured cycle. Unlinked literals are leaves and lie on no
//! cycle, so the test means the same on a partial structure.
//!
//! By Yeo's theorem, a graph without properly coloured cycle has a vertex
//! that every component of the graph without it meets in edges of one
//! colour only, and deleting such a vertex, which lies on no properly
//! coloured cycle, keeps the property. Repeating the deletion until it
//! stops decides the question exactly: everything goes, or a properly
//! coloured cycle is left. With this colouring the deletion condition needs
//! no colours: a vertex that is not a `⅋` has an edge of its own colour to
//! every neighbour, so it is deletable when every incident edge is a
//! bridge; a `⅋` is deletable when its parent edge is a bridge, or absent,
//! since its premise edges share a colour. Bridges come from one
//! depth-first search per round.

use super::NONE;
use crate::occurrences::{Forest, OccId, OccSet};
use crate::sequents::Kind;

/// The coloured structure graph in compressed sparse row layout: per
/// vertex, its edge to its parent first, then to its children, and for a
/// literal its axiom link last, with `NONE` while it is unlinked.
#[derive(Clone, Debug)]
pub(super) struct Graph {
    /// Per vertex, where its slots start in `to`; one more entry holds the
    /// total.
    start: Box<[u32]>,
    /// Per slot, the other end of the edge.
    to: Box<[u32]>,
    /// The edges every switching keeps: two per `⊗`, one per `⅋`.
    switched_tree_edges: usize,
}

/// Working memory for the correctness tests, sized for one structure, so
/// that running a test allocates nothing; [`ProofStructure::scratch`] makes
/// one.
///
/// [`ProofStructure::scratch`]: super::ProofStructure::scratch
#[derive(Clone, Debug)]
pub struct Scratch {
    /// The vertices the deletion procedure has removed.
    deleted: OccSet,
    /// The vertices deleted when the procedure got stuck, kept while a
    /// cycle is being isolated among the others.
    stuck: OccSet,
    /// Per slot, whether the edge is taken out of the graph, which the
    /// witness searches use.
    removed: Box<[bool]>,
    /// Per vertex, its discovery time in the last search, or `NONE`.
    disc: Box<[u32]>,
    /// Per vertex, the least discovery time it reaches through its
    /// search-tree descendants and one back edge.
    low: Box<[u32]>,
    /// Per vertex, its parent in the search tree, or `NONE` for a root.
    dfs_parent: Box<[u32]>,
    /// Per vertex, whether the edge to its search-tree parent is a bridge.
    bridge: Box<[bool]>,
    /// Per vertex, the root of the search tree that reached it.
    component: Box<[u32]>,
    /// The search stack: a vertex and its next slot to look at.
    stack: Vec<(u32, u32)>,
}

impl Scratch {
    /// Returns working memory for a graph.
    fn new(graph: &Graph) -> Self {
        let n = graph.start.len() - 1;
        Self {
            deleted: OccSet::empty(n),
            stuck: OccSet::empty(n),
            removed: vec![false; graph.to.len()].into_boxed_slice(),
            disc: vec![NONE; n].into_boxed_slice(),
            low: vec![NONE; n].into_boxed_slice(),
            dfs_parent: vec![NONE; n].into_boxed_slice(),
            bridge: vec![false; n].into_boxed_slice(),
            component: vec![NONE; n].into_boxed_slice(),
            stack: Vec::with_capacity(n),
        }
    }

    /// Returns whether a vertex is deleted.
    fn is_deleted(&self, v: u32) -> bool {
        self.deleted.contains(OccId::new(v))
    }

    /// Makes every vertex present again.
    pub(super) fn restore(&mut self) {
        self.deleted.clear();
    }

    /// Takes a vertex out of the graph.
    pub(super) fn delete(&mut self, v: OccId) {
        self.deleted.insert(v);
    }

    /// Returns whether the last search reached the vertex.
    pub(super) fn reached(&self, v: OccId) -> bool {
        self.disc[v.index()] != NONE
    }

    /// Returns the root of the search tree that reached the vertex in the
    /// last search.
    pub(super) fn component(&self, v: OccId) -> OccId {
        OccId::new(self.component[v.index()])
    }

    /// Returns whether the edge between two adjacent vertices is a bridge
    /// according to the last search, which must have reached them.
    pub(super) fn is_bridge(&self, v: u32, w: u32) -> bool {
        debug_assert!(
            self.reached(OccId::new(v)) && self.reached(OccId::new(w)),
            "the last search did not reach both ends"
        );
        (self.dfs_parent[w as usize] == v && self.bridge[w as usize])
            || (self.dfs_parent[v as usize] == w && self.bridge[v as usize])
    }
}

impl Graph {
    /// Builds the graph of a forest, with every literal unlinked.
    pub(super) fn new(forest: &Forest) -> Self {
        let n = forest.len();
        let mut start = Vec::with_capacity(n + 1);
        let mut to = Vec::with_capacity(3 * n);
        let mut switched_tree_edges = 0;
        for o in forest.ids() {
            start.push(to.len() as u32);
            if let Some(p) = forest.parent(o) {
                to.push(p.get());
            }
            to.extend(forest.children(o).map(OccId::get));
            match forest.kind(o) {
                Kind::Tensor => switched_tree_edges += 2,
                Kind::Par => switched_tree_edges += 1,
                _ => to.push(NONE),
            }
        }
        start.push(to.len() as u32);
        Self {
            start: start.into_boxed_slice(),
            to: to.into_boxed_slice(),
            switched_tree_edges,
        }
    }

    /// Returns working memory for this graph.
    pub(super) fn scratch(&self) -> Scratch {
        Scratch::new(self)
    }

    /// Returns the number of vertices.
    fn len(&self) -> usize {
        self.start.len() - 1
    }

    /// Returns the slots of a vertex.
    fn slots(&self, v: u32) -> std::ops::Range<usize> {
        self.start[v as usize] as usize..self.start[v as usize + 1] as usize
    }

    /// Returns the slot of a literal's axiom link.
    fn axiom_slot(&self, l: u32) -> usize {
        self.start[l as usize + 1] as usize - 1
    }

    /// Returns the slot at `w` of the edge to `v`.
    fn twin(&self, v: u32, w: u32) -> usize {
        self.slots(w)
            .find(|&s| self.to[s] == v)
            .expect("the edge exists at both ends")
    }

    /// Adds the axiom link between two literals.
    pub(super) fn link(&mut self, x: u32, y: u32) {
        self.to[self.axiom_slot(x)] = y;
        self.to[self.axiom_slot(y)] = x;
    }

    /// Removes the axiom link between two literals.
    pub(super) fn unlink(&mut self, x: u32, y: u32) {
        self.to[self.axiom_slot(x)] = NONE;
        self.to[self.axiom_slot(y)] = NONE;
    }

    /// Returns the number of edges every switching keeps when the
    /// structure has `links` axiom links.
    pub(super) fn switched_edges(&self, links: usize) -> usize {
        self.switched_tree_edges + links
    }

    /// Searches depth-first from each of `roots` in turn, over the vertices
    /// not deleted and the edges not removed, computing for every vertex
    /// reached its discovery time, its search-tree parent, the root that
    /// reached it, and whether its parent edge is a bridge. Returns the
    /// number of roots that reached something new, which is the number of
    /// connected components among the vertices reached.
    pub(super) fn search(
        &self,
        scratch: &mut Scratch,
        roots: impl IntoIterator<Item = u32>,
    ) -> u32 {
        debug_assert_eq!(
            scratch.removed.len(),
            self.to.len(),
            "a scratch of another graph"
        );
        scratch.disc.fill(NONE);
        let mut time = 0;
        let mut trees = 0;
        for r in roots {
            if scratch.is_deleted(r) || scratch.disc[r as usize] != NONE {
                continue;
            }
            trees += 1;
            self.discover(scratch, r, NONE, r, &mut time);
            while let Some(&(v, next)) = scratch.stack.last() {
                if next < self.start[v as usize + 1] {
                    scratch.stack.last_mut().unwrap().1 = next + 1;
                    let w = self.to[next as usize];
                    if w == NONE
                        || scratch.removed[next as usize]
                        || scratch.is_deleted(w)
                        || w == scratch.dfs_parent[v as usize]
                    {
                        continue;
                    }
                    if scratch.disc[w as usize] == NONE {
                        self.discover(scratch, w, v, r, &mut time);
                    } else {
                        let (lv, dw) = (scratch.low[v as usize], scratch.disc[w as usize]);
                        scratch.low[v as usize] = lv.min(dw);
                    }
                } else {
                    scratch.stack.pop();
                    let p = scratch.dfs_parent[v as usize];
                    if p != NONE {
                        let (lp, lv) = (scratch.low[p as usize], scratch.low[v as usize]);
                        scratch.low[p as usize] = lp.min(lv);
                        scratch.bridge[v as usize] = lv > scratch.disc[p as usize];
                    }
                }
            }
        }
        trees
    }

    /// Enters a vertex in the search: from `parent`, in the tree of `root`.
    fn discover(&self, scratch: &mut Scratch, v: u32, parent: u32, root: u32, time: &mut u32) {
        scratch.disc[v as usize] = *time;
        scratch.low[v as usize] = *time;
        *time += 1;
        scratch.dfs_parent[v as usize] = parent;
        scratch.component[v as usize] = root;
        scratch.bridge[v as usize] = false;
        scratch.stack.push((v, self.start[v as usize]));
    }

    /// Returns whether a vertex may be deleted according to the last
    /// search: it is a `⅋` whose parent edge is absent or a bridge, or any
    /// other vertex all of whose edges are bridges.
    fn deletable(&self, forest: &Forest, scratch: &Scratch, v: u32) -> bool {
        let present = |s: usize| {
            let w = self.to[s];
            w != NONE && !scratch.removed[s] && !scratch.is_deleted(w)
        };
        if forest.kind(OccId::new(v)) == Kind::Par {
            match forest.parent(OccId::new(v)) {
                Some(p) => !present(self.slots(v).start) || scratch.is_bridge(v, p.get()),
                None => true,
            }
        } else {
            self.slots(v)
                .all(|s| !present(s) || scratch.is_bridge(v, self.to[s]))
        }
    }

    /// Runs the deletion procedure on the vertices not yet deleted: in
    /// rounds, deletes every deletable vertex until none is. Returns
    /// whether every vertex went, which means the graph has no switching
    /// cycle; otherwise the vertices left in place are those the procedure
    /// got stuck on, and a switching cycle runs among them.
    pub(super) fn acyclic(&self, forest: &Forest, scratch: &mut Scratch) -> bool {
        let n = self.len();
        let mut remaining = n - scratch.deleted.len();
        while remaining > 0 {
            self.search(scratch, 0..n as u32);
            let before = remaining;
            for v in 0..n as u32 {
                if !scratch.is_deleted(v) && self.deletable(forest, scratch, v) {
                    scratch.delete(OccId::new(v));
                    remaining -= 1;
                }
            }
            if remaining == before {
                return false;
            }
        }
        true
    }

    /// Isolates one switching cycle among the vertices [`acyclic`](Self::acyclic)
    /// got stuck on, which it must have just done, and returns its vertices
    /// in order along the cycle. Every edge among those vertices is taken
    /// out in turn and kept out when a switching cycle survives without it;
    /// an edge found necessary stays necessary as edges go, so one pass
    /// leaves exactly one cycle.
    pub(super) fn cycle(&self, forest: &Forest, scratch: &mut Scratch) -> Vec<OccId> {
        let n = self.len();
        scratch.stuck.clone_from(&scratch.deleted);
        for v in 0..n as u32 {
            for s in self.slots(v) {
                let w = self.to[s];
                if w == NONE
                    || w < v
                    || scratch.removed[s]
                    || scratch.stuck.contains(OccId::new(v))
                    || scratch.stuck.contains(OccId::new(w))
                {
                    continue;
                }
                let t = self.twin(v, w);
                scratch.removed[s] = true;
                scratch.removed[t] = true;
                scratch.deleted.clone_from(&scratch.stuck);
                if self.acyclic(forest, scratch) {
                    scratch.removed[s] = false;
                    scratch.removed[t] = false;
                } else {
                    scratch.stuck.clone_from(&scratch.deleted);
                }
            }
        }
        // What is left is one cycle: walk it from its first vertex.
        let first = (0..n as u32)
            .find(|&v| !scratch.stuck.contains(OccId::new(v)))
            .expect("a switching cycle is left");
        let mut cycle = Vec::new();
        let (mut previous, mut current) = (NONE, first);
        loop {
            cycle.push(OccId::new(current));
            let next = self
                .slots(current)
                .map(|s| (s, self.to[s]))
                .find(|&(s, w)| {
                    w != NONE
                        && w != previous
                        && !scratch.removed[s]
                        && !scratch.stuck.contains(OccId::new(w))
                })
                .map(|(_, w)| w)
                .expect("every vertex of the cycle has two edges on it");
            if next == first {
                break;
            }
            (previous, current) = (current, next);
        }
        scratch.removed.fill(false);
        cycle
    }

    /// Returns the connected components of the switching that keeps the
    /// left premise of every `⅋`, each as the vertices in it that have no
    /// parent edge there: the roots and the right premises of `⅋` nodes.
    /// Components come in the order of their smallest vertex.
    pub(super) fn parts(&self, forest: &Forest, scratch: &mut Scratch) -> Vec<Vec<OccId>> {
        let n = self.len();
        scratch.restore();
        for p in forest.ids().filter(|&p| forest.kind(p) == Kind::Par) {
            let r = forest.right(p).unwrap();
            scratch.removed[self.twin(r.get(), p.get())] = true;
            scratch.removed[self.slots(r.get()).start] = true;
        }
        self.search(scratch, 0..n as u32);
        scratch.removed.fill(false);
        let mut parts: Vec<(u32, Vec<OccId>)> = Vec::new();
        for v in forest.ids() {
            let cut = match forest.parent(v) {
                None => true,
                Some(p) => forest.kind(p) == Kind::Par && forest.right(p) == Some(v),
            };
            if !cut {
                continue;
            }
            let label = scratch.component[v.index()];
            match parts.iter_mut().find(|(l, _)| *l == label) {
                Some((_, part)) => part.push(v),
                None => parts.push((label, vec![v])),
            }
        }
        parts.sort_by_key(|(label, _)| *label);
        parts.into_iter().map(|(_, part)| part).collect()
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::super::{NetError, ProofStructure};
    use crate::occurrences::{Forest, OccId};
    use crate::search::generate::{self, Rng, Rules};
    use crate::sequents::{Kind, Sequent};

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

    /// The Danos–Regnier criterion by brute force: every switching is
    /// built and tested for a cycle and for connectedness with a
    /// union-find. Returns whether every switching is acyclic and whether
    /// every switching is connected.
    fn enumerate(net: &ProofStructure) -> (bool, bool) {
        let f = net.forest();
        let pars: Vec<OccId> = f.ids().filter(|&p| f.kind(p) == Kind::Par).collect();
        assert!(pars.len() <= 16, "too many switchings to enumerate");
        let (mut acyclic, mut connected) = (true, true);
        for switching in 0..1u32 << pars.len() {
            let keeps_left = |p: OccId| {
                let i = pars.iter().position(|&q| q == p).unwrap();
                switching & (1 << i) == 0
            };
            let mut parent: Vec<usize> = (0..f.len()).collect();
            fn find(parent: &mut [usize], mut x: usize) -> usize {
                while parent[x] != x {
                    x = parent[x];
                }
                x
            }
            let mut components = f.len();
            let mut cyclic = false;
            let mut join = |x: OccId, y: OccId| {
                let (a, b) = (find(&mut parent, x.index()), find(&mut parent, y.index()));
                if a == b {
                    cyclic = true;
                } else {
                    parent[a] = b;
                    components -= 1;
                }
            };
            for v in f.ids() {
                if let Some(p) = f.parent(v) {
                    let kept = match f.kind(p) {
                        Kind::Tensor => true,
                        Kind::Par => keeps_left(p) == (f.left(p) == Some(v)),
                        _ => unreachable!(),
                    };
                    if kept {
                        join(p, v);
                    }
                }
            }
            for &(x, y) in net.links() {
                join(x, y);
            }
            acyclic &= !cyclic;
            connected &= components == 1;
        }
        (acyclic, connected)
    }

    /// Checks that a reported cycle is a switching cycle: a simple cycle of
    /// the graph that uses at most one premise edge of every `⅋`.
    fn check_cycle(net: &ProofStructure, cycle: &[OccId]) {
        let f = net.forest();
        assert!(cycle.len() >= 3, "{cycle:?}");
        let mut sorted = cycle.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), cycle.len(), "simple: {cycle:?}");
        for i in 0..cycle.len() {
            let (a, b, c) = (
                cycle[(i + cycle.len() - 1) % cycle.len()],
                cycle[i],
                cycle[(i + 1) % cycle.len()],
            );
            let adjacent =
                f.parent(b) == Some(c) || f.parent(c) == Some(b) || net.partner(b) == Some(c);
            assert!(adjacent, "{b:?} and {c:?} are not adjacent in {cycle:?}");
            if f.kind(b) == Kind::Par {
                assert!(
                    f.parent(a) != Some(b) || f.parent(c) != Some(b),
                    "{cycle:?} passes {b:?} through both premises"
                );
            }
        }
    }

    /// The classic nets: the axiom-tensor example is a net, a link below a
    /// `⊗` is a cycle, below a `⅋` it is a net, two separate axioms under a
    /// `⅋` are a net with Mix only, and a partial or empty structure is
    /// neither.
    #[test]
    fn classic_structures() {
        // ⊢ ~A, A ⊗ ~B, B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B.
        assert_eq!(
            net("A, A -o B |- B", false, &[(0, 2), (3, 4)]).is_correct(),
            Ok(())
        );
        let partial = net("A, A -o B |- B", false, &[(0, 2)]);
        assert_eq!(partial.is_correct(), Err(NetError::Unlinked(o(4))));
        assert!(partial.is_acyclic(&mut partial.scratch()));
        // ⊢ A ⊗ ~A: 0 ⊗, 1 A, 2 ~A.
        let cyclic = net("|- A * ~A", true, &[(1, 2)]);
        let Err(NetError::SwitchingCycle(cycle)) = cyclic.is_correct() else {
            panic!("a link below a ⊗ is a switching cycle");
        };
        check_cycle(&cyclic, &cycle);
        assert_eq!(cycle, [o(0), o(1), o(2)]);
        assert!(!cyclic.is_acyclic(&mut cyclic.scratch()));
        // ⊢ A ⅋ ~A: the cycle through both premises does not count.
        assert_eq!(net("|- A par ~A", false, &[(1, 2)]).is_correct(), Ok(()));
        // ⊢ A ⅋ B, ~A, ~B: 0 ⅋, 1 A, 2 B, 3 ~A, 4 ~B.
        assert_eq!(
            net("|- A par B, ~A, ~B", false, &[(1, 3), (2, 4)]).is_correct(),
            Err(NetError::Disconnected(vec![
                vec![o(0), o(3)],
                vec![o(2), o(4)]
            ]))
        );
        assert_eq!(
            net("|- A par B, ~A, ~B", true, &[(1, 3), (2, 4)]).is_correct(),
            Ok(())
        );
        // ⊢ (A ⅋ ~A) ⅋ (B ⅋ ~B): 0 ⅋, 1 ⅋, 2 A, 3 ~A, 4 ⅋, 5 B, 6 ~B: the
        // right part hangs off a cut premise.
        assert_eq!(
            net("|- (A par ~A) par (B par ~B)", false, &[(2, 3), (5, 6)]).is_correct(),
            Err(NetError::Disconnected(vec![
                vec![o(0), o(3)],
                vec![o(4), o(6)]
            ]))
        );
        // ⊢ A ⊗ B, ~A ⊗ ~B: 0 ⊗, 1 A, 2 B, 3 ⊗, 4 ~A, 5 ~B: a cycle through
        // both tensors.
        let cyclic = net("|- A * B, ~A * ~B", true, &[(1, 4), (2, 5)]);
        let Err(NetError::SwitchingCycle(cycle)) = cyclic.is_correct() else {
            panic!("two tensors joined twice are a switching cycle");
        };
        check_cycle(&cyclic, &cycle);
        assert_eq!(cycle.len(), 6);
        assert_eq!(net("|-", true, &[]).is_correct(), Err(NetError::Empty));
    }

    /// The criterion agrees with the brute-force enumeration of switchings
    /// on random linkings of random sequents, with and without Mix,
    /// partial and complete, and every witness it names is one.
    #[test]
    fn agrees_with_enumeration() {
        let mut cases = (0, 0, 0);
        for (seed, mix) in [(1, false), (2, true)] {
            let mut rng = Rng::new(seed);
            for i in 0..200 {
                // Sequents that need Mix give disconnected structures when
                // Mix is not allowed; mutants give partial and cyclic ones.
                let rules = Rules {
                    units: false,
                    additives: false,
                    mix: i % 3 == 2,
                };
                let budget = 2 + rng.below(9);
                let mut formulas = generate::provable(&mut rng, rules, 3, budget);
                if i % 2 == 1 {
                    generate::mutate(&mut rng, &mut formulas, 3);
                }
                let text = generate::sequent(&formulas);
                let s: Sequent = text.parse().unwrap();
                let forest = Forest::new(&s).unwrap();
                // A random pairing per atom, as far as the counts allow.
                let mut pairs = Vec::new();
                for a in 0..s.atom_names().len() {
                    let atom = crate::sequents::Atom::new(a as u32);
                    let vars = forest.literals(atom, crate::occurrences::Sign::Var);
                    let mut duals = forest
                        .literals(atom, crate::occurrences::Sign::DualVar)
                        .to_vec();
                    for k in (1..duals.len()).rev() {
                        duals.swap(k, rng.below(k + 1));
                    }
                    pairs.extend(vars.iter().copied().zip(duals));
                }
                for k in (1..pairs.len()).rev() {
                    pairs.swap(k, rng.below(k + 1));
                }
                let mut net = ProofStructure::new(forest, mix).unwrap();
                let mut scratch = net.scratch();
                for &(x, y) in &pairs {
                    net.link(x, y);
                    let (acyclic, _) = enumerate(&net);
                    assert_eq!(
                        net.is_acyclic(&mut scratch),
                        acyclic,
                        "{text:?} with {:?}",
                        net.links()
                    );
                }
                let (acyclic, connected) = enumerate(&net);
                let verdict = net.is_correct();
                let expected =
                    !net.forest().is_empty() && net.is_complete() && acyclic && (mix || connected);
                assert_eq!(
                    verdict.is_ok(),
                    expected,
                    "{text:?} with {:?}: {verdict:?}",
                    net.links()
                );
                match &verdict {
                    Ok(()) => cases.0 += 1,
                    Err(NetError::SwitchingCycle(cycle)) => {
                        check_cycle(&net, cycle);
                        cases.1 += 1;
                    }
                    Err(NetError::Disconnected(parts)) => {
                        let f = net.forest();
                        let switched = f.ids().filter(|&v| f.kind(v) == Kind::Tensor).count() * 2
                            + f.ids().filter(|&v| f.kind(v) == Kind::Par).count()
                            + net.links().len();
                        assert_eq!(parts.len(), f.len() - switched, "{text:?}");
                        for &r in f.roots() {
                            assert_eq!(
                                parts.iter().filter(|p| p.contains(&r)).count(),
                                1,
                                "{text:?}"
                            );
                        }
                        cases.2 += 1;
                    }
                    Err(NetError::Unlinked(_)) => {}
                    Err(e) => panic!("{text:?}: {e}"),
                }
            }
        }
        assert!(cases.0 > 20 && cases.1 > 20 && cases.2 > 5, "{cases:?}");
    }
}
