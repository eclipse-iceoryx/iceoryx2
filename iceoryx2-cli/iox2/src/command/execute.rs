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

use anyhow::{anyhow, Context, Result};

use crate::command::{
    CommandExecutor, CommandFinder, Environment, HostEnvironment, IceoryxCommandExecutor,
    IceoryxCommandFinder,
};

fn execute_impl<E>(command_name: &str, args: Option<&[String]>) -> Result<()>
where
    E: Environment,
{
    let all_commands =
        IceoryxCommandFinder::<E>::commands().context("Failed to find command binaries")?;

    let command = all_commands
        .into_iter()
        .find(|command| command.name == command_name)
        .ok_or_else(|| anyhow!("Command not found: {}", command_name))?;

    IceoryxCommandExecutor::execute(&command, args)
}

pub(crate) fn execute(command_name: &str, args: Option<&[String]>) -> Result<()> {
    execute_impl::<HostEnvironment>(command_name, args)
}

#[cfg(test)]
mod tests {

    use super::*;
    use iceoryx2_bb_testing::assert_that;
    use std::env;
    use std::fs::File;
    use tempfile::TempDir;

    const IOX2_PREFIX: &str = "iox2-";
    const FOO_COMMAND: &str = "Xt7bK9pL";
    const BAR_COMMAND: &str = "m3Qf8RzN";
    const BAZ_COMMAND: &str = "P5hJ2wAc";

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
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to compile noop executable: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
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
            env::set_var("PATH", &new_path);

            create_file!(temp_path, format!("{}{}", IOX2_PREFIX, FOO_COMMAND));
            create_file!(temp_path, format!("{}{}.d", IOX2_PREFIX, FOO_COMMAND));
            create_file!(temp_path, format!("{}{}.exe", IOX2_PREFIX, FOO_COMMAND));
            create_file!(temp_path, format!("{}{}", IOX2_PREFIX, BAR_COMMAND));
            create_file!(temp_path, format!("{}{}.d", IOX2_PREFIX, BAR_COMMAND));
            create_file!(temp_path, format!("{}{}.exe", IOX2_PREFIX, BAR_COMMAND));
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
            env::set_var("PATH", &self.original_path);
        }
    }

    #[test]
    fn test_list() {
        let _test_env = TestEnv::setup();

        let commands = IceoryxCommandFinder::<HostEnvironment>::commands()
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
    fn test_execute() {
        let _test_env = TestEnv::setup();

        let commands = IceoryxCommandFinder::<HostEnvironment>::commands().unwrap_or_else(|e| {
            panic!("Failed to retrieve commands: {}", e);
        });

        let [foo_command, ..] = commands
            .iter()
            .filter(|cmd| cmd.name == FOO_COMMAND)
            .collect::<Vec<_>>()[..]
        else {
            panic!("Failed to extract CommandInfo of test files");
        };

        let result = IceoryxCommandExecutor::execute(&foo_command, None);
        if let Err(ref e) = result {
            println!("Error executing command: {}", e);
        }

        assert_that!(result, is_ok);

        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let result = IceoryxCommandExecutor::execute(&foo_command, Some(&args));
        assert_that!(result, is_ok);
    }
}
