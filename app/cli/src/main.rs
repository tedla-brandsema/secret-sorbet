use std::io::{self, BufRead};
use std::process::ExitCode;

use clap::Parser;
use secret_sorbet_cli::{Cli, init_diagnostics, run};
use tracing::error;

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Err(error) = init_diagnostics(cli.verbosity()) {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    // Do not hold the stdin lock while the hidden prompt owns the terminal.
    // This matters on platforms where console input and stdin share a handle.
    let mut input: Box<dyn BufRead + '_> = if cli.uses_standard_input() {
        Box::new(stdin.lock())
    } else {
        Box::new(io::empty())
    };
    let mut output = stdout.lock();

    match run(cli, &mut *input, &mut output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!(event = "command_failed", error_code = error.code());
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
