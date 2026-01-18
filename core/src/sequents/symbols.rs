// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum LLRawSymb {
    Var(usize),
    One,
    Bot,
    Top,
    Zero,
    Dual,
    Bang,
    Quest,
    Tensor,
    Par,
    With,
    Plus,
    Lollipop,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum LLSymb {
    Var(usize),
    DualVar(usize),
    One,
    Bot,
    Top,
    Zero,
    Bang,
    Quest,
    Tensor,
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

struct LL {}

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
            Dual => unreachable!(),
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
            Dual => unreachable!(),
        }
    }
}

// Multiplicative Exponential Linear Logic

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MELLRawSymb {
    Var(usize),
    One,
    Bot,
    Dual,
    Bang,
    Quest,
    Tensor,
    Par,
    Lollipop,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MELLSymb {
    Var(usize),
    DualVar(usize),
    One,
    Bot,
    Bang,
    Quest,
    Tensor,
    Par,
}

impl Token for MELLRawSymb {
    fn arity(&self) -> Arity {
        use MELLRawSymb::*;
        match self {
            Var(_) | One | Bot => Arity::Nullary,
            Dual => Arity::Dual,
            Bang | Quest => Arity::Unary,
            Tensor | Par => Arity::Binary,
            Lollipop => Arity::Lollipop,
        }
    }
}

impl RawSymbol for MELLRawSymb {
    fn show(&self) -> RawSymbolShow {
        use MELLRawSymb::*;
        use RawSymbolShow::*;
        match self {
            Var(n) => Variable(*n),
            One => Char(SHOW_ONE),
            Bot => Char(SHOW_BOT),
            Dual => Variant(SHOW_DUAL_PREFIX, SHOW_DUAL_POSTFIX),
            Bang => Char(SHOW_BANG),
            Quest => Char(SHOW_QUEST),
            Tensor => Char(SHOW_TENSOR),
            Par => Char(SHOW_PAR),
            Lollipop => Char(SHOW_LOLLIPOP),
        }
    }
}

impl Token for MELLSymb {
    fn arity(&self) -> Arity {
        use MELLSymb::*;
        match self {
            Var(_) | DualVar(_) | One | Bot => Arity::Nullary,
            Bang | Quest => Arity::Unary,
            Tensor | Par => Arity::Binary,
        }
    }
}

impl Symbol for MELLSymb {
    fn show(&self) -> SymbolShow {
        use MELLSymb::*;
        use SymbolShow::*;
        match self {
            Var(n) => Variable(*n),
            DualVar(n) => DualVariable(*n),
            One => Char(SHOW_ONE),
            Bot => Char(SHOW_BOT),
            Bang => Char(SHOW_BANG),
            Quest => Char(SHOW_QUEST),
            Tensor => Char(SHOW_TENSOR),
            Par => Char(SHOW_PAR),
        }
    }
}

struct MELL {}

impl SymbolSet for MELL {
    type Symb = MELLSymb;
    type RawSymb = MELLRawSymb;

    fn from_raw_pos(r: MELLRawSymb) -> MELLSymb {
        use MELLRawSymb::*;
        match r {
            Var(n) => MELLSymb::Var(n),
            One => MELLSymb::One,
            Bot => MELLSymb::Bot,
            Bang => MELLSymb::Bang,
            Quest => MELLSymb::Quest,
            Tensor => MELLSymb::Tensor,
            Par => MELLSymb::Par,
            Lollipop => MELLSymb::Par,
            Dual => unreachable!(),
        }
    }

    fn from_raw_neg(r: MELLRawSymb) -> MELLSymb {
        use MELLRawSymb::*;
        match r {
            Var(n) => MELLSymb::DualVar(n),
            One => MELLSymb::Bot,
            Bot => MELLSymb::One,
            Bang => MELLSymb::Quest,
            Quest => MELLSymb::Bang,
            Tensor => MELLSymb::Par,
            Par => MELLSymb::Tensor,
            Lollipop => MELLSymb::Par,
            Dual => unreachable!(),
        }
    }
}

// convert between LL <-> MELL

impl From<MELLRawSymb> for LLRawSymb {
    fn from(s: MELLRawSymb) -> Self {
        use LLRawSymb as LL;
        use MELLRawSymb::*;
        match s {
            Var(n) => LL::Var(n),
            One => LL::One,
            Bot => LL::Bot,
            Dual => LL::Dual,
            Bang => LL::Bang,
            Quest => LL::Quest,
            Tensor => LL::Tensor,
            Par => LL::Par,
            Lollipop => LL::Lollipop,
        }
    }
}

impl From<MELLSymb> for LLSymb {
    fn from(s: MELLSymb) -> Self {
        use LLSymb as LL;
        use MELLSymb::*;
        match s {
            Var(n) => LL::Var(n),
            DualVar(n) => LL::DualVar(n),
            One => LL::One,
            Bot => LL::Bot,
            Bang => LL::Bang,
            Quest => LL::Quest,
            Tensor => LL::Tensor,
            Par => LL::Par,
        }
    }
}

impl TryFrom<LLRawSymb> for MELLRawSymb {
    type Error = crate::Error;

    fn try_from(s: LLRawSymb) -> Result<MELLRawSymb, Self::Error> {
        use LLRawSymb as LL;
        use MELLRawSymb::*;
        match s {
            LL::Var(n) => Ok(Var(n)),
            LL::One => Ok(One),
            LL::Bot => Ok(Bot),
            LL::Top => Err(Self::Error::LLToMELLConversion(SHOW_TOP.to_string())),
            LL::Zero => Err(Self::Error::LLToMELLConversion(SHOW_ZERO.to_string())),
            LL::Dual => Ok(Dual),
            LL::Bang => Ok(Bang),
            LL::Quest => Ok(Quest),
            LL::Tensor => Ok(Tensor),
            LL::Par => Ok(Par),
            LL::With => Err(Self::Error::LLToMELLConversion(SHOW_WITH.to_string())),
            LL::Plus => Err(Self::Error::LLToMELLConversion(SHOW_PLUS.to_string())),
            LL::Lollipop => Ok(Lollipop),
        }
    }
}

impl TryFrom<LLSymb> for MELLSymb {
    type Error = crate::Error;

    fn try_from(s: LLSymb) -> Result<MELLSymb, Self::Error> {
        use LLSymb as LL;
        use MELLSymb::*;
        match s {
            LL::Var(n) => Ok(Var(n)),
            LL::DualVar(n) => Ok(DualVar(n)),
            LL::One => Ok(One),
            LL::Bot => Ok(Bot),
            LL::Top => Err(Self::Error::LLToMELLConversion(SHOW_TOP.to_string())),
            LL::Zero => Err(Self::Error::LLToMELLConversion(SHOW_ZERO.to_string())),
            LL::Bang => Ok(Bang),
            LL::Quest => Ok(Quest),
            LL::Tensor => Ok(Tensor),
            LL::Par => Ok(Par),
            LL::With => Err(Self::Error::LLToMELLConversion(SHOW_WITH.to_string())),
            LL::Plus => Err(Self::Error::LLToMELLConversion(SHOW_PLUS.to_string())),
        }
    }
}

// Multiplicative Linear Logic

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MLLRawSymb {
    Var(usize),
    One,
    Bot,
    Dual,
    Tensor,
    Par,
    Lollipop,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MLLSymb {
    Var(usize),
    DualVar(usize),
    One,
    Bot,
    Tensor,
    Par,
}

impl Token for MLLRawSymb {
    fn arity(&self) -> Arity {
        use MLLRawSymb::*;
        match self {
            Var(_) | One | Bot => Arity::Nullary,
            Dual => Arity::Dual,
            Tensor | Par => Arity::Binary,
            Lollipop => Arity::Lollipop,
        }
    }
}

impl RawSymbol for MLLRawSymb {
    fn show(&self) -> RawSymbolShow {
        use MLLRawSymb::*;
        use RawSymbolShow::*;
        match self {
            Var(n) => Variable(*n),
            One => Char(SHOW_ONE),
            Bot => Char(SHOW_BOT),
            Dual => Variant(SHOW_DUAL_PREFIX, SHOW_DUAL_POSTFIX),
            Tensor => Char(SHOW_TENSOR),
            Par => Char(SHOW_PAR),
            Lollipop => Char(SHOW_LOLLIPOP),
        }
    }
}

impl Token for MLLSymb {
    fn arity(&self) -> Arity {
        use MLLSymb::*;
        match self {
            Var(_) | DualVar(_) | One | Bot => Arity::Nullary,
            Tensor | Par => Arity::Binary,
        }
    }
}

impl Symbol for MLLSymb {
    fn show(&self) -> SymbolShow {
        use MLLSymb::*;
        use SymbolShow::*;
        match self {
            Var(n) => Variable(*n),
            DualVar(n) => DualVariable(*n),
            One => Char(SHOW_ONE),
            Bot => Char(SHOW_BOT),
            Tensor => Char(SHOW_TENSOR),
            Par => Char(SHOW_PAR),
        }
    }
}

struct MLL {}

impl SymbolSet for MLL {
    type Symb = MLLSymb;
    type RawSymb = MLLRawSymb;

    fn from_raw_pos(r: MLLRawSymb) -> MLLSymb {
        use MLLRawSymb::*;
        match r {
            Var(n) => MLLSymb::Var(n),
            One => MLLSymb::One,
            Bot => MLLSymb::Bot,
            Tensor => MLLSymb::Tensor,
            Par => MLLSymb::Par,
            Lollipop => MLLSymb::Par,
            Dual => unreachable!(),
        }
    }

    fn from_raw_neg(r: MLLRawSymb) -> MLLSymb {
        use MLLRawSymb::*;
        match r {
            Var(n) => MLLSymb::DualVar(n),
            One => MLLSymb::Bot,
            Bot => MLLSymb::One,
            Tensor => MLLSymb::Par,
            Par => MLLSymb::Tensor,
            Lollipop => MLLSymb::Par,
            Dual => unreachable!(),
        }
    }
}

// convert between LL <-> MLL

impl From<MLLRawSymb> for LLRawSymb {
    fn from(s: MLLRawSymb) -> Self {
        use LLRawSymb as LL;
        use MLLRawSymb::*;
        match s {
            Var(n) => LL::Var(n),
            One => LL::One,
            Bot => LL::Bot,
            Dual => LL::Dual,
            Tensor => LL::Tensor,
            Par => LL::Par,
            Lollipop => LL::Lollipop,
        }
    }
}

impl From<MLLSymb> for LLSymb {
    fn from(s: MLLSymb) -> Self {
        use LLSymb as LL;
        use MLLSymb::*;
        match s {
            Var(n) => LL::Var(n),
            DualVar(n) => LL::DualVar(n),
            One => LL::One,
            Bot => LL::Bot,
            Tensor => LL::Tensor,
            Par => LL::Par,
        }
    }
}

impl TryFrom<LLRawSymb> for MLLRawSymb {
    type Error = crate::Error;

    fn try_from(s: LLRawSymb) -> Result<MLLRawSymb, Self::Error> {
        use LLRawSymb as LL;
        use MLLRawSymb::*;
        match s {
            LL::Var(n) => Ok(Var(n)),
            LL::One => Ok(One),
            LL::Bot => Ok(Bot),
            LL::Top => Err(Self::Error::LLToMLLConversion(SHOW_TOP.to_string())),
            LL::Zero => Err(Self::Error::LLToMLLConversion(SHOW_ZERO.to_string())),
            LL::Dual => Ok(Dual),
            LL::Bang => Err(Self::Error::LLToMLLConversion(SHOW_BANG.to_string())),
            LL::Quest => Err(Self::Error::LLToMLLConversion(SHOW_QUEST.to_string())),
            LL::Tensor => Ok(Tensor),
            LL::Par => Ok(Par),
            LL::With => Err(Self::Error::LLToMLLConversion(SHOW_WITH.to_string())),
            LL::Plus => Err(Self::Error::LLToMLLConversion(SHOW_PLUS.to_string())),
            LL::Lollipop => Ok(Lollipop),
        }
    }
}

impl TryFrom<LLSymb> for MLLSymb {
    type Error = crate::Error;

    fn try_from(s: LLSymb) -> Result<MLLSymb, Self::Error> {
        use LLSymb as LL;
        use MLLSymb::*;
        match s {
            LL::Var(n) => Ok(Var(n)),
            LL::DualVar(n) => Ok(DualVar(n)),
            LL::One => Ok(One),
            LL::Bot => Ok(Bot),
            LL::Top => Err(Self::Error::LLToMLLConversion(SHOW_TOP.to_string())),
            LL::Zero => Err(Self::Error::LLToMLLConversion(SHOW_ZERO.to_string())),
            LL::Bang => Err(Self::Error::LLToMLLConversion(SHOW_BANG.to_string())),
            LL::Quest => Err(Self::Error::LLToMLLConversion(SHOW_QUEST.to_string())),
            LL::Tensor => Ok(Tensor),
            LL::Par => Ok(Par),
            LL::With => Err(Self::Error::LLToMLLConversion(SHOW_WITH.to_string())),
            LL::Plus => Err(Self::Error::LLToMLLConversion(SHOW_PLUS.to_string())),
        }
    }
}

// convert between MELL <-> MLL

impl From<MLLRawSymb> for MELLRawSymb {
    fn from(s: MLLRawSymb) -> Self {
        use MELLRawSymb as LL;
        use MLLRawSymb::*;
        match s {
            Var(n) => LL::Var(n),
            One => LL::One,
            Bot => LL::Bot,
            Dual => LL::Dual,
            Tensor => LL::Tensor,
            Par => LL::Par,
            Lollipop => LL::Lollipop,
        }
    }
}

impl From<MLLSymb> for MELLSymb {
    fn from(s: MLLSymb) -> Self {
        use MELLSymb as LL;
        use MLLSymb::*;
        match s {
            Var(n) => LL::Var(n),
            DualVar(n) => LL::DualVar(n),
            One => LL::One,
            Bot => LL::Bot,
            Tensor => LL::Tensor,
            Par => LL::Par,
        }
    }
}

impl TryFrom<MELLRawSymb> for MLLRawSymb {
    type Error = crate::Error;

    fn try_from(s: MELLRawSymb) -> Result<MLLRawSymb, Self::Error> {
        use MELLRawSymb as LL;
        use MLLRawSymb::*;
        match s {
            LL::Var(n) => Ok(Var(n)),
            LL::One => Ok(One),
            LL::Bot => Ok(Bot),
            LL::Dual => Ok(Dual),
            LL::Bang => Err(Self::Error::MELLToMLLConversion(SHOW_BANG.to_string())),
            LL::Quest => Err(Self::Error::MELLToMLLConversion(SHOW_QUEST.to_string())),
            LL::Tensor => Ok(Tensor),
            LL::Par => Ok(Par),
            LL::Lollipop => Ok(Lollipop),
        }
    }
}

impl TryFrom<MELLSymb> for MLLSymb {
    type Error = crate::Error;

    fn try_from(s: MELLSymb) -> Result<MLLSymb, Self::Error> {
        use MELLSymb as LL;
        use MLLSymb::*;
        match s {
            LL::Var(n) => Ok(Var(n)),
            LL::DualVar(n) => Ok(DualVar(n)),
            LL::One => Ok(One),
            LL::Bot => Ok(Bot),
            LL::Bang => Err(Self::Error::MELLToMLLConversion(SHOW_BANG.to_string())),
            LL::Quest => Err(Self::Error::MELLToMLLConversion(SHOW_QUEST.to_string())),
            LL::Tensor => Ok(Tensor),
            LL::Par => Ok(Par),
        }
    }
}
