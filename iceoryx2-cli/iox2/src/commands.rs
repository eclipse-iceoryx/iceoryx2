// Copyright (c) 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use colored::*;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
enum CommandType {
    Installed,
    Development,
}

#[derive(Clone, Debug)]
struct CommandInfo {
    path: PathBuf,
    command_type: CommandType,
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

    if let Ok(mut commands) = find_command_binaries() {
        commands.sort_by_cached_key(|command| {
            command
                .path
                .file_name()
                .expect("Could not extract file name from command binary path")
                .to_os_string()
        });
        commands
            .iter()
            .map(|command| {
                format!(
                    "  {} {}",
                    // TODO: Simplify logic for extracting name to remove this duplication
                    command
                        .path
                        .file_name()
                        .and_then(|os_str| os_str.to_str())
                        .and_then(|command_name| command_name.strip_prefix("iox2-"))
                        .expect("Unable to extract command name from command path")
                        .bold(),
                    if command.command_type == CommandType::Development {
                        "(dev)".italic()
                    } else {
                        "".italic()
                    },
                )
            })
            .for_each(|formatted_command| println!("{}", formatted_command));
    } else {
        // TODO: handle error ...
    }
}

fn build_dirs() -> Result<Vec<PathBuf>, std::io::Error> {
    let current_exe = env::current_exe()?;

    let build_type = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    let paths = current_exe
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join(build_type))
        .filter(|p| p.is_dir())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Unable to determine build path from executable path",
            )
        })?;

    Ok(vec![paths])
}

fn install_dirs() -> Result<Vec<PathBuf>, std::io::Error> {
    env::var("PATH")
        .map(|paths| {
            paths
                .split(':')
                .map(PathBuf::from)
                .filter(|p| p.is_dir())
                .collect()
        })
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Unable to determine install path from environment",
            )
        })
}

fn find_command_binaries_in_path(dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    dirs.into_iter()
        .flat_map(|dir| fs::read_dir(dir).into_iter().flatten())
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file()) // filter out dirs
        .filter(|path| path.extension().is_none()) // valid binaries have no extension (e.g. '.d')
        .filter(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .filter(|name| name.starts_with("iox2-")) // iox2 command start with 'iox2-'
                .is_some()
        })
        .collect()
}

fn find_command_binaries() -> Result<Vec<CommandInfo>, io::Error> {
    let mut command_binaries: Vec<CommandInfo> = Vec::new();

    // Find binaries in both build dirs and the install path
    command_binaries.extend(
        find_command_binaries_in_path(build_dirs()?)
            .into_iter()
            .map(|path| CommandInfo {
                path,
                command_type: CommandType::Development,
            }),
    );
    command_binaries.extend(
        find_command_binaries_in_path(install_dirs()?)
            .into_iter()
            .map(|path| CommandInfo {
                path,
                command_type: CommandType::Installed,
            }),
    );

    Ok(command_binaries)
}

pub fn paths() {
    match build_dirs() {
        Ok(dirs) => {
            println!("{}", "Development Binary Paths:".bright_green().bold());
            for dir in dirs {
                println!("  {}", dir.display().to_string().bold());
            }
        }
        Err(e) => {
            println!("Error retrieving build dirs: {e}");
        }
    }
    match install_dirs() {
        Ok(dirs) => {
            println!("{}", "Installed Binary Paths:".bright_green().bold());
            for dir in dirs {
                println!("  {}", dir.display().to_string().bold());
            }
        }
        Err(e) => {
            println!("Error retrieving install dirs: {e}");
        }
    }
}

pub fn execute_external_command(
    command_name: &str,
    args: &[String],
    dev_flag_present: bool,
) -> Result<(), ExecutionError> {
    if let Ok(commands) = find_command_binaries() {
        let command_info = commands
            .into_iter()
            .filter(|command| {
                if dev_flag_present {
                    command.command_type == CommandType::Development
                } else {
                    command.command_type == CommandType::Installed
                }
            })
            .find(|command| {
                command
                    // TODO: Simplify logic for extracting name to remove this duplication
                    .path
                    .file_name()
                    .and_then(|os_str| os_str.to_str())
                    .and_then(|command_name| command_name.strip_prefix("iox2-"))
                    .expect("Unable to extract command name from command path")
                    == command_name
            })
            .ok_or_else(|| ExecutionError::NotFound(command_name.to_string()))?;
        execute(&command_info, Some(args)) // TODO: Remove Some() - pass optional directly
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
