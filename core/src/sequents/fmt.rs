// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::SequentNew;
use super::expressions::LLExpressionNew;
use crate::index::Index;
use crate::logics::LogicNew;
use std::fmt::{Display, Formatter, Result as FmtResult};

impl<I: Index, L: LogicNew<I>> SequentNew<I, L> {
    fn display_term<const NEEDS_BRACKETS: bool>(
        &self,
        index: I,
        f: &mut Formatter<'_>,
    ) -> FmtResult {
        use LLExpressionNew::*;
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

impl<I: Index, L: LogicNew<I>> Display for SequentNew<I, L> {
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
