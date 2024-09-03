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

#[cfg(test)]
mod tests {

    use iceoryx2_bb_testing::assert_that;
    use iox2::commands::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[cfg(unix)]
    use std::{
        fs::{metadata, set_permissions},
        os::unix::fs::PermissionsExt,
    };

    #[cfg(windows)]
    const NOOP: &[u8] = include_bytes!("noop.exe");

    const IOX2_PREFIX: &str = "iox2-";
    const FOO_COMMAND: &str = "Xt7bK9pL";
    const BAR_COMMAND: &str = "m3Qf8RzN";
    const BAZ_COMMAND: &str = "P5hJ2wAc";

    macro_rules! create_file {
        ($path:expr, $file:expr) => {{
            let file_path = $path.join($file);
            #[cfg(unix)]
            {
                let mut file = File::create(&file_path).expect("Failed to create file");
                file.write_all(b"#!/bin/sh\nexit 0\n").expect("");

                let mut perms = metadata(&file_path)
                    .expect("Failed to get metadata")
                    .permissions();
                perms.set_mode(0o755);
                set_permissions(&file_path, perms).expect("Failed to set permissions");
            }
            #[cfg(windows)]
            {
                const COMMAND_EXT: &str = "exe";

                let mut file = File::create(&file_path).expect("Failed to create file");

                if file_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    == COMMAND_EXT
                {
                    file.write_all(&NOOP)
                        .expect("Failed to write executable content");
                }
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
