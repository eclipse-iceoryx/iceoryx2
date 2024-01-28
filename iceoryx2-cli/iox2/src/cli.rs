use clap::Parser;

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
}

fn help_template() -> &'static str {
    "\
    USAGE: iox2 [OPTIONS] <COMMAND>\n\n\
    OPTIONS:\n{options}\n\n\
    COMMANDS:\n{subcommands}\n\
    \u{00A0}\u{00A0}...         See all installed commands with --list"
}
