// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::logics::LL;
use crate::logics::Logic;
use crate::sequents::Sequent as Seq;
use crate::sequents::expressions::{Expression, LLExpression};
use chumsky::pratt::*;
use chumsky::prelude::*;

/// A parsed formula, before it is lowered into an arena.
#[derive(Debug)]
pub(super) enum Term<'a> {
    /// A variable, by name.
    Var(&'a str),
    /// `0`
    Zero,
    /// `1`
    One,
    /// `⊥`
    Bot,
    /// `⊤`
    Top,
    /// Linear negation, `~A` or `A^`.
    Dual(Box<Term<'a>>),
    /// `!A`
    Bang(Box<Term<'a>>),
    /// `?A`
    Quest(Box<Term<'a>>),
    /// `A ⊗ B`
    Tensor(Box<Term<'a>>, Box<Term<'a>>),
    /// `A ⅋ B`
    Par(Box<Term<'a>>, Box<Term<'a>>),
    /// `A & B`
    With(Box<Term<'a>>, Box<Term<'a>>),
    /// `A ⊕ B`
    Plus(Box<Term<'a>>, Box<Term<'a>>),
    /// `A ⊸ B`
    Lollipop(Box<Term<'a>>, Box<Term<'a>>),
}

/// A parsed two-sided sequent, `left ⊢ right`.
#[derive(Debug)]
pub(super) struct Sequent<'a> {
    /// The formulas left of the turnstile.
    pub(super) left: Vec<Term<'a>>,
    /// The formulas right of the turnstile.
    pub(super) right: Vec<Term<'a>>,
}

/// Parses a unit constant: `0`, `1`, `bot`/`⊥` or `top`/`⊤`.
fn constant_parser<'a>() -> impl Parser<'a, &'a str, Term<'a>, extra::Err<Simple<'a, char>>> + Clone
{
    // `text::keyword` only matches identifiers, so digits and symbols need `just`.
    choice((
        just('0').map(|_| Term::Zero),
        just('1').map(|_| Term::One),
        text::keyword("bot").map(|_| Term::Bot),
        just('⊥').map(|_| Term::Bot),
        text::keyword("top").map(|_| Term::Top),
        just('⊤').map(|_| Term::Top),
    ))
}

/// Parses a variable name: a Unicode identifier.
fn variable_parser<'a>() -> impl Parser<'a, &'a str, Term<'a>, extra::Err<Simple<'a, char>>> + Clone
{
    text::ident().map(Term::Var)
}

/// Parses a formula, with its operators' precedence and associativity.
fn term_parser<'a>() -> impl Parser<'a, &'a str, Term<'a>, extra::Err<Simple<'a, char>>> + Clone {
    recursive(|term| {
        let atom = choice((
            constant_parser(),
            variable_parser(),
            term.delimited_by(just('('), just(')')),
        ))
        .padded();

        // binding strength:
        // postfix '^' > prefix '~', '!', '?' > Tensor > Par > With > Sum > Lollipop
        atom.pratt((
            // Unaries
            postfix(7, just('^').padded(), |lhs, _, _| Term::Dual(Box::new(lhs))),
            prefix(6, just('~').padded(), |_, rhs, _| Term::Dual(Box::new(rhs))),
            prefix(6, just('!').padded(), |_, rhs, _| Term::Bang(Box::new(rhs))),
            prefix(6, just('?').padded(), |_, rhs, _| {
                Term::Quest(Box::new(rhs))
            }),
            // Tensor
            infix(left(5), just('*').padded(), |l, _, r, _| {
                Term::Tensor(Box::new(l), Box::new(r))
            }),
            infix(left(5), just('⊗').padded(), |l, _, r, _| {
                Term::Tensor(Box::new(l), Box::new(r))
            }),
            // Par
            infix(left(4), text::keyword("par").padded(), |l, _, r, _| {
                Term::Par(Box::new(l), Box::new(r))
            }),
            infix(left(4), just('|').padded(), |l, _, r, _| {
                Term::Par(Box::new(l), Box::new(r))
            }),
            infix(left(4), just('⅋').padded(), |l, _, r, _| {
                Term::Par(Box::new(l), Box::new(r))
            }),
            // With
            infix(left(3), just('&').padded(), |l, _, r, _| {
                Term::With(Box::new(l), Box::new(r))
            }),
            // Plus
            infix(left(2), just('+').padded(), |l, _, r, _| {
                Term::Plus(Box::new(l), Box::new(r))
            }),
            infix(left(2), just('⊕').padded(), |l, _, r, _| {
                Term::Plus(Box::new(l), Box::new(r))
            }),
            // Lollipop
            infix(right(1), just("-o").padded(), |l, _, r, _| {
                Term::Lollipop(Box::new(l), Box::new(r))
            }),
            infix(right(1), just('⊸').padded(), |l, _, r, _| {
                Term::Lollipop(Box::new(l), Box::new(r))
            }),
        ))
    })
}

/// Parses a two-sided sequent, `Γ |- Δ`, which must span the whole input.
fn sequent_parser<'a>() -> impl Parser<'a, &'a str, Sequent<'a>, extra::Err<Simple<'a, char>>> {
    let terms_list = term_parser()
        .clone()
        .separated_by(just(',').padded())
        .collect::<Vec<Term<'a>>>();

    let tack = choice((just("|-"), just("⊢"))).padded();

    terms_list
        .clone() // Parse LHS
        .then(tack) // Ignore the tack (but consume it)
        .then(terms_list) // Parse RHS
        .map(|((left, _), right)| Sequent { left, right })
        .then_ignore(end()) // Ensure the parser consumes the entire input string
}

impl<'a> TryFrom<&'a str> for Sequent<'a> {
    type Error = crate::Error;

    /// Parses `input` into a syntax tree that borrows its variable names, or
    /// returns every parse error.
    fn try_from(input: &'a str) -> Result<Sequent<'a>, Self::Error> {
        sequent_parser()
            .parse(input)
            .into_result()
            .map_err(|borrowed_errors| {
                let owned_errors: Vec<crate::errors::ParseError> = borrowed_errors
                    .into_iter()
                    .map(crate::errors::ParseError::from)
                    .collect();

                crate::Error::SequentParsing(owned_errors)
            })
    }
}

impl<'a> From<(Term<'a>, bool)> for Seq<usize, LL> {
    /// Lowers a formula into a one-sided sequent of that formula alone, dualised
    /// if `polarity` is false.
    fn from((t, polarity): (Term<'a>, bool)) -> Self {
        /// Pushes `t` and its subterms onto the arena, dualised if `polarity` is
        /// false, and returns the index of `t`.
        fn recursion_helper<'a>(
            t: Term<'a>,
            polarity: bool,
            term_arena: &mut Vec<<LL as Logic<usize>>::Expression>,
            variable_dict: &mut Vec<String>,
        ) -> usize {
            use LLExpression as E;
            use Term::*;
            if let Dual(nt) = t {
                recursion_helper(*nt, !polarity, term_arena, variable_dict)
            } else {
                let e = match t {
                    Var(s) => {
                        let var_index = variable_dict.len();
                        variable_dict.push(s.to_string());
                        E::Var(var_index)
                    }
                    One => E::One,
                    Bot => E::Bot,
                    Top => E::Top,
                    Zero => E::Zero,
                    Bang(nt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        E::Bang(n)
                    }
                    Quest(nt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        E::Quest(n)
                    }
                    Tensor(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        E::Tensor(n, m)
                    }
                    Par(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        E::Par(n, m)
                    }
                    With(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        E::With(n, m)
                    }
                    Plus(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        E::Plus(n, m)
                    }
                    Lollipop(nt, mt) => {
                        // Lollipop is a Par where the first element has its polarity inverted
                        let n = recursion_helper(*nt, !polarity, term_arena, variable_dict);
                        let m = recursion_helper(*mt, polarity, term_arena, variable_dict);
                        E::Par(n, m)
                    }
                    Dual(nt) => unreachable!(),
                };

                let index = term_arena.len();
                let e = if polarity { e } else { e.dualize() };
                term_arena.push(e);
                index
            }
        }

        let mut term_arena = Vec::<<LL as Logic<usize>>::Expression>::new();
        let mut variable_dict = Vec::<String>::new();
        let index = recursion_helper(t, polarity, &mut term_arena, &mut variable_dict);

        Self {
            term_arena,
            term_ids: vec![index],
            variable_dict,
        }
    }
}

impl<'a> From<Sequent<'a>> for Seq<usize, LL> {
    /// Lowers a parsed two-sided sequent into an optimized one-sided one, with
    /// the left side dualised.
    fn from(s: Sequent<'a>) -> Self {
        let lhs_terms = s.left.into_iter().map(|t| (t, false));
        let rhs_terms = s.right.into_iter().map(|t| (t, true));
        let mut sequent = Seq::new();
        lhs_terms
            .chain(rhs_terms)
            .map(Seq::from)
            .for_each(|s| sequent.add(s));
        sequent.optimize().unwrap();
        sequent
    }
}

impl std::str::FromStr for Seq<usize, LL> {
    type Err = crate::Error;

    /// Parses a two-sided sequent such as `A, B |- A * B`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Seq::from(Sequent::try_from(s)?))
    }
}
