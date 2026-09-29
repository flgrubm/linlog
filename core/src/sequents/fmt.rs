// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::{Sequent, Term, TermId};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A formula of a sequent, as a value that prints it in one-sided notation.
#[derive(Clone, Copy, Debug)]
pub struct Formula<'a> {
    /// The sequent whose arena and atom names the formula lives in.
    sequent: &'a Sequent,
    /// The formula's root term.
    id: TermId,
}

impl Sequent {
    /// Returns the formula rooted at `id`, which must belong to this sequent,
    /// as a value that prints it.
    pub fn formula(&self, id: TermId) -> Formula<'_> {
        debug_assert!(id.index() < self.terms.len());
        Formula { sequent: self, id }
    }

    /// Writes the term at `id`, in brackets if it is binary and `brackets` is
    /// set.
    fn fmt_term(&self, id: TermId, f: &mut Formatter<'_>, brackets: bool) -> FmtResult {
        use Term::*;
        debug_assert!(id.index() < self.terms.len());
        let term = self.terms[id.index()];
        for k in term.subterms() {
            debug_assert!(k < id);
        }
        let binary = |f: &mut Formatter<'_>, k: TermId, symbol: &str, l: TermId| {
            if brackets {
                write!(f, "(")?;
            }
            self.fmt_term(k, f, true)?;
            write!(f, " {symbol} ")?;
            self.fmt_term(l, f, true)?;
            if brackets {
                write!(f, ")")?;
            }
            Ok(())
        };
        match term {
            Var(a) => write!(f, "{}", self.atom_name(a)),
            DualVar(a) => write!(f, "~{}", self.atom_name(a)),
            One => write!(f, "1"),
            Bot => write!(f, "⊥"),
            Top => write!(f, "⊤"),
            Zero => write!(f, "0"),
            Tensor(k, l) => binary(f, k, "⊗", l),
            Par(k, l) => binary(f, k, "⅋", l),
            With(k, l) => binary(f, k, "&", l),
            Plus(k, l) => binary(f, k, "⊕", l),
            Bang(k) => {
                write!(f, "!")?;
                self.fmt_term(k, f, true)
            }
            Quest(k) => {
                write!(f, "?")?;
                self.fmt_term(k, f, true)
            }
        }
    }
}

impl Display for Formula<'_> {
    /// Writes the formula with brackets around every binary subformula.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.sequent.fmt_term(self.id, f, false)
    }
}

impl Display for Sequent {
    /// Writes the sequent one-sided: `⊢` followed by its formulas, separated by
    /// commas.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "⊢")?;

        let mut it = self.roots.iter();

        if let Some(n) = it.next() {
            write!(f, " ")?;
            self.fmt_term(*n, f, false)?;
        }

        for n in it {
            write!(f, ", ")?;
            self.fmt_term(*n, f, false)?;
        }
        Ok(())
    }
}
