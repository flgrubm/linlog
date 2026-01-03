// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use chumsky::pratt::*;
use chumsky::prelude::*;

enum Term<'a> {
    Var(&'a str),
    Zero,
    One,
    Bot,
    Top,
    Neg(Box<Term<'a>>),
    Bang(Box<Term<'a>>),
    Quest(Box<Term<'a>>),
    Tensor(Box<Term<'a>>, Box<Term<'a>>),
    Par(Box<Term<'a>>, Box<Term<'a>>),
    With(Box<Term<'a>>, Box<Term<'a>>),
    Plus(Box<Term<'a>>, Box<Term<'a>>),
    Lollipop(Box<Term<'a>>, Box<Term<'a>>),
}

struct Sequent<'a> {
    left: Vec<Term<'a>>,
    right: Vec<Term<'a>>,
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
            // Unaties
            postfix(7, just('^'), |lhs, _, _| Term::Neg(Box::new(lhs))),
            prefix(6, just('~'), |_, rhs, _| Term::Neg(Box::new(rhs))),
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
