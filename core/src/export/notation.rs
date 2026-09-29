// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What the export targets share: a symbol table per target, and the
//! printers that write formulas and sequents through it with the
//! bracketing of `Display`.

use crate::occurrences::{Forest, OccId, Position, Reading};
use crate::sequents::{Kind, Sequent, Term, TermId};

/// How a target writes formulas and sequents: the spelling of every
/// connective, unit and of the turnstile, and how an atom's name is
/// escaped. Binary connectives are written between spaces, `!` and `?`
/// directly before their operand.
pub(crate) struct Notation {
    /// `⊗`
    pub(crate) tensor: &'static str,
    /// `⅋`
    pub(crate) par: &'static str,
    /// `&`
    pub(crate) with: &'static str,
    /// `⊕`
    pub(crate) plus: &'static str,
    /// `⊸`, which only a two-sided sequent shows.
    pub(crate) lollipop: &'static str,
    /// `!`
    pub(crate) bang: &'static str,
    /// `?`
    pub(crate) quest: &'static str,
    /// `1`
    pub(crate) one: &'static str,
    /// `⊥`
    pub(crate) bot: &'static str,
    /// `⊤`
    pub(crate) top: &'static str,
    /// `0`
    pub(crate) zero: &'static str,
    /// The mark after a negated atom's name.
    pub(crate) dual: &'static str,
    /// `⊢`
    pub(crate) turnstile: &'static str,
    /// What goes before the turnstile of a two-sided sequent in a proof
    /// tree so that the turnstiles of a unary inference line up; empty
    /// where the target cannot align them.
    pub(crate) align: &'static str,
    /// Writes an atom's name.
    pub(crate) atom: fn(&mut String, &str),
}

impl Notation {
    /// Writes the formula rooted at `id` of `sequent` one-sided, in
    /// brackets if it is binary and `brackets` is set.
    pub(crate) fn term(&self, out: &mut String, sequent: &Sequent, id: TermId, brackets: bool) {
        use Term::*;
        let binary = |out: &mut String, k: TermId, symbol: &str, l: TermId| {
            if brackets {
                out.push('(');
            }
            self.term(out, sequent, k, true);
            out.push(' ');
            out.push_str(symbol);
            out.push(' ');
            self.term(out, sequent, l, true);
            if brackets {
                out.push(')');
            }
        };
        match sequent.term(id) {
            Var(a) => (self.atom)(out, sequent.atom_name(a)),
            DualVar(a) => {
                (self.atom)(out, sequent.atom_name(a));
                out.push_str(self.dual);
            }
            One => out.push_str(self.one),
            Bot => out.push_str(self.bot),
            Top => out.push_str(self.top),
            Zero => out.push_str(self.zero),
            Tensor(k, l) => binary(out, k, self.tensor, l),
            Par(k, l) => binary(out, k, self.par, l),
            With(k, l) => binary(out, k, self.with, l),
            Plus(k, l) => binary(out, k, self.plus, l),
            Bang(k) => {
                out.push_str(self.bang);
                self.term(out, sequent, k, true);
            }
            Quest(k) => {
                out.push_str(self.quest);
                self.term(out, sequent, k, true);
            }
        }
    }

    /// Writes the intuitionistic formula at `o` as the reading reads it
    /// (`⊸`, `1` for `⊥`, `⊤` for an input `0`, and so on), in brackets if
    /// it is binary and `brackets` is set.
    pub(crate) fn ill(&self, out: &mut String, reading: &Reading, o: OccId, brackets: bool) {
        use Kind::*;
        let forest = reading.forest();
        let binary = |out: &mut String, k: OccId, symbol: &str, l: OccId| {
            if brackets {
                out.push('(');
            }
            self.ill(out, reading, k, true);
            out.push(' ');
            out.push_str(symbol);
            out.push(' ');
            self.ill(out, reading, l, true);
            if brackets {
                out.push(')');
            }
        };
        let (l, r) = (forest.left(o), forest.right(o));
        match (forest.kind(o), reading.position(o)) {
            (Var | DualVar, _) => {
                (self.atom)(out, forest.sequent().atom_name(forest.atom(o).unwrap()));
            }
            (One | Bot, _) => out.push_str(self.one),
            (Top, Position::Output) | (Zero, Position::Input) => out.push_str(self.top),
            (Zero, Position::Output) | (Top, Position::Input) => out.push_str(self.zero),
            (Tensor, Position::Output) | (Par, Position::Input) => {
                binary(out, l.unwrap(), self.tensor, r.unwrap());
            }
            (Tensor, Position::Input) | (Par, Position::Output) => {
                let (antecedent, consequent) = reading.implication(o).unwrap();
                binary(out, antecedent, self.lollipop, consequent);
            }
            (With, Position::Output) | (Plus, Position::Input) => {
                binary(out, l.unwrap(), self.with, r.unwrap());
            }
            (Plus, Position::Output) | (With, Position::Input) => {
                binary(out, l.unwrap(), self.plus, r.unwrap());
            }
            (Bang | Quest, _) => {
                out.push_str(self.bang);
                self.ill(out, reading, l.unwrap(), true);
            }
        }
    }

    /// Writes a sequent one-sided: the turnstile, then its root formulas,
    /// comma-separated.
    pub(crate) fn one_sided(&self, out: &mut String, sequent: &Sequent) {
        out.push_str(self.turnstile);
        for (i, &id) in sequent.roots().iter().enumerate() {
            out.push_str(if i == 0 { " " } else { ", " });
            self.term(out, sequent, id, false);
        }
    }

    /// Writes a sequent of a derivation, occurrences in ascending order:
    /// one-sided, or two-sided under a reading with the hypotheses in id
    /// order before the turnstile and the goal after it, which `aligned`
    /// lines up with the turnstiles above and below.
    pub(crate) fn sequent(
        &self,
        out: &mut String,
        forest: &Forest,
        reading: Option<&Reading>,
        sequent: &[OccId],
        aligned: bool,
    ) {
        let Some(reading) = reading else {
            out.push_str(self.turnstile);
            for (i, &o) in sequent.iter().enumerate() {
                out.push_str(if i == 0 { " " } else { ", " });
                self.term(out, forest.sequent(), forest.term(o), false);
            }
            return;
        };
        let mut goal = None;
        let mut hypotheses = 0;
        for &o in sequent {
            if reading.position(o) == Position::Output {
                goal = Some(o);
                continue;
            }
            if hypotheses > 0 {
                out.push_str(", ");
            }
            self.ill(out, reading, o, false);
            hypotheses += 1;
        }
        if hypotheses > 0 {
            out.push(' ');
        }
        if aligned {
            out.push_str(self.align);
        }
        out.push_str(self.turnstile);
        if let Some(goal) = goal {
            out.push(' ');
            self.ill(out, reading, goal, false);
        }
    }
}
