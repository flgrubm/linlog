// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The intuitionistic reading of a sequent: which occurrences are
//! hypotheses and which one is the goal, so that a one-sided sequent in
//! negation normal form is read as a two-sided sequent `Γ ⊢ A` of
//! intuitionistic linear logic without a second representation.
//!
//! An intuitionistic formula stands in *output position* when it is the goal
//! or the antecedent of a hypothesis: it is built from `⊗ ⊕ & ! 1 ⊤ 0`,
//! atoms `a`, and `A ⊸ B`, which the one-sided arena stores as the par
//! `A⊥ ⅋ B`. In *input position*, as a hypothesis or as the antecedent of
//! the goal, a formula is the negation of such a formula: `⅋ & ⊕ ? ⊥ 0 ⊤`,
//! negated atoms `~a`, and `A ⊗ B⊥` for a hypothesis `A ⊸ B`. The position
//! flips at the antecedent of an implication and nowhere else, which is
//! Lamarche's polarization of intuitionistic proof structures. An
//! intuitionistic sequent is a one-sided sequent with exactly one
//! output-shaped root, the goal, and every other root input-shaped.

use super::{Forest, OccId};
use crate::sequents::Kind;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The side of `⊢` an occurrence stands on under the intuitionistic reading
/// of its sequent: a hypothesis, or a subformula that behaves like one, is
/// input; the goal, or a subformula that behaves like it, is output.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Position {
    /// A hypothesis, or the antecedent of a formula in output position.
    Input,
    /// The goal, or the antecedent of a formula in input position.
    Output,
}

impl std::ops::Not for Position {
    type Output = Self;

    /// The other position.
    fn not(self) -> Self {
        match self {
            Position::Input => Position::Output,
            Position::Output => Position::Input,
        }
    }
}

/// Why a sequent has no intuitionistic reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeError {
    /// No root formula can be the goal: every one is input-shaped only, or
    /// the sequent is empty.
    NoGoal,
    /// Two root formulas can only be goals.
    SeveralGoals(OccId, OccId),
    /// A subformula is neither an intuitionistic formula nor the negation of
    /// one, in any position.
    Formula(OccId),
}

impl Display for ShapeError {
    /// Writes the reason with occurrence ids, such as `subformula 3 has no
    /// intuitionistic reading`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write(f, None)
    }
}

impl ShapeError {
    /// Returns the error for display with formulas instead of occurrence
    /// ids, read from `forest`, the forest the reading was attempted on.
    pub fn describe<'a>(&'a self, forest: &'a Forest) -> DescribedShape<'a> {
        DescribedShape {
            error: self,
            forest,
        }
    }

    /// Writes the reason as [`Display`] does, with formulas instead of ids
    /// when a forest is given.
    fn write(&self, f: &mut Formatter<'_>, forest: Option<&Forest>) -> FmtResult {
        let occurrence = |f: &mut Formatter<'_>, o: OccId| match forest {
            Some(forest) => write!(f, "{}", forest.formula(o)),
            None => write!(f, "{}", o.get()),
        };
        match self {
            ShapeError::NoGoal => f.write_str(
                "no formula can be the goal: an intuitionistic sequent has exactly one formula on the right of ⊢",
            ),
            ShapeError::SeveralGoals(a, b) => {
                f.write_str(if forest.is_some() {
                    "both "
                } else {
                    "both formula "
                })?;
                occurrence(f, *a)?;
                f.write_str(" and ")?;
                occurrence(f, *b)?;
                f.write_str(" can only be the goal, but an intuitionistic sequent has one")
            }
            ShapeError::Formula(o) => {
                f.write_str(if forest.is_some() {
                    "the subformula "
                } else {
                    "subformula "
                })?;
                occurrence(f, *o)?;
                if let Some(forest) = forest
                    && forest.root(*o) != *o
                {
                    write!(f, " (in {})", forest.formula(forest.root(*o)))?;
                }
                f.write_str(
                    " is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)",
                )
            }
        }
    }
}

impl std::error::Error for ShapeError {}

/// A [`ShapeError`] displayed with formulas, as [`ShapeError::describe`]
/// returns it.
pub struct DescribedShape<'a> {
    /// The error.
    error: &'a ShapeError,
    /// The forest the reading was attempted on.
    forest: &'a Forest,
}

impl Display for DescribedShape<'_> {
    /// Writes the reason with formulas instead of occurrence ids.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.error.write(f, Some(self.forest))
    }
}

/// An occurrence's possible positions, as two bits.
const IN: u8 = 1;
/// See [`IN`].
const OUT: u8 = 2;

/// The intuitionistic reading of a sequent: the position of every
/// occurrence of its forest and which root is the goal. It prints the
/// sequent two-sided, `Γ ⊢ A`, with intuitionistic formulas.
///
/// The reading is deterministic. A root that can only be a goal is the
/// goal; otherwise the last root, in id order, that can be one. Only a
/// formula built from `⊤` and `0` alone can stand on either side, and for
/// those the written succedent is not recoverable: the arena keeps its
/// roots sorted by term, not in the order they were written, so `0, ⊤ ⊢ ⊤`
/// reads as `0, 0 ⊢ 0`. Both readings of such a sequent are provable or
/// neither is, so the verdict does not depend on the choice, only the
/// two-sided print does. Inside a
/// formula the only choice is which factor of an implication is the
/// antecedent, `⅋` in output position being `A ⊸ B` for `A⊥ ⅋ B` and `⊗` in
/// input position `A ⊗ B⊥`: the left factor is the antecedent when that
/// reading works, and the right one otherwise, so that `b ⅋ ~a` is read
/// as `a ⊸ b` too.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Forest, Position, Reading, Sequent};
///
/// // ⊢ ~A, A ⊗ ~B, B
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let forest = Forest::new(&sequent)?;
/// let reading = Reading::new(&forest)?;
/// assert_eq!(reading.to_string(), "A, A ⊸ B ⊢ B");
/// assert_eq!(reading.goal(), forest.roots()[2]);
/// let hypothesis = forest.roots()[1];
/// assert_eq!(reading.position(hypothesis), Position::Input);
/// assert_eq!(reading.position(forest.left(hypothesis).unwrap()), Position::Output);
/// assert_eq!(reading.formula(hypothesis).to_string(), "A ⊸ B");
///
/// let classical: Sequent = "|- A par B".parse()?;
/// assert!(Reading::new(&Forest::new(&classical)?).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct Reading<'a> {
    /// The forest read.
    forest: &'a Forest,
    /// Per occurrence, its position.
    position: Box<[Position]>,
    /// The goal.
    goal: OccId,
}

impl<'a> Reading<'a> {
    /// Reads the forest as an intuitionistic sequent, or says why it is
    /// none: a subformula with no reading in any position, no root that can
    /// be the goal, or two that must be.
    pub fn new(forest: &'a Forest) -> Result<Self, ShapeError> {
        use Kind::*;
        // Which positions each occurrence can take, children before parents.
        let mut can = vec![0u8; forest.len()];
        for o in forest.ids().rev() {
            let child = |k: Option<OccId>| k.map_or(0, |k| can[k.index()]);
            let (l, r) = (child(forest.left(o)), child(forest.right(o)));
            let (l_in, l_out, r_in, r_out) = (l & IN != 0, l & OUT != 0, r & IN != 0, r & OUT != 0);
            let both = |i: bool, u: bool| (u8::from(i) * IN) | (u8::from(u) * OUT);
            can[o.index()] = match forest.kind(o) {
                Var | One | Bang => both(false, l_out || forest.kind(o) != Bang),
                DualVar | Bot | Quest => both(l_in || forest.kind(o) != Quest, false),
                Top | Zero => IN | OUT,
                With | Plus => both(l_in && r_in, l_out && r_out),
                Tensor => both((l_out && r_in) || (l_in && r_out), l_out && r_out),
                Par => both(l_in && r_in, (l_in && r_out) || (l_out && r_in)),
            };
            if can[o.index()] == 0 {
                return Err(ShapeError::Formula(o));
            }
        }

        // The goal: the root that can only be output, or the last that can
        // be output at all.
        let roots = forest.roots();
        let only_goals: Vec<OccId> = roots
            .iter()
            .copied()
            .filter(|&o| can[o.index()] == OUT)
            .collect();
        let goal = match only_goals[..] {
            [goal] => goal,
            [a, b, ..] => return Err(ShapeError::SeveralGoals(a, b)),
            [] => roots
                .iter()
                .copied()
                .rev()
                .find(|&o| can[o.index()] & OUT != 0)
                .ok_or(ShapeError::NoGoal)?,
        };

        // The positions, parents before children.
        let mut position = vec![Position::Input; forest.len()].into_boxed_slice();
        position[goal.index()] = Position::Output;
        for o in forest.ids() {
            let p = position[o.index()];
            let Some((l, r)) = forest.left(o).zip(forest.right(o)) else {
                if let Some(l) = forest.left(o) {
                    position[l.index()] = p;
                }
                continue;
            };
            let (cl, cr) = (can[l.index()], can[r.index()]);
            // An implication: the left factor is the antecedent unless only
            // the right one can be.
            let flipped = match (forest.kind(o), p) {
                (Tensor, Position::Input) => {
                    if cl & OUT != 0 && cr & IN != 0 {
                        l
                    } else {
                        r
                    }
                }
                (Par, Position::Output) => {
                    if cl & IN != 0 && cr & OUT != 0 {
                        l
                    } else {
                        r
                    }
                }
                _ => {
                    position[l.index()] = p;
                    position[r.index()] = p;
                    continue;
                }
            };
            position[l.index()] = if flipped == l { !p } else { p };
            position[r.index()] = if flipped == r { !p } else { p };
        }
        Ok(Self {
            forest,
            position,
            goal,
        })
    }

    /// Returns the forest read.
    pub fn forest(&self) -> &'a Forest {
        self.forest
    }

    /// Returns the position of an occurrence.
    pub fn position(&self, o: OccId) -> Position {
        self.position[o.index()]
    }

    /// Returns the goal: the one root in output position.
    pub fn goal(&self) -> OccId {
        self.goal
    }

    /// Returns the hypotheses: every root but the goal, in id order.
    pub fn hypotheses(&self) -> impl Iterator<Item = OccId> + '_ {
        self.forest
            .roots()
            .iter()
            .copied()
            .filter(move |&o| o != self.goal)
    }

    /// Returns the antecedent and the consequent of an implication `A ⊸ B`:
    /// a `⅋` in output position or a `⊗` in input position, whose antecedent
    /// is the factor in the other position. `None` for any other occurrence.
    pub fn implication(&self, o: OccId) -> Option<(OccId, OccId)> {
        let p = self.position(o);
        match (self.forest.kind(o), p) {
            (Kind::Par, Position::Output) | (Kind::Tensor, Position::Input) => {
                let (l, r) = (self.forest.left(o)?, self.forest.right(o)?);
                Some(if self.position(l) != p {
                    (l, r)
                } else {
                    (r, l)
                })
            }
            _ => None,
        }
    }

    /// Counts the occurrences in output position among `ids`: an
    /// intuitionistic sequent has exactly one.
    pub fn outputs(&self, ids: impl IntoIterator<Item = OccId>) -> usize {
        ids.into_iter()
            .filter(|&o| self.position(o) == Position::Output)
            .count()
    }

    /// Returns the intuitionistic formula at `o`, as a value that prints it
    /// with `⊸`, `1` for `⊥`, `⊤` for an input `0` and so on, according to
    /// the position of `o`.
    pub fn formula(&self, o: OccId) -> IllFormula<'_> {
        IllFormula {
            reading: self,
            id: o,
        }
    }

    /// Writes the formula at `o`, in brackets if it is binary and `brackets`
    /// is set.
    fn fmt_formula(&self, o: OccId, f: &mut Formatter<'_>, brackets: bool) -> FmtResult {
        use Kind::*;
        let forest = self.forest;
        let p = self.position(o);
        let binary = |f: &mut Formatter<'_>, k: OccId, symbol: &str, l: OccId| {
            if brackets {
                f.write_str("(")?;
            }
            self.fmt_formula(k, f, true)?;
            write!(f, " {symbol} ")?;
            self.fmt_formula(l, f, true)?;
            if brackets {
                f.write_str(")")?;
            }
            Ok(())
        };
        let (l, r) = (forest.left(o), forest.right(o));
        match (forest.kind(o), p) {
            (Var | DualVar, _) => f.write_str(forest.sequent().atom_name(forest.atom(o).unwrap())),
            (One | Bot, _) => f.write_str("1"),
            (Top, Position::Output) | (Zero, Position::Input) => f.write_str("⊤"),
            (Zero, Position::Output) | (Top, Position::Input) => f.write_str("0"),
            (Tensor, Position::Output) | (Par, Position::Input) => {
                binary(f, l.unwrap(), "⊗", r.unwrap())
            }
            (Tensor, Position::Input) | (Par, Position::Output) => {
                let (antecedent, consequent) = self.implication(o).unwrap();
                binary(f, antecedent, "⊸", consequent)
            }
            (With, Position::Output) | (Plus, Position::Input) => {
                binary(f, l.unwrap(), "&", r.unwrap())
            }
            (Plus, Position::Output) | (With, Position::Input) => {
                binary(f, l.unwrap(), "⊕", r.unwrap())
            }
            (Bang | Quest, _) => {
                f.write_str("!")?;
                self.fmt_formula(l.unwrap(), f, true)
            }
        }
    }
}

impl Display for Reading<'_> {
    /// Writes the sequent two-sided: the hypotheses, `⊢`, the goal, with
    /// intuitionistic formulas.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for (i, h) in self.hypotheses().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            self.fmt_formula(h, f, false)?;
        }
        if self.hypotheses().next().is_some() {
            f.write_str(" ")?;
        }
        f.write_str("⊢ ")?;
        self.fmt_formula(self.goal, f, false)
    }
}

/// An intuitionistic formula of a read sequent, as a value that prints it.
#[derive(Clone, Copy, Debug)]
pub struct IllFormula<'a> {
    /// The reading the formula's position comes from.
    reading: &'a Reading<'a>,
    /// The formula's occurrence.
    id: OccId,
}

impl Display for IllFormula<'_> {
    /// Writes the formula with brackets around every binary subformula.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.reading.fmt_formula(self.id, f, false)
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;
    use crate::search::generate::Rng;

    /// Parses `input` and reads it.
    fn read(input: &str) -> Result<String, String> {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let forest = Forest::new(&s).unwrap();
        Reading::new(&forest)
            .map(|r| r.to_string())
            .map_err(|e| e.describe(&forest).to_string())
    }

    /// Intuitionistic sequents print back two-sided with `⊸`, `1`, `⊤` and
    /// `0` recovered from their one-sided forms, and the goal is the root
    /// that can only be one, else the last that can be.
    #[test]
    fn shapes() {
        for (input, two_sided) in [
            ("A, A -o B |- B", "A, A ⊸ B ⊢ B"),
            ("|- A", "⊢ A"),
            ("A -o B -o C |- (A * B) -o C", "A ⊸ (B ⊸ C) ⊢ (A ⊗ B) ⊸ C"),
            ("|- A & B -o A + B", "⊢ (A & B) ⊸ (A ⊕ B)"),
            ("A & B |- A + B", "A & B ⊢ A ⊕ B"),
            ("!A, !(A -o B) |- !B", "!A, !(A ⊸ B) ⊢ !B"),
            ("(A -o B) -o C |- D", "(A ⊸ B) ⊸ C ⊢ D"),
            // The symmetric reading of an implication: `b ⅋ ~a` is `a ⊸ b`
            // and its hypothesis form `~b ⊗ a` too.
            ("|- B par ~A", "⊢ A ⊸ B"),
            ("~B * A |-", "⊢ A ⊸ B"),
            // Ambiguous roots: `0` and `⊤` can stand on either side, so the
            // last root that can be the goal is it.
            ("|- 0, top", "⊤ ⊢ ⊤"),
            ("|- top, top * top", "0 ⊢ ⊤ ⊗ ⊤"),
            ("top |- top * top", "⊤ ⊢ ⊤ ⊗ ⊤"),
            ("|- A, top", "0 ⊢ A"),
            ("A * top |- A", "A ⊗ ⊤ ⊢ A"),
            ("1, top, 0 |- 1 * top * 0", "1, ⊤, 0 ⊢ (1 ⊗ ⊤) ⊗ 0"),
        ] {
            assert_eq!(read(input).as_deref(), Ok(two_sided), "{input:?}");
        }
        for (input, message) in [
            (
                "|-",
                "no formula can be the goal: an intuitionistic sequent has exactly one formula on the right of ⊢",
            ),
            (
                "A, B |-",
                "no formula can be the goal: an intuitionistic sequent has exactly one formula on the right of ⊢",
            ),
            (
                "A |- B, C",
                "both B and C can only be the goal, but an intuitionistic sequent has one",
            ),
            (
                "|- A par B",
                "the subformula A ⅋ B is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)",
            ),
            (
                "A * (B par C) |- D",
                "the subformula ~B ⊗ ~C (in ~A ⅋ (~B ⊗ ~C)) is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)",
            ),
            (
                "|- ?A",
                "the subformula ?A is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)",
            ),
            (
                "|- ~A",
                "no formula can be the goal: an intuitionistic sequent has exactly one formula on the right of ⊢",
            ),
        ] {
            assert_eq!(
                read(input).as_ref().map_err(String::as_str),
                Err(message),
                "{input:?}"
            );
        }
        let s: Sequent = "|- A par B".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        assert_eq!(
            Reading::new(&forest).unwrap_err().to_string(),
            "subformula 0 is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)"
        );
    }

    /// The positions: the antecedent of an implication flips, the rest
    /// inherits, and `implication` names the antecedent first.
    #[test]
    fn positions() {
        // ⊢ ~A ⅋ (B ⊗ ~C), D: 0 ⅋, 1 ~A, 2 ⊗, 3 B, 4 ~C, 5 D
        let s: Sequent = "|- A -o (B -o C), D".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        // The goal is `D`, the only root that must be output; the other root
        // is the hypothesis `(A ⊸ (B ⊸ C))`... whose one-sided form is an
        // output-shaped par, so it can only be a goal too.
        assert!(matches!(
            Reading::new(&forest),
            Err(ShapeError::SeveralGoals(a, b)) if a == OccId::new(0) && b == OccId::new(5)
        ));
        // ⊢ A ⊗ (B ⊗ ~C), D: hypothesis A ⊸ (B ⊸ C): 0 ⊗, 1 A, 2 ⊗, 3 B, 4 ~C, 5 D
        let s: Sequent = "A -o (B -o C) |- D".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let r = Reading::new(&forest).unwrap();
        let o = OccId::new;
        assert_eq!(r.goal(), o(5));
        assert_eq!(r.hypotheses().collect::<Vec<_>>(), [o(0)]);
        let positions: Vec<Position> = forest.ids().map(|i| r.position(i)).collect();
        use Position::*;
        assert_eq!(positions, [Input, Output, Input, Output, Input, Output]);
        assert_eq!(r.implication(o(0)), Some((o(1), o(2))));
        assert_eq!(r.implication(o(2)), Some((o(3), o(4))));
        assert_eq!(r.implication(o(1)), None);
        assert_eq!(r.outputs(forest.roots().iter().copied()), 1);
        assert_eq!(r.formula(o(2)).to_string(), "B ⊸ C");
        assert_eq!(r.formula(o(3)).to_string(), "B");
    }

    /// A random intuitionistic formula in the parser's syntax.
    fn formula(rng: &mut Rng, depth: usize) -> String {
        if depth == 0 || rng.one_in(3) {
            return match rng.below(6) {
                0 => "1".into(),
                1 => "top".into(),
                2 => "0".into(),
                _ => ["a", "b", "c"][rng.below(3)].into(),
            };
        }
        let sub = |rng: &mut Rng| formula(rng, depth - 1);
        match rng.below(6) {
            0 => format!("({} * {})", sub(rng), sub(rng)),
            1 => format!("({} -o {})", sub(rng), sub(rng)),
            2 => format!("({} & {})", sub(rng), sub(rng)),
            3 => format!("({} + {})", sub(rng), sub(rng)),
            4 => format!("!{}", sub(rng)),
            _ => format!("({} -o {})", sub(rng), sub(rng)),
        }
    }

    /// Parsing a random intuitionistic sequent, printing it two-sided and
    /// parsing the print gives the same sequent.
    #[test]
    fn print_parse_round_trip() {
        let mut rng = Rng::new(8);
        for _ in 0..500 {
            let hypotheses: Vec<String> = (0..rng.below(4)).map(|_| formula(&mut rng, 3)).collect();
            let input = format!("{} |- {}", hypotheses.join(", "), formula(&mut rng, 3));
            let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
            let forest = Forest::new(&s).unwrap();
            let printed = Reading::new(&forest)
                .unwrap_or_else(|e| panic!("{input:?}: {}", e.describe(&forest)))
                .to_string();
            let back: Sequent = printed
                .parse()
                .unwrap_or_else(|e| panic!("{input:?} printed as {printed:?}: {e}"));
            // The print is canonical (the antecedent of an implication comes
            // first), so its own reading prints the same, over the same
            // fragment and as many formulas.
            assert_eq!(
                back.fragment(),
                s.fragment(),
                "{input:?} printed as {printed:?}"
            );
            assert_eq!(
                back.roots().len(),
                s.roots().len(),
                "{input:?} printed as {printed:?}"
            );
            let forest = Forest::new(&back).unwrap();
            assert_eq!(Reading::new(&forest).unwrap().to_string(), printed);
        }
    }
}
