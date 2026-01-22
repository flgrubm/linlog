// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod modes;
pub mod parsing;
pub mod serialize;
pub mod symbols;

use modes::Mode;
use serde::{Deserialize, Serialize};
use symbols::{Symbol, SymbolSet};

#[derive(Debug)]
pub struct Sequent<S: SymbolSet, M: Mode> {
    lhs_symbols: M::LHSSymbols<S>,
    lhs_terms: M::LHSTerms,

    rhs_symbols: Box<[Symbol<S, M>]>,
    rhs_terms: Box<[usize]>,

    variable_names: Box<[String]>,
}
