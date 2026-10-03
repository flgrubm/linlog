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

/// One stop of a [`Walk`] over a formula.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Visit<T> {
    /// A subformula is reached, before anything below it. The flag says
    /// whether it stands inside another formula, where a binary one is
    /// written in brackets; for the formula the walk started on it is the
    /// flag the walk was given.
    Enter(T, bool),
    /// The place between the two subformulas of a binary formula.
    Between(T),
    /// A formula with subformulas is left, after the last of them, with
    /// the flag it was entered with.
    Exit(T, bool),
}

/// What a walk has yet to do with a formula.
#[derive(Clone, Copy, Debug)]
enum Step<T> {
    /// Enter it.
    Enter(T, bool),
    /// Its first subformula is done: go on to the second.
    Second(T, bool),
    /// Its last subformula is done: leave it.
    Exit(T, bool),
}

/// How many pending steps a walk holds before it allocates: one for each
/// formula it is inside of, so an ordinary formula is written without an
/// allocation.
const INLINE: usize = 16;

/// The stops of a formula in the order it is written: every subformula
/// when it is reached, a binary one again between its two subformulas, and
/// a compound one once more when it is left. The walk keeps the formulas
/// it is inside of in a stack of its own, so a formula of any depth takes
/// no more of the caller's stack than a literal does.
pub(crate) struct Walk<T, F> {
    /// The first pending steps, the next one last.
    inline: [Step<T>; INLINE],
    /// How many entries of `inline` are pending.
    len: usize,
    /// The pending steps beyond `inline`, the next one last.
    spilled: Vec<Step<T>>,
    /// Returns the first and the second subformula of a formula, each if
    /// there is one, in the order they are written.
    operands: F,
}

impl<T: Copy, F: Fn(T) -> (Option<T>, Option<T>)> Walk<T, F> {
    /// Returns the walk over the formula `root`, whose subformulas are
    /// what `operands` returns; `nested` is the flag `root` is entered
    /// with.
    pub(crate) fn new(root: T, nested: bool, operands: F) -> Self {
        Self {
            inline: [Step::Enter(root, nested); INLINE],
            len: 1,
            spilled: Vec::new(),
            operands,
        }
    }

    /// Adds a step to take before the pending ones.
    fn push(&mut self, step: Step<T>) {
        if self.len < INLINE {
            self.inline[self.len] = step;
            self.len += 1;
        } else {
            self.spilled.push(step);
        }
    }
}

impl<T: Copy, F: Fn(T) -> (Option<T>, Option<T>)> Iterator for Walk<T, F> {
    type Item = Visit<T>;

    /// Returns the next stop, or `None` once the formula is left.
    fn next(&mut self) -> Option<Visit<T>> {
        let step = match self.spilled.pop() {
            Some(step) => step,
            None => {
                self.len = self.len.checked_sub(1)?;
                self.inline[self.len]
            }
        };
        Some(match step {
            Step::Enter(formula, nested) => {
                match (self.operands)(formula) {
                    (Some(first), Some(_)) => {
                        self.push(Step::Second(formula, nested));
                        self.push(Step::Enter(first, true));
                    }
                    (Some(only), None) | (None, Some(only)) => {
                        self.push(Step::Exit(formula, nested));
                        self.push(Step::Enter(only, true));
                    }
                    (None, None) => {}
                }
                Visit::Enter(formula, nested)
            }
            Step::Second(formula, nested) => {
                self.push(Step::Exit(formula, nested));
                if let (_, Some(second)) = (self.operands)(formula) {
                    self.push(Step::Enter(second, true));
                }
                Visit::Between(formula)
            }
            Step::Exit(formula, nested) => Visit::Exit(formula, nested),
        })
    }
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
        for visit in Walk::new(id, brackets, |k: TermId| self.terms[k.index()].operands()) {
            match visit {
                Visit::Enter(k, nested) => {
                    let term = self.terms[k.index()];
                    debug_assert!(term.subterms().all(|sub| sub < k));
                    match term {
                        Var(a) => f.write_str(self.atom_name(a))?,
                        DualVar(a) => write!(f, "~{}", self.atom_name(a))?,
                        One => f.write_str("1")?,
                        Bot => f.write_str("⊥")?,
                        Top => f.write_str("⊤")?,
                        Zero => f.write_str("0")?,
                        Tensor(..) | Par(..) | With(..) | Plus(..) if nested => f.write_str("(")?,
                        Tensor(..) | Par(..) | With(..) | Plus(..) => {}
                        Bang(_) => f.write_str("!")?,
                        Quest(_) => f.write_str("?")?,
                    }
                }
                Visit::Between(k) => f.write_str(match self.terms[k.index()] {
                    Tensor(..) => " ⊗ ",
                    Par(..) => " ⅋ ",
                    With(..) => " & ",
                    _ => " ⊕ ",
                })?,
                Visit::Exit(k, nested) => {
                    if nested && self.terms[k.index()].kind().arity() == 2 {
                        f.write_str(")")?;
                    }
                }
            }
        }
        Ok(())
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
