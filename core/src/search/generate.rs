// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Random provable sequents for testing the engines: a cut-free proof is
//! built bottom-up by choosing rules at random, and only its conclusion is
//! kept, as text for the parser. The engine under test must prove it. A
//! mutant of such a sequent, with one literal's atom swapped, is usually
//! unprovable and tests the negative side.

use std::fmt::{Display, Formatter, Result as FmtResult};

/// A small deterministic pseudo-random generator (SplitMix64), so that a
/// test's sample is the same on every run.
#[derive(Clone, Debug)]
pub(crate) struct Rng(u64);

impl Rng {
    /// Starts the sequence at a seed.
    pub(crate) fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Returns the next 64 random bits.
    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns a number below `n`, which must be positive.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// Returns true with probability `1 / n`.
    pub(crate) fn one_in(&mut self, n: usize) -> bool {
        self.below(n) == 0
    }
}

/// Which rules a generated proof may use, beyond the axiom, `⊗` and `⅋`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rules {
    /// `1` and `⊥`.
    pub(crate) units: bool,
    /// `&`, `⊕`, `⊤` and `0`.
    pub(crate) additives: bool,
    /// Mix.
    pub(crate) mix: bool,
}

impl Rules {
    /// Every combination of the three switches.
    pub(crate) const ALL: [Rules; 8] = {
        let mut all = [Rules {
            units: false,
            additives: false,
            mix: false,
        }; 8];
        let mut i = 0;
        while i < 8 {
            all[i] = Rules {
                units: i & 1 != 0,
                additives: i & 2 != 0,
                mix: i & 4 != 0,
            };
            i += 1;
        }
        all
    };
}

/// A formula as a tree, printed in the parser's syntax with every binary
/// connective parenthesised. Atoms are numbered and print as letters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Tree {
    /// An atom.
    Var(u8),
    /// A negated atom.
    Dual(u8),
    /// `1`
    One,
    /// `⊥`
    Bot,
    /// `⊤`
    Top,
    /// `0`
    Zero,
    /// `A ⊗ B`
    Tensor(Box<Tree>, Box<Tree>),
    /// `A ⅋ B`
    Par(Box<Tree>, Box<Tree>),
    /// `A & B`
    With(Box<Tree>, Box<Tree>),
    /// `A ⊕ B`
    Plus(Box<Tree>, Box<Tree>),
}

impl Tree {
    /// Returns every literal of the tree, for a mutation to pick from.
    fn literals(&mut self) -> Vec<&mut u8> {
        use Tree::*;
        match self {
            Var(a) | Dual(a) => vec![a],
            One | Bot | Top | Zero => vec![],
            Tensor(l, r) | Par(l, r) | With(l, r) | Plus(l, r) => {
                let mut all = l.literals();
                all.extend(r.literals());
                all
            }
        }
    }
}

impl Display for Tree {
    /// Writes the formula for the parser: `((a * ~b) par top)`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use Tree::*;
        let binary =
            |f: &mut Formatter<'_>, l: &Tree, op: &str, r: &Tree| write!(f, "({l} {op} {r})");
        match self {
            Var(a) => write!(f, "{}", (b'a' + a) as char),
            Dual(a) => write!(f, "~{}", (b'a' + a) as char),
            One => f.write_str("1"),
            Bot => f.write_str("bot"),
            Top => f.write_str("top"),
            Zero => f.write_str("0"),
            Tensor(l, r) => binary(f, l, "*", r),
            Par(l, r) => binary(f, l, "par", r),
            With(l, r) => binary(f, l, "&", r),
            Plus(l, r) => binary(f, l, "+", r),
        }
    }
}

/// Writes a sequent for the parser: `|- A, B, C`.
pub(crate) fn sequent(formulas: &[Tree]) -> String {
    let mut text = String::from("|-");
    for (i, formula) in formulas.iter().enumerate() {
        text.push_str(if i == 0 { " " } else { ", " });
        text.push_str(&formula.to_string());
    }
    text
}

/// Builds a random provable sequent: a cut-free proof of about `budget`
/// rule applications over `atoms` atom names, using the rules allowed, and
/// returns its conclusion.
pub(crate) fn provable(rng: &mut Rng, rules: Rules, atoms: u8, budget: usize) -> Vec<Tree> {
    let mut generator = Generator { rng, rules, atoms };
    generator.proof(budget)
}

/// Mutates a sequent by giving one literal another atom name, and returns
/// whether there was a literal to mutate. The result is a sequent that is
/// usually unprovable but passes many of the count checks.
pub(crate) fn mutate(rng: &mut Rng, formulas: &mut [Tree], atoms: u8) -> bool {
    let mut literals: Vec<&mut u8> = formulas.iter_mut().flat_map(Tree::literals).collect();
    if literals.is_empty() || atoms < 2 {
        return false;
    }
    let pick = rng.below(literals.len());
    let shift = 1 + rng.below(atoms as usize - 1) as u8;
    let atom = &mut *literals[pick];
    *atom = (*atom + shift) % atoms;
    true
}

/// The state of one generation.
struct Generator<'a> {
    /// The random source.
    rng: &'a mut Rng,
    /// The rules allowed.
    rules: Rules,
    /// How many atom names to draw from.
    atoms: u8,
}

impl Generator<'_> {
    /// Builds the conclusion of a random proof of about `budget` rules.
    fn proof(&mut self, budget: usize) -> Vec<Tree> {
        if budget <= 1 {
            return self.leaf();
        }
        // Weighted choice among the rules the switches allow.
        let mut choices: Vec<u8> = vec![b'a', b'a', b't', b't', b'p', b'p'];
        if self.rules.units {
            choices.extend(*b"1b");
        }
        if self.rules.additives {
            choices.extend(*b"&++T");
        }
        if self.rules.mix {
            choices.push(b'm');
        }
        match choices[self.rng.below(choices.len())] {
            b'a' => self.leaf(),
            b't' => {
                // ⊢ Γ, A and ⊢ Δ, B give ⊢ Γ, Δ, A ⊗ B.
                let left_budget = 1 + self.rng.below(budget - 1);
                let mut left = self.proof(left_budget);
                let mut right = self.proof(budget - left_budget);
                let a = left.swap_remove(self.rng.below(left.len()));
                let b = right.swap_remove(self.rng.below(right.len()));
                left.append(&mut right);
                left.push(Tree::Tensor(Box::new(a), Box::new(b)));
                left
            }
            b'p' => {
                // ⊢ Γ, A, B gives ⊢ Γ, A ⅋ B; with one formula only, `⊥`
                // serves as the other when units are allowed.
                let mut premise = self.proof(budget - 1);
                if premise.len() < 2 {
                    if !self.rules.units {
                        return premise;
                    }
                    premise.push(Tree::Bot);
                }
                let a = premise.swap_remove(self.rng.below(premise.len()));
                let b = premise.swap_remove(self.rng.below(premise.len()));
                premise.push(Tree::Par(Box::new(a), Box::new(b)));
                premise
            }
            b'1' => vec![Tree::One],
            b'b' => {
                let mut premise = self.proof(budget - 1);
                premise.push(Tree::Bot);
                premise
            }
            b'&' => {
                // ⊢ Γ, A and ⊢ Γ, B give ⊢ Γ, A & B: B is a twin of A that
                // the same context proves.
                let mut premise = self.proof(budget - 1);
                let a = premise.swap_remove(self.rng.below(premise.len()));
                let b = self.twin(&a, 2);
                premise.push(Tree::With(Box::new(a), Box::new(b)));
                premise
            }
            b'+' => {
                let mut premise = self.proof(budget - 1);
                let a = premise.swap_remove(self.rng.below(premise.len()));
                let junk = self.junk(3);
                premise.push(if self.rng.one_in(2) {
                    Tree::Plus(Box::new(a), Box::new(junk))
                } else {
                    Tree::Plus(Box::new(junk), Box::new(a))
                });
                premise
            }
            b'T' => {
                // ⊢ ⊤, Γ for any Γ.
                let mut sequent = vec![Tree::Top];
                for _ in 0..self.rng.below(3) {
                    sequent.push(self.junk(3));
                }
                sequent
            }
            b'm' => {
                let left_budget = 1 + self.rng.below(budget - 1);
                let mut left = self.proof(left_budget);
                left.append(&mut self.proof(budget - left_budget));
                left
            }
            _ => unreachable!(),
        }
    }

    /// A proof with no premises: an axiom, or `1` when units are allowed.
    fn leaf(&mut self) -> Vec<Tree> {
        if self.rules.units && self.rng.one_in(4) {
            return vec![Tree::One];
        }
        let a = self.rng.below(self.atoms as usize) as u8;
        vec![Tree::Var(a), Tree::Dual(a)]
    }

    /// A formula that every context proving `a` proves as well, for the
    /// other side of a `&`: `a` itself, `⊤`, `a` under a `⊕` with anything,
    /// or, with units, `a ⅋ ⊥` or `a ⊗ 1`, nested at most `depth` deep.
    fn twin(&mut self, a: &Tree, depth: usize) -> Tree {
        let mut choices: Vec<u8> = vec![b'a', b'T', b'+', b'+'];
        if self.rules.units {
            choices.extend(*b"pt");
        }
        if depth > 0 {
            choices.push(b'&');
        }
        let inner = |this: &mut Self| {
            if depth > 0 {
                this.twin(a, depth - 1)
            } else {
                a.clone()
            }
        };
        match choices[self.rng.below(choices.len())] {
            b'a' => a.clone(),
            b'T' => Tree::Top,
            b'+' => {
                let (x, junk) = (inner(self), self.junk(2));
                if self.rng.one_in(2) {
                    Tree::Plus(Box::new(x), Box::new(junk))
                } else {
                    Tree::Plus(Box::new(junk), Box::new(x))
                }
            }
            b'p' => Tree::Par(Box::new(inner(self)), Box::new(Tree::Bot)),
            b't' => Tree::Tensor(Box::new(Tree::One), Box::new(inner(self))),
            b'&' => {
                let (x, y) = (inner(self), inner(self));
                Tree::With(Box::new(x), Box::new(y))
            }
            _ => unreachable!(),
        }
    }

    /// An arbitrary formula of the allowed connectives, provable or not, of
    /// at most `size` connectives.
    fn junk(&mut self, size: usize) -> Tree {
        let mut choices: Vec<u8> = vec![b'v', b'd'];
        if self.rules.units {
            choices.extend(*b"1b");
        }
        if self.rules.additives {
            choices.extend(*b"T0");
        }
        if size > 0 {
            choices.extend(*b"tp");
            if self.rules.additives {
                choices.extend(*b"&+");
            }
        }
        let a = self.rng.below(self.atoms as usize) as u8;
        let sub = |this: &mut Self| Box::new(this.junk(size - 1));
        match choices[self.rng.below(choices.len())] {
            b'v' => Tree::Var(a),
            b'd' => Tree::Dual(a),
            b'1' => Tree::One,
            b'b' => Tree::Bot,
            b'T' => Tree::Top,
            b'0' => Tree::Zero,
            b't' => Tree::Tensor(sub(self), sub(self)),
            b'p' => Tree::Par(sub(self), sub(self)),
            b'&' => Tree::With(sub(self), sub(self)),
            b'+' => Tree::Plus(sub(self), sub(self)),
            _ => unreachable!(),
        }
    }
}
