// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

pub mod fmt;
pub mod parsing;
pub mod raw;
pub mod symbols;
pub mod terms;

pub mod raw_old;

use serde::{Deserialize, Serialize};
use symbols::{RawSymbol, Symbol, Token};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sequent<S: Token + Symbol> {
    symbols: Box<[S]>,
    terms: Box<[usize]>,
    variable_names: Box<[String]>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawSequent<S: Token + RawSymbol> {
    symbols_lhs: Box<[S]>,
    terms_lhs: Box<[usize]>,

    symbols_rhs: Box<[S]>,
    terms_rhs: Box<[usize]>,

    variable_names: Box<[String]>,
}
