use colored::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Clone, Debug)]
struct CommandInfo {
    name: String,
    path: PathBuf,
    is_development: bool,
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Command not found: {0}")]
    NotFound(String),
    #[error("Execution failed: {0}")]
    Failed(String),
}

pub fn list() {
    println!("Installed Commands:");
    let installed_commands = find();
    for command in installed_commands {
        println!(
            "  {}",
            format!(
                "{}{}",
                if command.is_development { "(dev) " } else { "" },
                command.name.bold(),
            )
        );
    }
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
    env::var("PATH")
        .ok()
        .into_iter()
        .flat_map(|path_var| env::split_paths(&path_var).collect::<Vec<_>>())
        .flat_map(|path: PathBuf| {
            fs::read_dir(path)
                .into_iter()
                .flat_map(|read_dir| read_dir.filter_map(Result::ok))
        })
        .filter_map(|entry| {
            let path = entry.path();
            if is_valid_command_binary(&path) {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|command_name| CommandInfo {
                        name: command_name.to_string(),
                        path: path.clone(),
                        is_development: false,
                    })
            } else {
                None
            }
        })
        .collect()
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

pub fn execute_external_command(
    command_name: &str,
    args: &[String],
    dev_flag_present: bool,
) -> Result<(), ExecutionError> {
    let available_commands = find();
    if let Some(command_info) = available_commands.into_iter().find(|c| {
        &c.name == command_name
            && if dev_flag_present {
                c.is_development == true
            } else {
                if c.is_development {
                    println!(
                        "Development version of {} found but --dev flag is not set.",
                        command_name
                    )
                }
                false
            }
    }) {
        execute(&command_info, Some(args))
    } else {
        Err(ExecutionError::NotFound(command_name.to_string()))
    }
}

fn execute(command_info: &CommandInfo, args: Option<&[String]>) -> Result<(), ExecutionError> {
    let mut command = Command::new(&command_info.path);
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    if let Some(arguments) = args {
        command.args(arguments);
    }
    match command.status() {
        Ok(_) => Ok(()),
        Err(e) => Err(ExecutionError::Failed(format!(
            "Failed to execute command: {}",
            e
        ))),
    }
}
