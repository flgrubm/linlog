// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod fmt;
pub mod parsing;
pub mod raw;
pub mod symbols;
pub mod terms;

use serde::{Deserialize, Serialize};
use symbols::{RawSymbol, Symbol, SymbolSet, Token};

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "S::Symb: Serialize + for<'a> Deserialize<'a>")]
pub struct Sequent<S: SymbolSet> {
    symbols: Box<[S::Symb]>,
    terms: Box<[usize]>,
    variable_names: Box<[String]>,
}
