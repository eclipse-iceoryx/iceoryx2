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

use anyhow::{Context, Result, anyhow};
use cargo_metadata::MetadataCommand;
use iceoryx2_bb_print::*;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

#[cfg(windows)]
const PATH_ENV_VAR_SEPARATOR: char = ';';
#[cfg(windows)]
const COMMAND_EXT: &str = "exe";

#[cfg(not(windows))]
const PATH_ENV_VAR_SEPARATOR: char = ':';
#[cfg(not(windows))]
const COMMAND_EXT: &str = "";

/// Separates the segments of a command's name, e.g. `iox2-link-tunnel-zenoh`.
const NAME_SEPARATOR: char = '-';

#[derive(Clone, Debug, PartialEq)]
pub enum CommandType {
    Installed,
    Development,
}

#[derive(Clone, Debug)]
pub struct CommandInfo {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathsList {
    pub build: Vec<PathBuf>,
    pub install: Vec<PathBuf>,
}

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
}

impl Environment for HostEnvironment {
    // TODO: This can be optimized to make the command look-up quicker
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
                if let Ok(entry) = entry
                    && entry.path().is_dir()
                {
                    return Some(entry.path());
                }
                None
            })
            .collect();

        Ok(build_paths)
    }
}

pub struct ExternalCommandFinder<E: Environment> {
    _phantom: core::marker::PhantomData<E>,
}

impl<E> ExternalCommandFinder<E>
where
    E: Environment,
{
    fn parse_command_name(path: &Path, prefix: &str) -> Result<String> {
        let file_stem = path
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .ok_or_else(|| anyhow!("Invalid file name"))?;

        let command_name = file_stem.strip_prefix(prefix).ok_or_else(|| {
            anyhow!(
                "Not a {} command: {}",
                prefix.trim_end_matches('-'),
                file_stem
            )
        })?;

        if command_name.contains(NAME_SEPARATOR) {
            return Err(anyhow!(
                "Not a direct {} command: {}",
                prefix.trim_end_matches(NAME_SEPARATOR),
                file_stem
            ));
        }

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        if extension == COMMAND_EXT {
            Ok(command_name.to_string())
        } else {
            Err(anyhow!("Invalid file extension: {}", extension))
        }
    }

    fn list_commands_in_path(
        path: &Path,
        prefix: &str,
        command_type: CommandType,
    ) -> Result<Vec<CommandInfo>> {
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
                        Self::parse_command_name(entry_path_buf, prefix)
                            .map(|parsed_name| {
                                // Given that development builds can have different build types
                                // (debug, release, something else), the name needs to be unique to
                                // allow for selection.
                                // Thus, the build type is appended as a suffix.
                                // e.g. foo-debug or foo-release
                                let mut command_name = parsed_name.to_string();
                                if command_type == CommandType::Development
                                    && let Some(build_type) =
                                        path.file_name().and_then(|os_str| os_str.to_str())
                                {
                                    command_name.push(NAME_SEPARATOR);
                                    command_name.push_str(build_type);
                                }

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

    pub fn paths_for_prefix(_prefix: &str) -> Result<PathsList> {
        let build = E::build_paths().unwrap_or_default();
        let install = E::install_paths().unwrap_or_default();

        Ok(PathsList { build, install })
    }

    /// The commands directly under `prefix`, executables named `prefix`
    /// followed by a single name segment. A command nested deeper is found
    /// through the command it is nested under.
    pub fn commands_with_prefix(prefix: &str) -> Result<Vec<CommandInfo>> {
        let search_paths = Self::paths_for_prefix(prefix).context("Failed to list paths")?;
        let mut commands = Vec::new();

        for path in &search_paths.build {
            commands.extend(Self::list_commands_in_path(
                path,
                prefix,
                CommandType::Development,
            )?);
        }
        for path in &search_paths.install {
            commands.extend(Self::list_commands_in_path(
                path,
                prefix,
                CommandType::Installed,
            )?);
        }
        commands.sort_by_cached_key(|command| {
            command.path.file_name().unwrap_or_default().to_os_string()
        });

        Ok(commands)
    }
}

pub trait CommandExecutor {
    /// Runs the command to completion and returns its exit status.
    fn execute(command_info: &CommandInfo, args: Option<&[String]>) -> Result<ExitStatus>;
}

pub struct ExternalCommandExecutor;

impl ExternalCommandExecutor {
    fn command(command_info: &CommandInfo, args: Option<&[String]>) -> Command {
        let mut command = Command::new(&command_info.path);
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        if let Some(arguments) = args {
            command.args(arguments);
        }
        command
    }

    /// Hands this process over to the command, so its exit status and the
    /// signals sent to this process come from the command. Returns only when
    /// the command could not be started.
    #[cfg(unix)]
    pub fn replace(command_info: &CommandInfo, args: Option<&[String]>) -> anyhow::Error {
        use std::os::unix::process::CommandExt;

        let error = Self::command(command_info, args).exec();
        anyhow!(
            "Failed to execute command {:?}: {}",
            command_info.path,
            error
        )
    }

    /// Runs the command and exits with its exit status, so the status is the
    /// from the command. Returns only when the command could not be started.
    #[cfg(not(unix))]
    pub fn replace(command_info: &CommandInfo, args: Option<&[String]>) -> anyhow::Error {
        match Self::execute(command_info, args) {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(error) => error,
        }
    }
}

impl CommandExecutor for ExternalCommandExecutor {
    fn execute(command_info: &CommandInfo, args: Option<&[String]>) -> Result<ExitStatus> {
        Self::command(command_info, args)
            .status()
            .with_context(|| format!("Failed to execute command: {:?}", command_info.path))
    }
}

/// Runs the command `command_name` found under `prefix` in place of this
/// process. Returns only when the command is not found or could not be
/// started.
pub fn execute<E: Environment>(
    prefix: &str,
    command_name: &str,
    args: Option<&[String]>,
) -> Result<()> {
    let all_commands = ExternalCommandFinder::<E>::commands_with_prefix(prefix)
        .context("Failed to find command binaries")?;

    let command = all_commands
        .into_iter()
        .find(|command| command.name == command_name)
        .ok_or_else(|| anyhow!("Command not found: {}", command_name))?;

    Err(ExternalCommandExecutor::replace(&command, args))
}

pub fn list<E: Environment>(prefix: &str) -> Result<()> {
    let commands = ExternalCommandFinder::<E>::commands_with_prefix(prefix)?;

    println!("{BRIGHT_GREEN}{BOLD}Discovered Commands:{RESET}");
    for command in commands {
        println!("{BOLD}  {}{RESET}", command.name);
    }

    Ok(())
}

pub fn paths<E: Environment>(prefix: &str) -> Result<()> {
    let search_paths = ExternalCommandFinder::<E>::paths_for_prefix(prefix)
        .context("Failed to list search paths")?;

    if !search_paths.build.is_empty() {
        println!("{BRIGHT_GREEN}{BOLD}Build Paths:{RESET}",);
        for dir in &search_paths.build {
            println!("{BOLD}  {}{RESET}", dir.display());
        }
        println!();
    }
    if !search_paths.install.is_empty() {
        println!("{BRIGHT_GREEN}{BOLD}Install Paths:{RESET}");
        for dir in &search_paths.install {
            println!("{BOLD}  {}{RESET}", dir.display(),);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use iceoryx2_bb_testing::assert_that;
    use std::env;
    use std::fs::File;
    use tempfile::TempDir;

    const PREFIX: &str = "iox2-";
    const FOO_COMMAND: &str = "Xt7bK9pL";
    const BAR_COMMAND: &str = "m3Qf8RzN";
    const BAZ_COMMAND: &str = "P5hJ2wAc";
    const NESTED_COMMAND: &str = "Qw4Er7Ty";

    fn create_noop_executable(file_path: &std::path::Path) -> std::io::Result<()> {
        use std::process::Command;

        let src_file = file_path.with_extension("rs");
        std::fs::write(&src_file, "fn main() {}")?;
        let output = Command::new("rustc")
            .arg(&src_file)
            .arg("-o")
            .arg(file_path)
            .arg("--crate-type")
            .arg("bin")
            .output()?;
        std::fs::remove_file(&src_file).ok();

        if !output.status.success() {
            return Err(std::io::Error::other(format!(
                "Failed to compile noop executable: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    macro_rules! create_file {
        ($path:expr, $file:expr) => {{
            let file_path = $path.join($file);

            #[cfg(unix)]
            const COMMAND_EXT: &str = "";
            #[cfg(windows)]
            const COMMAND_EXT: &str = "exe";

            let extension = file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            if extension == COMMAND_EXT || (cfg!(unix) && extension.is_empty()) {
                create_noop_executable(&file_path).expect("Failed to create noop executable");
            } else {
                // For non-executable files (like .d files), just create an empty file
                File::create(&file_path).expect("Failed to create file");
            }
        }};
    }

    struct TestEnv {
        _temp_dir: TempDir,
        original_path: String,
    }

    impl TestEnv {
        fn setup() -> Self {
            let original_path = env::var("PATH").expect("Failed to get PATH");

            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let temp_path = temp_dir.path().to_path_buf();

            let mut paths = env::split_paths(&original_path).collect::<Vec<_>>();
            paths.push(temp_path.clone());
            let new_path = env::join_paths(paths).expect("Failed to join paths");
            unsafe {
                env::set_var("PATH", &new_path);
            }

            create_file!(temp_path, format!("{}{}", PREFIX, FOO_COMMAND));
            create_file!(temp_path, format!("{}{}.d", PREFIX, FOO_COMMAND));
            create_file!(temp_path, format!("{}{}.exe", PREFIX, FOO_COMMAND));
            create_file!(temp_path, format!("{}{}", PREFIX, BAR_COMMAND));
            create_file!(temp_path, format!("{}{}.d", PREFIX, BAR_COMMAND));
            create_file!(temp_path, format!("{}{}.exe", PREFIX, BAR_COMMAND));
            create_file!(
                temp_path,
                format!("{}{}-{}", PREFIX, FOO_COMMAND, NESTED_COMMAND)
            );
            create_file!(temp_path, BAZ_COMMAND);
            create_file!(temp_path, format!("{}.d", BAZ_COMMAND));
            create_file!(temp_path, format!("{}.exe", BAZ_COMMAND));

            TestEnv {
                _temp_dir: temp_dir,
                original_path,
            }
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            unsafe {
                env::set_var("PATH", &self.original_path);
            }
        }
    }

    #[test]
    fn test_list() {
        let _test_env = TestEnv::setup();

        let commands = ExternalCommandFinder::<HostEnvironment>::commands_with_prefix(PREFIX)
            .expect("Failed to retrieve commands");

        assert_that!(
            commands,
            contains_match | command | command.name == FOO_COMMAND
        );
        assert_that!(
            commands,
            contains_match | command | command.name == BAR_COMMAND
        );
        assert_that!(
            commands,
            not_contains_match | command | command.name == BAZ_COMMAND
        );
    }

    #[test]
    fn a_nested_command_is_listed_only_under_the_command_it_nests_in() {
        let _test_env = TestEnv::setup();

        let top_level = ExternalCommandFinder::<HostEnvironment>::commands_with_prefix(PREFIX)
            .expect("Failed to retrieve commands");
        let nested = ExternalCommandFinder::<HostEnvironment>::commands_with_prefix(&format!(
            "{}{}-",
            PREFIX, FOO_COMMAND
        ))
        .expect("Failed to retrieve nested commands");

        assert_that!(
            top_level,
            not_contains_match | command | command.name.contains(NESTED_COMMAND)
        );
        assert_that!(
            nested,
            contains_match | command | command.name == NESTED_COMMAND
        );
    }

    #[test]
    fn test_execute() {
        let _test_env = TestEnv::setup();

        let commands = ExternalCommandFinder::<HostEnvironment>::commands_with_prefix(PREFIX)
            .unwrap_or_else(|e| {
                panic!("Failed to retrieve commands: {}", e);
            });

        let [foo_command, ..] = commands
            .iter()
            .filter(|cmd| cmd.name == FOO_COMMAND)
            .collect::<Vec<_>>()[..]
        else {
            panic!("Failed to extract CommandInfo of test files");
        };

        let result = ExternalCommandExecutor::execute(foo_command, None);
        if let Err(ref e) = result {
            println!("Error executing command: {}", e);
        }

        assert_that!(result, is_ok);

        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let result = ExternalCommandExecutor::execute(foo_command, Some(&args));
        assert_that!(result, is_ok);
    }
}
