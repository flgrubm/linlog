// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

mod argument_parsing;

use argument_parsing::Cli;
use clap::Parser;

/// Parses the command line arguments and runs the CLI.
fn main() {
    let cli = Cli::parse();

    if cli.bb {
        println!("bb is true")
    } else {
        println!("bb is false")
    }
}
