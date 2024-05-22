use clap::Parser;
use colored::*;

#[derive(Parser, Debug)]
#[command(
    name = "iox2",
    about = "The command-line interface to iceoryx2",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = true,
    help_template = help_template(),
)]
pub struct Cli {
    #[arg(short, long, help = "List all installed commands")]
    pub list: bool,

    #[arg(
        short,
        long,
        help = "Specify to execute development versions of commands if they exist"
    )]
    pub dev: bool,

    #[arg(hide = true, required = false)]
    pub external_command: Vec<String>,
}

fn help_template() -> String {
    format!(
        "{}{}{}\n\n{}\n{{options}}\n\n{}\n{{subcommands}}{}{}",
        "Usage: ".bright_green().bold(),
        "iox2 ".bold(),
        "[OPTIONS] [COMMAND]",
        "Options:".bright_green().bold(),
        "Commands:".bright_green().bold(),
        "  ...         ".bold(),
        "See all installed commands with --list"
    )
}
