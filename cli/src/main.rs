// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The `linlog` binary: one call into the library of the same package.

/// Runs the command line interface.
fn main() -> std::process::ExitCode {
    linlog_cli::main()
}
