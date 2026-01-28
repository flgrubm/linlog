// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::Sequent;
use super::expressions::LLExpression;
use crate::index::Index;
use crate::logics::Logic;
use std::fmt::{Display, Formatter, Result as FmtResult};

impl<I: Index, L: Logic<I>> Sequent<I, L> {
    fn display_term<const NEEDS_BRACKETS: bool>(
        &self,
        index: I,
        f: &mut Formatter<'_>,
    ) -> FmtResult {
        use LLExpression::*;
        debug_assert!(index < Index::from_usize(self.term_arena.len()));
        match (self.term_arena[index.as_usize()]).into() {
            Var(var_index) => {
                debug_assert!(var_index < Index::from_usize(self.variable_dict.len()));
                write!(f, "{}", &self.variable_dict[var_index.as_usize()])
            }
            DualVar(var_index) => {
                debug_assert!(var_index < Index::from_usize(self.variable_dict.len()));
                write!(f, "~{}", &self.variable_dict[var_index.as_usize()])
            }
            One => write!(f, "1"),
            Bot => write!(f, "⊥"),
            Top => write!(f, "⊤"),
            Zero => write!(f, "0"),
            Tensor(m, n) => {
                debug_assert!(m < index);
                debug_assert!(n < index);
                if NEEDS_BRACKETS {
                    write!(f, "(")?
                }
                self.display_term::<true>(m, f)?;
                write!(f, " ⊗ ")?;
                self.display_term::<true>(n, f)?;
                if NEEDS_BRACKETS {
                    write!(f, ")")?
                }
                Ok(())
            }
            Par(m, n) => {
                debug_assert!(m < index);
                debug_assert!(n < index);
                if NEEDS_BRACKETS {
                    write!(f, "(")?
                }
                self.display_term::<true>(m, f)?;
                write!(f, " ⅋ ")?;
                self.display_term::<true>(n, f)?;
                if NEEDS_BRACKETS {
                    write!(f, ")")?
                }
                Ok(())
            }
            With(m, n) => {
                debug_assert!(m < index);
                debug_assert!(n < index);
                if NEEDS_BRACKETS {
                    write!(f, "(")?
                }
                self.display_term::<true>(m, f)?;
                write!(f, " & ")?;
                self.display_term::<true>(n, f)?;
                if NEEDS_BRACKETS {
                    write!(f, ")")?
                }
                Ok(())
            }
            Plus(m, n) => {
                debug_assert!(m < index);
                debug_assert!(n < index);
                if NEEDS_BRACKETS {
                    write!(f, "(")?
                }
                self.display_term::<true>(m, f)?;
                write!(f, " ⊕ ")?;
                self.display_term::<true>(n, f)?;
                if NEEDS_BRACKETS {
                    write!(f, ")")?
                }
                Ok(())
            }
            Bang(m) => {
                debug_assert!(m < index);
                write!(f, "!")?;
                self.display_term::<true>(m, f)
            }
            Quest(m) => {
                debug_assert!(m < index);
                write!(f, "?")?;
                self.display_term::<true>(m, f)
            }
        }
    }
}

impl<I: Index, L: Logic<I>> Display for Sequent<I, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "⊢")?;

        let mut it = self.term_ids.iter();

        if let Some(n) = it.next() {
            write!(f, " ")?;
            self.display_term::<false>(*n, f)?;
        }

        for n in it {
            write!(f, ", ")?;
            self.display_term::<false>(*n, f)?;
        }
        Ok(())
    }
}
