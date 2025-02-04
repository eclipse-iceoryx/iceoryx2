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

mod node_name {
    use iceoryx2::prelude::*;
    use iceoryx2_bb_system_types::file_path::*;
    use iceoryx2_bb_system_types::path::*;
    use iceoryx2_bb_testing::{assert_that, test_requires};

    #[test]
    fn config_file_settings_and_default_config_are_equal() {
        // NOTE: The test is skipped when not run with cargo but with bazel,
        //       since it does not work with the sandbox.
        //       The CI which runs with cargo ensures that the settings are equal.
        test_requires!(option_env!("CARGO").is_some());

        let default_config = Config::default();
        let top_level_dir = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .unwrap();

        let mut config_file_path =
            Path::new(&top_level_dir.stdout.as_slice()[..top_level_dir.stdout.len() - 1]).unwrap();

        config_file_path
            .add_path_entry(&Config::default_config_file_path().into())
            .unwrap();

        let file_config =
            Config::from_file(&FilePath::new(config_file_path.as_string()).unwrap()).unwrap();

        assert_that!(default_config, eq file_config);
    }
}
