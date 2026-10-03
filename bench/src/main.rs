// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! `linlog-bench`, the benchmark harness: runs the generated families, the
//! problems of the LLTP library and problem files of its own through
//! linlog's engines, one child process per run with a time limit, writes
//! one CSV row per run, and summarises CSV files as Markdown tables.

/// Two sets of rows compared problem by problem.
mod compare;
/// Where problems come from: the families, LLTP files, problem files.
mod problems;
/// Running problems: the parent that spawns a child per run, and the child.
mod run;
/// Markdown tables from the CSV rows.
mod summary;

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

/// Benchmark linlog's engines on generated families, the LLTP library and
/// problem files
#[derive(Parser, Debug)]
#[command(name = "linlog-bench", version)]
struct Cli {
    /// What to do
    #[command(subcommand)]
    command: Command,
}

/// The commands.
#[derive(Subcommand, Debug)]
enum Command {
    /// Run problems, each in a child process with a time limit, and write
    /// one CSV row per run
    Run(RunArgs),
    /// Print Markdown tables of CSV files that `run` wrote: the problems
    /// solved within the time limit per family and configuration, and the
    /// time of every generated or listed problem per configuration
    Summary {
        /// The CSV files
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Compare every file with the file of the same name in this
        /// directory, an earlier baseline, problem by problem, instead
        #[arg(long, value_name = "DIR", conflicts_with = "against")]
        before: Option<PathBuf>,
        /// Compare every other file with this one, problem by problem
        /// whatever the files' names, instead (the passes under one atom
        /// bias against the default)
        #[arg(long, value_name = "FILE")]
        against: Option<PathBuf>,
    },
    /// List the generated families with their default sizes
    Families,
    /// Run one problem and print its result as the tail of a CSV row: what
    /// `run` starts for every run
    #[command(hide = true)]
    One(OneArgs),
}

/// The seconds a child gets to load its problem by default: several times
/// what the largest file of the LLTP library takes (16 s for 103 MB).
const DEFAULT_LOAD_LIMIT: f64 = 120.0;

/// The arguments of `run`.
#[derive(Args, Debug)]
pub struct RunArgs {
    /// A generated family, with its default sizes or the sizes given
    /// (`--family partition-no=4,5,6`); repeatable
    #[arg(long, value_name = "NAME[=SIZES]")]
    family: Vec<String>,
    /// Every generated family at its default sizes
    #[arg(long)]
    all_families: bool,
    /// An LLTP problem file, or a directory searched for `*.p` files; a
    /// problem under a directory named `ILL` is intuitionistic, any other
    /// classical; repeatable
    #[arg(long, value_name = "PATH")]
    lltp: Vec<PathBuf>,
    /// A problem file of lines `name; mode; expected; copies; sequent` (see
    /// `bench/problems/`); repeatable
    #[arg(long, value_name = "FILE")]
    problems: Vec<PathBuf>,
    /// Run only the problems whose family and name, `FAMILY/NAME` (as
    /// `ILL/KLE-cbn/KLE017+1.p`), contain one of these
    #[arg(long, value_delimiter = ',', value_name = "TEXT")]
    only: Vec<String>,
    /// Run the problems in the reverse of their order: two runs over one
    /// library, one of them reversed, then meet its largest problems at
    /// different times
    #[arg(long)]
    reverse: bool,
    /// The modes to run every problem in: `given` (the problem's own),
    /// `classical` or `intuitionistic`
    #[arg(long, value_delimiter = ',', default_value = "given")]
    modes: Vec<run::ModeChoice>,
    /// The engines to run every problem with: `auto` (the one the fragment
    /// calls for), `focus`, `net`, `two-sided` or `additive`; a forced
    /// engine that does not apply to a problem gives a `refused` row
    #[arg(long, value_delimiter = ',', default_value = "auto")]
    engines: Vec<run::EngineChoice>,
    /// The thread counts to run every problem with; `all` is every core
    #[arg(long, value_delimiter = ',', default_value = "1")]
    jobs: Vec<String>,
    /// The copy bound, overriding the one a generated problem names
    /// (default: the problem's, else 3)
    #[arg(long)]
    copies: Option<u32>,
    /// How the focused engines pick the positive literal of every atom:
    /// `auto`, `rarer` or `factors`
    #[arg(long, default_value = "auto")]
    bias: run::BiasChoice,
    /// The copy bound of the forward search that the bias `auto` runs on
    /// a Horn program (default: the library's)
    #[arg(long)]
    forward_copies: Option<u32>,
    /// The most bytes a search may hold at once, 0 for no bound (default:
    /// the library's)
    #[arg(long, value_name = "BYTES")]
    memory_limit: Option<u64>,
    /// The net engine's links between two exact acyclicity tests
    /// (default: every link up to 200 occurrences, every fourth above)
    #[arg(long)]
    test_period: Option<u32>,
    /// The recursion limit of the search (default 2048), which the LLTP
    /// library's Petri nets with long markings reach
    #[arg(long)]
    recursion_limit: Option<u32>,
    /// The time limit per run, in seconds
    #[arg(long, default_value_t = 60.0)]
    timeout: f64,
    /// The seconds a child may run past its time limit, counted from the
    /// end of its load, before it is killed: for the pool's teardown and
    /// the proof check
    /// (default: a tenth of the limit and five seconds)
    #[arg(long, value_name = "SECONDS")]
    grace: Option<f64>,
    /// Seconds a child may take to load its problem before its search
    /// starts and the time limit counts; a child that takes longer is
    /// killed
    #[arg(long, value_name = "SECONDS", default_value_t = DEFAULT_LOAD_LIMIT)]
    load_limit: f64,
    /// How often to run a problem whose first run took less than
    /// `--repeat-under` seconds; the summary takes the median
    #[arg(long, default_value_t = 1)]
    repeat: u32,
    /// Repeat only runs faster than this many seconds
    #[arg(long, default_value_t = 1.0)]
    repeat_under: f64,
    /// Write the CSV here instead of to standard output
    #[arg(long, short)]
    output: Option<PathBuf>,
    /// Append to the output file, writing the header only if it is empty
    #[arg(long, requires = "output")]
    append: bool,
    /// Skip every problem and configuration the output file already has a
    /// row for: finishes an interrupted run
    #[arg(long, requires = "append")]
    resume: bool,
}

/// The arguments of `one`.
#[derive(Args, Debug)]
pub struct OneArgs {
    /// The problem, as `run` names it (`family:NAME:SIZE:INDEX`,
    /// `lltp:PATH`, `file:PATH:LINE`)
    #[arg(long)]
    problem: String,
    /// The mode to run in
    #[arg(long)]
    mode: run::ModeChoice,
    /// The engine to run
    #[arg(long)]
    engine: run::EngineChoice,
    /// The threads
    #[arg(long)]
    jobs: usize,
    /// The copy bound, overriding the problem's
    #[arg(long)]
    copies: Option<u32>,
    /// The bias
    #[arg(long, default_value = "auto")]
    bias: run::BiasChoice,
    /// The forward search's copy bound
    #[arg(long)]
    forward_copies: Option<u32>,
    /// The memory bound in bytes, 0 for none
    #[arg(long)]
    memory_limit: Option<u64>,
    /// The net engine's test period
    #[arg(long)]
    test_period: Option<u32>,
    /// The recursion limit
    #[arg(long)]
    recursion_limit: Option<u32>,
    /// The time limit, in seconds
    #[arg(long)]
    timeout: f64,
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Command::Run(args) => run::run(&args),
        Command::Summary {
            files,
            before,
            against,
        } => match (before, against) {
            (Some(dir), _) => compare::before(&dir, &files),
            (_, Some(file)) => compare::against(&file, &files),
            _ => summary::summary(&files),
        },
        Command::Families => {
            for family in linlog::families::FAMILIES {
                let sizes: Vec<String> = family.sizes.iter().map(u32::to_string).collect();
                println!("{} ({}): {}", family.name, sizes.join(","), family.summary);
            }
            Ok(())
        }
        Command::One(args) => run::one(args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// The clap definitions are consistent.
    #[test]
    fn arguments() {
        Cli::command().debug_assert();
    }
}
