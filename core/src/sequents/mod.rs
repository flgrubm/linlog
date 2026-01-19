// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod fmt;
pub mod parsing;
pub mod raw;
pub mod symbols;
pub mod terms;

use raw::Sequent as RawSequent;
use serde::{Deserialize, Serialize};
use symbols::{RawSymbol, Symbol, SymbolSet, Token};
use terms::Term;

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "S::Symb: Serialize + for<'a> Deserialize<'a>")]
pub struct Sequent<S: SymbolSet> {
    symbols: Box<[S::Symb]>,
    terms: Box<[usize]>,
    variable_names: Box<[String]>,
}

impl<S: SymbolSet> From<RawSequent<S>> for Sequent<S> {
    fn from(s: RawSequent<S>) -> Sequent<S> {
        let symbols: Box<[S::Symb]> =
            terms::reduce::<S>(Term { tm: &s.symbols_lhs }, Term { tm: &s.symbols_rhs });

        let offset = s.terms_lhs.len();
        let mut terms = s.terms_lhs.into_vec();
        terms.reserve_exact(s.terms_rhs.len());
        terms.extend(s.terms_rhs.iter().map(|x| x + offset));
        Sequent {
            symbols: symbols,
            terms: terms.into_boxed_slice(),
            variable_names: s.variable_names,
        }
    }
}

impl<S: SymbolSet> From<Sequent<S>> for RawSequent<S> {
    fn from(s: Sequent<S>) -> RawSequent<S> {
        unimplemented!()
    }
}
