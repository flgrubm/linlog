// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// Printing sequents and formulas in one-sided notation.
pub mod fmt;
/// The terms an arena is built from.
pub mod term;

pub use fmt::Formula;
pub use term::{Atom, Kind, Term, TermId};

use crate::hash::HashMap;

/// A one-sided sequent in negation normal form: root formulas over an arena of
/// shared subterms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sequent {
    /// Every subformula; a term refers only to terms at lower indices.
    pub(crate) terms: Vec<Term>,
    /// The root formulas, as arena indices, in the order the sequent lists
    /// them.
    pub(crate) roots: Vec<TermId>,
    /// The atom names that `Var` and `DualVar` refer to by index.
    pub(crate) atoms: Vec<String>,
}

impl std::default::Default for Sequent {
    /// Returns the empty sequent.
    fn default() -> Self {
        Self::new()
    }
}

impl Sequent {
    /// Returns the empty sequent.
    pub const fn new() -> Self {
        Self {
            terms: vec![],
            roots: vec![],
            atoms: vec![],
        }
    }

    /// Returns the arena: every subformula, each referring only to terms at
    /// lower indices.
    pub fn terms(&self) -> &[Term] {
        &self.terms
    }

    /// Returns the term at `id`, which must belong to this sequent.
    pub fn term(&self, id: TermId) -> Term {
        self.terms[id.index()]
    }

    /// Returns the root formulas, in the order the sequent lists them.
    pub fn roots(&self) -> &[TermId] {
        &self.roots
    }

    /// Returns the atom names, in the order `Atom` indexes them.
    pub fn atom_names(&self) -> &[String] {
        &self.atoms
    }

    /// Returns the name of `atom`, which must belong to this sequent.
    pub fn atom_name(&self, atom: Atom) -> &str {
        &self.atoms[atom.index()]
    }

    /// Returns the atom called `name`, if the sequent has one.
    pub fn atom(&self, name: &str) -> Option<Atom> {
        self.atoms
            .iter()
            .position(|n| n == name)
            .map(|i| Atom::new(i as u32))
    }

    /// Returns, per arena term, the number of subformula occurrences of the
    /// formula it is the top of, itself included: a subterm counts once
    /// for every place it stands in. A number that passes `u64::MAX` is
    /// that.
    pub(crate) fn sizes(&self) -> Vec<u64> {
        // A subterm precedes its parent, so its size is there when the
        // parent's is added up.
        let mut sizes = vec![0u64; self.terms.len()];
        for (n, term) in self.terms.iter().enumerate() {
            sizes[n] = term
                .subterms()
                .fold(1u64, |sum, k| sum.saturating_add(sizes[k.index()]));
        }
        sizes
    }

    /// Returns the number of subformula occurrences of the sequent, which
    /// is the number of nodes of its formulas written out as trees: a
    /// term that several terms or root formulas share counts once for
    /// each. That is how many occurrences a [`Forest`](crate::Forest) of
    /// the sequent has, and it can be exponentially more than the arena
    /// has terms; a number that passes `u64::MAX` is returned as that.
    /// One pass over the arena.
    pub fn occurrences(&self) -> u64 {
        let sizes = self.sizes();
        self.roots
            .iter()
            .fold(0u64, |sum, r| sum.saturating_add(sizes[r.index()]))
    }

    /// Check whether the internal data structure is correct
    pub fn verify_integrity(&self) -> Result<(), crate::Error> {
        let num_atoms = self.atoms.len() as u32;
        let num_terms = self.terms.len() as u32;

        // check that terms only reference
        //   - other terms with lower IDs than themselves
        //   - existing atom IDS
        self.terms
            .iter()
            .enumerate()
            .try_for_each(|(n, e)| e.check_bounds(num_atoms, n as u32))?;

        // check that all root indices are valid
        self.roots.iter().try_for_each(|n| {
            if n.get() >= num_terms {
                Err(crate::Error::TermIndexOutOfBounds(
                    n.index(),
                    num_terms as usize,
                ))
            } else {
                Ok(())
            }
        })?;

        Ok(())
    }

    /// Sorts the root indices and checks that each names a term.
    fn optimize_roots(&mut self) -> Result<(), crate::Error> {
        self.roots.sort();
        self.roots.shrink_to_fit();
        let Some(n) = self.roots.last() else {
            return Ok(());
        };
        let num_terms = self.terms.len() as u32;
        if n.get() >= num_terms {
            Err(crate::Error::TermIndexOutOfBounds(
                n.index(),
                num_terms as usize,
            ))
        } else {
            Ok(())
        }
    }

    /// Drops the terms no root formula reaches and merges equal terms, keeping
    /// the arena topologically sorted. Fails if an index breaks that order or
    /// points outside the arena.
    fn optimize_terms(&mut self) -> Result<(), crate::Error> {
        let num_terms = self.terms.len();

        let mut reachable = vec![false; num_terms];
        for n in self.roots.iter() {
            match reachable.get_mut(n.index()) {
                Some(r) => *r = true,
                None => return Err(crate::Error::TermIndexOutOfBounds(n.index(), num_terms)),
            }
        }

        // A subterm precedes its parent, so one pass from the top marks every
        // reachable term before it is visited.
        for n in (0..num_terms).rev() {
            if !reachable[n] {
                continue;
            }
            for k in self.terms[n].subterms() {
                if k.index() >= n {
                    return Err(crate::Error::SubtermIndexNotDecreasing(k.index(), n));
                }
                reachable[k.index()] = true;
            }
        }

        // In index order, every subterm has its final index before its parents
        // are rebuilt, so hashing the rebuilt terms merges equal terms at any
        // depth.
        let mut new_index = vec![None; num_terms];
        let mut terms = Vec::with_capacity(num_terms);
        let mut kept = HashMap::<Term, TermId>::default();

        for (n, e) in self.terms.iter().enumerate() {
            if !reachable[n] {
                continue;
            }
            // The subterms of a reachable term are reachable and precede it.
            let e = e.map_subterms(|k| new_index[k.index()].unwrap());
            let fresh_index = TermId::new(terms.len() as u32);
            let index = *kept.entry(e).or_insert(fresh_index);
            if index == fresh_index {
                terms.push(e);
            }
            new_index[n] = Some(index);
        }

        self.terms = terms;
        self.roots = self
            .roots
            .iter()
            .map(|k| new_index[k.index()].unwrap())
            .collect();

        Ok(())
    }

    /// Merges atoms of the same name, numbered in order of first occurrence.
    fn optimize_atoms(&mut self) -> Result<(), crate::Error> {
        use Term::*;
        let num_atoms = self.atoms.len();
        let mut atoms = Vec::<String>::with_capacity(num_atoms);
        let mut seen =
            HashMap::<&str, Atom>::with_capacity_and_hasher(num_atoms, Default::default());

        for e in self.terms.iter_mut() {
            let (Var(a) | DualVar(a)) = *e else {
                continue;
            };
            let name: &str = self
                .atoms
                .get(a.index())
                .ok_or(crate::Error::InvalidVariableIndex(a.index(), num_atoms))?;
            let merged = *seen.entry(name).or_insert_with(|| {
                atoms.push(name.to_string());
                Atom::new((atoms.len() - 1) as u32)
            });
            *e = match *e {
                Var(_) => Var(merged),
                _ => DualVar(merged),
            };
        }
        drop(seen);
        atoms.shrink_to_fit();
        self.atoms = atoms;
        Ok(())
    }

    /// Merges atoms of the same name and equal terms, drops the terms no
    /// root formula reaches and sorts the root formulas. Fails if the arena
    /// breaks its invariants.
    pub fn optimize(&mut self) -> Result<(), crate::Error> {
        self.optimize_atoms()?;
        self.optimize_terms()?;
        self.optimize_roots()?;
        self.optimize_atoms()?;
        Ok(())
    }

    /// Gives atoms of the same name one index, the first's, and drops the
    /// later entries of the table, so that a name is an atom; a table of
    /// distinct names stays as it is. The terms must refer to atoms of the
    /// table.
    pub(crate) fn merge_atoms(&mut self) {
        use Term::*;
        let mut seen =
            HashMap::<&str, Atom>::with_capacity_and_hasher(self.atoms.len(), Default::default());
        let mut merged = Vec::with_capacity(self.atoms.len());
        for name in &self.atoms {
            let fresh = Atom::new(seen.len() as u32);
            merged.push(*seen.entry(name).or_insert(fresh));
        }
        let distinct = seen.len();
        drop(seen);
        if distinct == self.atoms.len() {
            return;
        }
        for e in &mut self.terms {
            *e = match *e {
                Var(a) => Var(merged[a.index()]),
                DualVar(a) => DualVar(merged[a.index()]),
                other => other,
            };
        }
        // An entry stays when it is the first of its name, which is when
        // its new index is the number of entries kept before it.
        let (mut old, mut kept) = (0, 0);
        self.atoms.retain(|_| {
            let first = merged[old].index() == kept;
            old += 1;
            kept += usize::from(first);
            first
        });
    }

    /// Appends another sequent's arena, root formulas and atom table to this
    /// one's as they are, so that a name both have is two entries until
    /// [`optimize`](Self::optimize) or [`merge_atoms`](Self::merge_atoms)
    /// runs.
    pub(crate) fn append(&mut self, s: Self) {
        let offset_atoms = self.atoms.len() as u32;
        let offset_terms = self.terms.len() as u32;

        self.terms.extend(
            s.terms
                .into_iter()
                .map(|e| e.offset(offset_atoms, offset_terms)),
        );

        self.roots.extend(
            s.roots
                .into_iter()
                .map(|n| TermId::new(n.get() + offset_terms)),
        );

        self.atoms.extend(s.atoms);
    }

    /// Consumes another sequent and appends its root formulas to this one,
    /// with the arenas concatenated; an atom of the other sequent is the
    /// atom of the same name here, if there is one.
    pub fn add(&mut self, s: Self) {
        self.append(s);
        self.merge_atoms();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a sequent from raw parts, as a test would write them.
    pub(crate) fn raw(terms: Vec<Term>, roots: &[u32], atoms: &[&str]) -> Sequent {
        Sequent {
            terms,
            roots: roots.iter().map(|&n| TermId::new(n)).collect(),
            atoms: atoms.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Shorthand for `Term::Var`.
    pub(crate) const fn var(a: u32) -> Term {
        Term::Var(Atom::new(a))
    }

    /// Shorthand for `Term::DualVar`.
    pub(crate) const fn dual_var(a: u32) -> Term {
        Term::DualVar(Atom::new(a))
    }

    /// Shorthand for `Term::Tensor`.
    pub(crate) const fn tensor(k: u32, l: u32) -> Term {
        Term::Tensor(TermId::new(k), TermId::new(l))
    }

    /// Shorthand for `Term::Bang`.
    pub(crate) const fn bang(k: u32) -> Term {
        Term::Bang(TermId::new(k))
    }

    /// Adding a sequent identifies its atoms with those of the same name,
    /// and leaves the others in the order they came.
    #[test]
    fn add_merges_atoms_by_name() {
        let mut s = raw(vec![dual_var(0), var(1)], &[0, 1], &["A", "B"]);
        s.add(raw(
            vec![var(0), var(1), var(2)],
            &[0, 1, 2],
            &["C", "A", "B"],
        ));
        assert_eq!(s.atoms, ["A", "B", "C"]);
        assert_eq!(s.terms, [dual_var(0), var(1), var(2), var(0), var(1)]);
        assert_eq!(s.roots.len(), 5);
    }

    /// The empty sequent survives optimisation unchanged.
    #[test]
    fn optimize_empty() {
        let mut s = Sequent::new();
        s.optimize().unwrap();
        assert!(s.terms.is_empty() && s.roots.is_empty() && s.atoms.is_empty());
    }

    /// Atoms of one name merge, and the rest are renumbered to match, also
    /// when a repeated name comes before a new one.
    #[test]
    fn optimize_merges_atoms() {
        let mut s = raw(
            vec![var(0), dual_var(1), var(2)],
            &[0, 1, 2],
            &["A", "A", "B"],
        );
        s.optimize().unwrap();
        assert_eq!(s.atoms, ["A", "B"]);
        assert_eq!(s.terms, [var(0), dual_var(0), var(1)]);
    }

    /// References to a duplicate term point to its kept copy, also after
    /// unreachable terms before it are dropped.
    #[test]
    fn optimize_redirects_duplicates() {
        use Term::*;
        let mut s = raw(vec![Zero, One, Top, Top, tensor(1, 3)], &[2, 4], &[]);
        s.optimize().unwrap();
        assert_eq!(s.terms, [One, Top, tensor(0, 1)]);
        assert_eq!(s.roots, [TermId::new(1), TermId::new(2)]);
    }

    /// A subterm that does not precede its parent is reported with each index
    /// in its place.
    #[test]
    fn subterm_order_error_names_both_terms() {
        use Term::*;
        let s = raw(vec![One, bang(2), Bot], &[1], &[]);
        assert_eq!(
            s.verify_integrity().unwrap_err().to_string(),
            "term 1 refers to term 2, but a subterm must come before the terms that use it"
        );
        assert!(s.clone().optimize().is_err());
    }

    /// Equal terms merge at every depth, not only equal leaves.
    #[test]
    fn optimize_merges_equal_subterms() {
        let mut s = raw(
            vec![
                var(0),
                var(1),
                tensor(0, 1),
                bang(2),
                var(0),
                var(1),
                tensor(4, 5),
                bang(6),
            ],
            &[3, 7],
            &["A", "B"],
        );
        s.optimize().unwrap();
        assert_eq!(s.terms, [var(0), var(1), tensor(0, 1), bang(2)]);
        assert_eq!(s.roots, [TermId::new(3), TermId::new(3)]);
    }

    /// Appending a sequent shifts its term and atom indices past the existing
    /// ones.
    #[test]
    fn add_offsets_indices() {
        let mut s = raw(vec![var(0)], &[0], &["A"]);
        s.add(raw(vec![dual_var(0), bang(0)], &[1], &["B"]));
        assert_eq!(s.terms, [var(0), dual_var(1), bang(1)]);
        assert_eq!(s.roots, [TermId::new(0), TermId::new(2)]);
        assert_eq!(s.atoms, ["A", "B"]);
    }
}
