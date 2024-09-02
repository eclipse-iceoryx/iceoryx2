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

use anyhow::{anyhow, Context, Result};
use colored::*;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Clone, Debug, PartialEq)]
enum CommandType {
    Installed,
    Development,
}

#[derive(Clone, Debug)]
struct CommandInfo {
    name: String,
    path: PathBuf,
    command_type: CommandType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PathsList {
    dev_dirs: Vec<PathBuf>,
    install_dirs: Vec<PathBuf>,
}

fn build_dirs() -> Result<Vec<PathBuf>> {
    let current_exe = env::current_exe().context("Failed to get current executable path")?;

    let build_type = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    let path = current_exe
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join(build_type))
        .filter(|p| p.is_dir())
        .with_context(|| {
            format!(
                "Unable to determine build path from executable: {:?}",
                current_exe
            )
        })?;

    Ok(vec![path])
}

fn install_dirs() -> Result<Vec<PathBuf>> {
    env::var("PATH")
        .context("Failed to read PATH environment variable")?
        .split(':')
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .map(Ok)
        .collect()
}

fn list_paths() -> Result<PathsList> {
    let dev_dirs = build_dirs().context("Failed to retrieve development binary paths")?;
    let install_dirs = install_dirs().context("Failed to retrieve installed binary paths")?;

    Ok(PathsList {
        dev_dirs,
        install_dirs,
    })
}

pub fn paths() -> Result<()> {
    let paths = list_paths().context("Failed to list paths")?;

    println!("{}", "Development Paths:".bright_green().bold());
    for dir in paths.dev_dirs {
        println!("  {}", dir.display().to_string().bold());
    }

    println!("\n{}", "Installed Paths:".bright_green().bold());
    for dir in paths.install_dirs {
        println!("  {}", dir.display().to_string().bold());
    }

    Ok(())
}

fn parse_command_name(path: &PathBuf) -> Result<String> {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| anyhow!("Invalid file name"))
        .and_then(|file_name| {
            if path.extension().is_some() {
                Err(anyhow!("File has an extension: {}", file_name))
            } else {
                Ok(file_name)
            }
        })
        .and_then(|file_name| {
            file_name
                .strip_prefix("iox2-")
                .map(String::from)
                .ok_or_else(|| anyhow!("Not an iox2 command: {}", file_name))
        })
}

fn list_commands_at_path(dir: &PathBuf, command_type: CommandType) -> Result<Vec<CommandInfo>> {
    let commands = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            path.file_name().and_then(OsStr::to_str).and_then(|_| {
                parse_command_name(&path)
                    .ok()
                    .map(|parsed_name| CommandInfo {
                        name: parsed_name,
                        path: path.clone(),
                        command_type: command_type.clone(),
                    })
            })
        })
        .collect();

    Ok(commands)
}

fn list_commands() -> Result<Vec<CommandInfo>> {
    let paths = list_paths().context("Failed to list paths")?;
    let mut commands = Vec::new();

    for dir in &paths.dev_dirs {
        commands.extend(list_commands_at_path(dir, CommandType::Development)?);
    }
    for dir in &paths.install_dirs {
        commands.extend(list_commands_at_path(dir, CommandType::Installed)?);
    }
    commands
        .sort_by_cached_key(|command| command.path.file_name().unwrap_or_default().to_os_string());

    Ok(commands)
}

pub fn list() -> Result<()> {
    let commands = list_commands()?;

    println!("{}", "Installed Commands:".bright_green().bold());
    for command in commands {
        let dev_indicator = if command.command_type == CommandType::Development {
            " (dev)".italic()
        } else {
            "".into()
        };
        println!("  {}{}", command.name.bold(), dev_indicator);
    }

    Ok(())
}

pub fn execute_external_command(
    command_name: &str,
    args: Option<&[String]>,
    dev_flag_present: bool,
) -> Result<()> {
    let commands = list_commands().context("Failed to find command binaries")?;
    let command_info = commands
        .into_iter()
        .filter(|command| {
            if dev_flag_present {
                command.command_type == CommandType::Development
            } else {
                command.command_type == CommandType::Installed
            }
        })
        .find(|command| command.name == command_name)
        .ok_or_else(|| anyhow!("Command not found: {}", command_name))?;
    execute(&command_info, args)
}

fn execute(command_info: &CommandInfo, args: Option<&[String]>) -> Result<()> {
    let mut command = Command::new(&command_info.path);
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    if let Some(arguments) = args {
        command.args(arguments);
    }
    command
        .status()
        .with_context(|| format!("Failed to execute command: {:?}", command_info.path))?;
    Ok(())
}
