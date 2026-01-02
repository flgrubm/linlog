// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use std::collections::HashMap;

/// Symbols for the `Term`s in polish notation
#[derive(Debug)]
pub enum Symbol {
    Var(usize),
    Zero,
    One,
    Bot,
    Top,
    Dual,
    Tensor,
    Plus,
    With,
    Par,
    Bang,
    Quest,
    Lollipop,
}

pub struct Term<'a> {
    term: Box<[Symbol]>,
    labels: Box<[&'a str]>,
}

impl<'a> Term<'a> {
    pub fn parse(t: &'a str) -> Result<Self, ()> {
        let mut label_map: HashMap<&'a str, usize> = HashMap::with_capacity(t.len());
        let mut term: Vec<Symbol> = Vec::with_capacity(t.len());
        let mut labels: Vec<&'a str> = Vec::with_capacity(t.len());

        todo!();
        Ok(Self {
            term: term.into_boxed_slice(),
            labels: labels.into_boxed_slice(),
        })
    }

    pub fn try_new(term: Box<[Symbol]>, labels: Box<[&'a str]>) -> Result<Self, ()> {
        if Self::verify(&term) {
            Ok(Self { term, labels })
        } else {
            Err(())
        }
    }

    pub fn verify(t: &[Symbol]) -> bool {
        use Symbol::*;
        let mut leaves_left = 1usize;
        for sym in t.iter() {
            match sym {
                Var(_) | Zero | One | Bot | Top => {
                    if leaves_left == 0 {
                        return false;
                    } else {
                        leaves_left -= 1;
                    }
                }
                Tensor | Plus | With | Par | Lollipop => {
                    leaves_left += 1;
                }
                Dual | Bang | Quest => {}
            }
        }
        leaves_left == 0
    }
}
