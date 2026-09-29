// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use clap::{Args, Parser, Subcommand, ValueEnum};
use linlog::search::{Engine, Options};
use linlog::{Fragment, Mode};
use std::path::PathBuf;
use std::time::Duration;

/// How to write a sequent, shown under the help of every command that reads
/// one.
const SYNTAX: &str = "\
Sequent syntax: `A, B |- C, D` (or `⊢`), either side may be empty.
  tensor  A * B, A ⊗ B        par   A | B, A par B, A ⅋ B
  with    A & B               plus  A + B, A ⊕ B
  linear implication  A -o B, A ⊸ B
  negation  ~A, A^            exponentials  !A, ?A
  units  1, bot, ⊥, top, ⊤, 0
Binding, tightest first: A^, then ~ ! ?, then *, |, &, +, and last -o, which
groups to the right.";

/// Decide, print and convert sequents of linear logic.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, after_help = SYNTAX)]
pub struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub command: Command,
}

/// The commands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Decide whether a sequent is provable, and print a proof if it is
    ///
    /// The fragment of the sequent (MLL, MALL, …) is detected and picks the
    /// engine; the first line of the output names both. The exit status is 0
    /// for provable, 1 for unprovable, 3 when the search stopped before
    /// deciding, and 2 for an error.
    #[command(after_help = SYNTAX)]
    Prove(ProveArgs),
    /// Check a proof read from JSON, as `prove --format json` writes it
    ///
    /// The mode comes from the flags, not from the file: pass the flags the
    /// proof was found with. The exit status is 0 for a valid proof, 1 for
    /// an invalid one, and 2 for an error.
    Check(CheckArgs),
    /// Prove a sequent step by step, reading commands from standard input
    ///
    /// The session starts with the whole sequent as the one open goal.
    /// Commands, one per line: `goals` lists the open goals with the
    /// positions of their formulas; `rules G P` lists the rules that act on
    /// formula P of goal G; `apply G P RULE [P…]` applies a rule, the
    /// further positions being the formulas that go to the left premise of
    /// a ⊗ or Mix; `undo` retracts the last step; `close [G]` lets the
    /// search close goal G, or every open goal; `show` draws the derivation
    /// so far; `proof [FILE]` checks the finished proof and prints it, or
    /// writes it as JSON for `check`; `save FILE` and `load FILE` keep and
    /// resume the session as JSON; `help`; `quit`. The exit status is 0
    /// when the session ends with a finished proof that checks, 1
    /// otherwise, and 2 for an error.
    #[command(after_help = SYNTAX)]
    Interact(InteractArgs),
    /// Print a sequent, convert it to JSON, or name its fragment
    Seq {
        /// What to do with the sequent.
        #[command(subcommand)]
        command: SeqCommand,
    },
}

/// The arguments of `prove`.
#[derive(Args, Debug)]
pub struct ProveArgs {
    /// The sequent.
    #[command(flatten)]
    pub input: SequentInput,
    /// The logic.
    #[command(flatten)]
    pub mode: ModeArgs,
    /// Search in this fragment instead of the detected one
    ///
    /// A sequent outside it is an error. A larger fragment than the detected
    /// one switches off the prunes that only hold in the smaller one, which
    /// is a way to compare them.
    #[arg(long, value_enum, value_name = "FRAGMENT")]
    pub fragment: Option<FragmentArg>,
    /// The engine to search with
    #[arg(long, value_enum, value_name = "ENGINE", default_value_t = EngineArg::Auto)]
    pub engine: EngineArg,
    /// How often `?` formulas may be copied on one branch of the proof
    ///
    /// The search tries the bounds 0, 1, … up to this one. A sequent with
    /// exponentials that has no proof within the bound is unknown (exit
    /// status 3) unless a smaller bound already exhausted the search space,
    /// in which case it is unprovable. Sequents without exponentials are
    /// not affected.
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_COPIES)]
    pub copies: u32,
    /// Give up after this long, such as 500ms, 10s, 2m or 1h
    ///
    /// The verdict is then unknown (exit status 3). Without it, the search
    /// runs until it decides or is interrupted with Ctrl-C.
    #[arg(long, value_name = "DURATION", value_parser = parse_duration)]
    pub timeout: Option<Duration>,
    /// The most decided sequents the search remembers at once
    ///
    /// When the memo is full it is emptied, which costs time but not
    /// correctness; lower the limit if memory runs out. Zero switches the memo
    /// off.
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_MEMO_LIMIT)]
    pub memo_limit: usize,
    /// The deepest nesting of rules on one branch before the search gives up
    ///
    /// Raise it for sequents with thousands of connectives; the search runs on
    /// a thread whose stack grows with the limit.
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_RECURSION_LIMIT)]
    pub recursion_limit: u32,
    /// Where and how to write the result.
    #[command(flatten)]
    pub output: OutputArgs,
    /// Also print what the search cost: the sequents visited, memo use and
    /// splits tried of the focus engine, or the literals chosen, links tried
    /// and exact tests run of the net engine, and the time
    ///
    /// JSON output always carries the counts; the time is printed only as
    /// text.
    #[arg(long)]
    pub stats: bool,
}

/// The arguments of `interact`.
#[derive(Args, Debug)]
pub struct InteractArgs {
    /// The sequent, as an argument or with --file, since standard input
    /// carries the commands.
    #[command(flatten)]
    pub input: SequentInput,
    /// Resume a session saved with `save` instead of starting from a
    /// sequent; the mode is the file's, so the mode flags are refused
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with_all = ["sequent", "file", "intuitionistic", "affine", "mix"]
    )]
    pub state: Option<PathBuf>,
    /// The logic.
    #[command(flatten)]
    pub mode: ModeArgs,
    /// How often `?` formulas may be copied on one branch when the search
    /// closes a goal; see `prove --copies`
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_COPIES)]
    pub copies: u32,
    /// Give up on a `close` after this long, such as 500ms, 10s, 2m or 1h
    #[arg(long, value_name = "DURATION", value_parser = parse_duration)]
    pub timeout: Option<Duration>,
    /// The most decided sequents a `close` remembers at once; see
    /// `prove --memo-limit`
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_MEMO_LIMIT)]
    pub memo_limit: usize,
    /// The deepest nesting of rules on one branch before a `close` gives
    /// up; see `prove --recursion-limit`
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_RECURSION_LIMIT)]
    pub recursion_limit: u32,
}

/// The arguments of `check`.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// The proof file, or standard input when absent or `-`
    #[arg(value_name = "PROOF")]
    pub proof: Option<PathBuf>,
    /// The logic the proof must hold in.
    #[command(flatten)]
    pub mode: ModeArgs,
    /// Where to write the result.
    #[command(flatten)]
    pub output: OutputArgs,
}

/// The subcommands of `seq`.
#[derive(Subcommand, Debug)]
pub enum SeqCommand {
    /// Print a sequent one-sided, in negation normal form, or two-sided
    ///
    /// `A, A -o B |- B` prints as `⊢ ~A, A ⊗ ~B, B`: hypotheses are negated
    /// onto the right, implications become pars, and negation is pushed down
    /// to the atoms. With --intuitionistic it prints as `A, A ⊸ B ⊢ B`
    /// again, which requires an intuitionistic sequent: one formula on the
    /// right, and pars only as implications.
    #[command(after_help = SYNTAX)]
    Print {
        /// The sequent.
        #[command(flatten)]
        input: SequentInput,
        /// Print the sequent two-sided, as intuitionistic linear logic
        /// reads it
        #[arg(short, long)]
        intuitionistic: bool,
        /// Write to this file instead of standard output
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Print a sequent as JSON, the form `--json-input` reads
    #[command(after_help = SYNTAX)]
    Json {
        /// The sequent.
        #[command(flatten)]
        input: SequentInput,
        /// Merge equal subformulas, drop unreferenced ones and sort the
        /// formulas of a JSON input, as parsing text always does
        #[arg(long)]
        optimize: bool,
        /// Write to this file instead of standard output
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Print the smallest fragment a sequent lives in: MLL, MLL with units,
    /// ALL, MALL, MELL or LL, or with --intuitionistic IMLL, IMLL with
    /// units, IALL, IMALL, IMELL or ILL
    #[command(after_help = SYNTAX)]
    Fragment {
        /// The sequent.
        #[command(flatten)]
        input: SequentInput,
        /// Name the intuitionistic fragment
        #[arg(short, long)]
        intuitionistic: bool,
    },
}

/// Where a sequent comes from.
#[derive(Args, Debug)]
pub struct SequentInput {
    /// The sequent, such as "A, A -o B |- B"; read from --file or standard
    /// input when absent
    #[arg(value_name = "SEQUENT", conflicts_with = "file")]
    pub sequent: Option<String>,
    /// Read the sequent from this file, or `-` for standard input
    #[arg(short, long, value_name = "PATH")]
    pub file: Option<PathBuf>,
    /// Read the sequent as JSON, as `seq json` writes it, instead of as text
    #[arg(long)]
    pub json_input: bool,
}

/// The logic a sequent is proved in.
#[derive(Args, Debug)]
pub struct ModeArgs {
    /// Intuitionistic linear logic: one formula on the right of ⊢, pars
    /// only as implications, and two-sided derivations
    #[arg(short, long)]
    pub intuitionistic: bool,
    /// Affine logic: a hypothesis may go unused (weakening)
    #[arg(short, long)]
    pub affine: bool,
    /// Allow the Mix rule, which proves ⊢ Γ, Δ from ⊢ Γ and ⊢ Δ
    #[arg(long)]
    pub mix: bool,
}

impl ModeArgs {
    /// Returns the mode the flags ask for.
    pub fn mode(&self) -> Mode {
        Mode {
            intuitionistic: self.intuitionistic,
            affine: self.affine,
            mix: self.mix,
        }
    }
}

/// Where and how a command writes its result.
#[derive(Args, Debug)]
pub struct OutputArgs {
    /// The output format
    #[arg(long, value_enum, value_name = "FORMAT", default_value_t = Format::Text)]
    pub format: Format,
    /// Write to this file instead of standard output
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,
    /// Print only the verdict line, not the derivation (text format)
    #[arg(short, long)]
    pub quiet: bool,
}

/// The output formats of `prove` and `check`.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    /// The verdict on one line, then the derivation as a text tree
    Text,
    /// One JSON object: verdict, fragment, mode, engine, statistics, and the
    /// proof, which `check` reads
    Json,
    /// The verdict on one line, then the proof net of the proof: the
    /// sequent, its axiom links as pairs of literals with their occurrence
    /// numbers, and the verdict of the correctness criterion; for MLL
    /// without units, in classical mode with or without Mix
    Net,
}

/// The fragments `--fragment` names.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentArg {
    /// Multiplicatives: ⊗ ⅋
    Mll,
    /// Multiplicatives and their units: ⊗ ⅋ 1 ⊥
    MllUnits,
    /// Additives only: & ⊕ ⊤ 0
    All,
    /// Multiplicatives and additives with units
    Mall,
    /// Multiplicatives and exponentials with units
    Mell,
    /// Every connective
    Ll,
}

impl From<FragmentArg> for Fragment {
    /// Returns the fragment the name stands for.
    fn from(f: FragmentArg) -> Self {
        match f {
            FragmentArg::Mll => Fragment::MLL,
            FragmentArg::MllUnits => Fragment::MLL_WITH_UNITS,
            FragmentArg::All => Fragment::ALL,
            FragmentArg::Mall => Fragment::MALL,
            FragmentArg::Mell => Fragment::MELL,
            FragmentArg::Ll => Fragment::LL,
        }
    }
}

/// The engines `--engine` names.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineArg {
    /// The engine the fragment and the mode call for
    Auto,
    /// Focused sequent search, for every classical fragment, with or
    /// without units, Mix, exponentials and weakening
    Focus,
    /// Proof-net search, for MLL without units, with or without Mix; in
    /// intuitionistic mode for IMLL without units, by its embedding into
    /// MLL
    Net,
    /// Focused sequent search two-sided, for every intuitionistic fragment
    TwoSided,
    /// The fast path for a sequent of two additive-only formulas, in every
    /// mode
    Additive,
}

impl From<EngineArg> for Option<Engine> {
    /// Returns the engine the name forces, or `None` for automatic choice.
    fn from(e: EngineArg) -> Self {
        match e {
            EngineArg::Auto => None,
            EngineArg::Focus => Some(Engine::Focus),
            EngineArg::Net => Some(Engine::Net),
            EngineArg::TwoSided => Some(Engine::TwoSided),
            EngineArg::Additive => Some(Engine::Additive),
        }
    }
}

/// Parses a duration such as `500ms`, `10s`, `1.5m` or `1h`; a bare number
/// is seconds.
fn parse_duration(text: &str) -> Result<Duration, String> {
    let split = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let seconds_per_unit = match unit.trim() {
        "ms" => 0.001,
        "" | "s" => 1.0,
        "m" | "min" => 60.0,
        "h" => 3600.0,
        _ => return Err(format!("unknown unit {unit:?}; use ms, s, m or h")),
    };
    let number: f64 = number
        .parse()
        .map_err(|_| format!("{text:?} is not a duration such as 500ms, 10s, 2m or 1h"))?;
    Duration::try_from_secs_f64(number * seconds_per_unit).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// The argument definitions are consistent, as clap checks them.
    #[test]
    fn arguments_are_consistent() {
        Cli::command().debug_assert();
    }

    /// Durations take a unit, seconds by default.
    #[test]
    fn durations() {
        for (text, millis) in [
            ("500ms", 500),
            ("10s", 10_000),
            ("1.5m", 90_000),
            ("1h", 3_600_000),
            ("2", 2000),
        ] {
            assert_eq!(parse_duration(text), Ok(Duration::from_millis(millis)));
        }
        for text in ["", "s", "10 days", "-1s", "1e3s"] {
            assert!(parse_duration(text).is_err(), "{text:?}");
        }
    }
}
