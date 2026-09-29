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
    /// `!` and `?`: promotion, dereliction, contraction and weakening.
    pub(crate) exponentials: bool,
}

impl Rules {
    /// Every combination of the four switches.
    pub(crate) const ALL: [Rules; 16] = {
        let mut all = [Rules {
            units: false,
            additives: false,
            mix: false,
            exponentials: false,
        }; 16];
        let mut i = 0;
        while i < 16 {
            all[i] = Rules {
                units: i & 1 != 0,
                additives: i & 2 != 0,
                mix: i & 4 != 0,
                exponentials: i & 8 != 0,
            };
            i += 1;
        }
        all
    };
}

/// A generated provable sequent: its formulas, and the derelictions of the
/// proof it was read off, which bound the copies a focused dyadic proof of
/// it needs on any branch. (The most derelictions on one branch would
/// not: a focused proof copies at a stable sequent whatever the positive
/// phase above needs, on every branch of that phase, where the standard
/// proof derelicts on each branch separately.)
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Provable {
    /// The formulas.
    pub(crate) formulas: Vec<Tree>,
    /// The derelictions of the proof, all branches together.
    pub(crate) copies: u32,
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
    /// `!A`
    Bang(Box<Tree>),
    /// `?A`
    Quest(Box<Tree>),
    /// `A ⊸ B`, in intuitionistic sequents only.
    Lolli(Box<Tree>, Box<Tree>),
}

impl Tree {
    /// Returns every literal of the tree, for a mutation to pick from.
    fn literals(&mut self) -> Vec<&mut u8> {
        use Tree::*;
        match self {
            Var(a) | Dual(a) => vec![a],
            One | Bot | Top | Zero => vec![],
            Tensor(l, r) | Par(l, r) | With(l, r) | Plus(l, r) | Lolli(l, r) => {
                let mut all = l.literals();
                all.extend(r.literals());
                all
            }
            Bang(a) | Quest(a) => a.literals(),
        }
    }

    /// Whether the formula is a `!` formula.
    fn is_bang(&self) -> bool {
        matches!(self, Tree::Bang(_))
    }

    /// Whether the formula is a `?` formula.
    fn is_quest(&self) -> bool {
        matches!(self, Tree::Quest(_))
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
            Bang(a) => write!(f, "!{a}"),
            Quest(a) => write!(f, "?{a}"),
            Lolli(l, r) => binary(f, l, "-o", r),
        }
    }
}

/// Writes an intuitionistic sequent for the parser: `A, B |- C`.
pub(crate) fn two_sided(hypotheses: &[Tree], goal: &Tree) -> String {
    let mut text = String::new();
    for (i, formula) in hypotheses.iter().enumerate() {
        if i > 0 {
            text.push_str(", ");
        }
        text.push_str(&formula.to_string());
    }
    if !hypotheses.is_empty() {
        text.push(' ');
    }
    text.push_str("|- ");
    text.push_str(&goal.to_string());
    text
}

/// Which rules a generated intuitionistic proof may use, beyond the axiom,
/// `⊗` and `⊸`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct IllRules {
    /// `1`.
    pub(crate) units: bool,
    /// `&`, `⊕` and `⊤`.
    pub(crate) additives: bool,
    /// `0`, which makes classical linear logic non-conservative over the
    /// intuitionistic one.
    pub(crate) zero: bool,
    /// `!`: promotion, dereliction, contraction and weakening.
    pub(crate) exponentials: bool,
}

impl IllRules {
    /// Every combination of the four switches, `zero` only with additives.
    pub(crate) const ALL: [IllRules; 12] = {
        let mut all = [IllRules {
            units: false,
            additives: false,
            zero: false,
            exponentials: false,
        }; 12];
        let (mut i, mut n) = (0, 0);
        while i < 16 {
            let zero = i & 4 != 0;
            let additives = i & 2 != 0;
            if !zero || additives {
                all[n] = IllRules {
                    units: i & 1 != 0,
                    additives,
                    zero,
                    exponentials: i & 8 != 0,
                };
                n += 1;
            }
            i += 1;
        }
        all
    };
}

/// A generated provable intuitionistic sequent: its hypotheses, its goal,
/// and the derelictions of the proof it was read off.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Ill {
    /// The hypotheses.
    pub(crate) hypotheses: Vec<Tree>,
    /// The goal.
    pub(crate) goal: Tree,
    /// The derelictions of the proof, all branches together.
    pub(crate) copies: u32,
}

/// Builds a random provable intuitionistic sequent: a cut-free two-sided
/// proof of about `budget` rule applications over `atoms` atom names, using
/// the rules allowed, and returns its conclusion with the derelictions of
/// the proof.
pub(crate) fn ill(rng: &mut Rng, rules: IllRules, atoms: u8, budget: usize) -> Ill {
    let mut generator = IllGenerator { rng, rules, atoms };
    let (hypotheses, goal, copies) = generator.proof(budget);
    Ill {
        hypotheses,
        goal,
        copies,
    }
}

/// The state of one intuitionistic generation.
struct IllGenerator<'a> {
    /// The random source.
    rng: &'a mut Rng,
    /// The rules allowed.
    rules: IllRules,
    /// How many atom names to draw from.
    atoms: u8,
}

impl IllGenerator<'_> {
    /// Builds the conclusion of a random two-sided proof of about `budget`
    /// rules: the hypotheses, the goal and the derelictions.
    fn proof(&mut self, budget: usize) -> (Vec<Tree>, Tree, u32) {
        use Tree::*;
        if budget <= 1 {
            return self.leaf();
        }
        let (mut hypotheses, mut goal, mut copies) = self.proof(budget - 1);
        // Weighted choice among the rules the switches allow and the
        // premise admits: the rules on a hypothesis need one, `⊗L` two.
        let mut choices: Vec<u8> = vec![b'a', b't', b't'];
        if !hypotheses.is_empty() {
            choices.extend(*b"llLL");
            if self.rules.additives {
                choices.extend(*b"wo");
            }
            if self.rules.exponentials {
                choices.push(b'd');
            }
        }
        if hypotheses.len() >= 2 {
            choices.push(b'T');
        }
        if self.rules.units {
            choices.push(b'1');
        }
        if self.rules.additives {
            choices.extend(*b"&+");
        }
        if self.rules.exponentials {
            choices.extend(*b"!cW");
        }
        let boxed = |t: Tree| Box::new(t);
        match choices[self.rng.below(choices.len())] {
            b'a' => return self.leaf(),
            b't' => {
                // Γ ⊢ A and Δ ⊢ B give Γ, Δ ⊢ A ⊗ B.
                let other = 1 + self.rng.below(budget - 1);
                let (delta, b, c) = self.proof(other);
                hypotheses.extend(delta);
                goal = Tensor(boxed(goal), boxed(b));
                copies += c;
            }
            b'l' => {
                // Γ, A ⊢ B gives Γ ⊢ A ⊸ B.
                let a = self.take_hypothesis(&mut hypotheses);
                goal = Lolli(boxed(a), boxed(goal));
            }
            b'L' => {
                // Γ ⊢ A and Δ, B ⊢ C give Γ, Δ, A ⊸ B ⊢ C.
                let other = 1 + self.rng.below(budget - 1);
                let (gamma, a, c) = self.proof(other);
                let b = self.take_hypothesis(&mut hypotheses);
                hypotheses.extend(gamma);
                hypotheses.push(Lolli(boxed(a), boxed(b)));
                copies += c;
            }
            b'T' => {
                // Γ, A, B ⊢ C gives Γ, A ⊗ B ⊢ C.
                let a = self.take_hypothesis(&mut hypotheses);
                let b = self.take_hypothesis(&mut hypotheses);
                hypotheses.push(Tensor(boxed(a), boxed(b)));
            }
            b'1' => hypotheses.push(One),
            b'&' => {
                // Γ ⊢ A and Γ ⊢ T give Γ ⊢ A & T, with T a twin of A.
                let twin = self.twin(&goal);
                goal = if self.rng.one_in(2) {
                    With(boxed(goal), boxed(twin))
                } else {
                    With(boxed(twin), boxed(goal))
                };
            }
            b'+' => {
                let junk = self.junk(2);
                goal = if self.rng.one_in(2) {
                    Plus(boxed(goal), boxed(junk))
                } else {
                    Plus(boxed(junk), boxed(goal))
                };
            }
            b'w' => {
                // Γ, A ⊢ C gives Γ, A & J ⊢ C.
                let a = self.take_hypothesis(&mut hypotheses);
                let junk = self.junk(2);
                hypotheses.push(if self.rng.one_in(2) {
                    With(boxed(a), boxed(junk))
                } else {
                    With(boxed(junk), boxed(a))
                });
            }
            b'o' => {
                // Γ, A ⊢ C and Γ, T ⊢ C give Γ, A ⊕ T ⊢ C, with T a twin of
                // A as a hypothesis: A itself, `0`, or A & anything.
                let a = self.take_hypothesis(&mut hypotheses);
                let twin = match self.rng.below(3) {
                    0 if self.rules.zero => Zero,
                    1 => With(boxed(a.clone()), boxed(self.junk(2))),
                    _ => a.clone(),
                };
                hypotheses.push(if self.rng.one_in(2) {
                    Plus(boxed(a), boxed(twin))
                } else {
                    Plus(boxed(twin), boxed(a))
                });
            }
            b'd' => {
                // Dereliction: Γ, A ⊢ C gives Γ, !A ⊢ C.
                let a = self.take_hypothesis(&mut hypotheses);
                hypotheses.push(Bang(boxed(a)));
                copies += 1;
            }
            b'!' => {
                // Promotion: !Γ ⊢ A gives !Γ ⊢ !A, once every hypothesis
                // is a `!` formula, which dereliction arranges.
                for h in &mut hypotheses {
                    if !h.is_bang() {
                        *h = Bang(boxed(std::mem::replace(h, One)));
                        copies += 1;
                    }
                }
                goal = Bang(boxed(goal));
            }
            b'c' => {
                // Contraction: Γ, !A, !A ⊢ C gives Γ, !A ⊢ C, on two equal
                // `!` hypotheses if there are any.
                let repeated = (0..hypotheses.len())
                    .find(|&j| hypotheses[j].is_bang() && hypotheses[..j].contains(&hypotheses[j]));
                if let Some(j) = repeated {
                    hypotheses.remove(j);
                }
            }
            b'W' => {
                // Weakening: Γ ⊢ C gives Γ, !J ⊢ C.
                let junk = self.junk(2);
                hypotheses.push(Bang(boxed(junk)));
            }
            _ => unreachable!(),
        }
        (hypotheses, goal, copies)
    }

    /// A proof with no premises: an axiom, `⊢ 1`, `Γ ⊢ ⊤` or `Γ, 0 ⊢ A` for
    /// arbitrary `Γ` and `A`, as the rules allow.
    fn leaf(&mut self) -> (Vec<Tree>, Tree, u32) {
        let mut choices: Vec<u8> = vec![b'a', b'a'];
        if self.rules.units {
            choices.push(b'1');
        }
        if self.rules.additives {
            choices.push(b'T');
        }
        if self.rules.zero {
            choices.push(b'0');
        }
        let junk_context = |this: &mut Self| -> Vec<Tree> {
            (0..this.rng.below(3)).map(|_| this.junk(2)).collect()
        };
        match choices[self.rng.below(choices.len())] {
            b'a' => {
                let a = self.rng.below(self.atoms as usize) as u8;
                (vec![Tree::Var(a)], Tree::Var(a), 0)
            }
            b'1' => (vec![], Tree::One, 0),
            b'T' => (junk_context(self), Tree::Top, 0),
            b'0' => {
                let mut context = junk_context(self);
                context.push(Tree::Zero);
                let goal = self.junk(2);
                (context, goal, 0)
            }
            _ => unreachable!(),
        }
    }

    /// Removes a random hypothesis, of which there must be one, and returns
    /// it.
    fn take_hypothesis(&mut self, hypotheses: &mut Vec<Tree>) -> Tree {
        hypotheses.remove(self.rng.below(hypotheses.len()))
    }

    /// A formula that every context proving `a` proves as well, for the
    /// other side of a `&`: `a` itself, `⊤`, or `a` under a `⊕`.
    fn twin(&mut self, a: &Tree) -> Tree {
        match self.rng.below(3) {
            0 => Tree::Top,
            1 => {
                let junk = self.junk(2);
                if self.rng.one_in(2) {
                    Tree::Plus(Box::new(a.clone()), Box::new(junk))
                } else {
                    Tree::Plus(Box::new(junk), Box::new(a.clone()))
                }
            }
            _ => a.clone(),
        }
    }

    /// An arbitrary intuitionistic formula of the allowed connectives, of
    /// at most `size` connectives.
    fn junk(&mut self, size: usize) -> Tree {
        let mut choices: Vec<u8> = vec![b'v', b'v'];
        if self.rules.units {
            choices.push(b'1');
        }
        if self.rules.additives {
            choices.push(b'T');
        }
        if self.rules.zero {
            choices.push(b'0');
        }
        if size > 0 {
            choices.extend(*b"tl");
            if self.rules.additives {
                choices.extend(*b"&+");
            }
            if self.rules.exponentials {
                choices.push(b'!');
            }
        }
        let boxed = |t: Tree| Box::new(t);
        match choices[self.rng.below(choices.len())] {
            b'v' => Tree::Var(self.rng.below(self.atoms as usize) as u8),
            b'1' => Tree::One,
            b'T' => Tree::Top,
            b'0' => Tree::Zero,
            b't' => Tree::Tensor(boxed(self.junk(size - 1)), boxed(self.junk(size - 1))),
            b'l' => Tree::Lolli(boxed(self.junk(size - 1)), boxed(self.junk(size - 1))),
            b'&' => Tree::With(boxed(self.junk(size - 1)), boxed(self.junk(size - 1))),
            b'+' => Tree::Plus(boxed(self.junk(size - 1)), boxed(self.junk(size - 1))),
            b'!' => Tree::Bang(boxed(self.junk(size - 1))),
            _ => unreachable!(),
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
/// returns its conclusion with the derelictions of the proof.
pub(crate) fn provable(rng: &mut Rng, rules: Rules, atoms: u8, budget: usize) -> Provable {
    let mut generator = Generator { rng, rules, atoms };
    let (formulas, copies) = generator.proof(budget);
    Provable { formulas, copies }
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
    /// Builds the conclusion of a random proof of about `budget` rules,
    /// with its derelictions, all branches together.
    fn proof(&mut self, budget: usize) -> (Vec<Tree>, u32) {
        if budget <= 1 {
            return (self.leaf(), 0);
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
        if self.rules.exponentials {
            choices.extend(*b"dwc!");
        }
        match choices[self.rng.below(choices.len())] {
            b'a' => (self.leaf(), 0),
            b't' => {
                // ⊢ Γ, A and ⊢ Δ, B give ⊢ Γ, Δ, A ⊗ B.
                let left_budget = 1 + self.rng.below(budget - 1);
                let (mut left, lc) = self.proof(left_budget);
                let (mut right, rc) = self.proof(budget - left_budget);
                let a = left.swap_remove(self.rng.below(left.len()));
                let b = right.swap_remove(self.rng.below(right.len()));
                left.append(&mut right);
                left.push(Tree::Tensor(Box::new(a), Box::new(b)));
                (left, lc + rc)
            }
            b'p' => {
                // ⊢ Γ, A, B gives ⊢ Γ, A ⅋ B; with one formula only, `⊥`
                // serves as the other when units are allowed.
                let (mut premise, copies) = self.proof(budget - 1);
                if premise.len() < 2 {
                    if !self.rules.units {
                        return (premise, copies);
                    }
                    premise.push(Tree::Bot);
                }
                let a = premise.swap_remove(self.rng.below(premise.len()));
                let b = premise.swap_remove(self.rng.below(premise.len()));
                premise.push(Tree::Par(Box::new(a), Box::new(b)));
                (premise, copies)
            }
            b'1' => (vec![Tree::One], 0),
            b'b' => {
                let (mut premise, copies) = self.proof(budget - 1);
                premise.push(Tree::Bot);
                (premise, copies)
            }
            b'&' => {
                // ⊢ Γ, A and ⊢ Γ, B give ⊢ Γ, A & B: B is a twin of A that
                // the same context proves, by the same proof, whose
                // derelictions count twice.
                let (mut premise, copies) = self.proof(budget - 1);
                let a = premise.swap_remove(self.rng.below(premise.len()));
                let b = self.twin(&a, 2);
                premise.push(Tree::With(Box::new(a), Box::new(b)));
                (premise, 2 * copies)
            }
            b'+' => {
                let (mut premise, copies) = self.proof(budget - 1);
                let a = premise.swap_remove(self.rng.below(premise.len()));
                let junk = self.junk(3);
                premise.push(if self.rng.one_in(2) {
                    Tree::Plus(Box::new(a), Box::new(junk))
                } else {
                    Tree::Plus(Box::new(junk), Box::new(a))
                });
                (premise, copies)
            }
            b'T' => {
                // ⊢ ⊤, Γ for any Γ.
                let mut sequent = vec![Tree::Top];
                for _ in 0..self.rng.below(3) {
                    sequent.push(self.junk(3));
                }
                (sequent, 0)
            }
            b'm' => {
                let left_budget = 1 + self.rng.below(budget - 1);
                let (mut left, lc) = self.proof(left_budget);
                let (mut right, rc) = self.proof(budget - left_budget);
                left.append(&mut right);
                (left, lc + rc)
            }
            b'd' => {
                // Dereliction: ⊢ Γ, A gives ⊢ Γ, ?A.
                let (mut premise, copies) = self.proof(budget - 1);
                let a = premise.swap_remove(self.rng.below(premise.len()));
                premise.push(Tree::Quest(Box::new(a)));
                (premise, copies + 1)
            }
            b'w' => {
                // Weakening: ⊢ Γ gives ⊢ Γ, ?A for any A.
                let (mut premise, copies) = self.proof(budget - 1);
                let junk = self.junk(3);
                premise.push(Tree::Quest(Box::new(junk)));
                (premise, copies)
            }
            b'c' => {
                // Contraction, on a `?` formula both copies of a premise
                // hold: ⊢ Γ, A twice gives ⊢ Γ, Γ, A ⊗ A by `⊗`, and each
                // `?B` of Γ appears twice, which contracts to once.
                let (mut premise, copies) = self.proof(budget - 1);
                if !premise.iter().any(Tree::is_quest) {
                    let i = self.rng.below(premise.len());
                    let b = premise.swap_remove(i);
                    premise.push(Tree::Quest(Box::new(b)));
                    return (premise, copies + 1);
                }
                let a = premise.swap_remove(self.rng.below(premise.len()));
                let (quests, rest): (Vec<Tree>, Vec<Tree>) =
                    premise.into_iter().partition(Tree::is_quest);
                let mut sequent = rest.clone();
                sequent.extend(rest);
                sequent.extend(quests);
                sequent.push(Tree::Tensor(Box::new(a.clone()), Box::new(a)));
                (sequent, 2 * copies)
            }
            b'!' => {
                // Promotion: ⊢ ?Γ, A gives ⊢ ?Γ, !A; whatever of Γ is not a
                // `?` formula is derelicted first.
                let (mut premise, copies) = self.proof(budget - 1);
                let a = premise.swap_remove(self.rng.below(premise.len()));
                let mut derelicted = 0;
                for f in premise.iter_mut() {
                    if !f.is_quest() {
                        let inner = std::mem::replace(f, Tree::One);
                        *f = Tree::Quest(Box::new(inner));
                        derelicted += 1;
                    }
                }
                premise.push(Tree::Bang(Box::new(a)));
                (premise, copies + derelicted)
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
            if self.rules.exponentials {
                choices.extend(*b"!?");
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
            b'!' => Tree::Bang(sub(self)),
            b'?' => Tree::Quest(sub(self)),
            _ => unreachable!(),
        }
    }
}

/// Builds a random sequent of unit-free MLL over `atoms` atom names with
/// `pairs` dual pairs of literals that passes the count checks of the net
/// engine, provable or not: exactly `pairs − 1` tensors without Mix and at
/// most that many with it, any number of pars that leaves a conclusion,
/// joined in random order. Most such sequents are unprovable, so they test
/// the negative side beyond what the counts reject.
pub(crate) fn balanced(rng: &mut Rng, atoms: u8, pairs: usize, mix: bool) -> Vec<Tree> {
    let mut formulas: Vec<Tree> = Vec::with_capacity(2 * pairs);
    for _ in 0..pairs {
        let a = rng.below(atoms as usize) as u8;
        formulas.push(Tree::Var(a));
        formulas.push(Tree::Dual(a));
    }
    let tensors = if mix { rng.below(pairs) } else { pairs - 1 };
    let pars = rng.below(2 * pairs - tensors);
    let mut joins: Vec<bool> = vec![true; tensors];
    joins.extend(vec![false; pars]);
    for k in (1..joins.len()).rev() {
        joins.swap(k, rng.below(k + 1));
    }
    for tensor in joins {
        let a = formulas.swap_remove(rng.below(formulas.len()));
        let b = formulas.swap_remove(rng.below(formulas.len()));
        formulas.push(if tensor {
            Tree::Tensor(Box::new(a), Box::new(b))
        } else {
            Tree::Par(Box::new(a), Box::new(b))
        });
    }
    formulas
}
