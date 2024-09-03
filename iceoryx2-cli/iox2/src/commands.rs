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
use cargo_metadata::MetadataCommand;
use colored::*;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Clone, Debug, PartialEq)]
pub enum CommandType {
    Installed,
    Development,
}

#[derive(Clone, Debug)]
pub struct CommandInfo {
    pub name: String,
    pub path: PathBuf,
    pub command_type: CommandType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathsList {
    build: Vec<PathBuf>,
    install: Vec<PathBuf>,
}

#[cfg(windows)]
const PATH_ENV_VAR_SEPARATOR: char = ';';
#[cfg(windows)]
const COMMAND_EXT: &str = "exe";

#[cfg(not(windows))]
const PATH_ENV_VAR_SEPARATOR: char = ':';
#[cfg(not(windows))]
const COMMAND_EXT: &str = "";

pub trait Environment {
    fn install_paths() -> Result<Vec<PathBuf>>;
    fn build_paths() -> Result<Vec<PathBuf>>;
}

pub struct HostEnvironment;

impl HostEnvironment {
    pub fn target_dir() -> Result<PathBuf> {
        let target_dir = MetadataCommand::new()
            .exec()
            .context("Failed to execute cargo metadata")?
            .target_directory
            .into_std_path_buf();
        Ok(target_dir)
    }

    pub fn build_profile() -> Result<PathBuf> {
        let metadata = MetadataCommand::new()
            .exec()
            .context("Failed to execute cargo metadata")?;

        let profile = metadata
            .resolve
            .as_ref()
            .and_then(|resolve| resolve.root.as_ref())
            .and_then(|root| metadata.packages.iter().find(|p| p.id == *root))
            .and_then(|package| package.name.split('-').next())
            .context("Failed to extract profile information")?;

        // Profile could also be "dev", "test", or something else depending on context.
        // If not release, default to "debug" since this is where the binaries are placed
        // in these other profiles.
        Ok(PathBuf::from(if profile == "release" {
            "release"
        } else {
            "debug"
        }))
    }
}

impl Environment for HostEnvironment {
    fn install_paths() -> Result<Vec<PathBuf>> {
        env::var("PATH")
            .context("Failed to read PATH environment variable")?
            .split(PATH_ENV_VAR_SEPARATOR)
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .map(Ok)
            .collect()
    }

    fn build_paths() -> Result<Vec<PathBuf>> {
        let target_dir = Self::target_dir()?;
        let profile = Self::build_profile()?;

        Ok(vec![target_dir.join(profile)])
    }
}

pub trait CommandFinder<E: Environment> {
    fn paths() -> Result<PathsList>;
    fn commands() -> Result<Vec<CommandInfo>>;
}

pub struct IceoryxCommandFinder<E: Environment> {
    _phantom: std::marker::PhantomData<E>,
}

impl<E> IceoryxCommandFinder<E>
where
    E: Environment,
{
    fn parse_command_name(path: &Path) -> Result<String> {
        let file_stem = path
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .ok_or_else(|| anyhow!("Invalid file name"))?;

        let command_name = file_stem
            .strip_prefix("iox2-")
            .ok_or_else(|| anyhow!("Not an iox2 command: {}", file_stem))?;

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        if extension == COMMAND_EXT {
            Ok(command_name.to_string())
        } else {
            Err(anyhow!("Invalid file extension: {}", extension))
        }
    }

    fn list_commands_in_path(path: &Path, command_type: CommandType) -> Result<Vec<CommandInfo>> {
        let commands = fs::read_dir(path)
            .with_context(|| format!("Failed to read directory at: {:?}", path.to_str()))?
            .map(|entry| {
                entry.map(|e| e.path()).with_context(|| {
                    format!("Failed to read entry in directory: {:?}", path.to_str())
                })
            })
            .filter_map(|path| {
                path.as_ref()
                    .map_err(|e| anyhow!("Failed to get PathBuf: {}", e))
                    .and_then(|path_buf| {
                        Self::parse_command_name(path_buf)
                            .map(|parsed_name| CommandInfo {
                                name: parsed_name,
                                path: path_buf.to_owned(),
                                command_type: command_type.clone(),
                            })
                            .map_err(|e| anyhow!("Failed to parse command name: {}", e))
                    })
                    .ok()
            })
            .collect();

        Ok(commands)
    }
}

impl<E> CommandFinder<E> for IceoryxCommandFinder<E>
where
    E: Environment,
{
    fn paths() -> Result<PathsList> {
        let build = E::build_paths().context("Failed to retrieve build paths")?;
        let install = E::install_paths().context("Failed to retrieve install paths")?;

        Ok(PathsList { build, install })
    }

    fn commands() -> Result<Vec<CommandInfo>> {
        let paths = Self::paths().context("Failed to list paths")?;
        let mut commands = Vec::new();

        for path in &paths.build {
            commands.extend(Self::list_commands_in_path(path, CommandType::Development)?);
        }
        for path in &paths.install {
            commands.extend(Self::list_commands_in_path(path, CommandType::Installed)?);
        }
        commands.sort_by_cached_key(|command| {
            command.path.file_name().unwrap_or_default().to_os_string()
        });

        Ok(commands)
    }
}

pub trait CommandExecutor {
    fn execute(command_info: &CommandInfo, args: Option<&[String]>) -> Result<()>;
}

pub struct IceoryxCommandExecutor;

impl CommandExecutor for IceoryxCommandExecutor {
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
}

fn paths_impl<E>() -> Result<()>
where
    E: Environment,
{
    let paths = IceoryxCommandFinder::<E>::paths().context("Failed to list search paths")?;

    println!("{}", "Build Paths:".bright_green().bold());
    for dir in paths.build {
        println!("  {}", dir.display().to_string().bold());
    }

    println!("\n{}", "Install Paths:".bright_green().bold());
    for dir in paths.install {
        println!("  {}", dir.display().to_string().bold());
    }

    Ok(())
}

pub fn paths() -> Result<()> {
    paths_impl::<HostEnvironment>()
}

fn list_impl<E>() -> Result<()>
where
    E: Environment,
{
    let commands = IceoryxCommandFinder::<E>::commands()?;

    println!("{}", "Discovered Commands:".bright_green().bold());
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

pub fn list() -> Result<()> {
    list_impl::<HostEnvironment>()
}

fn execute_impl<E>(command_name: &str, args: Option<&[String]>, dev_flag: bool) -> Result<()>
where
    E: Environment,
{
    let all_commands =
        IceoryxCommandFinder::<E>::commands().context("Failed to find command binaries")?;

    let command = all_commands
        .into_iter()
        .filter(|command| {
            if dev_flag {
                command.command_type == CommandType::Development
            } else {
                command.command_type == CommandType::Installed
            }
        })
        .find(|command| command.name == command_name)
        .ok_or_else(|| anyhow!("Command not found: {}", command_name))?;

    IceoryxCommandExecutor::execute(&command, args)
}

pub fn execute(command_name: &str, args: Option<&[String]>, dev_flag: bool) -> Result<()> {
    execute_impl::<HostEnvironment>(command_name, args, dev_flag)
}
