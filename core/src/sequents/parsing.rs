// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use chumsky::pratt::*;
use chumsky::prelude::*;
use std::collections::HashMap;

use super::modes as Mode;
use super::symbols::{LL, Symbol};

#[derive(Debug)]
enum Term<'a> {
    Var(&'a str),
    Zero,
    One,
    Bot,
    Top,
    Dual(Box<Term<'a>>),
    Bang(Box<Term<'a>>),
    Quest(Box<Term<'a>>),
    Tensor(Box<Term<'a>>, Box<Term<'a>>),
    Par(Box<Term<'a>>, Box<Term<'a>>),
    With(Box<Term<'a>>, Box<Term<'a>>),
    Plus(Box<Term<'a>>, Box<Term<'a>>),
    Lollipop(Box<Term<'a>>, Box<Term<'a>>),
}

#[derive(Debug)]
struct Sequent<'a> {
    left: Vec<Term<'a>>,
    right: Vec<Term<'a>>,
}

struct RawSequent {
    lhs_symbols: Box<[Symbol<LL, Mode::Rich>]>,
    lhs_terms: Box<[usize]>,
    rhs_symbols: Box<[Symbol<LL, Mode::Rich>]>,
    rhs_terms: Box<[usize]>,
    variable_names: Box<[String]>,
}

fn constant_parser<'a>() -> impl Parser<'a, &'a str, Term<'a>, extra::Err<Simple<'a, char>>> + Clone
{
    choice((
        text::keyword("0").map(|_| Term::Zero),
        text::keyword("1").map(|_| Term::One),
        text::keyword("bot").map(|_| Term::Bot),
        text::keyword("⊥").map(|_| Term::Bot),
        text::keyword("top").map(|_| Term::Top),
        text::keyword("⊤").map(|_| Term::Top),
    ))
}

fn variable_parser<'a>() -> impl Parser<'a, &'a str, Term<'a>, extra::Err<Simple<'a, char>>> + Clone
{
    text::ident().map(Term::Var)
}

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
            postfix(7, just('^'), |lhs, _, _| Term::Dual(Box::new(lhs))),
            prefix(6, just('~'), |_, rhs, _| Term::Dual(Box::new(rhs))),
            prefix(6, just('!'), |_, rhs, _| Term::Bang(Box::new(rhs))),
            prefix(6, just('?'), |_, rhs, _| Term::Quest(Box::new(rhs))),
            // Tensor
            infix(left(5), just('*'), |l, _, r, _| {
                Term::Tensor(Box::new(l), Box::new(r))
            }),
            infix(left(5), just('⊗'), |l, _, r, _| {
                Term::Tensor(Box::new(l), Box::new(r))
            }),
            // Par
            infix(left(4), text::keyword("par"), |l, _, r, _| {
                Term::Par(Box::new(l), Box::new(r))
            }),
            infix(left(4), just('|'), |l, _, r, _| {
                Term::Par(Box::new(l), Box::new(r))
            }),
            infix(left(4), just('⅋'), |l, _, r, _| {
                Term::Par(Box::new(l), Box::new(r))
            }),
            // With
            infix(left(3), just('&'), |l, _, r, _| {
                Term::With(Box::new(l), Box::new(r))
            }),
            // Plus
            infix(left(2), just('+'), |l, _, r, _| {
                Term::Plus(Box::new(l), Box::new(r))
            }),
            infix(left(2), just('⊕'), |l, _, r, _| {
                Term::Plus(Box::new(l), Box::new(r))
            }),
            // Lollipop
            infix(right(1), just("-o"), |l, _, r, _| {
                Term::Lollipop(Box::new(l), Box::new(r))
            }),
            infix(right(0), just('⊸'), |l, _, r, _| {
                Term::Lollipop(Box::new(l), Box::new(r))
            }),
        ))
    })
}

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

fn flatten_append_term<'a>(
    (mut symbols, mut indices, mut variable_names, mut variable_map): (
        Vec<Symbol<LL, Mode::Rich>>,
        Vec<usize>,
        Vec<&'a str>,
        HashMap<&'a str, usize>,
    ),
    t: Term<'a>,
) -> (
    Vec<Symbol<LL, Mode::Rich>>,
    Vec<usize>,
    Vec<&'a str>,
    HashMap<&'a str, usize>,
) {
    use Term::*;

    let mut stack = vec![Box::new(t)];
    indices.push(symbols.len());

    while let Some(tm) = stack.pop() {
        match *tm {
            Var(s) => {
                if let Some(s_id) = variable_map.get(s) {
                    symbols.push(Symbol::Var(*s_id));
                } else {
                    let fresh_variable = variable_names.len();
                    let _ = variable_map.insert(s, fresh_variable);
                    symbols.push(Symbol::Var(fresh_variable));
                    variable_names.push(s);
                }
            }
            One => symbols.push(Symbol::One(())),
            Bot => symbols.push(Symbol::Bot(())),
            Top => symbols.push(Symbol::Top(())),
            Zero => symbols.push(Symbol::Zero(())),
            Dual(st) => {
                stack.push(st);
                symbols.push(Symbol::Dual(()));
            }
            Bang(st) => {
                stack.push(st);
                symbols.push(Symbol::Bang(()));
            }
            Quest(st) => {
                stack.push(st);
                symbols.push(Symbol::Quest(()));
            }
            Tensor(st1, st2) => {
                stack.push(st2);
                stack.push(st1);
                symbols.push(Symbol::Tensor(()));
            }
            Par(st1, st2) => {
                stack.push(st2);
                stack.push(st1);
                symbols.push(Symbol::Par(()));
            }
            With(st1, st2) => {
                stack.push(st2);
                stack.push(st1);
                symbols.push(Symbol::With(()));
            }
            Plus(st1, st2) => {
                stack.push(st2);
                stack.push(st1);
                symbols.push(Symbol::Plus(()));
            }
            Lollipop(st1, st2) => {
                stack.push(st2);
                stack.push(st1);
                symbols.push(Symbol::Lollipop(()));
            }
        }
    }

    (symbols, indices, variable_names, variable_map)
}

fn flatten_terms<'a>(
    terms: Vec<Term<'a>>,
    variable_names: Vec<&'a str>,
    variable_map: HashMap<&'a str, usize>,
) -> (
    Box<[Symbol<LL, Mode::Rich>]>,
    Box<[usize]>,
    Vec<&'a str>,
    HashMap<&'a str, usize>,
) {
    let symbols = Vec::new();
    let indices = Vec::with_capacity(terms.len() - 1);
    let (symbols, indices, variable_names, variable_map) = terms.into_iter().fold(
        (symbols, indices, variable_names, variable_map),
        flatten_append_term,
    );
    (
        symbols.into_boxed_slice(),
        indices.into_boxed_slice(),
        variable_names,
        variable_map,
    )
}

impl<'a> From<Sequent<'a>> for RawSequent {
    fn from(s: Sequent<'a>) -> RawSequent {
        let variable_names_borrowed = Vec::new();
        let variable_map = HashMap::new();
        let (lhs_symbols, lhs_terms, variable_names_borrowed, variable_map) =
            flatten_terms(s.left, variable_names_borrowed, variable_map);
        let (rhs_symbols, rhs_terms, variable_names_borrowed, _) =
            flatten_terms(s.right, variable_names_borrowed, variable_map);
        let variable_names: Box<[String]> = variable_names_borrowed
            .into_iter()
            .map(|x| x.to_string())
            .collect();

        RawSequent {
            lhs_symbols,
            lhs_terms,
            rhs_symbols,
            rhs_terms,
            variable_names,
        }
    }
}

fn parse_raw<'a>(input: &'a str) -> Result<RawSequent, crate::Error> {
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
        .map(RawSequent::from)
}
