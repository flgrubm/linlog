// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::Sequent;
use super::symbols::{Symbol, SymbolSet};
use std::convert::Infallible;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub trait Mode {
    type NegVar: Copy + Clone + Debug + PartialEq + Eq;
    type Dual: Copy + Clone + Debug + PartialEq + Eq;
    type Lollipop: Copy + Clone + Debug + PartialEq + Eq;
    type LHSSymbols<S: SymbolSet>: Debug;
    type LHSTerms: Debug + PartialEq + Eq;
}

pub trait GeneralSequent<S: SymbolSet, M: Mode> {
    fn get_lhs_symbols(&self) -> Option<&[Symbol<S, M>]>;
    fn get_lhs_terms(&self) -> Option<&[usize]>;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rich;

impl Mode for Rich {
    type NegVar = Infallible;
    type Dual = ();
    type Lollipop = ();
    type LHSSymbols<S: SymbolSet> = Box<[Symbol<S, Rich>]>;
    type LHSTerms = Box<[usize]>;
}

impl<S: SymbolSet> GeneralSequent<S, Rich> for Sequent<S, Rich> {
    fn get_lhs_symbols(&self) -> Option<&[Symbol<S, Rich>]> {
        if self.lhs_symbols.is_empty() {
            None
        } else {
            Some(&self.lhs_symbols)
        }
    }

    fn get_lhs_terms(&self) -> Option<&[usize]> {
        if self.lhs_symbols.is_empty() {
            None
        } else {
            Some(&self.lhs_terms)
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Efficient;

impl Mode for Efficient {
    type NegVar = ();
    type Dual = Infallible;
    type Lollipop = Infallible;
    type LHSSymbols<S: SymbolSet> = Infallible;
    type LHSTerms = Infallible;
}

impl<S: SymbolSet> GeneralSequent<S, Efficient> for Sequent<S, Efficient> {
    fn get_lhs_symbols(&self) -> Option<&[Symbol<S, Efficient>]> {
        None
    }

    fn get_lhs_terms(&self) -> Option<&[usize]> {
        None
    }
}
