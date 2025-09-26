// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

mod execute;
mod list;
mod paths;

pub(crate) use execute::*;
pub(crate) use list::*;
pub(crate) use paths::*;

use anyhow::{anyhow, Context, Result};
use cargo_metadata::MetadataCommand;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(windows)]
const PATH_ENV_VAR_SEPARATOR: char = ';';
#[cfg(windows)]
const COMMAND_EXT: &str = "exe";

#[cfg(not(windows))]
const PATH_ENV_VAR_SEPARATOR: char = ':';
#[cfg(not(windows))]
const COMMAND_EXT: &str = "";

#[derive(Clone, Debug, PartialEq)]
pub enum CommandType {
    Installed,
    Development,
}

#[derive(Clone, Debug)]
pub(crate) struct CommandInfo {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathsList {
    build: Vec<PathBuf>,
    install: Vec<PathBuf>,
}

pub(crate) trait Environment {
    fn install_paths() -> Result<Vec<PathBuf>>;
    fn build_paths() -> Result<Vec<PathBuf>>;
}

pub(crate) struct HostEnvironment;

impl HostEnvironment {
    pub fn target_dir() -> Result<PathBuf> {
        let target_dir = MetadataCommand::new()
            .exec()
            .context("Failed to execute cargo metadata")?
            .target_directory
            .into_std_path_buf();
        Ok(target_dir)
    }
}

impl Environment for HostEnvironment {
    fn install_paths() -> Result<Vec<PathBuf>> {
        let mut install_paths: Vec<PathBuf> = env::var("PATH")
            .context("Failed to read PATH environment variable")?
            .split(PATH_ENV_VAR_SEPARATOR)
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .collect();

        install_paths.sort();
        install_paths.dedup();

        Ok(install_paths)
    }

    fn build_paths() -> Result<Vec<PathBuf>> {
        let target_dir = Self::target_dir()?;
        let build_paths: Vec<PathBuf> = fs::read_dir(target_dir)?
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        return Some(entry.path());
                    }
                }
                None
            })
            .collect();

        Ok(build_paths)
    }
}

pub(crate) trait CommandFinder<E: Environment> {
    fn paths() -> Result<PathsList>;
    fn commands() -> Result<Vec<CommandInfo>>;
}

pub(crate) struct IceoryxCommandFinder<E: Environment> {
    _phantom: core::marker::PhantomData<E>,
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
            .filter_map(|entry_path| {
                entry_path
                    .as_ref()
                    .map_err(|e| anyhow!("Failed to get PathBuf: {}", e))
                    .and_then(|entry_path_buf| {
                        Self::parse_command_name(entry_path_buf)
                            .map(|parsed_name| {
                                // Given that development builds can have different build types
                                // (debug, release, something else), the name needs to be unique to
                                // allow for selection.
                                // Thus, the build type is appended as a suffix.
                                // e.g. foo-debug or foo-release
                                let mut command_name = parsed_name.to_string();
                                if command_type == CommandType::Development {
                                    if let Some(build_type) =
                                        path.file_name().and_then(|os_str| os_str.to_str())
                                    {
                                        const NAME_SEPARATOR: &str = "-";
                                        command_name.push_str(NAME_SEPARATOR);
                                        command_name.push_str(build_type);
                                    }
                                };

                                CommandInfo {
                                    name: command_name,
                                    path: entry_path_buf.to_owned(),
                                }
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
        let build = E::build_paths().unwrap_or_default();
        let install = E::install_paths().unwrap_or_default();

        Ok(PathsList { build, install })
    }

    fn commands() -> Result<Vec<CommandInfo>> {
        let search_paths = Self::paths().context("Failed to list paths")?;
        let mut commands = Vec::new();

        for path in &search_paths.build {
            commands.extend(Self::list_commands_in_path(path, CommandType::Development)?);
        }
        for path in &search_paths.install {
            commands.extend(Self::list_commands_in_path(path, CommandType::Installed)?);
        }
        commands.sort_by_cached_key(|command| {
            command.path.file_name().unwrap_or_default().to_os_string()
        });

        Ok(commands)
    }
}

pub(crate) trait CommandExecutor {
    fn execute(command_info: &CommandInfo, args: Option<&[String]>) -> Result<()>;
}

pub(crate) struct IceoryxCommandExecutor;

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
