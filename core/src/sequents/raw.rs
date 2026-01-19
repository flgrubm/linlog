// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::sequents::symbols::LLRawSymb;

use super::symbols::SymbolSet;
use serde::{Deserialize, Serialize};

// pub(super) in order to convert from parsed sequent

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "S::RawSymb: Serialize + for<'a> Deserialize<'a>")]
pub struct Sequent<S: SymbolSet> {
    pub(super) symbols_lhs: Box<[S::RawSymb]>,
    pub(super) terms_lhs: Box<[usize]>,

    pub(super) symbols_rhs: Box<[S::RawSymb]>,
    pub(super) terms_rhs: Box<[usize]>,

    pub(super) variable_names: Box<[String]>,
}
