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
    /// A sequent has more subformula occurrences than the limit its forest
    /// was to be built within, or than the parser reads.
    #[error(
        "the sequent unfolds to at least {occurrences} subformula occurrences, more than the \
         limit of {limit}; Forest::within builds a forest within another limit, of at most {}",
        crate::occurrences::Forest::MOST
    )]
    TooManyOccurrences {
        /// How many occurrences the sequent has, or a number of them at
        /// which the count ended: the parser stops at the first beyond
        /// the limit, and the count of an arena saturates at `u64::MAX`.
        occurrences: u64,
        /// The most occurrences that were allowed.
        limit: u64,
    },
    /// The input is not a sequent, for each of the reasons listed.
    #[cfg(feature = "parse")]
    #[error("cannot parse the sequent: {}", .0.iter().map(ToString::to_string).collect::<Vec<_>>().join("; "))]
    SequentParsing(Vec<ParseError>),
    /// The input is not an LLTP problem, for the reason given.
    #[cfg(feature = "parse")]
    #[error("not an LLTP problem: {0}")]
    Lltp(String),
    /// A proof node index (first) lies outside the arena (its length second).
    #[error("a proof refers to node {0}, but it has {1} nodes")]
    NodeIndexOutOfBounds(usize, usize),
    /// A proof has this many nodes, more than a node index counts.
    #[error("the proof has {0} nodes, more than a proof can index (2³² − 1)")]
    TooManyNodes(usize),
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
    /// A proof has no derivation to show: it is larger than the options
    /// of the view allow, or its building was stopped.
    #[error("{0}")]
    View(crate::proofs::ViewError),
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
    /// The checker rejected the proof a search found: a defect of the
    /// engine that found it, and no verdict on the sequent.
    #[error(
        "the proof the search found does not pass the checker, which is a defect of the engine \
         and no verdict on the sequent: {0}"
    )]
    Rejected(Box<CheckError>),
    /// The threads of a parallel search could not be started.
    #[cfg(feature = "parallel")]
    #[error("cannot start {0} search threads: {1}")]
    ThreadPool(usize, String),
    /// A rule does not apply to a goal of an interactive proof as asked.
    #[cfg(feature = "interactive")]
    #[error(transparent)]
    Refused(#[from] crate::proofs::Refusal),
    /// The parts of an interactive proof read back do not fit together, as
    /// the message says.
    #[cfg(feature = "interactive")]
    #[error("not a proof in progress: {0}")]
    InconsistentState(&'static str),
    /// An interactive proof still has this many open goals, so there is no
    /// proof term to make of it yet.
    #[cfg(feature = "interactive")]
    #[error("the proof is not finished: {0} goals are open")]
    OpenGoals(usize),
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

impl From<crate::proofs::ViewError> for Error {
    /// Wraps the reason, an invalid proof as [`Error::InvalidProof`].
    fn from(error: crate::proofs::ViewError) -> Self {
        match error {
            crate::proofs::ViewError::Invalid(error) => Self::InvalidProof(error),
            other => Self::View(other),
        }
    }
}
