// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::Error;

/// The index of a term in a sequent's arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TermId(u32);

impl TermId {
    /// Wraps a raw arena index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw arena index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the arena.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// An atom (a propositional variable), by its index in a sequent's dictionary
/// of atom names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Atom(u32);

impl Atom {
    /// Wraps a raw dictionary index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw dictionary index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the dictionary.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A node of a formula in negation normal form, which refers to its subterms
/// by arena index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Term {
    /// An atom `a`.
    Var(Atom),
    /// The negation `~a` of an atom.
    DualVar(Atom),
    /// `1`, the unit of `⊗`.
    One,
    /// `⊥`, the unit of `⅋`.
    Bot,
    /// `⊤`, the unit of `&`.
    Top,
    /// `0`, the unit of `⊕`.
    Zero,
    /// `A ⊗ B`
    Tensor(TermId, TermId),
    /// `A ⅋ B`
    Par(TermId, TermId),
    /// `A & B`
    With(TermId, TermId),
    /// `A ⊕ B`
    Plus(TermId, TermId),
    /// `!A`
    Bang(TermId),
    /// `?A`
    Quest(TermId),
}

/// The kind of a term: its connective, or which literal it is.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// An atom `a`.
    Var,
    /// The negation `~a` of an atom.
    DualVar,
    /// `1`
    One,
    /// `⊥`
    Bot,
    /// `⊤`
    Top,
    /// `0`
    Zero,
    /// `⊗`
    Tensor,
    /// `⅋`
    Par,
    /// `&`
    With,
    /// `⊕`
    Plus,
    /// `!`
    Bang,
    /// `?`
    Quest,
}

impl Kind {
    /// Returns the kind of the dual term: `⊗` for `⅋`, `Var` for `DualVar`, …
    pub const fn dual(self) -> Self {
        use Kind::*;
        match self {
            Var => DualVar,
            DualVar => Var,
            One => Bot,
            Bot => One,
            Top => Zero,
            Zero => Top,
            Tensor => Par,
            Par => Tensor,
            With => Plus,
            Plus => With,
            Bang => Quest,
            Quest => Bang,
        }
    }

    /// Returns how many subterms a term of this kind has: 0, 1 or 2.
    pub const fn arity(self) -> u8 {
        use Kind::*;
        match self {
            Var | DualVar | One | Bot | Top | Zero => 0,
            Bang | Quest => 1,
            Tensor | Par | With | Plus => 2,
        }
    }

    /// Returns whether this is a literal, `a` or `~a`.
    pub const fn is_literal(self) -> bool {
        matches!(self, Kind::Var | Kind::DualVar)
    }
}

impl Term {
    /// Returns the kind of this term.
    pub const fn kind(self) -> Kind {
        use Term::*;
        match self {
            Var(_) => Kind::Var,
            DualVar(_) => Kind::DualVar,
            One => Kind::One,
            Bot => Kind::Bot,
            Top => Kind::Top,
            Zero => Kind::Zero,
            Tensor(..) => Kind::Tensor,
            Par(..) => Kind::Par,
            With(..) => Kind::With,
            Plus(..) => Kind::Plus,
            Bang(_) => Kind::Bang,
            Quest(_) => Kind::Quest,
        }
    }

    /// Returns the atom of a literal, or `None` for any other term.
    pub const fn atom(self) -> Option<Atom> {
        match self {
            Term::Var(a) | Term::DualVar(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the term with its top node dualised (`⊗` to `⅋`, `a` to `~a`,
    /// …) and the same subterm indices. Dualising a whole formula dualises
    /// every node, which `Sequent` does when it lowers a two-sided sequent.
    pub const fn dual(self) -> Self {
        use Term::*;
        match self {
            Var(a) => DualVar(a),
            DualVar(a) => Var(a),
            One => Bot,
            Bot => One,
            Top => Zero,
            Zero => Top,
            Tensor(k, l) => Par(k, l),
            Par(k, l) => Tensor(k, l),
            With(k, l) => Plus(k, l),
            Plus(k, l) => With(k, l),
            Bang(k) => Quest(k),
            Quest(k) => Bang(k),
        }
    }

    /// Returns the subterms in order: none for a literal or a unit, one for
    /// `!` and `?`, the left one then the right one for a binary connective.
    pub fn subterms(self) -> impl Iterator<Item = TermId> {
        let (first, second) = self.operands();
        first.into_iter().chain(second)
    }

    /// Returns the first and the second subterm, each if there is one.
    pub(crate) const fn operands(self) -> (Option<TermId>, Option<TermId>) {
        use Term::*;
        match self {
            Tensor(k, l) | Par(k, l) | With(k, l) | Plus(k, l) => (Some(k), Some(l)),
            Bang(k) | Quest(k) => (Some(k), None),
            Var(_) | DualVar(_) | One | Bot | Top | Zero => (None, None),
        }
    }

    /// Returns the term with every subterm index replaced by what `f` maps it
    /// to.
    pub(crate) fn map_subterms(self, mut f: impl FnMut(TermId) -> TermId) -> Self {
        use Term::*;
        match self {
            Tensor(k, l) => Tensor(f(k), f(l)),
            Par(k, l) => Par(f(k), f(l)),
            With(k, l) => With(f(k), f(l)),
            Plus(k, l) => Plus(f(k), f(l)),
            Bang(k) => Bang(f(k)),
            Quest(k) => Quest(f(k)),
            leaf => leaf,
        }
    }

    /// Shifts the atom and subterm indices by the given offsets.
    pub(crate) fn offset(self, atoms: u32, terms: u32) -> Self {
        use Term::*;
        match self {
            Var(a) => Var(Atom(a.0 + atoms)),
            DualVar(a) => DualVar(Atom(a.0 + atoms)),
            other => other.map_subterms(|k| TermId(k.0 + terms)),
        }
    }

    /// Checks that the term refers only to atoms below `atom_bound` and to
    /// subterms below `term_bound`.
    pub(crate) fn check_bounds(self, atom_bound: u32, term_bound: u32) -> Result<(), Error> {
        if let Some(a) = self.atom()
            && a.0 >= atom_bound
        {
            return Err(Error::InvalidVariableIndex(a.index(), atom_bound as usize));
        }
        for k in self.subterms() {
            if k.0 >= term_bound {
                return Err(Error::SubtermIndexNotDecreasing(
                    k.index(),
                    term_bound as usize,
                ));
            }
        }
        Ok(())
    }
}
