use colored::*;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn list() {
    println!("Installed Commands:");
    let installed_commands = find();
    for command in installed_commands {
        println!("  {}", command.bold());
    }
}

fn find() -> Vec<String> {
    let mut commands = find_command_binaries_in_development_dirs();
    if commands.is_empty() {
        commands = find_command_binaries_in_system_path();
    }
    commands
}

fn find_command_binaries_in_development_dirs() -> Vec<String> {
    let mut commands = Vec::new();
    let current_exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(_) => return commands,
    };
    let target_dir_name = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let target_dir = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(target_dir_name);

    if let Ok(entries) = fs::read_dir(&target_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if is_valid_command_binary(&path) {
                if let Some(command_name) = path.file_name().and_then(|n| n.to_str()) {
                    let stripped = command_name.strip_prefix("iox2-").unwrap_or(command_name);
                    commands.push(stripped.to_string());
                }
            }
        }
    }

    commands
}

fn find_command_binaries_in_system_path() -> Vec<String> {
    let mut commands = Vec::new();
    if let Ok(path_var) = env::var("PATH") {
        for path in env::split_paths(&path_var) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if is_valid_command_binary(&path) {
                        if let Some(command_name) = path.file_name().and_then(|n| n.to_str()) {
                            commands.push(command_name.to_string());
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
