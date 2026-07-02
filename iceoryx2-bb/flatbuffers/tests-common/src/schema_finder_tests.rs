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

extern crate alloc;

use alloc::{vec, vec::Vec};
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_flatbuffers::{find_best_fitting_schema_file, type_name};
use iceoryx2_bb_posix::config::TEST_DIRECTORY;
use iceoryx2_bb_posix::directory::Directory;
use iceoryx2_bb_posix::file::{CreationMode, File, FileBuilder, Permission};
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

struct Test {
    files: Vec<FilePath>,
    directories: Vec<Path>,
}

impl Drop for Test {
    fn drop(&mut self) {
        for file in &self.files {
            File::remove(file).unwrap();
        }

        for dir in self.directories.iter().rev() {
            Directory::remove(dir).unwrap();
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

    fn root_path(&self) -> &Path {
        &TEST_DIRECTORY
    }

    fn is_same_path(&self, relative: &[&str], absolute: &FilePath) -> bool {
        let lhs = Self::to_path(relative);
        lhs == *absolute
    }

    fn to_path(path: &[&str]) -> FilePath {
        let mut full_path = TEST_DIRECTORY;
        for entry in path {
            full_path
                .add_path_entry(&Path::new(entry.as_bytes()).unwrap())
                .unwrap();
        }
        FilePath::new(full_path.as_bytes()).unwrap()
    }

    fn create_file(&mut self, path: &[&str]) {
        let path = Self::to_path(path);
        self.files.push(path);
        FileBuilder::new(&path)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create()
            .unwrap();
    }

    fn create_directory(&mut self, path: &[&str]) {
        let path = Self::to_path(path);
        self.directories.push(path.into());
        Directory::create(&path.into(), Permission::OWNER_ALL).unwrap();
    }
}

pub mod test_name_space {
    pub struct FlatbufferTestType {}
}

#[test]
pub fn find_schema_works() {
    let mut test = Test::new();
    let path = vec!["flatbuffer_test_type.fbs"];
    test.create_file(&path);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&path, &sut.unwrap()), eq true);
}

#[test]
pub fn when_the_schema_is_not_existing_it_returns_none() {
    let test = Test::new();

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(sut, is_none);
}

#[test]
pub fn schema_in_namespace_is_preferred() {
    let mut test = Test::new();
    let path = vec!["test_name_space", "flatbuffer_test_type.fbs"];
    test.create_directory(&["test_name_space"]);
    test.create_file(&["flatbuffer_test_type.fbs"]);
    test.create_file(&path);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&path, &sut.unwrap()), eq true);
}

#[test]
pub fn schema_requires_flatbuffer_extension() {
    let mut test = Test::new();
    let path = vec!["flatbuffer_test_type.bs"];
    test.create_file(&path);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(sut, is_none);
}

#[test]
pub fn file_extension_can_have_lower_and_upper_case() {
    let mut test = Test::new();
    let path = vec!["flatbuffer_test_type.FbS"];
    test.create_file(&path);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&path, &sut.unwrap()), eq true);
}

#[test]
pub fn name_can_have_lower_and_upper_case() {
    let mut test = Test::new();
    let path = vec!["flatbUFFER_test_type.fbs"];
    test.create_file(&path);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&path, &sut.unwrap()), eq true);
}

#[test]
pub fn name_can_be_camel_case() {
    let mut test = Test::new();
    let path = vec!["FlatbufferTestType.fbs"];
    test.create_file(&path);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&path, &sut.unwrap()), eq true);
}

#[test]
pub fn schema_in_camel_case_namespace_is_preferred() {
    let mut test = Test::new();
    let path = vec!["TestNameSpace", "flatbuffer_test_type.fbs"];
    test.create_directory(&["TestNameSpace"]);
    test.create_file(&["flatbuffer_test_type.fbs"]);
    test.create_file(&path);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&path, &sut.unwrap()), eq true);
}

#[test]
pub fn schema_is_preferred_over_non_namespace_directory() {
    let mut test = Test::new();
    test.create_directory(&["fuu"]);
    test.create_file(&["flatbuffer_test_type.fbs"]);
    test.create_file(&["fuu", "flatbuffer_test_type.fbs"]);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&["flatbuffer_test_type.fbs"], &sut.unwrap()), eq true);
}

#[test]
pub fn the_correct_schema_file_is_picked() {
    let mut test = Test::new();
    test.create_file(&["flatbuffer_test_type.fbs"]);
    test.create_file(&["fer_test_type.fbs"]);
    test.create_file(&["flatbuffr_type.fbs"]);
    test.create_file(&["flatbuype.fbs"]);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&["flatbuffer_test_type.fbs"], &sut.unwrap()), eq true);
}

#[test]
pub fn namespace_directory_is_preferred_over_any_other_directory() {
    let mut test = Test::new();
    test.create_directory(&["apfelkopf"]);
    test.create_directory(&["TestNameSpace"]);
    test.create_file(&["apfelkopf", "flatbuffer_test_type.fbs"]);
    test.create_file(&["apfelkopf", "fuu_type.fbs"]);
    test.create_file(&["TestNameSpace", "flatbuffer_test_type.fbs"]);
    test.create_file(&["TestNameSpace", "fuu_type.fbs"]);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&["TestNameSpace", "flatbuffer_test_type.fbs"], &sut.unwrap()), eq true);
}

#[test]
pub fn namespace_directory_is_preferred_over_any_other_directory_in_subdirectory() {
    let mut test = Test::new();
    test.create_directory(&["apfelkopf"]);
    test.create_directory(&["birnenschaedel"]);
    test.create_directory(&["birnenschaedel", "Knoedel"]);
    test.create_directory(&["birnenschaedel", "TestNameSpace"]);
    test.create_file(&["birnenschaedel", "Knoedel", "flatbuffer_test_type.fbs"]);
    test.create_file(&["birnenschaedel", "Knoedel", "fuu_type.fbs"]);
    test.create_file(&[
        "birnenschaedel",
        "TestNameSpace",
        "flatbuffer_test_type.fbs",
    ]);
    test.create_file(&["birnenschaedel", "TestNameSpace", "fuu_type.fbs"]);

    let sut = find_best_fitting_schema_file(
        &type_name::<test_name_space::FlatbufferTestType>(),
        test.root_path(),
    )
    .unwrap();

    assert_that!(test.is_same_path(&["birnenschaedel", "TestNameSpace", "flatbuffer_test_type.fbs"], &sut.unwrap()), eq true);
}
