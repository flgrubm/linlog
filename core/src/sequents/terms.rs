// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::sequents::symbols::{Arity, SymbolSet, Token};

#[allow(type_alias_bounds)] // trait bound is not checked in current compiler version
pub type PreTerm<'a, T: Token> = &'a [T];

#[derive(Copy, Clone, Debug)]
pub struct Term<'a, T: Token> {
    pub(crate) tm: PreTerm<'a, T>,
}

impl<'a, T: Token> TryFrom<PreTerm<'a, T>> for Term<'a, T> {
    type Error = crate::Error;

    fn try_from(p: PreTerm<'a, T>) -> Result<Self, Self::Error> {
        use Arity::*;
        let mut leaves_left = 1usize;
        for token in p.iter() {
            match token.arity() {
                Nullary => {
                    if leaves_left > 0 {
                        leaves_left -= 1
                    } else {
                        return Err(crate::Error::MalformedTerm("Leaf".to_string()));
                    }
                }
                Unary | Dual => {}
                Binary | Lollipop => leaves_left += 1,
            }
        }

        if leaves_left > 0 {
            Err(crate::Error::MalformedTerm("EOF".to_string()))
        } else {
            Ok(Term { tm: p })
        }
    }
}

pub enum SubTerm<'a, T: Token> {
    Nothing,
    One(Term<'a, T>),
    Two(Term<'a, T>, Term<'a, T>),
}

impl<'a, T: Token> Term<'a, T> {
    fn subterms(&self) -> SubTerm<'a, T> {
        use Arity::*;
        match self.tm[0].arity() {
            Nullary => SubTerm::Nothing,
            Unary | Dual => SubTerm::One(Term {
                tm: &self.tm[1..self.tm.len()],
            }),
            Binary | Lollipop => {
                let mut leaves_left = 1usize;
                for (n, token) in self.tm[1..self.tm.len()].iter().enumerate() {
                    match token.arity() {
                        Nullary => match leaves_left {
                            0 => unreachable!(),
                            1 => {
                                let split_index = n + 1;
                                return SubTerm::Two(
                                    Term {
                                        tm: &self.tm[1..split_index],
                                    },
                                    Term {
                                        tm: &self.tm[split_index..self.tm.len()],
                                    },
                                );
                            }
                            _ => leaves_left -= 1,
                        },
                        Unary | Dual => {}
                        Binary | Lollipop => leaves_left += 1,
                    }
                }
                unreachable!()
            }
        }
    }
}

#[allow(type_alias_bounds)] // trait bound is not checked in current compiler version
pub type RawTerm<'a, S: SymbolSet> = Term<'a, <S as SymbolSet>::RawSymb>;

#[allow(type_alias_bounds)] // trait bound is not checked in current compiler version
pub type ReducedTerm<'a, S: SymbolSet> = Term<'a, <S as SymbolSet>::Symb>;

fn reduce_append<'a, S: SymbolSet>(
    term: Term<'a, S::RawSymb>,
    polarity: bool,
    mut symbols: Vec<S::Symb>,
) -> Vec<S::Symb> {
    let mut stack = vec![(term, polarity)];

    while let Some((current_subterm, polarity)) = stack.pop() {
        let head = current_subterm.tm[0];
        match current_subterm.subterms() {
            SubTerm::Nothing => symbols.push(S::from_raw(head, polarity)),
            SubTerm::One(st) => match head.arity() {
                Arity::Unary => {
                    symbols.push(S::from_raw(head, polarity));
                    stack.push((st, polarity));
                }
                Arity::Dual => stack.push((st, !polarity)),
                _ => unreachable!(),
            },
            SubTerm::Two(st1, st2) => {
                symbols.push(S::from_raw(head, polarity));

                // second one needs to be deeper in the stack
                stack.push((st2, polarity));

                // first one needs polarity inverted if head is lollipop
                match head.arity() {
                    Arity::Binary => {
                        stack.push((st1, polarity));
                    }
                    Arity::Lollipop => {
                        stack.push((st1, !polarity));
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    symbols
}

// // Handling local offset:
// // Dual decreases offset by one
// // Var with negative polarity increases offset by one
// // usize will underflow since Dual is likely read without/before any negative polarity Var
// // Solution: track offset by (pos, neg), where
// //     - Dual decreases pos until it reaches zero, then increases neg
// //     - negative polarity Var decreases neg until it reaches zero, then increases pos
// type OffsetCounter = (usize, usize);

// let mut local_offset: OffsetCounter = (0, 0);

// enum OffsetChange {
//     IncrementOne,
//     DecrementOne,
// }

// fn apply_offset((pos, neg): OffsetCounter, previous_offset: usize) -> usize {
//     debug_assert!(pos == 0 || neg == 0);
//     if neg == 0 {
//         previous_offset + pos
//     } else {
//         // function is only ever called in the very end with the so-far total accumulated offset
//         // it is clear that the total offset of the next term cannot be negative
//         debug_assert!(previous_offset >= neg);
//         previous_offset - neg
//     }
// }

// fn change_offset((pos, neg): OffsetCounter, change: OffsetChange) -> OffsetCounter {
//     match change {
//         OffsetChange::IncrementOne => {
//             if neg == 0 {
//                 (pos + 1, 0)
//             } else {
//                 (0, neg - 1)
//             }
//         }
//         OffsetChange::DecrementOne => {
//             if pos == 0 {
//                 (0, neg + 1)
//             } else {
//                 (pos - 1, 0)
//             }
//         }
//     }
// }

pub(super) fn fold_reduce_terms<'a, S: SymbolSet>(
    (mut symbols, mut offsets): (Vec<S::Symb>, Vec<usize>),
    (term, original_polarity): (RawTerm<'a, S>, bool),
) -> (Vec<S::Symb>, Vec<usize>) {
    offsets.push(symbols.len());

    let mut stack = vec![(term, original_polarity)];

    while let Some((current_subterm, polarity)) = stack.pop() {
        debug_assert!(!current_subterm.tm.is_empty());

        let head = current_subterm.tm[0];
        match current_subterm.subterms() {
            SubTerm::Nothing => symbols.push(S::from_raw(head, polarity)),
            SubTerm::One(st) => match head.arity() {
                Arity::Unary => {
                    symbols.push(S::from_raw(head, polarity));
                    stack.push((st, polarity));
                }
                Arity::Dual => stack.push((st, !polarity)),
                _ => unreachable!(),
            },
            SubTerm::Two(st1, st2) => {
                symbols.push(S::from_raw(head, polarity));

                // second one needs to be deeper in the stack
                stack.push((st2, polarity));

                // first one needs polarity inverted if head is lollipop
                match head.arity() {
                    Arity::Binary => {
                        stack.push((st1, polarity));
                    }
                    Arity::Lollipop => {
                        stack.push((st1, !polarity));
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    (symbols, offsets)
}

pub(super) fn reduce<'a, S: SymbolSet>(
    t1: Term<'a, S::RawSymb>,
    t2: Term<'a, S::RawSymb>,
) -> Box<[S::Symb]> {
    unimplemented!()
    // let v = Vec::<S::Symb>::new();
    // let v = reduce_append::<S>(t1, false, v);
    // reduce_append::<S>(t2, true, v).into_boxed_slice()
}
