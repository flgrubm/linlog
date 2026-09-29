// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The `⅋`-free skeleton of a proof structure: the occurrences joined by the
//! premise edges of `⊗` nodes and by the axiom links, as a union-find. A
//! cycle in the skeleton avoids every `⅋` and so survives every switching,
//! which is why a link between two literals of one component is never
//! admissible. The union-find has an undo log so that the last link can be
//! taken back in constant time.

/// The raw index that stands for "no vertex".
const NONE: u32 = u32::MAX;

/// A union-find with union by rank and no path compression, so that every
/// union changes one parent and at most one rank, which the undo log
/// records. Finding a root costs the height of its tree, at most
/// logarithmic in the number of vertices.
#[derive(Clone, Debug)]
pub(super) struct Skeleton {
    /// Per vertex, its parent in the union-find forest; a root is its own
    /// parent.
    parent: Box<[u32]>,
    /// Per root, a bound on the height of its tree.
    rank: Box<[u8]>,
    /// One entry per [`union`](Self::union) since the log was last cleared:
    /// the root that was attached under the other, or `NONE` when the union
    /// joined nothing, and whether the other root's rank grew.
    log: Vec<(u32, bool)>,
}

impl Skeleton {
    /// Returns `n` singleton components.
    pub(super) fn new(n: usize) -> Self {
        Self {
            parent: (0..n as u32).collect(),
            rank: vec![0; n].into_boxed_slice(),
            log: Vec::new(),
        }
    }

    /// Returns the root of the component of `x`.
    pub(super) fn find(&self, mut x: u32) -> u32 {
        while self.parent[x as usize] != x {
            x = self.parent[x as usize];
        }
        x
    }

    /// Returns whether `x` and `y` lie in one component.
    pub(super) fn same(&self, x: u32, y: u32) -> bool {
        self.find(x) == self.find(y)
    }

    /// Joins the components of `x` and `y`, records the change in the log,
    /// and returns whether they were different components.
    pub(super) fn union(&mut self, x: u32, y: u32) -> bool {
        let (mut a, mut b) = (self.find(x), self.find(y));
        if a == b {
            self.log.push((NONE, false));
            return false;
        }
        if self.rank[a as usize] < self.rank[b as usize] {
            (a, b) = (b, a);
        }
        // `a` has the larger rank, or the same, and takes `b` under it.
        self.parent[b as usize] = a;
        let bumped = self.rank[a as usize] == self.rank[b as usize];
        if bumped {
            self.rank[a as usize] += 1;
        }
        self.log.push((b, bumped));
        true
    }

    /// Takes back the last union that the log records. Panics if the log is
    /// empty.
    pub(super) fn undo(&mut self) {
        let (b, bumped) = self.log.pop().expect("no union to undo");
        if b == NONE {
            return;
        }
        let a = self.parent[b as usize];
        self.parent[b as usize] = b;
        if bumped {
            self.rank[a as usize] -= 1;
        }
    }

    /// Forgets the unions made so far, so that they can no longer be undone,
    /// and makes room for `capacity` more log entries.
    pub(super) fn commit(&mut self, capacity: usize) {
        self.log.clear();
        self.log.reserve(capacity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unions join components, undo takes the last one back in order, and
    /// a union within a component is a no-op that still undoes cleanly.
    #[test]
    fn union_find_with_undo() {
        let mut s = Skeleton::new(6);
        assert!(s.union(0, 1));
        assert!(s.union(2, 3));
        s.commit(4);
        assert!(s.same(0, 1) && s.same(2, 3) && !s.same(1, 2));

        assert!(s.union(1, 2));
        assert!(!s.union(0, 3), "already joined");
        assert!(s.union(4, 5));
        assert!(s.same(0, 3) && s.same(4, 5) && !s.same(3, 4));

        s.undo();
        assert!(!s.same(4, 5));
        s.undo();
        assert!(s.same(0, 3), "undoing a no-op changes nothing");
        s.undo();
        assert!(!s.same(1, 2) && s.same(0, 1) && s.same(2, 3));
        assert!(s.log.is_empty());
        assert!(s.rank.iter().all(|&r| r <= 1), "ranks went back down");
    }
}
