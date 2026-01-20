// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod fmt;
pub mod parsing;
pub mod raw;
pub mod symbols;
pub mod terms;

use crate::utils::SlidingIterator;
use raw::Sequent as RawSequent;
use serde::{Deserialize, Serialize};
use std::iter::Iterator;
use symbols::{RawSymbol, Symbol, SymbolSet, Token};
use terms::{RawTerm, Term, fold_reduce_terms};

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "S::Symb: Serialize + for<'a> Deserialize<'a>")]
pub struct Sequent<S: SymbolSet> {
    symbols: Box<[S::Symb]>,
    terms: Box<[usize]>,
    variable_names: Box<[String]>,
}

pub struct SequentView<'a, S: SymbolSet> {
    terms: Box<&'a [S::Symb]>,
}

impl<S: SymbolSet> From<RawSequent<S>> for Sequent<S> {
    fn from(s: RawSequent<S>) -> Sequent<S> {
        let mut terms: Vec<(RawTerm<'_, S>, bool)> =
            Vec::with_capacity(s.terms_lhs.len() + s.terms_rhs.len());

        for (lower, upper) in s.terms_lhs.into_iter().slide_iter(s.symbols_lhs.len()) {
            terms.push((
                Term {
                    tm: &s.symbols_lhs[lower..upper],
                },
                false,
            ));
        }

        for (lower, upper) in s.terms_rhs.into_iter().slide_iter(s.symbols_rhs.len()) {
            terms.push((
                Term {
                    tm: &s.symbols_rhs[lower..upper],
                },
                true,
            ));
        }

        let symbols: Vec<S::Symb> = Vec::with_capacity(s.symbols_lhs.len() + s.symbols_rhs.len());
        let offsets: Vec<usize> = Vec::with_capacity(symbols.len());

        let (symbols, terms) = terms
            .into_iter()
            .fold((symbols, offsets), fold_reduce_terms::<S>);

        Sequent {
            symbols: symbols.into_boxed_slice(),
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
