// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::sequents::{Sequent, Term, TermId};
use chumsky::pratt::*;
use chumsky::prelude::*;

/// A parsed formula, before it is lowered into an arena.
#[derive(Debug)]
pub(super) enum Tree<'a> {
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
    Dual(Box<Tree<'a>>),
    /// `!A`
    Bang(Box<Tree<'a>>),
    /// `?A`
    Quest(Box<Tree<'a>>),
    /// `A ⊗ B`
    Tensor(Box<Tree<'a>>, Box<Tree<'a>>),
    /// `A ⅋ B`
    Par(Box<Tree<'a>>, Box<Tree<'a>>),
    /// `A & B`
    With(Box<Tree<'a>>, Box<Tree<'a>>),
    /// `A ⊕ B`
    Plus(Box<Tree<'a>>, Box<Tree<'a>>),
    /// `A ⊸ B`
    Lollipop(Box<Tree<'a>>, Box<Tree<'a>>),
}

/// A parsed two-sided sequent, `left ⊢ right`.
#[derive(Debug)]
pub(super) struct TwoSided<'a> {
    /// The formulas left of the turnstile.
    pub(super) left: Vec<Tree<'a>>,
    /// The formulas right of the turnstile.
    pub(super) right: Vec<Tree<'a>>,
}

/// Parses a unit constant: `0`, `1`, `bot`/`⊥` or `top`/`⊤`.
fn constant_parser<'a>() -> impl Parser<'a, &'a str, Tree<'a>, extra::Err<Simple<'a, char>>> + Clone
{
    // `text::keyword` only matches identifiers, so digits and symbols need `just`.
    choice((
        just('0').map(|_| Tree::Zero),
        just('1').map(|_| Tree::One),
        text::keyword("bot").map(|_| Tree::Bot),
        just('⊥').map(|_| Tree::Bot),
        text::keyword("top").map(|_| Tree::Top),
        just('⊤').map(|_| Tree::Top),
    ))
}

/// Parses a variable name: a Unicode identifier.
fn variable_parser<'a>() -> impl Parser<'a, &'a str, Tree<'a>, extra::Err<Simple<'a, char>>> + Clone
{
    text::ident().map(Tree::Var)
}

/// Parses a formula, with its operators' precedence and associativity.
fn term_parser<'a>() -> impl Parser<'a, &'a str, Tree<'a>, extra::Err<Simple<'a, char>>> + Clone {
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
            postfix(7, just('^').padded(), |lhs, _, _| Tree::Dual(Box::new(lhs))),
            prefix(6, just('~').padded(), |_, rhs, _| Tree::Dual(Box::new(rhs))),
            prefix(6, just('!').padded(), |_, rhs, _| Tree::Bang(Box::new(rhs))),
            prefix(6, just('?').padded(), |_, rhs, _| {
                Tree::Quest(Box::new(rhs))
            }),
            // Tensor
            infix(left(5), just('*').padded(), |l, _, r, _| {
                Tree::Tensor(Box::new(l), Box::new(r))
            }),
            infix(left(5), just('⊗').padded(), |l, _, r, _| {
                Tree::Tensor(Box::new(l), Box::new(r))
            }),
            // Par
            infix(left(4), text::keyword("par").padded(), |l, _, r, _| {
                Tree::Par(Box::new(l), Box::new(r))
            }),
            infix(left(4), just('|').padded(), |l, _, r, _| {
                Tree::Par(Box::new(l), Box::new(r))
            }),
            infix(left(4), just('⅋').padded(), |l, _, r, _| {
                Tree::Par(Box::new(l), Box::new(r))
            }),
            // With
            infix(left(3), just('&').padded(), |l, _, r, _| {
                Tree::With(Box::new(l), Box::new(r))
            }),
            // Plus
            infix(left(2), just('+').padded(), |l, _, r, _| {
                Tree::Plus(Box::new(l), Box::new(r))
            }),
            infix(left(2), just('⊕').padded(), |l, _, r, _| {
                Tree::Plus(Box::new(l), Box::new(r))
            }),
            // Lollipop
            infix(right(1), just("-o").padded(), |l, _, r, _| {
                Tree::Lollipop(Box::new(l), Box::new(r))
            }),
            infix(right(1), just('⊸').padded(), |l, _, r, _| {
                Tree::Lollipop(Box::new(l), Box::new(r))
            }),
        ))
    })
}

/// Parses a two-sided sequent, `Γ |- Δ`, which must span the whole input.
fn sequent_parser<'a>() -> impl Parser<'a, &'a str, TwoSided<'a>, extra::Err<Simple<'a, char>>> {
    let terms_list = term_parser()
        .clone()
        .separated_by(just(',').padded())
        .collect::<Vec<Tree<'a>>>();

    let tack = choice((just("|-"), just("⊢"))).padded();

    terms_list
        .clone() // Parse LHS
        .then(tack) // Ignore the tack (but consume it)
        .then(terms_list) // Parse RHS
        .map(|((left, _), right)| TwoSided { left, right })
        .then_ignore(end()) // Ensure the parser consumes the entire input string
}

impl<'a> TryFrom<&'a str> for TwoSided<'a> {
    type Error = crate::Error;

    /// Parses `input` into a syntax tree that borrows its variable names, or
    /// returns every parse error.
    fn try_from(input: &'a str) -> Result<TwoSided<'a>, Self::Error> {
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

impl<'a> From<(Tree<'a>, bool)> for Sequent {
    /// Lowers a formula into a one-sided sequent of that formula alone, dualised
    /// if `polarity` is false.
    fn from((t, polarity): (Tree<'a>, bool)) -> Self {
        /// Pushes `t` and its subterms onto the arena, dualised if `polarity` is
        /// false, and returns the index of `t`.
        fn recursion_helper<'a>(
            t: Tree<'a>,
            polarity: bool,
            terms: &mut Vec<Term>,
            atoms: &mut Vec<String>,
        ) -> TermId {
            use Term as E;
            use Tree::*;
            if let Dual(nt) = t {
                recursion_helper(*nt, !polarity, terms, atoms)
            } else {
                let e = match t {
                    Var(s) => {
                        let atom = crate::sequents::Atom::new(atoms.len() as u32);
                        atoms.push(s.to_string());
                        E::Var(atom)
                    }
                    One => E::One,
                    Bot => E::Bot,
                    Top => E::Top,
                    Zero => E::Zero,
                    Bang(nt) => {
                        let n = recursion_helper(*nt, polarity, terms, atoms);
                        E::Bang(n)
                    }
                    Quest(nt) => {
                        let n = recursion_helper(*nt, polarity, terms, atoms);
                        E::Quest(n)
                    }
                    Tensor(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, terms, atoms);
                        let m = recursion_helper(*mt, polarity, terms, atoms);
                        E::Tensor(n, m)
                    }
                    Par(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, terms, atoms);
                        let m = recursion_helper(*mt, polarity, terms, atoms);
                        E::Par(n, m)
                    }
                    With(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, terms, atoms);
                        let m = recursion_helper(*mt, polarity, terms, atoms);
                        E::With(n, m)
                    }
                    Plus(nt, mt) => {
                        let n = recursion_helper(*nt, polarity, terms, atoms);
                        let m = recursion_helper(*mt, polarity, terms, atoms);
                        E::Plus(n, m)
                    }
                    Lollipop(nt, mt) => {
                        // Lollipop is a Par where the first element has its polarity inverted
                        let n = recursion_helper(*nt, !polarity, terms, atoms);
                        let m = recursion_helper(*mt, polarity, terms, atoms);
                        E::Par(n, m)
                    }
                    Dual(_) => unreachable!(),
                };

                let index = TermId::new(terms.len() as u32);
                let e = if polarity { e } else { e.dual() };
                terms.push(e);
                index
            }
        }

        let mut terms = Vec::<Term>::new();
        let mut atoms = Vec::<String>::new();
        let index = recursion_helper(t, polarity, &mut terms, &mut atoms);

        Self {
            terms,
            roots: vec![index],
            atoms,
        }
    }
}

impl<'a> From<TwoSided<'a>> for Sequent {
    /// Lowers a parsed two-sided sequent into an optimized one-sided one, with
    /// the left side dualised.
    fn from(s: TwoSided<'a>) -> Self {
        let lhs_terms = s.left.into_iter().map(|t| (t, false));
        let rhs_terms = s.right.into_iter().map(|t| (t, true));
        let mut sequent = Sequent::new();
        lhs_terms
            .chain(rhs_terms)
            .map(Sequent::from)
            .for_each(|s| sequent.add(s));
        sequent.optimize().unwrap();
        sequent
    }
}

impl std::str::FromStr for Sequent {
    type Err = crate::Error;

    /// Parses a two-sided sequent such as `A, B |- A * B`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Sequent::from(TwoSided::try_from(s)?))
    }
}
