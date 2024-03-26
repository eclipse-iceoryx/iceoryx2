#[cfg(not(debug_assertions))]
use human_panic::setup_panic;

#[cfg(debug_assertions)]
extern crate better_panic;

mod cli;
mod commands;

use clap::Parser;

fn main() {
    #[cfg(not(debug_assertions))]
    {
        setup_panic!();
    }
    #[cfg(debug_assertions)]
    {
        better_panic::Settings::debug()
            .most_recent_first(false)
            .lineno_suffix(true)
            .verbosity(better_panic::Verbosity::Full)
            .install();
    }

    let cli = cli::Cli::parse();

    if cli.list {
        commands::list();
    } else if !cli.external_command.is_empty() {
        let command_name = &cli.external_command[0];
        let command_args = &cli.external_command[1..];
        match commands::execute_external_command(command_name, command_args, cli.dev) {
            Ok(()) => {
                // Command executed successfully, nothing to do
            }
            Err(commands::ExecutionError::NotFound(_)) => {
                // Command not found, print help
                println!("Command not found. See all installed commands with --list.");
            }
            Err(commands::ExecutionError::Failed(_)) => {
                println!("Command found but execution failed ...");
            }
        }
    }
}
