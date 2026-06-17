// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_flatbuffers::schema_finder;
use iceoryx2_bb_posix::config::TEST_DIRECTORY;
use iceoryx2_bb_posix::directory::Directory;
use iceoryx2_bb_posix::file::File;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

struct Test {
    files: Vec<String>,
    directories: Vec<String>,
}

impl Drop for Test {
    fn drop(&mut self) {
        for file in &self.files {
            let file = FileName::new(file.as_bytes()).unwrap();
            let path = FilePath::from_path_and_file(&TEST_DIRECTORY, &file).unwrap();

            File::remove(&path).unwrap();
        }

        for dir in self.directories.iter().rev() {
            let mut path = TEST_DIRECTORY;
            let entry = FileName::new(dir.as_bytes()).unwrap();
            path.add_path_entry(&entry.into()).unwrap();

            Directory::remove(&path).unwrap();
        }
    }
}

impl Test {
    fn new() -> Self {
        create_test_directory();
        Self {
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    fn create_file(path: &str) {}

    fn create_directory(path: &str) {}
}

#[test]
pub fn works() {
    let test = Test::new();
}
