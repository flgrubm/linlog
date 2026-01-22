// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::modes::{Efficient, Mode, Rich};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::fmt::Debug;

pub trait SymbolSet: Copy + Clone + Debug {
    type One: Copy + Clone + Debug + PartialEq + Eq;
    type Bot: Copy + Clone + Debug + PartialEq + Eq;
    type Top: Copy + Clone + Debug + PartialEq + Eq;
    type Zero: Copy + Clone + Debug + PartialEq + Eq;
    type Bang: Copy + Clone + Debug + PartialEq + Eq;
    type Quest: Copy + Clone + Debug + PartialEq + Eq;
    type Tensor: Copy + Clone + Debug + PartialEq + Eq;
    type Par: Copy + Clone + Debug + PartialEq + Eq;
    type With: Copy + Clone + Debug + PartialEq + Eq;
    type Plus: Copy + Clone + Debug + PartialEq + Eq;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Symbol<S: SymbolSet, M: Mode> {
    Var(usize),
    NegVar(M::NegVar),
    One(S::One),
    Bot(S::Bot),
    Top(S::Top),
    Zero(S::Zero),
    Dual(M::Dual),
    Bang(S::Bang),
    Quest(S::Quest),
    Tensor(S::Tensor),
    Par(S::Par),
    With(S::With),
    Plus(S::Plus),
    Lollipop(M::Lollipop),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LL;

impl SymbolSet for LL {
    type One = ();
    type Bot = ();
    type Top = ();
    type Zero = ();
    type Bang = ();
    type Quest = ();
    type Tensor = ();
    type Par = ();
    type With = ();
    type Plus = ();
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MELL;

impl SymbolSet for MELL {
    type One = ();
    type Bot = ();
    type Top = Infallible;
    type Zero = Infallible;
    type Bang = ();
    type Quest = ();
    type Tensor = ();
    type Par = ();
    type With = Infallible;
    type Plus = Infallible;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct MLL;

impl SymbolSet for MLL {
    type One = ();
    type Bot = ();
    type Top = Infallible;
    type Zero = Infallible;
    type Bang = Infallible;
    type Quest = Infallible;
    type Tensor = ();
    type Par = ();
    type With = Infallible;
    type Plus = Infallible;
}

impl<M: Mode> From<Symbol<MELL, M>> for Symbol<LL, M> {
    fn from(s: Symbol<MELL, M>) -> Symbol<LL, M> {
        unimplemented!()
    }
}

impl<M: Mode> From<Symbol<MLL, M>> for Symbol<LL, M> {
    fn from(s: Symbol<MLL, M>) -> Symbol<LL, M> {
        unimplemented!()
    }
}

impl<M: Mode> From<Symbol<MLL, M>> for Symbol<MELL, M> {
    fn from(s: Symbol<MLL, M>) -> Symbol<MELL, M> {
        unimplemented!()
    }
}

impl<M: Mode> TryFrom<Symbol<LL, M>> for Symbol<MELL, M> {
    type Error = ();
    fn try_from(s: Symbol<LL, M>) -> Result<Symbol<MELL, M>, Self::Error> {
        unimplemented!()
    }
}

impl<M: Mode> TryFrom<Symbol<LL, M>> for Symbol<MLL, M> {
    type Error = ();
    fn try_from(s: Symbol<LL, M>) -> Result<Symbol<MLL, M>, Self::Error> {
        unimplemented!()
    }
}

impl<M: Mode> TryFrom<Symbol<MELL, M>> for Symbol<MLL, M> {
    type Error = ();
    fn try_from(s: Symbol<MELL, M>) -> Result<Symbol<MLL, M>, Self::Error> {
        unimplemented!()
    }
}

impl<S: SymbolSet, M: Mode> TryFrom<Symbol<S, M>> for char {
    type Error = ();

    fn try_from(s: Symbol<S, M>) -> Result<char, Self::Error> {
        use Symbol::*;
        match s {
            Var(_) | NegVar(_) => Err(()),
            One(_) => Ok('1'),
            Bot(_) => Ok('⊥'),
            Top(_) => Ok('⊤'),
            Zero(_) => Ok('0'),
            Dual(_) => Ok('~'),
            Bang(_) => Ok('!'),
            Quest(_) => Ok('?'),
            Tensor(_) => Ok('⊗'),
            Par(_) => Ok('⅋'),
            With(_) => Ok('&'),
            Plus(_) => Ok('⊕'),
            Lollipop(_) => Ok('⊸'),
        }
    }
}

#[test]
pub fn test() {
    assert_eq!(
        Symbol::<LL, Rich>::One(()),
        Symbol::from(Symbol::<MLL, Rich>::One(()))
    );
}
