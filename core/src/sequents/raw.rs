// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::sequents::symbols::LLRawSymb;

use super::symbols::{LL, MELL, MLL, SymbolSet, is_mell_raw, is_mll_raw};
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

pub fn is_mell_raw_sequent(s: &Sequent<LL>) -> bool {
    is_mell_raw(&s.symbols_lhs) && is_mell_raw(&s.symbols_rhs)
}

pub fn is_mll_raw_sequent(s: &Sequent<LL>) -> bool {
    is_mll_raw(&s.symbols_lhs) && is_mll_raw(&s.symbols_rhs)
}

impl TryFrom<Sequent<LL>> for Sequent<MELL> {
    type Error = crate::Error;
    fn try_from(_s: Sequent<LL>) -> Result<Sequent<MELL>, Self::Error> {
        unimplemented!()
    }
}

impl TryFrom<Sequent<LL>> for Sequent<MLL> {
    type Error = crate::Error;
    fn try_from(_s: Sequent<LL>) -> Result<Sequent<MLL>, Self::Error> {
        unimplemented!()
    }
}

pub unsafe fn to_mell_raw_sequent_unchecked(s: Sequent<LL>) -> Sequent<MELL> {
    unsafe { Sequent::<MELL>::try_from(s).unwrap_unchecked() }
}

pub unsafe fn to_mll_raw_sequent_unchecked(s: Sequent<LL>) -> Sequent<MLL> {
    unsafe { Sequent::<MLL>::try_from(s).unwrap_unchecked() }
}
