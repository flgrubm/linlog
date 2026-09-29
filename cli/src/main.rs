// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The linlog command line interface.

/// The command line arguments.
mod argument_parsing;
/// Reading input and writing output.
mod io;
/// The `prove` and `check` commands.
mod prove;

use anyhow::Result;
use argument_parsing::{Cli, Command, SeqCommand};
use clap::Parser;
use linlog::Error;
use std::fmt::Write;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

/// What a command found, which decides the exit status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Status {
    /// Provable, a valid proof, or a command that has no verdict: 0.
    Yes,
    /// Unprovable, or an invalid proof: 1.
    No,
    /// The search stopped before deciding: 3.
    Unknown,
}

/// The exit status of an error: bad arguments (as clap reports them),
/// unreadable input, a sequent no engine handles.
const ERROR: u8 = 2;

impl From<Status> for ExitCode {
    /// Returns the exit status for a verdict.
    fn from(s: Status) -> Self {
        ExitCode::from(match s {
            Status::Yes => 0,
            Status::No => 1,
            Status::Unknown => 3,
        })
    }
}

/// Set by the first Ctrl-C.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Whether the user pressed Ctrl-C to stop the search.
pub(crate) fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}

/// Makes the first Ctrl-C stop the search, so that its verdict is unknown
/// and the statistics still print, and a second one end the program as
/// Ctrl-C usually does.
pub(crate) fn catch_interrupt() {
    // The handler runs on a thread of its own, once per signal, so it may
    // exit. Without it Ctrl-C ends the program, which is a fine fallback.
    let _ = ctrlc::set_handler(|| {
        if INTERRUPTED.swap(true, Ordering::Relaxed) {
            std::process::exit(130);
        }
    });
}

/// Returns a parse error as a message that points at the place in the input
/// where parsing failed.
pub(crate) fn parse_error(input: &str, error: Error) -> anyhow::Error {
    let Error::SequentParsing(errors) = &error else {
        return error.into();
    };
    if input.is_empty() {
        return anyhow::Error::msg("the input is empty; the empty sequent is written |-");
    }
    let mut message = String::from("cannot parse the sequent");
    for e in errors {
        let column = input[..e.span.start].chars().count();
        let found = match &e.found {
            Some(token) => format!("unexpected {token:?}"),
            None => "unexpected end of input".into(),
        };
        write!(message, "\n  {input}\n  {:column$}^ {found}", "").unwrap();
    }
    anyhow::Error::msg(message)
}

/// Runs the command the arguments name.
fn run(cli: &Cli) -> Result<Status> {
    match &cli.command {
        Command::Prove(args) => prove::prove(args),
        Command::Check(args) => prove::check(args),
        Command::Seq { command } => {
            match command {
                SeqCommand::Print { input, output } => {
                    io::write(output.as_deref(), &input.sequent()?.to_string())?;
                }
                SeqCommand::Json {
                    input,
                    optimize,
                    output,
                } => {
                    let mut sequent = input.sequent()?;
                    if *optimize {
                        sequent.optimize()?;
                    }
                    io::write(output.as_deref(), &serde_json::to_string(&sequent)?)?;
                }
                SeqCommand::Fragment { input } => {
                    io::write(None, input.sequent()?.fragment().name())?;
                }
            }
            Ok(Status::Yes)
        }
    }
}

/// Parses the command line arguments, runs the command, and exits with the
/// status of its verdict, or with 2 after printing an error.
fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(status) => status.into(),
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(ERROR)
        }
    }
}
