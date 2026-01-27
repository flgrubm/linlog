// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
/// A linear logic suite for all your needs
#[command(version, about, long_about)]
pub(crate) struct Cli {
    #[arg(short, long)]
    pub(crate) bb: bool,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Transform sequent representations
    Seq {
        #[command(subcommand)]
        action: SeqCommand,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum SeqCommand {
    /// Pretty print a sequent
    Pretty {
        #[command(subcommand)]
        action: PrettyCommand,
    },
    /// Output a sequent in JSON format
    Serialize {
        #[command(subcommand)]
        action: SerializeCommand,
    },
}

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
