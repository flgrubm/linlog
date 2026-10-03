// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A precedence parser that keeps its own stack and writes the terms it
//! reads straight into the arena, so that neither the nesting of a formula
//! nor its length costs any of the caller's stack.
//!
//! A sequent is `formulas ⊢ formulas`, each side a list separated by commas
//! and possibly empty, with whitespace allowed between any two tokens. From
//! the loosest binding to the tightest, a formula is built with `-o`/`⊸`
//! (to the right), `+`/`⊕`, `&`, `|`/`par`/`⅋`, `*`/`⊗` (these to the
//! left), the prefix operators `~`, `!` and `?`, and the postfix `^`, over
//! the constants `0`, `1`, `bot`/`⊥` and `top`/`⊤`, variables and
//! parentheses. A variable is a Unicode identifier (it starts with `_` or a
//! character of `XID_Start` and goes on with characters of `XID_Continue`);
//! `bot`, `top` and `par` are that constant or connective only as whole
//! identifiers, and `par` only where a connective can stand, so that it is
//! a variable where a formula starts.
//!
//! The arena holds the formulas in negation normal form, one-sided: the
//! formulas left of the turnstile are negated, `A ⊸ B` is `A^⊥ ⅋ B`, and a
//! negation is pushed to the atoms. The parser writes every term as it
//! stands in the text and notes where a negation applies; one pass from
//! the last term to the first then dualises what stands under an odd
//! number of them.

use crate::Error;
use crate::errors::ParseError;
use crate::hash::HashMap;
use crate::occurrences::Forest;
use crate::sequents::{Atom, Sequent, Term, TermId};

/// A binary connective as it is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Binary {
    /// `*` or `⊗`
    Tensor,
    /// `|`, `par` or `⅋`
    Par,
    /// `&`
    With,
    /// `+` or `⊕`
    Plus,
    /// `-o` or `⊸`
    Lollipop,
}

impl Binary {
    /// Returns how tightly the connective binds: of two connectives around
    /// an operand, the one with the larger number takes it.
    const fn strength(self) -> u8 {
        match self {
            Binary::Tensor => 5,
            Binary::Par => 4,
            Binary::With => 3,
            Binary::Plus => 2,
            Binary::Lollipop => 1,
        }
    }

    /// Returns whether an operand between this connective on its left and
    /// `next` on its right is this one's: it binds tighter, or the two are
    /// the same connective and it associates to the left, as every one but
    /// `⊸` does.
    fn takes_before(self, next: Binary) -> bool {
        self.strength() > next.strength() || (self == next && self != Binary::Lollipop)
    }
}

/// What has been read and waits for the operand to its right to be
/// complete.
#[derive(Clone, Copy, Debug)]
enum Pending {
    /// `(`, which waits for its `)`.
    Open,
    /// `~`
    Dual,
    /// `!`
    Bang,
    /// `?`
    Quest,
    /// A binary connective with its left operand.
    Binary(Binary, TermId),
}

/// What the parser reads next.
enum State {
    /// Something a formula starts with.
    Operand,
    /// Something that follows the operand just read, which is the term
    /// given.
    Operator(TermId),
    /// Nothing: the sequent is complete.
    End,
}

/// Returns whether an identifier may start with `c`.
fn starts_identifier(c: char) -> bool {
    c == '_' || unicode_ident::is_xid_start(c)
}

/// A sequent in the reading: the text and the place in it, the arena so
/// far, and what waits for its right operand.
struct Parser<'a> {
    /// The text of the sequent.
    input: &'a str,
    /// The byte offset of the next character to read.
    at: usize,
    /// Whether the formulas being read stand left of the turnstile.
    left: bool,
    /// Whether nothing of this side of the turnstile has been read yet, so
    /// that the side may still turn out to have no formula.
    empty: bool,
    /// The terms read so far, each as it stands in the text.
    terms: Vec<Term>,
    /// Per term, whether the text negates it an odd number of times: by a
    /// `~` or a `^` on it, as the antecedent of a `⊸`, or as a formula
    /// left of the turnstile. The negations of the terms above it are not
    /// counted.
    negated: Vec<bool>,
    /// The formulas of the sequent, in the order of the text.
    roots: Vec<TermId>,
    /// The names of the atoms, in the order they first occur.
    atoms: Vec<String>,
    /// The atom of every name read so far.
    names: HashMap<&'a str, Atom>,
    /// The prefix operators, parentheses and binary connectives whose
    /// right operand is being read, the innermost last.
    pending: Vec<Pending>,
    /// How many parentheses are open.
    open: usize,
    /// The most terms the arena may hold.
    most: u64,
}

impl<'a> Parser<'a> {
    /// Returns a parser at the start of `input`, for a sequent of at most
    /// `most` terms.
    fn new(input: &'a str, most: u64) -> Self {
        Self {
            input,
            at: 0,
            left: true,
            empty: true,
            terms: Vec::new(),
            negated: Vec::new(),
            roots: Vec::new(),
            atoms: Vec::new(),
            names: HashMap::default(),
            pending: Vec::new(),
            open: 0,
            most,
        }
    }

    /// Returns the next character, or `None` at the end of the input.
    fn peek(&self) -> Option<char> {
        self.input[self.at..].chars().next()
    }

    /// Returns the error for the character at byte `at`, or for the end of
    /// the input there.
    fn unexpected(&self, at: usize) -> Error {
        let found = self.input[at..].chars().next();
        Error::SequentParsing(vec![ParseError {
            span: at..at + found.map_or(0, char::len_utf8),
            found: found.map(String::from),
            label: None,
            expected: Vec::new(),
        }])
    }

    /// Appends a term to the arena and returns its index, or fails when
    /// the arena is full.
    fn push(&mut self, term: Term) -> Result<TermId, Error> {
        let index = self.terms.len() as u64;
        if index >= self.most {
            // Every term of the text is an occurrence of its own.
            return Err(Error::TooManyOccurrences {
                occurrences: self.most.saturating_add(1),
                limit: self.most,
            });
        }
        self.terms.push(term);
        self.negated.push(false);
        Ok(TermId::new(index as u32))
    }

    /// Notes one more negation of a term.
    fn negate(&mut self, term: TermId) {
        self.negated[term.index()] ^= true;
    }

    /// Reads the rest of the identifier that starts at byte `start` and
    /// whose first character has been read, and returns it whole.
    fn identifier(&mut self, start: usize) -> &'a str {
        let rest = &self.input[self.at..];
        let end = rest
            .find(|c| !unicode_ident::is_xid_continue(c))
            .unwrap_or(rest.len());
        self.at += end;
        &self.input[start..self.at]
    }

    /// Returns the atom called `name`, a new one if the name is.
    fn atom(&mut self, name: &'a str) -> Atom {
        let fresh = Atom::new(self.atoms.len() as u32);
        *self.names.entry(name).or_insert_with(|| {
            self.atoms.push(name.to_string());
            fresh
        })
    }

    /// Gives `operand` to the pending operators that take it before the
    /// connective `next` does, or to all of them up to the innermost open
    /// parenthesis for `None`, the innermost first, and returns the term
    /// they make of it.
    fn reduce(&mut self, mut operand: TermId, next: Option<Binary>) -> Result<TermId, Error> {
        while let Some(&top) = self.pending.last() {
            let term = match top {
                Pending::Open => break,
                Pending::Dual => {
                    self.negate(operand);
                    self.pending.pop();
                    continue;
                }
                Pending::Bang => Term::Bang(operand),
                Pending::Quest => Term::Quest(operand),
                Pending::Binary(connective, left) => {
                    if next.is_some_and(|next| !connective.takes_before(next)) {
                        break;
                    }
                    match connective {
                        Binary::Tensor => Term::Tensor(left, operand),
                        Binary::Par => Term::Par(left, operand),
                        Binary::With => Term::With(left, operand),
                        Binary::Plus => Term::Plus(left, operand),
                        Binary::Lollipop => {
                            self.negate(left);
                            Term::Par(left, operand)
                        }
                    }
                }
            };
            self.pending.pop();
            operand = self.push(term)?;
        }
        Ok(operand)
    }

    /// Completes a formula of the sequent, outside every parenthesis, with
    /// its last operand.
    fn root(&mut self, operand: TermId) -> Result<(), Error> {
        let root = self.reduce(operand, None)?;
        if self.left {
            self.negate(root);
        }
        self.roots.push(root);
        Ok(())
    }

    /// Passes the turnstile: what follows is the right side, of which
    /// nothing has been read.
    fn turnstile(&mut self) -> State {
        self.left = false;
        self.empty = true;
        State::Operand
    }

    /// Notes an operator that stands before its operand, or an open
    /// parenthesis.
    fn prefix(&mut self, operator: Pending) -> State {
        self.pending.push(operator);
        self.empty = false;
        State::Operand
    }

    /// Reads what a formula starts with: a prefix operator, an open
    /// parenthesis, a constant or a variable. An empty left side may have
    /// the turnstile there and an empty right side the end of the input.
    fn operand(&mut self) -> Result<State, Error> {
        let start = self.at;
        let Some(c) = self.peek() else {
            return if self.empty && !self.left {
                Ok(State::End)
            } else {
                Err(self.unexpected(start))
            };
        };
        self.at += c.len_utf8();
        let term = match c {
            '~' => return Ok(self.prefix(Pending::Dual)),
            '!' => return Ok(self.prefix(Pending::Bang)),
            '?' => return Ok(self.prefix(Pending::Quest)),
            '(' => {
                self.open += 1;
                return Ok(self.prefix(Pending::Open));
            }
            '0' => Term::Zero,
            '1' => Term::One,
            '⊥' => Term::Bot,
            '⊤' => Term::Top,
            '⊢' if self.empty && self.left => return Ok(self.turnstile()),
            '|' if self.empty && self.left => {
                // Only `|-` can start a sequent with `|`, so what is wrong
                // is the character after it.
                if self.peek() != Some('-') {
                    return Err(self.unexpected(self.at));
                }
                self.at += 1;
                return Ok(self.turnstile());
            }
            _ if starts_identifier(c) => match self.identifier(start) {
                "bot" => Term::Bot,
                "top" => Term::Top,
                name => Term::Var(self.atom(name)),
            },
            _ => return Err(self.unexpected(start)),
        };
        self.empty = false;
        Ok(State::Operator(self.push(term)?))
    }

    /// Reads what follows the operand just read: a postfix `^`, a binary
    /// connective, a closing parenthesis, or outside every parenthesis a
    /// comma, the turnstile on the left side and the end of the input on
    /// the right side.
    fn operator(&mut self, operand: TermId) -> Result<State, Error> {
        let start = self.at;
        let outermost = self.open == 0;
        let Some(c) = self.peek() else {
            if self.left || !outermost {
                return Err(self.unexpected(start));
            }
            self.root(operand)?;
            return Ok(State::End);
        };
        self.at += c.len_utf8();
        let connective = match c {
            '^' => {
                self.negate(operand);
                return Ok(State::Operator(operand));
            }
            '*' | '⊗' => Binary::Tensor,
            '⅋' => Binary::Par,
            '&' => Binary::With,
            '+' | '⊕' => Binary::Plus,
            '⊸' => Binary::Lollipop,
            '-' => {
                // Only `-o` starts with `-`, so what is wrong is the
                // character after it.
                if self.peek() != Some('o') {
                    return Err(self.unexpected(self.at));
                }
                self.at += 1;
                Binary::Lollipop
            }
            '|' if self.left && outermost && self.peek() == Some('-') => {
                self.at += 1;
                self.root(operand)?;
                return Ok(self.turnstile());
            }
            // Where no turnstile can stand, `|-` is a `|` before a `-`,
            // which starts no formula and is reported as that.
            '|' => Binary::Par,
            '⊢' if self.left && outermost => {
                self.root(operand)?;
                return Ok(self.turnstile());
            }
            ',' if outermost => {
                self.root(operand)?;
                return Ok(State::Operand);
            }
            ')' if !outermost => {
                let inner = self.reduce(operand, None)?;
                // What stopped the reduction is the open parenthesis.
                self.pending.pop();
                self.open -= 1;
                return Ok(State::Operator(inner));
            }
            _ if starts_identifier(c) && self.identifier(start) == "par" => Binary::Par,
            _ => return Err(self.unexpected(start)),
        };
        let left = self.reduce(operand, Some(connective))?;
        self.pending.push(Pending::Binary(connective, left));
        Ok(State::Operand)
    }

    /// Reads the whole input as a sequent.
    fn sequent(mut self) -> Result<Sequent, Error> {
        let mut state = State::Operand;
        loop {
            let blank = &self.input[self.at..];
            self.at += blank.len() - blank.trim_start().len();
            state = match state {
                State::Operand => self.operand()?,
                State::Operator(operand) => self.operator(operand)?,
                State::End => return self.finish(),
            };
        }
    }

    /// Applies the negations noted and returns the sequent, optimized.
    fn finish(mut self) -> Result<Sequent, Error> {
        // A term comes after its subterms, so one pass from the last term
        // sees every negation of a term before it reaches the term.
        for n in (0..self.terms.len()).rev() {
            if self.negated[n] {
                let term = self.terms[n].dual();
                self.terms[n] = term;
                for k in term.subterms() {
                    self.negated[k.index()] ^= true;
                }
            }
        }
        let mut sequent = Sequent {
            terms: self.terms,
            roots: self.roots,
            atoms: self.atoms,
        };
        sequent.optimize()?;
        Ok(sequent)
    }
}

impl std::str::FromStr for Sequent {
    type Err = Error;

    /// Parses a two-sided sequent such as `A, B |- A * B` into a one-sided
    /// one, optimized. The nesting of its formulas may be of any depth; a
    /// sequent of more subformulas than a forest can index is refused.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::new(s, Forest::MOST).sequent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sequent of more terms than the arena may hold is refused, and one
    /// that fills it is read.
    #[test]
    fn refuses_more_terms_than_the_arena_holds() {
        let input = "A |- !A * B";
        assert_eq!(Parser::new(input, 5).sequent().unwrap().terms().len(), 5);
        assert!(matches!(
            Parser::new(input, 4).sequent(),
            Err(Error::TooManyOccurrences {
                occurrences: 5,
                limit: 4
            })
        ));
    }
}
