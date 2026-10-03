// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use clap::{Args, Parser, Subcommand, ValueEnum};
use linlog::search::{Engine, Options};
use linlog::{Bias, Forest, Fragment, Mode, ViewOptions};
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
    /// search close goal G, or every open goal; `show [latex|typst|svg]`
    /// draws the derivation so far, as text, as a LaTeX or Typst proof
    /// tree, or as an SVG document;
    /// `proof [FILE]` checks the finished proof and prints it, or writes it
    /// as JSON for `check`; `save FILE` and `load FILE` keep and resume the
    /// session as JSON; `help`; `quit`. The exit status is 0
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
    /// How the focused engines pick the positive literal of every atom
    ///
    /// No choice changes what is provable. `rarer` mostly chains backward
    /// from the goal, `factors` forward from the hypotheses, which on Horn
    /// clauses under `!`, such as a Petri net, is often much faster but
    /// takes one copy per step on a single branch. `auto` runs both
    /// searches on a sequent with exponentials and answers with the first
    /// that decides: alternating on one core, side by side on several.
    #[arg(long, value_enum, value_name = "BIAS", default_value_t = BiasArg::Auto)]
    pub bias: BiasArg,
    /// How often `?` formulas may be copied on one branch of the proof, or
    /// `none` for no bound
    ///
    /// The search tries the bounds 0, 1, … in turn. By default it goes on
    /// to the next bound until it decides or the time limit passes, and an
    /// unknown verdict says which bound it reached. With a bound, a sequent
    /// with exponentials that has no proof within it is unknown (exit
    /// status 3) unless a smaller bound already exhausted the search
    /// space, in which case it is unprovable. Sequents without exponentials
    /// are not affected.
    #[arg(long, value_name = "N", value_parser = parse_bound, default_value_t = Bound(None))]
    pub copies: Bound,
    /// How often `?` formulas may be copied on one branch of the forward
    /// search that `--bias auto` runs
    ///
    /// It applies to a Horn program: clauses such as `!(a * b -o c * d)`,
    /// a marking and a goal of atoms, as a Petri net is. There a copy is
    /// one step of a chain, and a chain of n steps needs n of them. The
    /// forward search never runs within less than `--copies`; on any other
    /// sequent, and under Mix, it runs within `--copies`. Without a bound
    /// from `--copies` it has no effect: both searches deepen while the
    /// time limit lasts.
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_FORWARD_COPIES)]
    pub forward_copies: u32,
    /// Give up after this long, such as 500ms, 10s, 2m or 1h, or `none`
    /// for no limit
    ///
    /// The verdict is then unknown (exit status 3). The time counts from
    /// the start of the command, so it covers reading and parsing the
    /// sequent as well as the search: a sequent that is not read in time
    /// is given up on like one that is not decided. Without the limit, the
    /// search runs until it decides or is interrupted with Ctrl-C, which
    /// with exponentials and no `--copies` bound can be forever.
    #[arg(long, value_name = "DURATION", value_parser = parse_time, default_value_t = Time(Some(DEFAULT_TIMEOUT)))]
    pub timeout: Time,
    /// The most decided sequents the search remembers at once
    ///
    /// When the memo is full it is emptied, which costs time but not
    /// correctness. Zero switches the memo off. `--memory-limit` bounds
    /// the memo in bytes; this is the finer knob beside it.
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_MEMO_LIMIT)]
    pub memo_limit: usize,
    /// The most memory the search may hold: a number of bytes with a unit
    /// such as 512MiB or 4GiB, or `none` for no limit
    ///
    /// Counted is what grows with the search: what it remembers, the
    /// proofs it keeps and what every level of its recursion takes; not
    /// the sequent itself. A memo that no longer fits is emptied first,
    /// as at `--memo-limit`; when that is not enough the verdict is
    /// unknown (exit status 3). The check of the proof and the derivation
    /// built from it are under the same limit: a derivation estimated
    /// above it is left out even with `--derivation-limit none`.
    #[arg(long, value_name = "SIZE", value_parser = parse_limit, default_value_t = Limit(Some(Options::DEFAULT_MEMORY_LIMIT)))]
    pub memory_limit: Limit,
    /// The deepest nesting of rules on one branch before the search gives up
    ///
    /// Raise it for sequents with thousands of connectives; the search runs on
    /// a thread whose stack grows with the limit.
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_RECURSION_LIMIT)]
    pub recursion_limit: u32,
    /// How many threads the search may use
    ///
    /// By default one thread searches first, for the time `--pool-after`
    /// gives, and then as many threads as the machine runs at once take
    /// over: a small sequent is decided at once and always the same way,
    /// a hard one gets every core. A number given here is used from the
    /// start, unless `--pool-after` is given too; a number above the
    /// machine's parallelism is taken as that, with a note on standard
    /// error. More than one runs the focused engine and the net engine on
    /// that many threads; the proof found may then differ from run to run,
    /// the verdict never does.
    #[arg(short, long, value_name = "N")]
    pub jobs: Option<usize>,
    /// How long one thread searches before the other threads take over,
    /// such as 100ms; 0 starts them at once
    ///
    /// The default is 100ms when `--jobs` is not given, and 0 when it is.
    /// The threads start the search afresh: what the single thread found
    /// is not handed on.
    #[arg(long, value_name = "DURATION", value_parser = parse_duration)]
    pub pool_after: Option<Duration>,
    /// Run the sequential engines, whose proof is a function of the input
    ///
    /// The same as `--jobs 1`, and takes precedence over `--jobs`.
    #[arg(long)]
    pub deterministic: bool,
    /// Report the proof the search found without checking it
    ///
    /// By default every proof passes the proof checker, which shares no
    /// code with the search, before anything is reported; a proof it
    /// rejects is an error (exit status 2), not a verdict.
    #[arg(long)]
    pub no_check: bool,
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
    /// closes a goal, or `none` for no bound; see `prove --copies`
    #[arg(long, value_name = "N", value_parser = parse_bound, default_value_t = Bound(None))]
    pub copies: Bound,
    /// How a `close` picks the positive literal of every atom; see
    /// `prove --bias`
    #[arg(long, value_enum, value_name = "BIAS", default_value_t = BiasArg::Auto)]
    pub bias: BiasArg,
    /// How often `?` formulas may be copied on one branch of the forward
    /// search of `--bias auto` when the search closes a goal; see `prove
    /// --forward-copies`
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_FORWARD_COPIES)]
    pub forward_copies: u32,
    /// Give up on a `close` after this long, such as 500ms, 10s, 2m or 1h,
    /// or `none` for no limit; see `prove --timeout`
    #[arg(long, value_name = "DURATION", value_parser = parse_time, default_value_t = Time(Some(DEFAULT_TIMEOUT)))]
    pub timeout: Time,
    /// The most decided sequents a `close` remembers at once; see
    /// `prove --memo-limit`
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_MEMO_LIMIT)]
    pub memo_limit: usize,
    /// The most memory a `close` may hold; see `prove --memory-limit`
    #[arg(long, value_name = "SIZE", value_parser = parse_limit, default_value_t = Limit(Some(Options::DEFAULT_MEMORY_LIMIT)))]
    pub memory_limit: Limit,
    /// The deepest nesting of rules on one branch before a `close` gives
    /// up; see `prove --recursion-limit`
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_RECURSION_LIMIT)]
    pub recursion_limit: u32,
    /// How many threads a `close` may use; see `prove --jobs`
    #[arg(short, long, value_name = "N")]
    pub jobs: Option<usize>,
    /// How long one thread searches before the other threads take over;
    /// see `prove --pool-after`
    #[arg(long, value_name = "DURATION", value_parser = parse_duration)]
    pub pool_after: Option<Duration>,
    /// Run the sequential engines; see `prove --deterministic`
    #[arg(long)]
    pub deterministic: bool,
    /// The largest derivation `close` grafts, by its estimated size; see
    /// `prove --derivation-limit`
    #[arg(long, value_name = "SIZE", value_parser = parse_limit, default_value_t = Limit::default())]
    pub derivation_limit: Limit,
}

/// The time limit of `prove` and of a session's `close` when none is
/// given: what a larger copy bound still decides against what every
/// undecided sequent then waits. Of the problems of the LLTP library that
/// a copy bound of 3 leaves undecided and a larger bound decides, nine in
/// ten are decided within a second or two, while the many that nothing
/// decides each wait the whole limit.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

/// How long one thread searches before the others take over, when
/// `--jobs` is not given: a small sequent is decided in microseconds, and
/// a pool costs milliseconds to start and makes the proof depend on the
/// threads' timing.
pub const DEFAULT_POOL_AFTER: Duration = Duration::from_millis(100);

/// The threads a search gets, and how long one thread searches before
/// they take over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Threads {
    /// The threads, at most those the machine runs at once.
    pub(crate) jobs: usize,
    /// How long one thread searches first, when the threads are more than
    /// one and do not start at once.
    pub(crate) alone: Option<Duration>,
}

/// The threads a search gets for `--jobs`, `--pool-after` and
/// `--deterministic`: one for the last; else the threads asked for, from
/// the start unless `--pool-after` says otherwise; and without `--jobs`
/// every thread the machine runs at once after [`DEFAULT_POOL_AFTER`]. The
/// library bounds the threads by those the machine runs at once; a number
/// above that bound is taken as the bound, and a note on standard error
/// says so. Where the machine does not tell, the library's own bound
/// stands in for a number asked, and one thread is the default.
pub(crate) fn threads(
    asked: Option<usize>,
    pool_after: Option<Duration>,
    deterministic: bool,
) -> Threads {
    if deterministic {
        return Threads {
            jobs: 1,
            alone: None,
        };
    }
    let machine = std::thread::available_parallelism().map(std::num::NonZero::get);
    let (jobs, alone) = match asked {
        Some(asked) => {
            let most = machine.map_or(Options::MAX_JOBS, |m| m.min(Options::MAX_JOBS));
            if asked > most {
                let threads = if most == 1 { "thread" } else { "threads" };
                eprintln!(
                    "note: --jobs {asked} is more than the {most} {threads} a search uses at most \
                     on this machine; it uses {most}"
                );
            }
            (asked.clamp(1, most), pool_after)
        }
        None => (
            machine.map_or(1, |m| m.min(Options::MAX_JOBS)),
            Some(pool_after.unwrap_or(DEFAULT_POOL_AFTER)),
        ),
    };
    Threads {
        jobs,
        alone: alone.filter(|t| jobs > 1 && !t.is_zero()),
    }
}

/// The arguments of `check`.
#[derive(Args, Debug)]
pub struct CheckArgs {
    /// The most memory the check may hold: a number of bytes with a unit
    /// such as 512MiB or 4GiB, or `none` for no limit
    ///
    /// A proof whose check would pass it is neither valid nor invalid:
    /// the command ends with an error. A derivation estimated above it
    /// is left out, as past `--derivation-limit`.
    #[arg(long, value_name = "SIZE", value_parser = parse_limit, default_value_t = Limit(Some(Options::DEFAULT_MEMORY_LIMIT)))]
    pub memory_limit: Limit,
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
        /// The output format
        #[arg(long, value_enum, value_name = "FORMAT", default_value_t = SequentFormat::Text)]
        format: SequentFormat,
        /// Write a document that compiles on its own instead of a fragment
        /// to paste (latex and typst)
        #[arg(long)]
        standalone: bool,
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
#[derive(Args, Clone, Debug)]
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
    /// The most subformula occurrences the sequent may have, or `none`
    ///
    /// A sequent in JSON can share subformulas, so a small file may stand
    /// for a sequent of any size; one beyond the limit is refused before
    /// it is unfolded. Occurrences cost some 25 bytes each before any
    /// search, and no limit lets through more than about four billion.
    #[arg(long, value_name = "N", value_parser = parse_most, default_value_t = Most::default())]
    pub occurrence_limit: Most,
}

/// The most occurrences a sequent may have, or none for no limit but the
/// library's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Most(pub Option<u64>);

impl Default for Most {
    /// The library's default.
    fn default() -> Self {
        Self(Some(Forest::DEFAULT_LIMIT))
    }
}

impl std::fmt::Display for Most {
    /// Writes the limit as `--occurrence-limit` reads it.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => f.write_str("none"),
            Some(most) => write!(f, "{most}"),
        }
    }
}

/// Parses a number of occurrences, or `none`.
fn parse_most(text: &str) -> Result<Most, String> {
    if text == "none" {
        return Ok(Most(None));
    }
    text.parse()
        .map(|most| Most(Some(most)))
        .map_err(|_| format!("{text:?} is neither a number nor `none`"))
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
    /// Print only the verdict line, not the derivation
    #[arg(short, long)]
    pub quiet: bool,
    /// Write a document that compiles on its own instead of a fragment to
    /// paste (latex and typst)
    #[arg(long)]
    pub standalone: bool,
    /// When to print the proof tree of the text format
    ///
    /// By default a tree is printed on a terminal only if it fits: no line
    /// wider than the terminal and no more than three screens of lines.
    /// Otherwise one line says how large the tree is and how to get it.
    /// Into a file or a pipe the tree is always written.
    #[arg(long, value_enum, value_name = "WHEN", default_value_t = Tree::Auto)]
    pub tree: Tree,
    /// The largest derivation to build, by its estimated size: a number
    /// of bytes with a unit such as 64MiB or 2GiB, or `none` for no limit
    ///
    /// A derivation writes out the whole sequent at every inference and
    /// repeats every subproof that the proof shares, so it can be larger
    /// than the proof by any factor. A derivation estimated above the
    /// limit is not built, in any format: the verdict is reported without
    /// it, with a line that says how large it is, and the exit status is
    /// the verdict's. `--format json` writes the proof itself at any size.
    #[arg(long, value_name = "SIZE", value_parser = parse_limit, default_value_t = Limit::default())]
    pub derivation_limit: Limit,
}

/// When the text format prints its proof tree.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tree {
    /// On a terminal if it fits, into a file or a pipe always
    Auto,
    /// Whatever its size, up to `--derivation-limit`
    Always,
    /// Not at all
    Never,
}

/// The largest derivation a command builds, in bytes of its estimated
/// size, or none for no limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit(pub Option<u64>);

impl Default for Limit {
    /// The library's default.
    fn default() -> Self {
        Self(Some(ViewOptions::DEFAULT_LIMIT))
    }
}

impl std::fmt::Display for Limit {
    /// Writes the limit as `--derivation-limit` reads it.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => f.write_str("none"),
            Some(bytes) => {
                let (mut value, mut unit) = (bytes, 0);
                while unit + 1 < UNITS.len() && value >= 1024 && value.is_multiple_of(1024) {
                    (value, unit) = (value / 1024, unit + 1);
                }
                write!(f, "{value}{}", UNITS[unit])
            }
        }
    }
}

impl From<Limit> for ViewOptions {
    /// Returns the options of a derivation built within the limit.
    fn from(limit: Limit) -> Self {
        Self::default().limit(limit.0)
    }
}

/// A copy bound, or none for a search that deepens until it decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bound(pub Option<u32>);

impl std::fmt::Display for Bound {
    /// Writes the bound as `--copies` reads it.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => f.write_str("none"),
            Some(n) => write!(f, "{n}"),
        }
    }
}

/// Parses a copy bound: a number, or `none`.
fn parse_bound(text: &str) -> Result<Bound, String> {
    if text == "none" {
        return Ok(Bound(None));
    }
    text.parse()
        .map(|n| Bound(Some(n)))
        .map_err(|_| format!("{text:?} is not a number of copies, or `none`"))
}

/// A time limit, or none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Time(pub Option<Duration>);

impl std::fmt::Display for Time {
    /// Writes the limit as `--timeout` reads it.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => f.write_str("none"),
            Some(t) => write!(f, "{t:?}"),
        }
    }
}

/// Parses a time limit: a duration, or `none`.
fn parse_time(text: &str) -> Result<Time, String> {
    if text == "none" {
        return Ok(Time(None));
    }
    parse_duration(text).map(|t| Time(Some(t)))
}

/// The units of a size, each 1024 of the one before.
const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

/// Parses a size such as `64MiB`, `2GiB` or `1000000` (bytes), or `none`.
fn parse_limit(text: &str) -> Result<Limit, String> {
    if text == "none" {
        return Ok(Limit(None));
    }
    let split = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let unit = unit.trim();
    let power = UNITS
        .iter()
        .position(|u| u.eq_ignore_ascii_case(unit) || (unit.is_empty() && *u == "B"))
        .ok_or_else(|| format!("unknown unit {unit:?}; use B, KiB, MiB, GiB or TiB, or `none`"))?;
    let number: u64 = number
        .parse()
        .map_err(|_| format!("{text:?} is not a size such as 64MiB or 2GiB, or `none`"))?;
    Ok(Limit(Some(
        number.saturating_mul(1024u64.saturating_pow(power as u32)),
    )))
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
    /// The verdict as a comment, then the derivation as a LaTeX proof tree
    /// of the ebproof package, with the connectives of cmll and amssymb
    Latex,
    /// The verdict as a comment, then the derivation as a Typst proof tree
    /// of the curryst package
    Typst,
    /// The verdict as an XML comment, then the derivation drawn as an SVG
    /// document, set in the Euler Math font
    Svg,
    /// The verdict as an XML comment, then the proof net of the proof
    /// drawn as an SVG document: the formula trees with the axiom links as
    /// arcs over the literals; for the sequents `net` takes
    NetSvg,
    /// The verdict as a comment, then the derivation as a Rocq proof
    /// script for NanoYalla 1.1.3, the kernel of Click & coLLecT (the
    /// nanoyalla directory of github.com/ComputerAidedLL/click-and-collect,
    /// built with Rocq 9 and its standard library, no Yalla needed): a
    /// lemma stating the sequent one-sided, proved rule by rule and closed
    /// by Qed; `--standalone` adds the import line; a proof with Mix or
    /// with the weakening of affine mode is refused
    Rocq,
}

/// The output formats of `seq print`.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequentFormat {
    /// Unicode text
    Text,
    /// LaTeX math, with the connectives of cmll and amssymb
    Latex,
    /// Typst math
    Typst,
    /// An SVG document of one line, set in the Euler Math font
    Svg,
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

/// The rules `--bias` names.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BiasArg {
    /// `factors` without exponentials, `rarer` with weakening, and with
    /// exponentials both, each in a search of its own
    Auto,
    /// The literal with fewer occurrences is positive
    Rarer,
    /// The literal that is more often a direct factor of a tensor is
    /// positive, so that more splits are forced
    Factors,
}

impl From<BiasArg> for Bias {
    /// Returns the rule the name stands for.
    fn from(b: BiasArg) -> Self {
        match b {
            BiasArg::Auto => Bias::Auto,
            BiasArg::Rarer => Bias::Rarer,
            BiasArg::Factors => Bias::Factors,
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
