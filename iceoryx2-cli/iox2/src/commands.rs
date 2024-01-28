use colored::*;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn list() {
    println!("Installed Commands:");
    let installed_commands = find();
    for command in installed_commands {
        println!(
            "  {}",
            format!(
                "{}{}",
                command.name.bold(),
                if command.is_development { " (dev)" } else { "" }
            )
        );
    }
}

#[derive(Clone, Debug)]
struct CommandInfo {
    name: String,
    path: PathBuf,
    is_development: bool,
}

fn find() -> Vec<CommandInfo> {
    let development_commands = find_command_binaries_in_development_dirs();
    let installed_commands = find_command_binaries_in_system_path();

    let mut all_commands = development_commands;
    all_commands.extend(installed_commands.iter().cloned());
    all_commands
}

fn find_command_binaries_in_development_dirs() -> Vec<CommandInfo> {
    let mut commands = Vec::new();
    let current_exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(_) => return commands,
    };
    let build_type = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    // Get the location of the binary directory for the build
    let binary_dir = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(build_type);

    if let Ok(entries) = fs::read_dir(&binary_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if is_valid_command_binary(&path) {
                if let Some(command_name) = path.file_name().and_then(|n| n.to_str()) {
                    let stripped = command_name.strip_prefix("iox2-").unwrap_or(command_name);
                    commands.push(CommandInfo {
                        name: stripped.to_string(),
                        path,
                        is_development: true,
                    });
                }
            }
        }
    }

    commands
}

fn find_command_binaries_in_system_path() -> Vec<CommandInfo> {
    let mut commands = Vec::new();
    if let Ok(path_var) = env::var("PATH") {
        for path in env::split_paths(&path_var) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if is_valid_command_binary(&path) {
                        if let Some(command_name) = path.file_name().and_then(|n| n.to_str()) {
                            commands.push(CommandInfo {
                                name: command_name.to_string(),
                                path,
                                is_development: false,
                            });
                        }
                    }
                }
            }
        }
    }

    commands
}

fn is_valid_command_binary(path: &PathBuf) -> bool {
    path.is_file()
        && path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("iox2-")
        && path.extension().is_none() // Exclude files with extensions (e.g. '.d')
}
