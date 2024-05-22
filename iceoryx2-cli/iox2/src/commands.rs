use colored::*;
use std::env;
use std::fs;
use std::io;
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
    println!("{}", "Installed Commands:".bright_green().bold());
    let mut installed_commands = find();
    installed_commands.sort_by_key(|command| command.name.clone());
    installed_commands
        .iter()
        .map(|command| {
            format!(
                "  {}",
                format!(
                    "{} {}",
                    command.name.bold(),
                    if command.is_development {
                        "(dev) ".italic()
                    } else {
                        "".italic()
                    },
                )
            )
        })
        .for_each(|formatted_command| println!("{}", formatted_command));
}

fn get_build_path_dirs() -> Result<std::path::PathBuf, io::Error> {
    let current_exe = env::current_exe()?;

    let build_type = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    let build_path = current_exe
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join(build_type))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Failed to construct path of development dir",
            )
        })?;

    Ok(build_path)
}

fn get_install_path_dirs() -> impl Iterator<Item = std::path::PathBuf> {
    env::var("PATH")
        .ok()
        .into_iter()
        .flat_map(|path_var| env::split_paths(&path_var).collect::<Vec<_>>())
}

fn find() -> Vec<CommandInfo> {
    let development_commands = find_development_command_binaries();
    let installed_commands = find_installed_command_binaries();

    let mut all_commands = development_commands;
    all_commands.extend(installed_commands.iter().cloned());
    all_commands
}

fn find_development_command_binaries() -> Vec<CommandInfo> {
    let development_binaries_dir = match get_build_path_dirs() {
        Ok(location) => location,
        Err(_) => return Vec::new(),
    };

    fs::read_dir(development_binaries_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| is_valid_command_binary(path))
        .filter_map(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|command_name| {
                    let stripped = command_name.strip_prefix("iox2-").unwrap_or(command_name);
                    CommandInfo {
                        name: stripped.to_string(),
                        path: path.clone(),
                        is_development: true,
                    }
                })
        })
        .collect()
}

fn find_installed_command_binaries() -> Vec<CommandInfo> {
    get_install_path_dirs()
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

pub fn paths() {
    let mut development_binaries_dirs = Vec::new();
    development_binaries_dirs.extend(get_build_path_dirs().ok());

    let mut installed_binaries_dirs = Vec::new();
    installed_binaries_dirs.extend(get_install_path_dirs());

    println!("{}", "Development Binary Paths:".bright_green().bold());
    for dir in &development_binaries_dirs {
        println!("  {}", dir.display().to_string().bold());
    }

    println!();

    println!("{}", "Installed Binary Paths:".bright_green().bold());
    for dir in &installed_binaries_dirs {
        println!("  {}", dir.display().to_string().bold());
    }
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
