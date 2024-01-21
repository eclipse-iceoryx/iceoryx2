#[cfg(not(debug_assertions))]
use human_panic::setup_panic;

#[cfg(debug_assertions)]
extern crate better_panic;

mod commands;

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

    let commands = commands::find();

    println!("Available commands:");
    for command in commands {
        println!("- {}", command);
    }
}
