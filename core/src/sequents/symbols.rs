// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use serde::{Deserialize, Serialize};
use subenum::subenum;

const SHOW_ONE: char = '1';
const SHOW_BOT: char = '⊥';
const SHOW_TOP: char = '⊤';
const SHOW_ZERO: char = '0';
const SHOW_DUAL_PREFIX: char = '~';
const SHOW_DUAL_POSTFIX: char = '⊥';
const SHOW_BANG: char = '!';
const SHOW_QUEST: char = '?';
const SHOW_TENSOR: char = '⊗';
const SHOW_PAR: char = '⅋';
const SHOW_WITH: char = '&';
const SHOW_PLUS: char = '⊕';
const SHOW_LOLLIPOP: char = '⊸';

// Utilities

#[derive(Copy, Clone, Debug)]
pub enum Arity {
    Nullary,
    Unary,
    Dual,
    Binary,
    Lollipop,
}

#[derive(Copy, Clone, Debug)]
pub enum SymbolShow {
    Variable(usize),
    DualVariable(usize),
    Char(char),
}

#[derive(Copy, Clone, Debug)]
pub enum RawSymbolShow {
    Variable(usize),
    Char(char),
    Variant(char, char),
}

pub trait Token: Copy + Clone {
    fn arity(&self) -> Arity;
}

pub trait Symbol: Copy + Clone {
    fn show(&self) -> SymbolShow;
}

pub trait RawSymbol: Copy + Clone {
    fn show(&self) -> RawSymbolShow;
}

pub trait SymbolSet {
    type Symb: Token + Symbol;
    type RawSymb: Token + RawSymbol;
    fn from_raw_pos(r: Self::RawSymb) -> Self::Symb;
    fn from_raw_neg(r: Self::RawSymb) -> Self::Symb;

    fn from_raw(r: Self::RawSymb, polarity: bool) -> Self::Symb {
        if polarity {
            Self::from_raw_pos(r)
        } else {
            Self::from_raw_neg(r)
        }
    }
}

// (Full) Linear Logic

#[subenum(MELLRawSymb, MLLRawSymb)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum LLRawSymb {
    #[subenum(MELLRawSymb, MLLRawSymb)]
    Var(usize),

    #[subenum(MELLRawSymb, MLLRawSymb)]
    One,

    #[subenum(MELLRawSymb, MLLRawSymb)]
    Bot,

    Top,

    Zero,

    #[subenum(MELLRawSymb, MLLRawSymb)]
    Dual,

    #[subenum(MELLRawSymb)]
    Bang,

    #[subenum(MELLRawSymb)]
    Quest,

    #[subenum(MELLRawSymb, MLLRawSymb)]
    Tensor,

    #[subenum(MELLRawSymb, MLLRawSymb)]
    Par,

    With,

    Plus,

    #[subenum(MELLRawSymb, MLLRawSymb)]
    Lollipop,
}

#[subenum(MELLSymb, MLLSymb)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum LLSymb {
    #[subenum(MELLSymb, MLLSymb)]
    Var(usize),
    #[subenum(MELLSymb, MLLSymb)]
    DualVar(usize),
    #[subenum(MELLSymb, MLLSymb)]
    One,
    #[subenum(MELLSymb, MLLSymb)]
    Bot,
    Top,
    Zero,
    #[subenum(MELLSymb)]
    Bang,
    #[subenum(MELLSymb)]
    Quest,
    #[subenum(MELLSymb, MLLSymb)]
    Tensor,
    #[subenum(MELLSymb, MLLSymb)]
    Par,
    With,
    Plus,
}

impl Token for LLRawSymb {
    fn arity(&self) -> Arity {
        use LLRawSymb::*;
        match self {
            Var(_) | One | Bot | Top | Zero => Arity::Nullary,
            Dual => Arity::Dual,
            Bang | Quest => Arity::Unary,
            Tensor | Par | With | Plus => Arity::Binary,
            Lollipop => Arity::Lollipop,
        }
    }
}

impl RawSymbol for LLRawSymb {
    fn show(&self) -> RawSymbolShow {
        use LLRawSymb::*;
        use RawSymbolShow::*;
        match self {
            Var(n) => Variable(*n),
            One => Char(SHOW_ONE),
            Bot => Char(SHOW_BOT),
            Top => Char(SHOW_TOP),
            Zero => Char(SHOW_ZERO),
            Dual => Variant(SHOW_DUAL_PREFIX, SHOW_DUAL_POSTFIX),
            Bang => Char(SHOW_BANG),
            Quest => Char(SHOW_QUEST),
            Tensor => Char(SHOW_TENSOR),
            Par => Char(SHOW_PAR),
            With => Char(SHOW_WITH),
            Plus => Char(SHOW_PLUS),
            Lollipop => Char(SHOW_LOLLIPOP),
        }
    }
}

impl Token for LLSymb {
    fn arity(&self) -> Arity {
        use LLSymb::*;
        match self {
            Var(_) | DualVar(_) | One | Bot | Top | Zero => Arity::Nullary,
            Bang | Quest => Arity::Unary,
            Tensor | Par | With | Plus => Arity::Binary,
        }
    }
}

impl Symbol for LLSymb {
    fn show(&self) -> SymbolShow {
        use LLSymb::*;
        use SymbolShow::*;
        match self {
            Var(n) => Variable(*n),
            DualVar(n) => DualVariable(*n),
            One => Char(SHOW_ONE),
            Bot => Char(SHOW_BOT),
            Top => Char(SHOW_TOP),
            Zero => Char(SHOW_ZERO),
            Bang => Char(SHOW_BANG),
            Quest => Char(SHOW_QUEST),
            Tensor => Char(SHOW_TENSOR),
            Par => Char(SHOW_PAR),
            With => Char(SHOW_WITH),
            Plus => Char(SHOW_PLUS),
        }
    }
}

pub struct LL {}

impl SymbolSet for LL {
    type Symb = LLSymb;
    type RawSymb = LLRawSymb;

    fn from_raw_pos(r: LLRawSymb) -> LLSymb {
        use LLRawSymb::*;
        match r {
            Var(n) => LLSymb::Var(n),
            One => LLSymb::One,
            Bot => LLSymb::Bot,
            Top => LLSymb::Top,
            Zero => LLSymb::Zero,
            Bang => LLSymb::Bang,
            Quest => LLSymb::Quest,
            Tensor => LLSymb::Tensor,
            Par => LLSymb::Par,
            With => LLSymb::With,
            Plus => LLSymb::Plus,
            Lollipop => LLSymb::Par,
            Dual => {
                unreachable!()
                // TODO: function is never called with Dual
                // unsafe { std::hint::unreachable_unchecked() }
            }
        }
    }

    fn from_raw_neg(r: LLRawSymb) -> LLSymb {
        use LLRawSymb::*;
        match r {
            Var(n) => LLSymb::DualVar(n),
            One => LLSymb::Bot,
            Bot => LLSymb::One,
            Top => LLSymb::Zero,
            Zero => LLSymb::Top,
            Bang => LLSymb::Quest,
            Quest => LLSymb::Bang,
            Tensor => LLSymb::Par,
            Par => LLSymb::Tensor,
            With => LLSymb::Plus,
            Plus => LLSymb::With,
            Lollipop => LLSymb::Par,
            Dual => {
                unreachable!()
                // TODO: function is never called with Dual
                // unsafe { std::hint::unreachable_unchecked() }
            }
        }
    }
}

// Multiplicative Exponential Linear Logic

impl Token for MELLRawSymb {
    fn arity(&self) -> Arity {
        LLRawSymb::from(self.clone()).arity()
    }
}

impl RawSymbol for MELLRawSymb {
    fn show(&self) -> RawSymbolShow {
        LLRawSymb::from(self.clone()).show()
    }
}

impl Token for MELLSymb {
    fn arity(&self) -> Arity {
        LLSymb::from(self.clone()).arity()
    }
}

impl Symbol for MELLSymb {
    fn show(&self) -> SymbolShow {
        LLSymb::from(self.clone()).show()
    }
}

pub struct MELL {}

impl SymbolSet for MELL {
    type Symb = MELLSymb;
    type RawSymb = MELLRawSymb;

    fn from_raw_pos(r: Self::RawSymb) -> Self::Symb {
        Self::Symb::try_from(LL::from_raw_pos(LLRawSymb::from(r))).unwrap()
        // TODO: use unwrap_unchecked since LL::from_raw_pos is essentially the identity function
        // unsafe { MELLSymb::try_from(LL::from_raw_pos(LLRawSymb::from(r))).unwrap_unchecked() }
    }

    fn from_raw_neg(r: Self::RawSymb) -> Self::Symb {
        Self::Symb::try_from(LL::from_raw_neg(LLRawSymb::from(r))).unwrap()
        // TODO: use unwrap_unchecked since LL::from_raw_neg just makes changes the polarity and doesn't change the symbol class
        // unsafe { MELLSymb::try_from(LL::from_raw_neg(LLRawSymb::from(r))).unwrap_unchecked() }
    }
}

// // Multiplicative Linear Logic

impl Token for MLLRawSymb {
    fn arity(&self) -> Arity {
        LLRawSymb::from(self.clone()).arity()
    }
}

impl RawSymbol for MLLRawSymb {
    fn show(&self) -> RawSymbolShow {
        LLRawSymb::from(self.clone()).show()
    }
}

impl Token for MLLSymb {
    fn arity(&self) -> Arity {
        LLSymb::from(self.clone()).arity()
    }
}

impl Symbol for MLLSymb {
    fn show(&self) -> SymbolShow {
        LLSymb::from(self.clone()).show()
    }
}

pub struct MLL {}

impl SymbolSet for MLL {
    type Symb = MLLSymb;
    type RawSymb = MLLRawSymb;

    fn from_raw_pos(r: Self::RawSymb) -> Self::Symb {
        Self::Symb::try_from(LL::from_raw_pos(LLRawSymb::from(r))).unwrap()
        // TODO: use unwrap_unchecked since LL::from_raw_pos is essentially the identity function
        // unsafe { MLLSymb::try_from(LL::from_raw_pos(LLRawSymb::from(r))).unwrap_unchecked() }
    }

    fn from_raw_neg(r: Self::RawSymb) -> Self::Symb {
        Self::Symb::try_from(LL::from_raw_neg(LLRawSymb::from(r))).unwrap()
        // TODO: use unwrap_unchecked since LL::from_raw_neg just makes changes the polarity and doesn't change the symbol class
        // unsafe { MLLSymb::try_from(LL::from_raw_neg(LLRawSymb::from(r))).unwrap_unchecked() }
    }
}
