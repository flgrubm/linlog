// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
/// A linear logic suite for all your needs
#[command(version, about, long_about)]
pub(crate) struct Cli {
    #[arg(short, long)]
    /// Placeholder flag, until the subcommands are dispatched
    pub(crate) bb: bool,
    #[command(subcommand)]
    /// The subcommand to run
    pub(crate) command: Command,
}

/// The top-level subcommands.
#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Transform sequent representations
    Seq {
        #[command(subcommand)]
        /// What to do with the sequent
        action: SeqCommand,
    },
}

/// The subcommands of `seq`.
#[derive(Subcommand, Debug)]
pub(crate) enum SeqCommand {
    /// Pretty print a sequent
    Pretty {
        #[command(subcommand)]
        /// The input format
        action: PrettyCommand,
    },
    /// Output a sequent in JSON format
    Serialize {
        #[command(subcommand)]
        /// The input format
        action: SerializeCommand,
    },
}

/// The input formats of `seq pretty`.
#[derive(Subcommand, Debug)]
pub(crate) enum PrettyCommand {
    /// Parse human readable sequent
    Prose {
        #[arg(short, long)]
        /// Input string
        input: Option<String>,
        #[arg(short = 'f', long = "file")]
        /// Input file location
        input_file: Option<PathBuf>,
        #[arg(short, long = "output")]
        /// Output file location
        output_file: Option<PathBuf>,
    },
    /// Parse JSON-encoded sequent
    Json {
        #[arg(short, long)]
        /// Input string
        input: Option<String>,
        #[arg(short = 'f', long = "file")]
        /// Input file location
        input_file: Option<PathBuf>,
        #[arg(short, long = "output")]
        /// Output file location
        output_file: Option<PathBuf>,
    },
}

/// The input formats of `seq serialize`.
#[derive(Subcommand, Debug)]
pub(crate) enum SerializeCommand {
    /// Parse human readable sequent
    Prose {
        #[arg(short, long)]
        /// Input string
        input: Option<String>,
        #[arg(short = 'f', long = "file")]
        /// Input file location
        input_file: Option<PathBuf>,
        #[arg(short, long = "output")]
        /// Output file location
        output_file: Option<PathBuf>,
    },
    /// Parse JSON-encoded sequent
    Json {
        #[arg(short, long)]
        /// Input string
        input: Option<String>,
        #[arg(short = 'f', long = "file")]
        /// Input file location
        input_file: Option<PathBuf>,
        #[arg(short, long = "output")]
        /// Output file location
        output_file: Option<PathBuf>,
        #[arg(short = 'p', long)]
        /// Optimize
        optimize: bool,
    },
}
