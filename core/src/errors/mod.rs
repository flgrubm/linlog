// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// The error type for text that is not a sequent.
#[cfg(feature = "parse")]
mod parse;

#[cfg(feature = "parse")]
pub use parse::ParseError;

use crate::fragment::{Fragment, Mode};
use crate::nets::NetError;
use crate::occurrences::ShapeError;
use crate::proofs::CheckError;
use crate::search::Engine;
use thiserror::Error;

/// Everything that can go wrong in this crate.
#[derive(Error, Debug)]
pub enum Error {
    /// A term refers to a variable index (first) outside the variable
    /// dictionary (its length second).
    #[error("a term refers to atom {0}, but the atom list has {1} names")]
    InvalidVariableIndex(usize, usize),
    /// A root formula index (first) lies outside the arena (its length second).
    #[error("a root formula is term {0}, but the arena has {1} terms")]
    TermIndexOutOfBounds(usize, usize),
    /// A subterm index (first) is not below the index of its parent term
    /// (second).
    #[error("term {1} refers to term {0}, but a subterm must come before the terms that use it")]
    SubtermIndexNotDecreasing(usize, usize),
    /// A sequent has at least this many subformula occurrences, more than a
    /// forest can index.
    #[error(
        "the sequent unfolds to at least {0} subformula occurrences, more than a forest can index (2³² − 1)"
    )]
    TooManyOccurrences(u64),
    /// The input is not a sequent, for each of the reasons listed.
    #[cfg(feature = "parse")]
    #[error("cannot parse the sequent: {}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join("; "))]
    SequentParsing(Vec<ParseError>),
    /// A proof node index (first) lies outside the arena (its length second).
    #[error("a proof refers to node {0}, but it has {1} nodes")]
    NodeIndexOutOfBounds(usize, usize),
    /// A premise's node index (first) is not below the index of the node it
    /// proves (second).
    #[error("proof node {1} has premise {0}, but a premise must come before the nodes that use it")]
    PremiseIndexNotDecreasing(usize, usize),
    /// A proof node refers to an occurrence (first) outside its forest (its
    /// length second).
    #[error("a proof node refers to occurrence {0}, but the sequent has {1} occurrences")]
    OccurrenceIndexOutOfBounds(usize, usize),
    /// A proof does not prove its sequent.
    #[error("invalid proof: {0}")]
    InvalidProof(#[from] CheckError),
    /// A list of links is not a proof structure, or a structure is not a
    /// proof net.
    #[error("not a proof net: {0}")]
    InvalidNet(#[from] NetError),
    /// Proof nets exist for unit-free MLL only, and the sequent lies in a
    /// larger fragment.
    #[error("proof nets exist for MLL without units only, not for {0}")]
    NetFragment(Fragment),
    /// Proof nets exist in classical mode only, and the mode is affine.
    #[error("proof nets exist in classical mode only, with or without Mix, not in {0} mode")]
    NetMode(Mode),
    /// The mode is intuitionistic and the sequent has no intuitionistic
    /// reading. The message names occurrences by id; [`ShapeError::describe`]
    /// names them by formula.
    #[error("not an intuitionistic sequent: {0}")]
    NotIntuitionistic(ShapeError),
    /// The engine forced by the options does not search in the mode: the
    /// two-sided engine is for intuitionistic mode, the focus engine for
    /// classical mode.
    #[error("the {engine} engine does not search in {mode} mode")]
    EngineMode {
        /// The engine the options force.
        engine: Engine,
        /// The mode the search was asked for.
        mode: Mode,
    },
    /// The additive engine, forced by the options, decides only sequents of
    /// exactly two additive-only formulas.
    #[error(
        "the additive engine decides a sequent of two additive-only formulas, not {roots} formulas of {fragment}"
    )]
    NotAdditive {
        /// The fragment the sequent was searched in.
        fragment: Fragment,
        /// How many formulas the sequent has.
        roots: usize,
    },
    /// Mix was asked for in intuitionistic mode, where it has no form: a
    /// premise of a Mix would have no goal.
    #[error("Mix has no intuitionistic form: a premise of a Mix would have no goal")]
    IntuitionisticMix,
    /// An intuitionistic goal has this many formulas on the right of `⊢`,
    /// where a sequent of intuitionistic linear logic has exactly one.
    #[error("an intuitionistic goal has {0} formulas on the right of ⊢ instead of one")]
    GoalOutputs(usize),
    /// The net engine, forced by the options, decides the sequent's roots
    /// only, not a goal deeper in the forest.
    #[error("the net engine decides the whole sequent only, not a goal within it")]
    NetGoal,
    /// No engine handles the fragment in the mode yet.
    #[error("no engine for {fragment} in {mode} mode yet")]
    NoEngine {
        /// The fragment the search was asked for.
        fragment: Fragment,
        /// The mode the search was asked for.
        mode: Mode,
    },
    /// The sequent uses connectives outside the fragment the search options
    /// assert.
    #[error("the sequent lies in {detected}, outside the asserted fragment {asserted}")]
    FragmentMismatch {
        /// The fragment the options assert.
        asserted: Fragment,
        /// The fragment the sequent was detected to lie in.
        detected: Fragment,
    },
}
